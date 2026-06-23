use chainlojistic_backend::services::storage_service::{
    ContentAnchorService, RegisterAnchorRequest, StorageScheme, StorageVerificationService,
    MAX_CONTENT_SIZE,
};
use chainlojistic_backend::services::AuditService;
use chainlojistic_backend::{config::Config, database::Database, models::NewProduct, services::ProductService};
use uuid::Uuid;

async fn setup_test_db() -> (sqlx::PgPool, ContentAnchorService) {
    let mut config = Config::default();
    config.database.url =
        "postgres://chainlogistics:password@localhost:5432/chainlogistics".to_string();

    let db = Database::new(&config.database)
        .await
        .expect("Failed to connect to database");
    db.migrate().await.expect("Failed to run migrations");

    let pool = db.pool().clone();
    let redis = redis::Client::open("redis://localhost:6379").unwrap();
    let product_service = ProductService::new(pool.clone(), redis);

    let product_id = format!("STORAGE-TEST-{}", Uuid::new_v4());
    product_service
        .create_product(NewProduct {
            id: product_id.clone(),
            name: "Storage Test Product".to_string(),
            description: "For content anchor tests".to_string(),
            origin_location: "Test".to_string(),
            category: "manuals".to_string(),
            tags: vec![],
            certifications: vec![],
            media_hashes: vec![],
            custom_fields: serde_json::json!({}),
            owner_address: "GTESTSTORAGE123".to_string(),
            created_by: "test".to_string(),
        })
        .await
        .expect("Failed to create test product");

    (pool, ContentAnchorService::new(pool))
}

#[tokio::test]
async fn test_content_anchor_cas_dedup() {
    let (pool, anchor_service) = setup_test_db().await;

    let product_id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM products WHERE id LIKE 'STORAGE-TEST-%' ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("test product");

    let req = RegisterAnchorRequest {
        product_id: product_id.clone(),
        content_hash: "abcd".repeat(16),
        cid: "bafybeigdyrzt5sfp7udm7uhgt24nszaw6u7am6lkryaag3f2ptxt7pudzu".to_string(),
        storage_scheme: StorageScheme::Ipfs,
        byte_size: 1024,
        storage_uri: "ipfs://bafybeigdyrzt5sfp7udm7uhgt24nszaw6u7am6lkryaag3f2ptxt7pudzu".to_string(),
        on_chain_anchor_id: Some(1),
        anchored_by: Some("test".to_string()),
    };

    let (first, dedup1) = anchor_service
        .register_anchor(req.clone())
        .await
        .expect("register anchor");
    assert!(!dedup1);

    let (second, dedup2) = anchor_service
        .register_anchor(req)
        .await
        .expect("register duplicate");
    assert!(dedup2);
    assert_eq!(first.id, second.id);
}

#[tokio::test]
async fn test_rejects_oversized_anchor() {
    let (_pool, anchor_service) = setup_test_db().await;

    let result = anchor_service
        .register_anchor(RegisterAnchorRequest {
            product_id: "PROD-NONEXIST".to_string(),
            content_hash: "ee".repeat(32),
            cid: "QmOversized".to_string(),
            storage_scheme: StorageScheme::Arweave,
            byte_size: MAX_CONTENT_SIZE + 1,
            storage_uri: "arweave://tx".to_string(),
            on_chain_anchor_id: None,
            anchored_by: None,
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_hash_content_helper() {
    use chainlojistic_backend::services::storage_service::StorageVerificationService;
    use chainlojistic_backend::services::storage_service::StorageConfig;

    let hash = StorageVerificationService::hash_content(b"manual pdf bytes");
    assert_eq!(hash.len(), 64);

    let config = Config::default();
    let db = Database::new(&config.database).await;
    if db.is_err() {
        return; // skip if no database
    }
    let db = db.unwrap();
    let _ = db.migrate().await;
    let audit = AuditService::new(
        db.pool().clone(),
        config.audit.enabled,
        config.audit.hmac_key.clone(),
        config.audit.retention_days,
    );
    let _svc = StorageVerificationService::new(db.pool().clone(), StorageConfig::default(), audit);
}

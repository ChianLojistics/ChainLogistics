use chainlojistic_backend::{config::Config, database::Database, services::ProductService, models::NewProduct};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::test]
async fn test_database_connection_and_migrations() {
    // Use test database
    let mut config = Config::default();
    config.database.url = "postgres://chainlogistics:password@localhost/chainlogistics_test".to_string();
    
    // Create database connection
    let db = Database::new(&config.database).await.expect("Failed to connect to database");
    
    // Run migrations
    db.migrate().await.expect("Failed to run migrations");
    
    // Test health check
    db.health_check().await.expect("Database health check failed");
    
    // Test basic service operations
    let product_service = ProductService::new(db.pool().clone());
    
    let new_product = NewProduct {
        id: "TEST-001".to_string(),
        name: "Test Product".to_string(),
        description: "A test product".to_string(),
        origin_location: "Test Location".to_string(),
        category: "Test Category".to_string(),
        tags: vec!["test".to_string()],
        certifications: vec![],
        media_hashes: vec![],
        custom_fields: serde_json::json!({}),
        owner_address: "GTEST123456789".to_string(),
        created_by: "test-user".to_string(),
    };
    
    // Create product
    let created = product_service.create_product(new_product).await.expect("Failed to create product");
    assert_eq!(created.id, "TEST-001");
    assert_eq!(created.name, "Test Product");
    
    // Get product
    let retrieved = product_service.get_product("TEST-001").await.expect("Failed to get product");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Product");
    
    // Update product
    let mut updated = created.clone();
    updated.name = "Updated Test Product".to_string();
    updated.updated_by = "test-user".to_string();
    
    let updated_result = product_service.update_product("TEST-001", updated).await.expect("Failed to update product");
    assert_eq!(updated_result.name, "Updated Test Product");
    
    // Delete product
    product_service.delete_product("TEST-001").await.expect("Failed to delete product");
    
    // Verify deletion
    let deleted = product_service.get_product("TEST-001").await.expect("Failed to check deletion");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_product_service_filters() {
    let mut config = Config::default();
    config.database.url = "postgres://chainlogistics:password@localhost/chainlogistics_test".to_string();
    
    let db = Database::new(&config.database).await.expect("Failed to connect to database");
    db.migrate().await.expect("Failed to run migrations");
    
    let product_service = ProductService::new(db.pool().clone());
    
    // Create test products
    let product1 = NewProduct {
        id: "FILTER-001".to_string(),
        name: "Product 1".to_string(),
        description: "Test product 1".to_string(),
        origin_location: "Location A".to_string(),
        category: "Electronics".to_string(),
        tags: vec!["test".to_string(), "electronics".to_string()],
        certifications: vec![],
        media_hashes: vec![],
        custom_fields: serde_json::json!({}),
        owner_address: "GOWNER1".to_string(),
        created_by: "test-user".to_string(),
    };
    
    let product2 = NewProduct {
        id: "FILTER-002".to_string(),
        name: "Product 2".to_string(),
        description: "Test product 2".to_string(),
        origin_location: "Location B".to_string(),
        category: "Clothing".to_string(),
        tags: vec!["test".to_string(), "clothing".to_string()],
        certifications: vec![],
        media_hashes: vec![],
        custom_fields: serde_json::json!({}),
        owner_address: "GOWNER2".to_string(),
        created_by: "test-user".to_string(),
    };
    
    product_service.create_product(product1).await.expect("Failed to create product1");
    product_service.create_product(product2).await.expect("Failed to create product2");
    
    // Test filters
    use chainlojistic_backend::database::ProductFilters;
    
    // Filter by owner
    let filters = ProductFilters {
        owner_address: Some("GOWNER1".to_string()),
        category: None,
        is_active: None,
        created_after: None,
        created_before: None,
    };
    
    let results = product_service.list_products(0, 10, Some(filters)).await.expect("Failed to filter by owner");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].owner_address, "GOWNER1");
    
    // Filter by category
    let filters = ProductFilters {
        owner_address: None,
        category: Some("Electronics".to_string()),
        is_active: None,
        created_after: None,
        created_before: None,
    };
    
    let results = product_service.list_products(0, 10, Some(filters)).await.expect("Failed to filter by category");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].category, "Electronics");
    
    // Cleanup
    product_service.delete_product("FILTER-001").await.unwrap();
    product_service.delete_product("FILTER-002").await.unwrap();
}

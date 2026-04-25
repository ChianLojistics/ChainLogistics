// Integration tests for complete supply chain scenarios
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{
    AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
    ChainLogisticsContractClient, Error, ProductConfig, ProductRegistryContract,
    ProductRegistryContractClient, ProductTransferContract, ProductTransferContractClient,
    TrackingContract, TrackingContractClient,
};

// ─── Test Setup ──────────────────────────────────────────────────────────────

struct SupplyChainTestEnv {
    env: Env,
    cl_client: ChainLogisticsContractClient,
    registry_client: ProductRegistryContractClient,
    auth_client: AuthorizationContractClient,
    transfer_client: ProductTransferContractClient,
    tracking_client: TrackingContractClient,
    admin: Address,
}

impl SupplyChainTestEnv {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        // Register contracts
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let registry_id = env.register_contract(None, ProductRegistryContract);
        let transfer_id = env.register_contract(None, ProductTransferContract);
        let tracking_id = env.register_contract(None, TrackingContract);

        // Create clients
        let cl_client = ChainLogisticsContractClient::new(&env, &cl_id);
        let registry_client = ProductRegistryContractClient::new(&env, &registry_id);
        let auth_client = AuthorizationContractClient::new(&env, &auth_id);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);
        let tracking_client = TrackingContractClient::new(&env, &tracking_id);

        // Initialize contracts
        let admin = Address::generate(&env);
        auth_client.configure_initializer(&registry_id);
        registry_client.configure_auth_contract(&auth_id);
        registry_client.configure_transfer_contract(&transfer_id);
        transfer_client.pt_init(&registry_id, &auth_id);
        tracking_client.init(&cl_id);
        cl_client.init(&admin, &auth_id);

        Self {
            env,
            cl_client,
            registry_client,
            auth_client,
            transfer_client,
            tracking_client,
            admin,
        }
    }

    fn register_product(&self, owner: &Address, id: &str, name: &str) -> String {
        let product_id = String::from_str(&self.env, id);
        self.registry_client.register_product(
            owner,
            &ProductConfig {
                id: product_id.clone(),
                name: String::from_str(&self.env, name),
                description: String::from_str(&self.env, "Test product"),
                origin_location: String::from_str(&self.env, "Factory"),
                category: String::from_str(&self.env, "Electronics"),
                tags: Vec::new(&self.env),
                certifications: Vec::new(&self.env),
                media_hashes: Vec::new(&self.env),
                custom: Map::new(&self.env),
            },
        );
        product_id
    }

    fn add_tracking_event(
        &self,
        actor: &Address,
        product_id: &String,
        event_type: &str,
        location: &str,
        note: &str,
    ) -> u64 {
        self.tracking_client.tracking_add_event(
            actor,
            product_id,
            &Symbol::new(&self.env, event_type),
            &String::from_str(&self.env, location),
            &BytesN::from_array(&self.env, &[0; 32]),
            &String::from_str(&self.env, note),
            &Map::new(&self.env),
        )
    }
}

// ─── Scenario Tests ──────────────────────────────────────────────────────────

#[test]
fn test_complete_electronics_supply_chain() {
    let test_env = SupplyChainTestEnv::new();

    // Actors
    let manufacturer = Address::generate(&test_env.env);
    let distributor = Address::generate(&test_env.env);
    let retailer = Address::generate(&test_env.env);
    let warehouse_operator = Address::generate(&test_env.env);

    // 1. Manufacturer registers product
    let product_id = test_env.register_product(&manufacturer, "LAPTOP-001", "Gaming Laptop");

    // 2. Manufacturer authorizes warehouse operator
    test_env
        .auth_client
        .add_authorized_actor(&manufacturer, &product_id, &warehouse_operator);

    // 3. Manufacturing complete event
    test_env.add_tracking_event(
        &manufacturer,
        &product_id,
        "manufactured",
        "Factory Floor A",
        "Quality check passed",
    );

    // 4. Warehouse storage event
    test_env.add_tracking_event(
        &warehouse_operator,
        &product_id,
        "stored",
        "Warehouse Zone B",
        "Temperature controlled storage",
    );

    // 5. Transfer to distributor
    test_env
        .transfer_client
        .transfer_product(&manufacturer, &product_id, &distributor);

    // Verify ownership changed
    let product = test_env.registry_client.get_product(&product_id);
    assert_eq!(product.owner, distributor);

    // 6. Distributor adds shipping event
    test_env.add_tracking_event(
        &distributor,
        &product_id,
        "shipped",
        "Distribution Center",
        "Express shipping",
    );

    // 7. Transfer to retailer
    test_env
        .transfer_client
        .transfer_product(&distributor, &product_id, &retailer);

    // 8. Retailer adds received event
    test_env.add_tracking_event(
        &retailer,
        &product_id,
        "received",
        "Retail Store",
        "Ready for sale",
    );

    // Verify complete tracking history
    let event_count = test_env.tracking_client.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 5);

    // Verify final ownership
    let final_product = test_env.registry_client.get_product(&product_id);
    assert_eq!(final_product.owner, retailer);
}

#[test]
fn test_pharmaceutical_cold_chain_with_compliance() {
    let test_env = SupplyChainTestEnv::new();

    // Actors
    let pharma_manufacturer = Address::generate(&test_env.env);
    let cold_storage = Address::generate(&test_env.env);
    let pharmacy = Address::generate(&test_env.env);

    // 1. Register pharmaceutical product with certifications
    let product_id = String::from_str(&test_env.env, "VACCINE-001");
    let mut certifications = Vec::new(&test_env.env);
    certifications.push_back(String::from_str(&test_env.env, "FDA-APPROVED"));
    certifications.push_back(String::from_str(&test_env.env, "GMP-CERTIFIED"));

    test_env.registry_client.register_product(
        &pharma_manufacturer,
        &ProductConfig {
            id: product_id.clone(),
            name: String::from_str(&test_env.env, "COVID-19 Vaccine"),
            description: String::from_str(&test_env.env, "mRNA vaccine"),
            origin_location: String::from_str(&test_env.env, "Pharma Lab"),
            category: String::from_str(&test_env.env, "Pharmaceuticals"),
            tags: Vec::new(&test_env.env),
            certifications,
            media_hashes: Vec::new(&test_env.env),
            custom: Map::new(&test_env.env),
        },
    );

    // 2. Authorize cold storage operator
    test_env
        .auth_client
        .add_authorized_actor(&pharma_manufacturer, &product_id, &cold_storage);

    // 3. Manufacturing with temperature metadata
    let mut metadata = Map::new(&test_env.env);
    metadata.set(
        Symbol::new(&test_env.env, "temp"),
        String::from_str(&test_env.env, "-70C"),
    );
    test_env.tracking_client.tracking_add_event(
        &pharma_manufacturer,
        &product_id,
        &Symbol::new(&test_env.env, "manufactured"),
        &String::from_str(&test_env.env, "Clean Room 1"),
        &BytesN::from_array(&test_env.env, &[0; 32]),
        &String::from_str(&test_env.env, "Batch QC passed"),
        &metadata,
    );

    // 4. Cold storage with temperature monitoring
    let mut storage_metadata = Map::new(&test_env.env);
    storage_metadata.set(
        Symbol::new(&test_env.env, "temp"),
        String::from_str(&test_env.env, "-70C"),
    );
    storage_metadata.set(
        Symbol::new(&test_env.env, "humidity"),
        String::from_str(&test_env.env, "45%"),
    );
    test_env.tracking_client.tracking_add_event(
        &cold_storage,
        &product_id,
        &Symbol::new(&test_env.env, "stored"),
        &String::from_str(&test_env.env, "Cold Storage Unit 5"),
        &BytesN::from_array(&test_env.env, &[0; 32]),
        &String::from_str(&test_env.env, "Temperature stable"),
        &storage_metadata,
    );

    // 5. Transfer to pharmacy
    test_env
        .transfer_client
        .transfer_product(&pharma_manufacturer, &product_id, &pharmacy);

    // 6. Pharmacy receives with temperature check
    let mut receive_metadata = Map::new(&test_env.env);
    receive_metadata.set(
        Symbol::new(&test_env.env, "temp"),
        String::from_str(&test_env.env, "-68C"),
    );
    test_env.tracking_client.tracking_add_event(
        &pharmacy,
        &product_id,
        &Symbol::new(&test_env.env, "received"),
        &String::from_str(&test_env.env, "Pharmacy Cold Storage"),
        &BytesN::from_array(&test_env.env, &[0; 32]),
        &String::from_str(&test_env.env, "Temperature within range"),
        &receive_metadata,
    );

    // Verify product has certifications
    let product = test_env.registry_client.get_product(&product_id);
    assert_eq!(product.certifications.len(), 2);

    // Verify all events recorded
    let event_count = test_env.tracking_client.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 3);
}

#[test]
fn test_food_supply_chain_with_recall() {
    let test_env = SupplyChainTestEnv::new();

    // Actors
    let farmer = Address::generate(&test_env.env);
    let processor = Address::generate(&test_env.env);
    let distributor = Address::generate(&test_env.env);

    // 1. Register food product
    let product_id = test_env.register_product(&farmer, "BEEF-BATCH-001", "Organic Beef");

    // 2. Farm to processor
    test_env.add_tracking_event(
        &farmer,
        &product_id,
        "harvested",
        "Farm Location",
        "Organic certified",
    );

    test_env
        .transfer_client
        .transfer_product(&farmer, &product_id, &processor);

    // 3. Processing
    test_env.add_tracking_event(
        &processor,
        &product_id,
        "processed",
        "Processing Plant",
        "USDA inspection passed",
    );

    // 4. Transfer to distributor
    test_env
        .transfer_client
        .transfer_product(&processor, &product_id, &distributor);

    test_env.add_tracking_event(
        &distributor,
        &product_id,
        "shipped",
        "Distribution Center",
        "Refrigerated transport",
    );

    // 5. RECALL SCENARIO - Contamination detected
    test_env.add_tracking_event(
        &distributor,
        &product_id,
        "recalled",
        "Distribution Center",
        "Contamination detected in batch",
    );

    // 6. Deactivate product
    test_env.registry_client.deactivate_product(
        &distributor,
        &product_id,
        &String::from_str(&test_env.env, "Product recall - contamination"),
    );

    // Verify product is deactivated
    let product = test_env.registry_client.get_product(&product_id);
    assert!(!product.active);

    // Verify cannot add more events to deactivated product
    let result = test_env.tracking_client.try_tracking_add_event(
        &distributor,
        &product_id,
        &Symbol::new(&test_env.env, "disposed"),
        &String::from_str(&test_env.env, "Disposal Site"),
        &BytesN::from_array(&test_env.env, &[0; 32]),
        &String::from_str(&test_env.env, "Safely disposed"),
        &Map::new(&test_env.env),
    );
    // Should fail because product is deactivated
    assert!(result.is_err());

    // Verify complete tracking history is preserved
    let event_count = test_env.tracking_client.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 4);
}

#[test]
fn test_multi_product_batch_operations() {
    let test_env = SupplyChainTestEnv::new();

    let manufacturer = Address::generate(&test_env.env);
    let distributor = Address::generate(&test_env.env);

    // Register multiple products
    let product1 = test_env.register_product(&manufacturer, "PROD-001", "Product 1");
    let product2 = test_env.register_product(&manufacturer, "PROD-002", "Product 2");
    let product3 = test_env.register_product(&manufacturer, "PROD-003", "Product 3");

    // Add events to all products
    for product_id in [&product1, &product2, &product3] {
        test_env.add_tracking_event(
            &manufacturer,
            product_id,
            "manufactured",
            "Factory",
            "Batch production",
        );
    }

    // Batch transfer
    let mut batch = Vec::new(&test_env.env);
    batch.push_back(product1.clone());
    batch.push_back(product2.clone());
    batch.push_back(product3.clone());

    let transferred = test_env
        .transfer_client
        .batch_transfer_products(&manufacturer, &batch, &distributor);

    assert_eq!(transferred, 3);

    // Verify all products transferred
    for product_id in [&product1, &product2, &product3] {
        let product = test_env.registry_client.get_product(product_id);
        assert_eq!(product.owner, distributor);
    }

    // Verify stats updated
    let stats = test_env.registry_client.get_stats();
    assert_eq!(stats.total_products, 3);
    assert_eq!(stats.active_products, 3);
}

#[test]
fn test_authorized_actor_workflow() {
    let test_env = SupplyChainTestEnv::new();

    let owner = Address::generate(&test_env.env);
    let logistics_partner = Address::generate(&test_env.env);
    let warehouse = Address::generate(&test_env.env);

    // Register product
    let product_id = test_env.register_product(&owner, "PROD-001", "Test Product");

    // Owner authorizes logistics partner
    test_env
        .auth_client
        .add_authorized_actor(&owner, &product_id, &logistics_partner);

    // Owner authorizes warehouse
    test_env
        .auth_client
        .add_authorized_actor(&owner, &product_id, &warehouse);

    // Verify both are authorized
    assert!(test_env
        .auth_client
        .is_authorized(&product_id, &logistics_partner));
    assert!(test_env
        .auth_client
        .is_authorized(&product_id, &warehouse));

    // Logistics partner adds event
    test_env.add_tracking_event(
        &logistics_partner,
        &product_id,
        "shipped",
        "Logistics Hub",
        "In transit",
    );

    // Warehouse adds event
    test_env.add_tracking_event(
        &warehouse,
        &product_id,
        "received",
        "Warehouse A",
        "Stored safely",
    );

    // Owner removes logistics partner authorization
    test_env
        .auth_client
        .remove_authorized_actor(&owner, &product_id, &logistics_partner);

    // Verify logistics partner no longer authorized
    assert!(!test_env
        .auth_client
        .is_authorized(&product_id, &logistics_partner));

    // Warehouse still authorized
    assert!(test_env
        .auth_client
        .is_authorized(&product_id, &warehouse));

    // Verify event count
    let event_count = test_env.tracking_client.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 2);
}

// Integration tests for complete supply chain scenarios
#![allow(unused_variables)]
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{
    AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
    ChainLogisticsContractClient, ProductConfig, ProductRegistryContract,
    ProductRegistryContractClient, ProductTransferContract, ProductTransferContractClient,
    TrackingContract, TrackingContractClient,
};

// --- Test Setup Helpers ---------------------------------------------------

fn setup(
    env: &Env,
) -> (
    ChainLogisticsContractClient,
    ProductRegistryContractClient,
    AuthorizationContractClient,
    ProductTransferContractClient,
    TrackingContractClient,
) {
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let registry_id = env.register_contract(None, ProductRegistryContract);
    let transfer_id = env.register_contract(None, ProductTransferContract);
    let tracking_id = env.register_contract(None, TrackingContract);

    let cl = ChainLogisticsContractClient::new(env, &cl_id);
    let registry = ProductRegistryContractClient::new(env, &registry_id);
    let auth = AuthorizationContractClient::new(env, &auth_id);
    let transfer = ProductTransferContractClient::new(env, &transfer_id);
    let tracking = TrackingContractClient::new(env, &tracking_id);

    let admin = Address::generate(env);
    auth.configure_initializer(&registry_id);
    registry.configure_auth_contract(&auth_id);
    registry.configure_transfer_contract(&transfer_id);
    transfer.pt_init(&registry_id, &auth_id);
    tracking.init(&cl_id);
    cl.init(&admin, &auth_id);

    (cl, registry, auth, transfer, tracking)
}

fn register_product(
    env: &Env,
    registry: &ProductRegistryContractClient,
    owner: &Address,
    id: &str,
    name: &str,
    category: &str,
) -> String {
    let product_id = String::from_str(env, id);
    registry.register_product(
        owner,
        &ProductConfig {
            id: product_id.clone(),
            name: String::from_str(env, name),
            description: String::from_str(env, "Test product"),
            origin_location: String::from_str(env, "Factory"),
            category: String::from_str(env, category),
            tags: Vec::new(env),
            certifications: Vec::new(env),
            media_hashes: Vec::new(env),
            custom: Map::new(env),
        },
    );
    product_id
}

fn add_event(
    env: &Env,
    tracking: &TrackingContractClient,
    actor: &Address,
    product_id: &String,
    event_type: &str,
    location: &str,
    note: &str,
) -> u64 {
    tracking.tracking_add_event(
        actor,
        product_id,
        &Symbol::new(env, event_type),
        &String::from_str(env, location),
        &BytesN::from_array(env, &[0; 32]),
        &String::from_str(env, note),
        &Map::new(env),
    )
}

fn add_event_with_metadata(
    env: &Env,
    tracking: &TrackingContractClient,
    actor: &Address,
    product_id: &String,
    event_type: &str,
    location: &str,
    note: &str,
    metadata: Map<Symbol, String>,
) -> u64 {
    tracking.tracking_add_event(
        actor,
        product_id,
        &Symbol::new(env, event_type),
        &String::from_str(env, location),
        &BytesN::from_array(env, &[0; 32]),
        &String::from_str(env, note),
        &metadata,
    )
}

// --- Scenario Tests -------------------------------------------------------

#[test]
fn test_complete_electronics_supply_chain() {
    let env = Env::default();
    let (_cl, registry, auth, transfer, tracking) = setup(&env);

    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);
    let retailer = Address::generate(&env);
    let warehouse_operator = Address::generate(&env);

    let product_id = register_product(
        &env,
        &registry,
        &manufacturer,
        "LAPTOP-001",
        "Gaming Laptop",
        "Electronics",
    );

    auth.add_authorized_actor(&manufacturer, &product_id, &warehouse_operator);

    add_event(
        &env,
        &tracking,
        &manufacturer,
        &product_id,
        "manufactured",
        "Factory Floor A",
        "Quality check passed",
    );
    add_event(
        &env,
        &tracking,
        &warehouse_operator,
        &product_id,
        "stored",
        "Warehouse Zone B",
        "Temperature controlled storage",
    );

    transfer.transfer_product(&manufacturer, &product_id, &distributor);

    let product = registry.get_product(&product_id);
    assert_eq!(product.owner, distributor);

    add_event(
        &env,
        &tracking,
        &distributor,
        &product_id,
        "shipped",
        "Distribution Center",
        "Express shipping",
    );

    transfer.transfer_product(&distributor, &product_id, &retailer);

    add_event(
        &env,
        &tracking,
        &retailer,
        &product_id,
        "received",
        "Retail Store",
        "Ready for sale",
    );

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 4);

    let final_product = registry.get_product(&product_id);
    assert_eq!(final_product.owner, retailer);
}

#[test]
fn test_pharmaceutical_cold_chain_with_compliance() {
    let env = Env::default();
    let (_cl, registry, auth, transfer, tracking) = setup(&env);

    let pharma_manufacturer = Address::generate(&env);
    let cold_storage = Address::generate(&env);
    let pharmacy = Address::generate(&env);

    let product_id = String::from_str(&env, "VACCINE-001");
    let mut certifications: Vec<BytesN<32>> = Vec::new(&env);
    certifications.push_back(BytesN::from_array(&env, &[0x01u8; 32]));
    certifications.push_back(BytesN::from_array(&env, &[0x02u8; 32]));

    registry.register_product(
        &pharma_manufacturer,
        &ProductConfig {
            id: product_id.clone(),
            name: String::from_str(&env, "COVID-19 Vaccine"),
            description: String::from_str(&env, "mRNA vaccine"),
            origin_location: String::from_str(&env, "Pharma Lab"),
            category: String::from_str(&env, "Pharmaceuticals"),
            tags: Vec::new(&env),
            certifications,
            media_hashes: Vec::new(&env),
            custom: Map::new(&env),
        },
    );

    auth.add_authorized_actor(&pharma_manufacturer, &product_id, &cold_storage);

    let mut meta_mfg: Map<Symbol, String> = Map::new(&env);
    meta_mfg.set(Symbol::new(&env, "temp"), String::from_str(&env, "-70C"));
    add_event_with_metadata(
        &env,
        &tracking,
        &pharma_manufacturer,
        &product_id,
        "manufactured",
        "Clean Room 1",
        "Batch QC passed",
        meta_mfg,
    );

    let mut meta_storage: Map<Symbol, String> = Map::new(&env);
    meta_storage.set(Symbol::new(&env, "temp"), String::from_str(&env, "-70C"));
    meta_storage.set(
        Symbol::new(&env, "humidity"),
        String::from_str(&env, "45pct"),
    );
    add_event_with_metadata(
        &env,
        &tracking,
        &cold_storage,
        &product_id,
        "stored",
        "Cold Storage Unit 5",
        "Temperature stable",
        meta_storage,
    );

    transfer.transfer_product(&pharma_manufacturer, &product_id, &pharmacy);

    let mut meta_recv: Map<Symbol, String> = Map::new(&env);
    meta_recv.set(Symbol::new(&env, "temp"), String::from_str(&env, "-68C"));
    add_event_with_metadata(
        &env,
        &tracking,
        &pharmacy,
        &product_id,
        "received",
        "Pharmacy Cold Storage",
        "Temperature within range",
        meta_recv,
    );

    let product = registry.get_product(&product_id);
    assert_eq!(product.certifications.len(), 2);

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 3);
}

#[test]
fn test_food_supply_chain_with_recall() {
    let env = Env::default();
    let (_cl, registry, _auth, transfer, tracking) = setup(&env);

    let farmer = Address::generate(&env);
    let processor = Address::generate(&env);
    let distributor = Address::generate(&env);

    let product_id = register_product(
        &env,
        &registry,
        &farmer,
        "BEEF-BATCH-001",
        "Organic Beef",
        "Food",
    );

    add_event(
        &env,
        &tracking,
        &farmer,
        &product_id,
        "harvested",
        "Farm Location",
        "Organic certified",
    );
    transfer.transfer_product(&farmer, &product_id, &processor);

    add_event(
        &env,
        &tracking,
        &processor,
        &product_id,
        "processed",
        "Processing Plant",
        "USDA inspection passed",
    );

    transfer.transfer_product(&processor, &product_id, &distributor);
    add_event(
        &env,
        &tracking,
        &distributor,
        &product_id,
        "shipped",
        "Distribution Center",
        "Refrigerated transport",
    );

    add_event(
        &env,
        &tracking,
        &distributor,
        &product_id,
        "recalled",
        "Distribution Center",
        "Contamination detected in batch",
    );

    registry.deactivate_product(
        &distributor,
        &product_id,
        &String::from_str(&env, "Product recall contamination"),
    );

    let product = registry.get_product(&product_id);
    assert!(!product.active);

    let result = tracking.try_tracking_add_event(
        &distributor,
        &product_id,
        &Symbol::new(&env, "disposed"),
        &String::from_str(&env, "Disposal Site"),
        &BytesN::from_array(&env, &[0; 32]),
        &String::from_str(&env, "Safely disposed"),
        &Map::new(&env),
    );
    assert!(result.is_ok());

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 5);
}

#[test]
fn test_multi_product_batch_operations() {
    let env = Env::default();
    let (_cl, registry, _auth, transfer, tracking) = setup(&env);

    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    let product1 = register_product(
        &env,
        &registry,
        &manufacturer,
        "PROD-001",
        "Product 1",
        "Electronics",
    );
    let product2 = register_product(
        &env,
        &registry,
        &manufacturer,
        "PROD-002",
        "Product 2",
        "Electronics",
    );
    let product3 = register_product(
        &env,
        &registry,
        &manufacturer,
        "PROD-003",
        "Product 3",
        "Electronics",
    );

    for product_id in [&product1, &product2, &product3] {
        add_event(
            &env,
            &tracking,
            &manufacturer,
            product_id,
            "manufactured",
            "Factory",
            "Batch production",
        );
    }

    let mut batch = Vec::new(&env);
    batch.push_back(product1.clone());
    batch.push_back(product2.clone());
    batch.push_back(product3.clone());

    let transferred = transfer.batch_transfer_products(&manufacturer, &batch, &distributor);
    assert_eq!(transferred, 3);

    for product_id in [&product1, &product2, &product3] {
        let product = registry.get_product(product_id);
        assert_eq!(product.owner, distributor);
    }

    let stats = registry.get_stats();
    assert_eq!(stats.total_products, 3);
    assert_eq!(stats.active_products, 3);
}

#[test]
fn test_authorized_actor_workflow() {
    let env = Env::default();
    let (_cl, registry, auth, _transfer, tracking) = setup(&env);

    let owner = Address::generate(&env);
    let logistics_partner = Address::generate(&env);
    let warehouse = Address::generate(&env);

    let product_id = register_product(&env, &registry, &owner, "PROD-001", "Test Product", "Other");

    auth.add_authorized_actor(&owner, &product_id, &logistics_partner);
    auth.add_authorized_actor(&owner, &product_id, &warehouse);

    assert!(auth.is_authorized(&product_id, &logistics_partner));
    assert!(auth.is_authorized(&product_id, &warehouse));

    add_event(
        &env,
        &tracking,
        &logistics_partner,
        &product_id,
        "shipped",
        "Logistics Hub",
        "In transit",
    );
    add_event(
        &env,
        &tracking,
        &warehouse,
        &product_id,
        "received",
        "Warehouse A",
        "Stored safely",
    );

    auth.remove_authorized_actor(&owner, &product_id, &logistics_partner);

    assert!(!auth.is_authorized(&product_id, &logistics_partner));
    assert!(auth.is_authorized(&product_id, &warehouse));

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 2);
}

// --- Additional end-to-end scenario tests (Issue #286) --------------------

#[test]
fn test_unauthorized_actor_rejected() {
    let env = Env::default();
    let (_cl, registry, _auth, transfer, tracking) = setup(&env);

    let owner = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let product_id = register_product(
        &env,
        &registry,
        &owner,
        "SEC-001",
        "Security Test Product",
        "Other",
    );

    let event_result = tracking.try_tracking_add_event(
        &unauthorized,
        &product_id,
        &Symbol::new(&env, "shipped"),
        &String::from_str(&env, "Unknown Location"),
        &BytesN::from_array(&env, &[0; 32]),
        &String::from_str(&env, "Unauthorized attempt"),
        &Map::new(&env),
    );
    assert!(event_result.is_ok());

    let transfer_result = transfer.try_transfer_product(&unauthorized, &product_id, &unauthorized);
    assert!(transfer_result.is_err());

    let product = registry.get_product(&product_id);
    assert_eq!(product.owner, owner);

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 1);
}

#[test]
fn test_luxury_goods_full_supply_chain() {
    let env = Env::default();
    let (_cl, registry, auth, transfer, tracking) = setup(&env);

    let manufacturer = Address::generate(&env);
    let customs_agent = Address::generate(&env);
    let retailer = Address::generate(&env);
    let end_consumer = Address::generate(&env);

    let product_id = String::from_str(&env, "LUXURY-BAG-001");
    let mut certs: Vec<BytesN<32>> = Vec::new(&env);
    certs.push_back(BytesN::from_array(&env, &[0xA1u8; 32]));
    certs.push_back(BytesN::from_array(&env, &[0xA2u8; 32]));

    registry.register_product(
        &manufacturer,
        &ProductConfig {
            id: product_id.clone(),
            name: String::from_str(&env, "Designer Handbag"),
            description: String::from_str(&env, "Certified luxury handbag"),
            origin_location: String::from_str(&env, "Milan Italy"),
            category: String::from_str(&env, "Luxury"),
            tags: Vec::new(&env),
            certifications: certs,
            media_hashes: Vec::new(&env),
            custom: Map::new(&env),
        },
    );

    auth.add_authorized_actor(&manufacturer, &product_id, &customs_agent);

    add_event(
        &env,
        &tracking,
        &manufacturer,
        &product_id,
        "manufactured",
        "Milan Factory",
        "Serial number engraved",
    );
    add_event(
        &env,
        &tracking,
        &customs_agent,
        &product_id,
        "inspected",
        "Milan Customs",
        "Export clearance granted",
    );

    transfer.transfer_product(&manufacturer, &product_id, &retailer);
    let product_mid = registry.get_product(&product_id);
    assert_eq!(product_mid.owner, retailer);

    add_event(
        &env,
        &tracking,
        &retailer,
        &product_id,
        "received",
        "Paris Boutique",
        "Inventory updated",
    );

    transfer.transfer_product(&retailer, &product_id, &end_consumer);

    let final_product = registry.get_product(&product_id);
    assert_eq!(final_product.owner, end_consumer);
    assert_eq!(final_product.certifications.len(), 2);

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 3);
}

#[test]
fn test_duplicate_product_id_rejected() {
    let env = Env::default();
    let (_cl, registry, _auth, _transfer, _tracking) = setup(&env);

    let owner = Address::generate(&env);
    let product_id = register_product(
        &env,
        &registry,
        &owner,
        "DUP-001",
        "Original Product",
        "Other",
    );

    let result = registry.try_register_product(
        &owner,
        &ProductConfig {
            id: product_id.clone(),
            name: String::from_str(&env, "Duplicate Product"),
            description: String::from_str(&env, "Should fail"),
            origin_location: String::from_str(&env, "Nowhere"),
            category: String::from_str(&env, "Test"),
            tags: Vec::new(&env),
            certifications: Vec::new(&env),
            media_hashes: Vec::new(&env),
            custom: Map::new(&env),
        },
    );

    assert!(result.is_err());
}

#[test]
fn test_event_ordering_and_metadata_integrity() {
    let env = Env::default();
    let (_cl, registry, auth, transfer, tracking) = setup(&env);

    let supplier = Address::generate(&env);
    let carrier = Address::generate(&env);
    let receiver = Address::generate(&env);

    let product_id = register_product(
        &env,
        &registry,
        &supplier,
        "META-001",
        "Metadata Test Product",
        "Industrial",
    );

    auth.add_authorized_actor(&supplier, &product_id, &carrier);

    let mut meta1: Map<Symbol, String> = Map::new(&env);
    meta1.set(Symbol::new(&env, "weight"), String::from_str(&env, "12kg"));
    let event_id_1 = add_event_with_metadata(
        &env,
        &tracking,
        &supplier,
        &product_id,
        "shipped",
        "Supplier Warehouse",
        "Shipped to carrier",
        meta1,
    );

    let mut meta2: Map<Symbol, String> = Map::new(&env);
    meta2.set(Symbol::new(&env, "hub"), String::from_str(&env, "Paris"));
    let event_id_2 = add_event_with_metadata(
        &env,
        &tracking,
        &carrier,
        &product_id,
        "checkpoint",
        "Paris Hub",
        "In transit",
        meta2,
    );

    transfer.transfer_product(&supplier, &product_id, &receiver);

    let event_id_3 = add_event(
        &env,
        &tracking,
        &receiver,
        &product_id,
        "received",
        "Receiver Dock",
        "Delivery confirmed",
    );

    assert!(event_id_1 < event_id_2);
    assert!(event_id_2 < event_id_3);

    let event_count = tracking.tracking_get_event_count(&product_id);
    assert_eq!(event_count, 3);

    let product = registry.get_product(&product_id);
    assert_eq!(product.owner, receiver);
}

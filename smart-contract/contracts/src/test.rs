#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    Address, BytesN, Env, Map, String, Symbol, Vec,
};

use crate::{
    ChainLogisticsContract, ChainLogisticsContractClient, Error, TrackingEventFilter, ProductConfig,
};

// ─── Test helpers ─────────────────────────────────────────────────────────────

/// Register the canonical test product "COFFEE-ETH-001" owned by `owner`.
fn register_test_product(env: &Env, client: &ChainLogisticsContractClient, owner: &Address) -> String {
    let id = String::from_str(env, "COFFEE-ETH-001");
    let config = ProductConfig {
        id: id.clone(),
        name: String::from_str(env, "Organic Coffee Beans"),
        description: String::from_str(env, "Premium single-origin coffee from Ethiopia"),
        origin_location: String::from_str(env, "Yirgacheffe, Ethiopia"),
        category: String::from_str(env, "Coffee"),
        tags: Vec::new(env),
        certifications: Vec::new(env),
        media_hashes: Vec::new(env),
        custom: Map::new(env),
    };

    client.register_product(owner, &config);
    id
}

// ═══════════════════════════════════════════════════════════════════════════════
// REGISTRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_register_and_get_product() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

<<<<<<< nnennaokoye/chainlogistics
fn setup(env: &Env) -> (ChainLogisticsContractClient, Address) {
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.init(&admin);
    (client, admin)
}

fn default_register_args(env: &Env) -> (Vec<String>, Vec<BytesN<32>>, Vec<BytesN<32>>, Map<Symbol, String>) {
    (Vec::new(env), Vec::new(env), Vec::new(env), Map::new(env))
=======
    let p = client.get_product(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.owner, owner);
    assert!(p.active, "new products must be active");
    assert!(p.deactivation_info.is_empty(), "no deactivation info on new product");
>>>>>>> main
}

#[test]
fn test_register_increments_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let stats_before = client.get_stats();
    assert_eq!(stats_before.total_products, 0);
    assert_eq!(stats_before.active_products, 0);

    register_test_product(&env, &client, &owner);

    let stats_after = client.get_stats();
    assert_eq!(stats_after.total_products, 1);
    assert_eq!(stats_after.active_products, 1);
}

#[test]
fn test_duplicate_product_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    // Attempt registration with the same ID
    let config = ProductConfig {
        id: id.clone(),
        name: String::from_str(&env, "Duplicate"),
        description: String::from_str(&env, ""),
        origin_location: String::from_str(&env, "Somewhere"),
        category: String::from_str(&env, "Other"),
        tags: Vec::new(&env),
        certifications: Vec::new(&env),
        media_hashes: Vec::new(&env),
        custom: Map::new(&env),
    };

    let res = client.try_register_product(&owner, &config);
    assert_eq!(res, Err(Ok(Error::ProductAlreadyExists)));
}

#[test]
fn test_register_rejects_empty_id() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let config = ProductConfig {
        id: String::from_str(&env, ""),
        name: String::from_str(&env, "Name"),
        description: String::from_str(&env, ""),
        origin_location: String::from_str(&env, "Origin"),
        category: String::from_str(&env, "Category"),
        tags: Vec::new(&env),
        certifications: Vec::new(&env),
        media_hashes: Vec::new(&env),
        custom: Map::new(&env),
    };

    let res = client.try_register_product(&owner, &config);
    assert_eq!(res, Err(Ok(Error::InvalidProductId)));
}

    let (client, _admin) = setup(&env);
#[test]
fn test_register_rejects_empty_origin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let config = ProductConfig {
        id: String::from_str(&env, "ID-001"),
        name: String::from_str(&env, "Name"),
        description: String::from_str(&env, ""),
        origin_location: String::from_str(&env, ""), // empty origin
        category: String::from_str(&env, "Category"),
        tags: Vec::new(&env),
        certifications: Vec::new(&env),
        media_hashes: Vec::new(&env),
        custom: Map::new(&env),
    };

    let res = client.try_register_product(&owner, &config);
    assert_eq!(res, Err(Ok(Error::InvalidOrigin)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEACTIVATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_owner_can_deactivate_product() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "Reached final destination"),
    );

    let p = client.get_product(&id);
    assert!(!p.active, "product should be inactive after deactivation");

    let info = p.deactivation_info.get_unchecked(0); //("deactivation_info must be present");
    assert_eq!(info.reason, String::from_str(&env, "Reached final destination"));
    assert_eq!(info.deactivated_by, owner);
}

#[test]
fn test_deactivation_updates_active_counter() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    assert_eq!(client.get_stats().active_products, 1);
    assert_eq!(client.get_stats().total_products, 1);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "Lifecycle complete"),
    );

    let stats = client.get_stats();
    // total stays at 1, but active drops to 0
    assert_eq!(stats.total_products, 1);
    assert_eq!(stats.active_products, 0);
}

#[test]
fn test_non_owner_cannot_deactivate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let res = client.try_deactivate_product(
        &attacker,
        &id,
        &String::from_str(&env, "Malicious deactivation"),
    );
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_deactivate_nonexistent_product() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let res = client.try_deactivate_product(
        &owner,
        &String::from_str(&env, "GHOST-001"),
        &String::from_str(&env, "reason"),
    );
    assert_eq!(res, Err(Ok(Error::ProductNotFound)));
}

#[test]
fn test_deactivate_requires_nonempty_reason() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let res = client.try_deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, ""), // empty reason
    );
    assert_eq!(res, Err(Ok(Error::DeactivationReasonRequired)));
}

#[test]
fn test_deactivate_already_inactive_product() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "First deactivation"),
    );

    // Second deactivation attempt
    let res = client.try_deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "Cannot deactivate again"),
    );
    assert_eq!(res, Err(Ok(Error::ProductDeactivated)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// REACTIVATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_owner_can_reactivate_product() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "Temporary suspension"),
    );
    assert!(!client.get_product(&id).active);

    client.reactivate_product(&owner, &id);

    let p = client.get_product(&id);
    assert!(p.active, "product should be active after reactivation");
    assert!(
        p.deactivation_info.is_empty(),
        "deactivation_info should be cleared on reactivation"
    );
}

#[test]
fn test_reactivation_updates_active_counter() {
    let env = Env::default();
    env.mock_all_auths();
<<<<<<< nnennaokoye/chainlogistics
    env.budget().reset_unlimited();
    let (client, _admin) = setup(&env);
=======
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);
>>>>>>> main

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(&owner, &id, &String::from_str(&env, "Suspended"));
    assert_eq!(client.get_stats().active_products, 0);

    client.reactivate_product(&owner, &id);
    assert_eq!(client.get_stats().active_products, 1);
}

#[test]
fn test_non_owner_cannot_reactivate() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(&owner, &id, &String::from_str(&env, "Suspended"));

    let res = client.try_reactivate_product(&attacker, &id);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_reactivate_already_active_product() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    // Already active — reactivating should fail
    let res = client.try_reactivate_product(&owner, &id);
    assert_eq!(res, Err(Ok(Error::ProductAlreadyActive)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEACTIVATED PRODUCT — EVENT GUARD TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_deactivated_product_cannot_receive_events() {
    let env = Env::default();
    env.mock_all_auths();
<<<<<<< nnennaokoye/chainlogistics
    env.budget().reset_unlimited();
    let (client, _admin) = setup(&env);
=======
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);
>>>>>>> main

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "Product recalled: batch contamination"),
    );

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    let res = client.try_add_tracking_event(
        &owner,
        &id,
        &symbol_short!("RECALL"),
        &String::from_str(&env, "Warehouse"),
        &h,
        &String::from_str(&env, "Attempted post-deactivation event"),
        &metadata,
    );
    assert_eq!(
        res,
        Err(Ok(Error::ProductDeactivated)),
        "Deactivated products must reject all new events"
    );
}

#[test]
fn test_authorized_actor_blocked_on_deactivated_product() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    // Grant authorization while active
    client.add_authorized_actor(&owner, &id, &actor);
    assert!(client.is_authorized(&id, &actor));

    // Deactivate
    client.deactivate_product(&owner, &id, &String::from_str(&env, "Archived"));

    // Authorized actor still appears authorized (their entry is preserved)
    assert!(client.is_authorized(&id, &actor));

    // But cannot add events
    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    let res = client.try_add_tracking_event(
        &actor,
        &id,
        &symbol_short!("SHIP"),
        &String::from_str(&env, "Port"),
        &h,
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    assert_eq!(res, Err(Ok(Error::ProductDeactivated)));
}

#[test]
fn test_deactivated_product_remains_readable() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    // Add some events before deactivation
    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    let event_id = client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"),
        &h,
        &String::from_str(&env, "Harvested"),
        &metadata,
    );

    client.deactivate_product(&owner, &id, &String::from_str(&env, "Lifecycle complete"));

    // Product is still readable
    let p = client.get_product(&id);
    assert_eq!(p.id, id);
    assert!(!p.active);

    // Past events are still readable
    let ev = client.get_event(&event_id);
    assert_eq!(ev.event_id, event_id);

    // Event history is still accessible
    let events = client.get_product_events(&id, &0, &10);
    assert_eq!(events.total_count, 1);
}

#[test]
fn test_reactivated_product_can_receive_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(&owner, &id, &String::from_str(&env, "Suspended"));
    client.reactivate_product(&owner, &id);

    // New events should be accepted again
    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    let event_id = client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("RESUME"),
        &String::from_str(&env, "Warehouse"),
        &h,
        &String::from_str(&env, "Shipment resumed"),
        &metadata,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event.event_id, event_id);
    assert_eq!(event.event_type, symbol_short!("RESUME"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEACTIVATION USE CASE SCENARIOS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_product_reaches_final_destination() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let farmer = Address::generate(&env);
    let distributor = Address::generate(&env);
    let id = register_test_product(&env, &client, &farmer);

    client.add_authorized_actor(&farmer, &id, &distributor);

    // Normal journey
    let h = BytesN::from_array(&env, &[0u8; 32]);
    let meta: Map<Symbol, String> = Map::new(&env);
    client.add_tracking_event(
        &farmer, &id, &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"), &h,
        &String::from_str(&env, "Picked at peak"), &meta,
    );
    client.add_tracking_event(
        &distributor, &id, &symbol_short!("SHIP"),
        &String::from_str(&env, "Port"), &h,
        &String::from_str(&env, "Shipped to Hamburg"), &meta,
    );
    client.add_tracking_event(
        &distributor, &id, &symbol_short!("RECEIVE"),
        &String::from_str(&env, "Hamburg Warehouse"), &h,
        &String::from_str(&env, "Received in good condition"), &meta,
    );

    // Deactivate — lifecycle complete
    client.deactivate_product(
        &farmer,
        &id,
        &String::from_str(&env, "Product delivered to final destination"),
    );

    let p = client.get_product(&id);
    assert!(!p.active);
    let info = p.deactivation_info.get_unchecked(0);
    assert_eq!(
        info.reason,
        String::from_str(&env, "Product delivered to final destination")
    );

    // All 3 events are still intact
    let page = client.get_product_events(&id, &0, &10);
    assert_eq!(page.total_count, 3);
}

#[test]
fn test_product_recalled_scenario() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let manufacturer = Address::generate(&env);
    let id = register_test_product(&env, &client, &manufacturer);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let mut meta = Map::new(&env);
    meta.set(
        Symbol::new(&env, "batch"),
        String::from_str(&env, "LOT-A-2024"),
    );
    client.add_tracking_event(
        &manufacturer, &id, &Symbol::new(&env, "MANUFACTURE"),
        &String::from_str(&env, "Factory"), &h,
        &String::from_str(&env, "Batch produced"), &meta,
    );

    // Recall detected — deactivate with reason
    client.deactivate_product(
        &manufacturer,
        &id,
        &String::from_str(&env, "RECALL: contamination detected in batch LOT-A-2024"),
    );

    let p = client.get_product(&id);
    assert!(!p.active);
    assert!(p.deactivation_info.get_unchecked(0).reason.len() > 0);

    // Cannot add further events on a recalled product
    let res = client.try_add_tracking_event(
        &manufacturer, &id, &symbol_short!("SHIP"),
        &String::from_str(&env, "Port"), &h,
        &String::from_str(&env, ""), &Map::new(&env),
    );
    assert_eq!(res, Err(Ok(Error::ProductDeactivated)));
}

#[test]
fn test_multiple_products_stats_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);

    for suffix in ["A", "B", "C"] {
        let id = String::from_str(&env, &["PROD-", suffix].concat());
        let config = ProductConfig {
            id,
            name: String::from_str(&env, "Product"),
            description: String::from_str(&env, ""),
            origin_location: String::from_str(&env, "Origin"),
            category: String::from_str(&env, "Category"),
            tags: Vec::new(&env),
            certifications: Vec::new(&env),
            media_hashes: Vec::new(&env),
            custom: Map::new(&env),
        };
        client.register_product(&owner, &config);
    }

    let stats = client.get_stats();
    assert_eq!(stats.total_products, 3);
    assert_eq!(stats.active_products, 3);

    // Deactivate 2 products
    client.deactivate_product(
        &owner,
        &String::from_str(&env, "PROD-A"),
        &String::from_str(&env, "Delivered"),
    );
    client.deactivate_product(
        &owner,
        &String::from_str(&env, "PROD-B"),
        &String::from_str(&env, "Recalled"),
    );

    let stats = client.get_stats();
    assert_eq!(stats.total_products, 3, "total includes inactive products");
    assert_eq!(stats.active_products, 1, "only 1 active remaining");

    // Reactivate one
    client.reactivate_product(&owner, &String::from_str(&env, "PROD-B"));

    let stats = client.get_stats();
    assert_eq!(stats.total_products, 3);
    assert_eq!(stats.active_products, 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTHORIZATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_authorized_actor_can_add_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let processor = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.add_authorized_actor(&owner, &id, &processor);
    assert!(client.is_authorized(&id, &processor));

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    let event_id = client.add_tracking_event(
        &processor,
        &id,
        &symbol_short!("PROCESS"),
        &String::from_str(&env, "Processing Mill"),
        &h,
        &String::from_str(&env, "Washed and dried"),
        &metadata,
    );

    let ev = client.get_event(&event_id);
    assert_eq!(ev.actor, processor);
}

#[test]
fn test_unauthorized_actor_cannot_add_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    let res = client.try_add_tracking_event(
        &attacker,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_non_owner_cannot_add_authorized_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let res = client.try_add_authorized_actor(&non_owner, &id, &actor);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_remove_authorized_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.add_authorized_actor(&owner, &id, &actor);
    assert!(client.is_authorized(&id, &actor));

    client.remove_authorized_actor(&owner, &id, &actor);
    assert!(!client.is_authorized(&id, &actor));
}

// ═══════════════════════════════════════════════════════════════════════════════
// OWNERSHIP TRANSFER TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_transfer_product_ownership() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.transfer_product(&owner, &id, &new_owner);

    let p = client.get_product(&id);
    assert_eq!(p.owner, new_owner);

    // Old owner loses authorization
    assert!(!client.is_authorized(&id, &owner));
    // New owner gains authorization
    assert!(client.is_authorized(&id, &new_owner));
}

#[test]
fn test_old_owner_cannot_add_actors_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.transfer_product(&owner, &id, &new_owner);

    let res = client.try_add_authorized_actor(&owner, &id, &actor);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_authorized_actors_persist_across_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.add_authorized_actor(&owner, &id, &actor);
    client.transfer_product(&owner, &id, &new_owner);

    // Third-party actor's authorization is preserved
    assert!(client.is_authorized(&id, &actor));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT QUERY TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_add_tracking_event_with_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let mut metadata: Map<Symbol, String> = Map::new(&env);
    metadata.set(Symbol::new(&env, "temperature"), String::from_str(&env, "22.5"));
    metadata.set(Symbol::new(&env, "humidity"), String::from_str(&env, "65"));
    metadata.set(Symbol::new(&env, "batch"), String::from_str(&env, "B2024-001"));

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let event_id = client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Yirgacheffe Farm"),
        &h,
        &String::from_str(&env, "Coffee harvested at peak ripeness"),
        &metadata,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event.event_id, event_id);
    assert_eq!(event.product_id, id);
    assert_eq!(event.actor, owner);
    assert_eq!(event.event_type, symbol_short!("HARVEST"));
    assert_eq!(event.location, String::from_str(&env, "Yirgacheffe Farm"));
    assert_eq!(
        event.metadata.get(Symbol::new(&env, "temperature")),
        Some(String::from_str(&env, "22.5"))
    );
}

#[test]
fn test_event_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    for _ in 0..10 {
        client.add_tracking_event(
            &owner,
            &id,
            &symbol_short!("SHIP"),
            &String::from_str(&env, "Port"),
            &h,
            &String::from_str(&env, ""),
            &metadata,
        );
    }

    let page1 = client.get_product_events(&id, &0, &5);
    assert_eq!(page1.events.len(), 5);
    assert!(page1.has_more);
    assert_eq!(page1.total_count, 10);

    let page2 = client.get_product_events(&id, &5, &5);
    assert_eq!(page2.events.len(), 5);
    assert!(!page2.has_more);
    assert_eq!(page2.total_count, 10);

    let page3 = client.get_product_events(&id, &20, &5);
    assert_eq!(page3.events.len(), 0);
    assert!(!page3.has_more);
}

#[test]
fn test_filter_events_by_type() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    for _ in 0..3 {
        client.add_tracking_event(
            &owner, &id, &symbol_short!("HARVEST"),
            &String::from_str(&env, "Farm"), &h,
            &String::from_str(&env, ""), &metadata,
        );
    }
    for _ in 0..2 {
        client.add_tracking_event(
            &owner, &id, &symbol_short!("SHIP"),
            &String::from_str(&env, "Port"), &h,
            &String::from_str(&env, ""), &metadata,
        );
    }

    let harvest = client.get_events_by_type(&id, &symbol_short!("HARVEST"), &0, &10);
    assert_eq!(harvest.total_count, 3);
    assert_eq!(harvest.events.len(), 3);

    let ship = client.get_events_by_type(&id, &symbol_short!("SHIP"), &0, &10);
    assert_eq!(ship.total_count, 2);

    let process = client.get_events_by_type(&id, &symbol_short!("PROCESS"), &0, &10);
    assert_eq!(process.total_count, 0);
}

#[test]
fn test_filter_events_by_time_range() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    let current_time = env.ledger().timestamp();

    client.add_tracking_event(
        &owner, &id, &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"), &h,
        &String::from_str(&env, ""), &metadata,
    );
    client.add_tracking_event(
        &owner, &id, &symbol_short!("SHIP"),
        &String::from_str(&env, "Port"), &h,
        &String::from_str(&env, ""), &metadata,
    );
    client.add_tracking_event(
        &owner, &id, &symbol_short!("RECEIVE"),
        &String::from_str(&env, "Warehouse"), &h,
        &String::from_str(&env, ""), &metadata,
    );

    let events = client.get_events_by_time_range(&id, &0, &(current_time + 1000), &0, &10);
    assert_eq!(events.total_count, 3);
}

#[test]
fn test_flexible_filter_by_location() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    client.add_tracking_event(
        &owner, &id, &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm A"), &h,
        &String::from_str(&env, ""), &metadata,
    );
    client.add_tracking_event(
        &owner, &id, &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm B"), &h,
        &String::from_str(&env, ""), &metadata,
    );
    client.add_tracking_event(
        &owner, &id, &symbol_short!("PROCESS"),
        &String::from_str(&env, "Mill"), &h,
        &String::from_str(&env, ""), &metadata,
    );

    // Filter by location "Farm A"
    let filter = TrackingEventFilter {
        event_type: Symbol::new(&env, ""),
        start_time: 0,
        end_time: u64::MAX,
        location: String::from_str(&env, "Farm A"),
    };
    let events = client.get_filtered_events(&id, &filter, &0, &10);
    assert_eq!(events.total_count, 1);
    assert_eq!(
        events.events.get_unchecked(0).location,
        String::from_str(&env, "Farm A")
    );

    // Filter by type "HARVEST"
    let filter = TrackingEventFilter {
        event_type: symbol_short!("HARVEST"),
        start_time: 0,
        end_time: u64::MAX,
        location: String::from_str(&env, ""),
    };
    let events = client.get_filtered_events(&id, &filter, &0, &10);
    assert_eq!(events.total_count, 2);
}

#[test]
fn test_event_count_functions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0u8; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    for _ in 0..5 {
        client.add_tracking_event(
            &owner, &id, &symbol_short!("HARVEST"),
            &String::from_str(&env, "Farm"), &h,
            &String::from_str(&env, ""), &metadata,
        );
    }
    for _ in 0..3 {
        client.add_tracking_event(
            &owner, &id, &symbol_short!("SHIP"),
            &String::from_str(&env, "Port"), &h,
            &String::from_str(&env, ""), &metadata,
        );
    }

    assert_eq!(client.get_event_count(&id), 8);
    assert_eq!(client.get_event_count_by_type(&id, &symbol_short!("HARVEST")), 5);
    assert_eq!(client.get_event_count_by_type(&id, &symbol_short!("SHIP")), 3);
    assert_eq!(client.get_event_count_by_type(&id, &symbol_short!("PROCESS")), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPLETE END-TO-END SCENARIO
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_coffee_supply_chain_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let farmer = Address::generate(&env);
    let processor = Address::generate(&env);
    let shipper = Address::generate(&env);
    let id = register_test_product(&env, &client, &farmer);

    client.add_authorized_actor(&farmer, &id, &processor);
    client.add_authorized_actor(&farmer, &id, &shipper);

    let h = BytesN::from_array(&env, &[0u8; 32]);

    // Journey: harvest → process → quality → package → ship
    let mut meta = Map::new(&env);
    meta.set(Symbol::new(&env, "gps"), String::from_str(&env, "6.5244,38.4356"));
    client.add_tracking_event(
        &farmer, &id, &Symbol::new(&env, "HARVEST"),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &h, &String::from_str(&env, "Hand-picked at peak"), &meta,
    );

    let mut meta = Map::new(&env);
    meta.set(Symbol::new(&env, "method"), String::from_str(&env, "Washed"));
    client.add_tracking_event(
        &processor, &id, &Symbol::new(&env, "PROCESS"),
        &String::from_str(&env, "Addis Mill"),
        &h, &String::from_str(&env, "Fermented 24h"), &meta,
    );

    let mut meta = Map::new(&env);
    meta.set(Symbol::new(&env, "carrier"), String::from_str(&env, "Maersk"));
    client.add_tracking_event(
        &shipper, &id, &Symbol::new(&env, "SHIP"),
        &String::from_str(&env, "Port of Djibouti"),
        &h, &String::from_str(&env, "Departed for Hamburg"), &meta,
    );

    // Product arrives — deactivate lifecycle
    client.deactivate_product(
        &farmer,
        &id,
        &String::from_str(&env, "Coffee delivered to roaster in Hamburg"),
    );

    // Verify final state
    let p = client.get_product(&id);
    assert!(!p.active);
    assert_eq!(
        p.deactivation_info.get_unchecked(0).reason,
        String::from_str(&env, "Coffee delivered to roaster in Hamburg")
    );

    // All events preserved
    let events = client.get_product_events(&id, &0, &10);
    assert_eq!(events.total_count, 3);

    // Statistics correct
    let stats = client.get_stats();
    assert_eq!(stats.total_products, 1);
    assert_eq!(stats.active_products, 0);
}

#[test]
fn test_pharma_cold_chain_with_recall() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    let id = String::from_str(&env, "VACCINE-2024-001");
    let config = ProductConfig {
        id: id.clone(),
        name: String::from_str(&env, "COVID-19 Vaccine Batch A"),
        description: String::from_str(&env, "mRNA vaccine requiring cold chain"),
        origin_location: String::from_str(&env, "Pfizer Manufacturing, Belgium"),
        category: String::from_str(&env, "Pharmaceutical"),
        tags: Vec::new(&env),
        certifications: Vec::new(&env),
        media_hashes: Vec::new(&env),
        custom: Map::new(&env),
    };
    client.register_product(&manufacturer, &config);

    client.add_authorized_actor(&manufacturer, &id, &distributor);

    let h = BytesN::from_array(&env, &[0u8; 32]);

    // Add temperature checks
    let temps = ["-75.0", "-74.5", "-75.2", "-74.8"];
    for temp in temps {
        let mut meta = Map::new(&env);
        meta.set(Symbol::new(&env, "temperature_c"), String::from_str(&env, temp));
        client.add_tracking_event(
            &manufacturer, &id, &Symbol::new(&env, "TEMP_CHECK"),
            &String::from_str(&env, "Cold Storage A"),
            &h, &String::from_str(&env, "Auto log"), &meta,
        );
    }

    // Temperature breach detected — recall and deactivate
    client.deactivate_product(
        &manufacturer,
        &id,
        &String::from_str(&env, "RECALL: temperature breach detected, batch compromised"),
    );

    let p = client.get_product(&id);
    assert!(!p.active);

    // Type query still works post-deactivation
    let temp_events = client.get_events_by_type(&id, &Symbol::new(&env, "TEMP_CHECK"), &0, &10);
    assert_eq!(temp_events.total_count, 4);

    // Distributor cannot add events to recalled product
    let res = client.try_add_tracking_event(
        &distributor, &id, &symbol_short!("SHIP"),
        &String::from_str(&env, "Port"), &h,
        &String::from_str(&env, ""), &Map::new(&env),
    );
    assert_eq!(res, Err(Ok(Error::ProductDeactivated)));
}

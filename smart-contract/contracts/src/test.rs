#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    Address, Env, Map, String, Symbol, Vec,
};

use crate::{
    ProductRegistryContract, ProductRegistryContractClient,
    Error, ProductConfig,
};

// ─── Test helpers ─────────────────────────────────────────────────────────────

fn setup(env: &Env) -> ProductRegistryContractClient {
    let contract_id = env.register_contract(None, ProductRegistryContract);
    ProductRegistryContractClient::new(env, &contract_id)
}

fn register_test_product(
    env: &Env,
    client: &ProductRegistryContractClient,
    owner: &Address,
) -> String {
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
    let client = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let p = client.get_product(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.owner, owner);
    assert!(p.active, "new products must be active");
    assert!(p.deactivation_info.is_empty(), "no deactivation info on new product");
}

#[test]
fn test_register_increments_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

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
    let client = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

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
    let client = setup(&env);

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

#[test]
fn test_register_rejects_empty_origin() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

    let owner = Address::generate(&env);
    let config = ProductConfig {
        id: String::from_str(&env, "ID-001"),
        name: String::from_str(&env, "Name"),
        description: String::from_str(&env, ""),
        origin_location: String::from_str(&env, ""),
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
    let client = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "Reached final destination"),
    );

    let p = client.get_product(&id);
    assert!(!p.active, "product should be inactive after deactivation");

    let info = p.deactivation_info.get_unchecked(0);
    assert_eq!(info.reason, String::from_str(&env, "Reached final destination"));
    assert_eq!(info.deactivated_by, owner);
}

#[test]
fn test_deactivation_updates_active_counter() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

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
    assert_eq!(stats.total_products, 1);
    assert_eq!(stats.active_products, 0);
}

#[test]
fn test_non_owner_cannot_deactivate() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

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
    let client = setup(&env);

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
    let client = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let res = client.try_deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, ""),
    );
    assert_eq!(res, Err(Ok(Error::DeactivationReasonRequired)));
}

#[test]
fn test_deactivate_already_inactive_product() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    client.deactivate_product(
        &owner,
        &id,
        &String::from_str(&env, "First deactivation"),
    );

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
    let client = setup(&env);

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
    let client = setup(&env);

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
    let client = setup(&env);

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
    let client = setup(&env);

    let owner = Address::generate(&env);
    let id = register_test_product(&env, &client, &owner);

    let res = client.try_reactivate_product(&owner, &id);
    assert_eq!(res, Err(Ok(Error::ProductAlreadyActive)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// MULTI-PRODUCT STATS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_products_stats_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

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

    client.reactivate_product(&owner, &String::from_str(&env, "PROD-B"));

    let stats = client.get_stats();
    assert_eq!(stats.total_products, 3);
    assert_eq!(stats.active_products, 2);
}

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String, Vec};

use crate::integrity_anchor::{IntegrityAnchorContract, IntegrityAnchorContractClient, MAX_CONTENT_SIZE};
use crate::{
    AuthorizationContract, AuthorizationContractClient, ProductConfig,
    ProductRegistryContract, ProductRegistryContractClient,
};
use crate::types::StorageScheme;

fn setup_registry(env: &Env) -> (ProductRegistryContractClient<'_>, Address) {
    let auth_id = env.register_contract(None, AuthorizationContract);
    let registry_id = env.register_contract(None, ProductRegistryContract);
    let registry = ProductRegistryContractClient::new(env, &registry_id);
    let auth_client = AuthorizationContractClient::new(env, &auth_id);
    auth_client.configure_initializer(&registry_id);
    registry.configure_auth_contract(&auth_id);
    (registry, registry_id)
}

fn register_test_product(
    env: &Env,
    registry: &ProductRegistryContractClient,
    owner: &Address,
    product_id: &str,
) {
    let config = ProductConfig {
        id: String::from_str(env, product_id),
        name: String::from_str(env, "Manual Test Product"),
        description: String::from_str(env, "Product for integrity anchor tests"),
        origin_location: String::from_str(env, "Test Origin"),
        category: String::from_str(env, "manuals"),
        tags: Vec::new(env),
        certifications: Vec::new(env),
        media_hashes: Vec::new(env),
        custom: Map::new(env),
    };
    registry.register_product(owner, &config);
}

#[test]
fn test_anchor_content_and_cas_dedup() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (registry, registry_id) = setup_registry(&env);
    register_test_product(&env, &registry, &admin, "PROD-MANUAL-001");

    let anchor_id = env.register_contract(None, IntegrityAnchorContract);
    let client = IntegrityAnchorContractClient::new(&env, &anchor_id);
    client.init_integrity_anchor(&admin, &registry_id);

    let hash = BytesN::from_array(&env, &[0xAB; 32]);
    let cid = String::from_str(&env, "bafybeigdyrzt5sfp7udm7uhgt24nszaw6u7am6lkryaag3f2ptxt7pudzu");

    let anchor1 = client.anchor_content(
        &admin,
        &String::from_str(&env, "PROD-MANUAL-001"),
        &hash,
        &cid,
        &StorageScheme::Ipfs,
        &1_048_576,
    );

    assert_eq!(anchor1.anchor_id, 1);
    assert_eq!(anchor1.byte_size, 1_048_576);

    let anchor2 = client.anchor_content(
        &admin,
        &String::from_str(&env, "PROD-MANUAL-001"),
        &hash,
        &cid,
        &StorageScheme::Ipfs,
        &1_048_576,
    );
    assert_eq!(anchor2.anchor_id, anchor1.anchor_id);
}

#[test]
fn test_verify_anchor_tamper_detection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (registry, registry_id) = setup_registry(&env);
    register_test_product(&env, &registry, &admin, "PROD-MANUAL-002");

    let anchor_contract = env.register_contract(None, IntegrityAnchorContract);
    let client = IntegrityAnchorContractClient::new(&env, &anchor_contract);
    client.init_integrity_anchor(&admin, &registry_id);

    let original_hash = BytesN::from_array(&env, &[0x01; 32]);
    let tampered_hash = BytesN::from_array(&env, &[0x02; 32]);
    let cid = String::from_str(&env, "QmTestCid123456789");

    let anchor = client.anchor_content(
        &admin,
        &String::from_str(&env, "PROD-MANUAL-002"),
        &original_hash,
        &cid,
        &StorageScheme::Arweave,
        &512_000,
    );

    assert!(client.verify_anchor(&anchor.anchor_id, &original_hash));
    assert!(!client.verify_anchor(&anchor.anchor_id, &tampered_hash));
}

#[test]
fn test_rejects_oversized_content() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (registry, registry_id) = setup_registry(&env);
    register_test_product(&env, &registry, &admin, "PROD-MANUAL-003");

    let anchor_contract = env.register_contract(None, IntegrityAnchorContract);
    let client = IntegrityAnchorContractClient::new(&env, &anchor_contract);
    client.init_integrity_anchor(&admin, &registry_id);

    let hash = BytesN::from_array(&env, &[0xFF; 32]);
    let cid = String::from_str(&env, "QmOversized");
    let result = client.try_anchor_content(
        &admin,
        &String::from_str(&env, "PROD-MANUAL-003"),
        &hash,
        &cid,
        &StorageScheme::Ipfs,
        &(MAX_CONTENT_SIZE + 1),
    );

    assert!(result.is_err());
}

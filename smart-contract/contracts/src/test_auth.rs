#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec, Map, Symbol, symbol_short, BytesN};
use crate::{
    ChainLogisticsContract, ChainLogisticsContractClient, 
    AuthorizationContract, AuthorizationContractClient,
    Error, ProductConfig,
};

fn setup(env: &Env) -> (ChainLogisticsContractClient, AuthorizationContractClient, Address) {
    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    
    let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
    let auth_client = AuthorizationContractClient::new(env, &auth_id);
    
    let admin = Address::generate(env);
    // Initialize ChainLogisticsContract with the address of AuthorizationContract
    cl_client.init(&admin, &auth_id);
    
    (cl_client, auth_client, admin)
}

fn create_test_product(env: &Env, client: &ChainLogisticsContractClient, owner: &Address) -> String {
    let id = String::from_str(env, "PROD1");
    client.register_product(
        owner,
        &ProductConfig {
            id: id.clone(),
            name: String::from_str(env, "Test Product"),
            description: String::from_str(env, "Description"),
            origin_location: String::from_str(env, "Origin"),
            category: String::from_str(env, "Category"),
            tags: Vec::new(env),
            certifications: Vec::new(env),
            media_hashes: Vec::new(env),
            custom: Map::new(env),
        },
    );
    id
}

#[test]
fn test_registration_and_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let (cl_client, _auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    
    let stats_before = cl_client.get_stats();
    assert_eq!(stats_before.total_products, 0);
    
    let id = create_test_product(&env, &cl_client, &owner);
    let product = cl_client.get_product(&id);
    
    assert_eq!(product.id, id);
    assert_eq!(product.owner, owner);
    
    let stats_after = cl_client.get_stats();
    assert_eq!(stats_after.total_products, 1);
}

#[test]
fn test_authorization_contract_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (cl_client, auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    
    let product_id = create_test_product(&env, &cl_client, &owner);
    
    // Check initial auth
    assert!(auth_client.is_authorized(&product_id, &owner));
    assert!(!auth_client.is_authorized(&product_id, &actor));
    
    // Grant
    auth_client.add_authorized_actor(&owner, &product_id, &actor);
    assert!(auth_client.is_authorized(&product_id, &actor));
    
    // Revoke
    auth_client.remove_authorized_actor(&owner, &product_id, &actor);
    assert!(!auth_client.is_authorized(&product_id, &actor));
}

#[test]
fn test_cl_contract_uses_shared_auth() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (cl_client, auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    
    let product_id = create_test_product(&env, &cl_client, &owner);
    
    let event_type = symbol_short!("HARVEST");
    let location = String::from_str(&env, "Farm A");
    let data_hash = BytesN::from_array(&env, &[0u8; 32]);
    let note = String::from_str(&env, "Initial harvest");
    let metadata = Map::new(&env);
    
    // 1. Unauthorized actor fails to add event
    let res = cl_client.try_add_tracking_event(
        &actor, &product_id, &event_type, &location, &data_hash, &note, &metadata
    );
    assert!(res.is_err());
    
    // 2. Grant auth via AuthorizationContract
    auth_client.add_authorized_actor(&owner, &product_id, &actor);
    
    // 3. Authorized actor succeeds
    let event_id = cl_client.add_tracking_event(
        &actor, &product_id, &event_type, &location, &data_hash, &note, &metadata
    );
    assert_eq!(event_id, 1);
}

#[test]
fn test_product_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (cl_client, _auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    
    let id = create_test_product(&env, &cl_client, &owner);
    
    // Deactivate
    cl_client.deactivate_product(&owner, &id, &String::from_str(&env, "Testing"));
    let p = cl_client.get_product(&id);
    assert!(!p.active);
    
    // Reactivate
    cl_client.reactivate_product(&owner, &id);
    let p = cl_client.get_product(&id);
    assert!(p.active);
}

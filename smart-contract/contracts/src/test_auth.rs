#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec, Map, Symbol, symbol_short, BytesN};
use crate::{
    ChainLogisticsContract, ChainLogisticsContractClient, 
    ProductRegistryContract, ProductRegistryContractClient,
    AuthorizationContract, AuthorizationContractClient,
    Error, ProductConfig,
};

fn setup(env: &Env) -> (ChainLogisticsContractClient, ProductRegistryContractClient, AuthorizationContractClient, Address) {
    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let pr_id = env.register_contract(None, ProductRegistryContract);
    
    let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
    let pr_client = ProductRegistryContractClient::new(env, &pr_id);
    let auth_client = AuthorizationContractClient::new(env, &auth_id);
    
    let admin = Address::generate(env);
    cl_client.init(&admin, &auth_id);
    
    (cl_client, pr_client, auth_client, admin)
}

fn create_test_product(env: &Env, pr_client: &ProductRegistryContractClient, owner: &Address) -> String {
    let id = String::from_str(env, "PROD1");
    pr_client.register_product(
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
    let (_cl_client, pr_client, _auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    
    let stats_before = pr_client.get_stats();
    assert_eq!(stats_before.total_products, 0);
    
    let id = create_test_product(&env, &pr_client, &owner);
    let product = pr_client.get_product(&id);
    
    assert_eq!(product.id, id);
    assert_eq!(product.owner, owner);
    
    let stats_after = pr_client.get_stats();
    assert_eq!(stats_after.total_products, 1);
}

#[test]
fn test_authorization_contract_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_cl_client, pr_client, auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    
    let product_id = create_test_product(&env, &pr_client, &owner);
    
    // Check initial auth — owner is authorized via ProductRegistryContract's set_auth call
    // Note: auth_client is a separate contract, so owner init happens through CL init_product_owner
    // For this test we directly test the AuthorizationContract
    auth_client.init_product_owner(&product_id, &owner);
    
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
fn test_product_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (_cl_client, pr_client, _auth_client, _admin) = setup(&env);
    let owner = Address::generate(&env);
    
    let id = create_test_product(&env, &pr_client, &owner);
    
    // Deactivate
    pr_client.deactivate_product(&owner, &id, &String::from_str(&env, "Testing"));
    let p = pr_client.get_product(&id);
    assert!(!p.active);
    
    // Reactivate
    pr_client.reactivate_product(&owner, &id);
    let p = pr_client.get_product(&id);
    assert!(p.active);
}

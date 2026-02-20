#![cfg(test)]

use crate::{ChainLogisticsContract, ChainLogisticsContractClient, Error};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_register_and_get_product() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Product 1 Metadata");

    let product_id = client.register_product(&owner, &origin, &metadata);
    assert_eq!(product_id, 1);

    let product = client.get_product(&1).unwrap();
    assert_eq!(product.id, 1);
    assert_eq!(product.owner, owner);
    assert_eq!(product.origin, origin);
    assert_eq!(product.metadata, metadata);
    assert!(product.active);
}

#[test]
fn test_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let origin = String::from_str(&env, "USA");
    let metadata = String::from_str(&env, "Metadata");

    for _ in 0..10 {
        client.register_product(&owner, &origin, &metadata);
    }

    let page1 = client.get_all_products(&0, &5);
    assert_eq!(page1.len(), 5);
    assert_eq!(page1.get(0).unwrap().id, 1);
    assert_eq!(page1.get(4).unwrap().id, 5);

    let page2 = client.get_all_products(&5, &5);
    assert_eq!(page2.len(), 5);
    assert_eq!(page2.get(0).unwrap().id, 6);
    assert_eq!(page2.get(4).unwrap().id, 10);
}

#[test]
fn test_filtering() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let origin1 = String::from_str(&env, "China");
    let origin2 = String::from_str(&env, "Germany");

    client.register_product(&owner1, &origin1, &String::from_str(&env, "P1")); // ID 1
    client.register_product(&owner2, &origin2, &String::from_str(&env, "P2")); // ID 2
    client.register_product(&owner1, &origin2, &String::from_str(&env, "P3")); // ID 3

    let owner1_products = client.get_products_by_owner(&owner1, &0, &10);
    assert_eq!(owner1_products.len(), 2);
    assert_eq!(owner1_products.get(0).unwrap().id, 1);
    assert_eq!(owner1_products.get(1).unwrap().id, 3);

    let origin2_products = client.get_products_by_origin(&origin2, &0, &10);
    assert_eq!(origin2_products.len(), 2);
    assert_eq!(origin2_products.get(0).unwrap().id, 2);
    assert_eq!(origin2_products.get(1).unwrap().id, 3);
}

#[test]
fn test_stats() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);
    
    let owner = Address::generate(&env);
    let origin = String::from_str(&env, "A");
    
    client.register_product(&owner, &origin, &String::from_str(&env, "M"));
    client.register_product(&owner, &origin, &String::from_str(&env, "M"));
    
    let stats = client.get_stats();
    assert_eq!(stats.total_products, 2);
    assert_eq!(stats.active_products, 2);
}

#[test]
fn test_add_authorized_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.add_authorized_actor(&owner, &id, &actor);
    assert!(client.is_authorized(&id, &actor));
    assert!(!client.is_authorized(&id, &Address::generate(&env)));
}

#[test]
fn test_non_owner_cannot_add_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    // Call add_authorized_actor, passing non_owner as the required caller.
    // The implementation should verify: `if product.owner != caller.owner`
    let result = client.try_add_authorized_actor(&non_owner, &id, &actor);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_remove_authorized_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.add_authorized_actor(&owner, &id, &actor);
    assert!(client.is_authorized(&id, &actor));

    client.remove_authorized_actor(&owner, &id, &actor);
    assert!(!client.is_authorized(&id, &actor));
}

#[test]
fn test_transfer_ownership() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.transfer_product(&owner, &id, &new_owner);

    let p = client.get_product(&id).unwrap();
    assert_eq!(p.owner, new_owner);

    // Old owner shouldn't be authorized anymore
    assert!(!client.is_authorized(&id, &owner));
    
    // Old owner shouldn't be able to add actors
    let actor = Address::generate(&env);
    let result = client.try_add_authorized_actor(&owner, &id, &actor);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_ownership_preserves_authorized_actors() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.add_authorized_actor(&owner, &id, &actor);
    client.transfer_product(&owner, &id, &new_owner);

    // Actor should still be authorized
    assert!(client.is_authorized(&id, &actor));
}

#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env, Map, String, Vec};

fn create_test_product(
    env: &Env,
    client: &ChainLogisticsContractClient,
    owner: &Address,
) -> String {
    let id = String::from_str(env, "PROD-01");
    let name = String::from_str(env, "Test Product");
    let desc = String::from_str(env, "Description");
    let loc = String::from_str(env, "Origin");
    let cat = String::from_str(env, "Category");
    
    client.register_product(
        owner,
        &id,
        &name,
        &desc,
        &loc,
        &cat,
        &Vec::new(env),
        &Vec::new(env),
        &Vec::new(env),
        &Map::new(env),
    );
    id
}

#[test]
fn test_register_and_get_product() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = create_test_product(&env, &client, &owner);

    let p = client.get_product(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.owner, owner);
    assert!(p.active);
}

#[test]
fn test_add_authorized_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = create_test_product(&env, &client, &owner);

    client.add_authorized_actor(&owner, &id, &actor);

    assert!(client.is_authorized(&id, &actor));
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
    let id = create_test_product(&env, &client, &owner);

    // mock_all_auths() makes all auth checks pass, so we can't test "NotAuthorized" easily 
    // without granular mocking, BUT explicit checks in code `if owner != caller` 
    // will still run if we call with a different address.
    // However, `register_product` requires owner auth.
    // `add_authorized_actor` takes `owner` arg and calls `require_owner`.
    
    // Upstream `add_authorized_actor(env, owner, ...)` takes explicit owner arg.
    // It calls `require_owner(&product, &owner)`.
    // Then checks `product.owner != owner`.
    
    // So if we pass `non_owner` as the `owner` argument:
    // It will check `if product.owner != non_owner` -> returns Unauthorized.
    
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
    let id = create_test_product(&env, &client, &owner);

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
    let id = create_test_product(&env, &client, &owner);

    client.transfer_product(&owner, &id, &new_owner);

    let p = client.get_product(&id);
    assert_eq!(p.owner, new_owner);

    // Old owner should no longer be able to add actors (Authorized failure)
    let actor = Address::generate(&env);
    let result = client.try_add_authorized_actor(&owner, &id, &actor);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_authorized_actor_can_add_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = create_test_product(&env, &client, &owner);

    client.add_authorized_actor(&owner, &id, &actor);

    let data = BytesN::from_array(&env, &[0; 32]);
    let note = String::from_str(&env, "Event Note");
    
    // Upstream `add_tracking_event(env, actor, ...)`
    let result = client.try_add_tracking_event(
        &actor, 
        &id, 
        &symbol_short!("EVENT"), 
        &data, 
        &note
    );
    assert!(result.is_ok());
}

#[test]
fn test_unauthorized_actor_cannot_add_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let random = Address::generate(&env);
    let id = create_test_product(&env, &client, &owner);

    let data = BytesN::from_array(&env, &[0; 32]);
    let note = String::from_str(&env, "Event Note");

    let result = client.try_add_tracking_event(
        &random, 
        &id, 
        &symbol_short!("EVENT"), 
        &data, 
        &note
    );
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
    let id = create_test_product(&env, &client, &owner);

    client.add_authorized_actor(&owner, &id, &actor);
    client.transfer_product(&owner, &id, &new_owner);

    // Actor should still be authorized
    assert!(client.is_authorized(&id, &actor));
}

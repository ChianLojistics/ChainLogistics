#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, SupplyChainContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SupplyChainContract);
    let client = SupplyChainContractClient::new(&env, &contract_id);
    (env, client)
}

fn create_test_product(env: &Env, client: &SupplyChainContractClient, owner: &Address) -> u64 {
    let name = String::from_str(env, "Test Product");
    client.create_product(owner, &name)
}

// ─── Test 1 ───────────────────────────────────────────────────────────────────

#[test]
fn test_add_authorized_actor() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);
    client.add_authorized_actor(&product_id, &owner, &actor);

    let product = client.get_product(&product_id);
    assert!(product.authorized_actors.contains(&actor));
}

// ─── Test 2 ───────────────────────────────────────────────────────────────────

#[test]
fn test_add_duplicate_actor_fails() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);
    client.add_authorized_actor(&product_id, &owner, &actor);

    let result = client.try_add_authorized_actor(&product_id, &owner, &actor);
    assert_eq!(result, Err(Ok(Error::AlreadyAuthorized)));
}

// ─── Test 3 ───────────────────────────────────────────────────────────────────

#[test]
fn test_non_owner_cannot_add_actor() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);

    let result = client.try_add_authorized_actor(&product_id, &non_owner, &actor);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ─── Test 4 ───────────────────────────────────────────────────────────────────

#[test]
fn test_remove_authorized_actor() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);
    client.add_authorized_actor(&product_id, &owner, &actor);
    client.remove_authorized_actor(&product_id, &owner, &actor);

    let product = client.get_product(&product_id);
    assert!(!product.authorized_actors.contains(&actor));
}

// ─── Test 5 ───────────────────────────────────────────────────────────────────

#[test]
fn test_owner_cannot_remove_self() {
    let (env, client) = setup();
    let owner = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);

    let result = client.try_remove_authorized_actor(&product_id, &owner, &owner);
    assert_eq!(result, Err(Ok(Error::CannotRemoveSelf)));
}

// ─── Test 6 ───────────────────────────────────────────────────────────────────

#[test]
fn test_remove_nonexistent_actor_fails() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);

    let result = client.try_remove_authorized_actor(&product_id, &owner, &actor);
    assert_eq!(result, Err(Ok(Error::ActorNotFound)));
}

// ─── Test 7 ───────────────────────────────────────────────────────────────────

#[test]
fn test_transfer_ownership() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env); // to verify authorized_actors preserved

    let product_id = create_test_product(&env, &client, &owner);
    client.add_authorized_actor(&product_id, &owner, &actor);
    client.transfer_ownership(&product_id, &owner, &new_owner);

    let product = client.get_product(&product_id);
    assert_eq!(product.owner, new_owner);

    // Old owner is no longer the owner — attempts owner-only ops must fail
    let result = client.try_add_authorized_actor(&product_id, &owner, &Address::generate(&env));
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ─── Test 8 ───────────────────────────────────────────────────────────────────

#[test]
fn test_authorized_actor_can_add_event() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);
    client.add_authorized_actor(&product_id, &owner, &actor);

    let desc = String::from_str(&env, "Shipped from warehouse");
    let result = client.try_add_event(&product_id, &actor, &desc);
    assert!(result.is_ok());
}

// ─── Test 9 ───────────────────────────────────────────────────────────────────

#[test]
fn test_unauthorized_actor_cannot_add_event() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let random = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);

    let desc = String::from_str(&env, "Unauthorized event");
    let result = client.try_add_event(&product_id, &random, &desc);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ─── Test 10 ──────────────────────────────────────────────────────────────────

#[test]
fn test_ownership_preserves_authorized_actors() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let actor1 = Address::generate(&env);
    let actor2 = Address::generate(&env);
    let new_owner = Address::generate(&env);

    let product_id = create_test_product(&env, &client, &owner);
    client.add_authorized_actor(&product_id, &owner, &actor1);
    client.add_authorized_actor(&product_id, &owner, &actor2);
    client.transfer_ownership(&product_id, &owner, &new_owner);

    let product = client.get_product(&product_id);
    assert!(product.authorized_actors.contains(&actor1));
    assert!(product.authorized_actors.contains(&actor2));
}

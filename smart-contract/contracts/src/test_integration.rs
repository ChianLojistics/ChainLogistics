use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

use crate::{
    AuthorizationContract, AuthorizationContractClient, Error, ProductConfig,
    ProductRegistryContract, ProductRegistryContractClient, ProductTransferContract,
    ProductTransferContractClient,
};

fn setup(
    env: &Env,
) -> (
    ProductRegistryContractClient<'_>,
    AuthorizationContractClient<'_>,
    ProductTransferContractClient<'_>,
) {
    let auth_id = env.register_contract(None, AuthorizationContract);
    let registry_id = env.register_contract(None, ProductRegistryContract);
    let transfer_id = env.register_contract(None, ProductTransferContract);

    let auth_client = AuthorizationContractClient::new(env, &auth_id);
    let registry_client = ProductRegistryContractClient::new(env, &registry_id);
    let transfer_client = ProductTransferContractClient::new(env, &transfer_id);

    auth_client.configure_initializer(&registry_id);
    registry_client.configure_auth_contract(&auth_id);
    transfer_client.pt_init(&registry_id, &auth_id);

    (registry_client, auth_client, transfer_client)
}

fn register_product(
    env: &Env,
    registry: &ProductRegistryContractClient,
    owner: &Address,
    id: &str,
) -> String {
    let product_id = String::from_str(env, id);
    registry.register_product(
        owner,
        &ProductConfig {
            id: product_id.clone(),
            name: String::from_str(env, "Integration Product"),
            description: String::from_str(env, "integration path"),
            origin_location: String::from_str(env, "Accra"),
            category: String::from_str(env, "Coffee"),
            tags: Vec::new(env),
            certifications: Vec::new(env),
            media_hashes: Vec::new(env),
            custom: Map::new(env),
        },
    );
    product_id
}

#[test]
fn test_transfer_updates_registry_and_authorization_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (registry, auth, transfer) = setup(&env);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = register_product(&env, &registry, &owner, "INTEG-001");

    auth.add_authorized_actor(&owner, &id, &actor);
    assert!(auth.is_authorized(&id, &owner));
    assert!(auth.is_authorized(&id, &actor));

    transfer.transfer_product(&owner, &id, &new_owner);

    let p = registry.get_product(&id);
    assert_eq!(p.owner, new_owner);

    assert!(!auth.is_authorized(&id, &owner));
    assert!(auth.is_authorized(&id, &new_owner));

    let old_owner_add = auth.try_add_authorized_actor(&owner, &id, &actor);
    assert_eq!(old_owner_add, Err(Ok(Error::Unauthorized)));

    auth.add_authorized_actor(&new_owner, &id, &actor);
    assert!(auth.is_authorized(&id, &actor));
}

#[test]
fn test_deactivated_product_blocks_transfer_until_reactivated() {
    let env = Env::default();
    env.mock_all_auths();
    let (registry, _auth, transfer) = setup(&env);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let id = register_product(&env, &registry, &owner, "INTEG-002");

    registry.deactivate_product(&owner, &id, &String::from_str(&env, "finalized"));

    let blocked = transfer.try_transfer_product(&owner, &id, &new_owner);
    assert_eq!(blocked, Err(Ok(Error::ProductDeactivated)));

    registry.reactivate_product(&owner, &id);
    transfer.transfer_product(&owner, &id, &new_owner);

    let p = registry.get_product(&id);
    assert_eq!(p.owner, new_owner);
}

#[test]
fn test_batch_transfer_mixed_owned_unowned_and_missing_ids() {
    let env = Env::default();
    env.mock_all_auths();
    let (registry, _auth, transfer) = setup(&env);

    let owner = Address::generate(&env);
    let other_owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    let own_a = register_product(&env, &registry, &owner, "INTEG-B1");
    let own_b = register_product(&env, &registry, &owner, "INTEG-B2");
    let foreign = register_product(&env, &registry, &other_owner, "INTEG-B3");

    let mut batch = Vec::new(&env);
    batch.push_back(own_a.clone());
    batch.push_back(String::from_str(&env, "INTEG-NOTFOUND"));
    batch.push_back(foreign.clone());
    batch.push_back(own_b.clone());

    let moved = transfer.batch_transfer_products(&owner, &batch, &new_owner);
    assert_eq!(moved, 2);

    assert_eq!(registry.get_product(&own_a).owner, new_owner);
    assert_eq!(registry.get_product(&own_b).owner, new_owner);
    assert_eq!(registry.get_product(&foreign).owner, other_owner);
}

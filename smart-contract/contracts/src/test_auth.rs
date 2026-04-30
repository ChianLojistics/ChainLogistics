use crate::{
    AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
    ChainLogisticsContractClient, Error, ProductConfig, ProductRegistryContract,
    ProductRegistryContractClient,
};
use soroban_sdk::{
    symbol_short, testutils::Address as _, Address, BytesN, Env, Map, String, Symbol, Vec,
};

fn setup(
    env: &Env,
) -> (
    ChainLogisticsContractClient<'_>,
    ProductRegistryContractClient<'_>,
    AuthorizationContractClient<'_>,
    Address,
    Address,
    Address,
) {
    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let pr_id = env.register_contract(None, ProductRegistryContract);

    let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
    let pr_client = ProductRegistryContractClient::new(env, &pr_id);
    let auth_client = AuthorizationContractClient::new(env, &auth_id);

    let admin = Address::generate(env);
    cl_client.init(&admin, &auth_id);

    auth_client.configure_initializer(&pr_id);
    pr_client.configure_auth_contract(&auth_id);

    (cl_client, pr_client, auth_client, admin, pr_id, auth_id)
}

fn create_test_product(
    env: &Env,
    pr_client: &ProductRegistryContractClient,
    owner: &Address,
) -> String {
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
    let (_cl_client, pr_client, _auth_client, _admin, _pr_id, _auth_id) = setup(&env);
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

    let (_cl_client, pr_client, auth_client, _admin, pr_id, _auth_id) = setup(&env);
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let product_id = create_test_product(&env, &pr_client, &owner);

    // Check initial auth — owner is authorized via ProductRegistryContract's set_auth call
    // Note: auth_client is a separate contract, so owner init happens through CL init_product_owner
    // For this test we directly test the AuthorizationContract
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
    let (_cl_client, pr_client, _auth_client, _admin, _pr_id, _auth_id) = setup(&env);
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

#[test]
fn test_auth_initializer_not_configured_rejects_init_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let caller = Address::generate(&env);
    let owner = Address::generate(&env);
    let product_id = String::from_str(&env, "PROD-NOINIT");

    let res = auth_client.try_init_product_owner(&caller, &product_id, &owner);
    assert_eq!(res, Err(Ok(Error::NotInitialized)));
}

#[test]
fn test_auth_configure_initializer_rejects_different_second_initializer() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let first = Address::generate(&env);
    let second = Address::generate(&env);

    auth_client.configure_initializer(&first);
    let res = auth_client.try_configure_initializer(&second);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_auth_init_product_owner_rejects_duplicate_id() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let initializer = Address::generate(&env);
    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let product_id = String::from_str(&env, "PROD-DUPE");

    auth_client.configure_initializer(&initializer);
    auth_client.init_product_owner(&initializer, &product_id, &owner_a);

    let res = auth_client.try_init_product_owner(&initializer, &product_id, &owner_b);
    assert_eq!(res, Err(Ok(Error::ProductAlreadyExists)));
}

#[test]
fn test_auth_update_owner_requires_current_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let initializer = Address::generate(&env);
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let product_id = String::from_str(&env, "PROD-UPD");

    auth_client.configure_initializer(&initializer);
    auth_client.init_product_owner(&initializer, &product_id, &owner);

    let res = auth_client.try_update_product_owner(&attacker, &product_id, &new_owner);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_auth_add_remove_actor_nonexistent_product() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let missing = String::from_str(&env, "PROD-MISSING");

    let add_res = auth_client.try_add_authorized_actor(&owner, &missing, &actor);
    assert_eq!(add_res, Err(Ok(Error::ProductNotFound)));

    let remove_res = auth_client.try_remove_authorized_actor(&owner, &missing, &actor);
    assert_eq!(remove_res, Err(Ok(Error::ProductNotFound)));
}

#[test]
fn test_auth_old_owner_loses_actor_management_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let initializer = Address::generate(&env);
    let old_owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let product_id = String::from_str(&env, "PROD-XFER-AUTH");

    auth_client.configure_initializer(&initializer);
    auth_client.init_product_owner(&initializer, &product_id, &old_owner);
    auth_client.update_product_owner(&old_owner, &product_id, &new_owner);

    let old_owner_res = auth_client.try_add_authorized_actor(&old_owner, &product_id, &actor);
    assert_eq!(old_owner_res, Err(Ok(Error::Unauthorized)));

    auth_client.add_authorized_actor(&new_owner, &product_id, &actor);
    assert!(auth_client.is_authorized(&product_id, &actor));
}

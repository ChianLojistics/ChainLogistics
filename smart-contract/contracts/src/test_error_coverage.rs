use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol, Vec};

use crate::{
    AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
    ChainLogisticsContractClient, Error, Origin, Product,
};

#[test]
fn test_chainlogistics_init_and_pause_error_paths() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let cl_client = ChainLogisticsContractClient::new(&env, &cl_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    cl_client.init(&admin, &auth_id);

    let second_init = cl_client.try_init(&admin, &auth_id);
    assert_eq!(second_init, Err(Ok(Error::AlreadyInitialized)));

    let unauthorized_pause = cl_client.try_pause(&attacker);
    assert_eq!(unauthorized_pause, Err(Ok(Error::Unauthorized)));

    cl_client.pause(&admin);

    let double_pause = cl_client.try_pause(&admin);
    assert_eq!(double_pause, Err(Ok(Error::ContractPaused)));

    cl_client.unpause(&admin);

    let double_unpause = cl_client.try_unpause(&admin);
    assert_eq!(double_unpause, Err(Ok(Error::ContractNotPaused)));
}

#[test]
fn test_chainlogistics_missing_product_and_event_errors() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let cl_client = ChainLogisticsContractClient::new(&env, &cl_id);

    let admin = Address::generate(&env);
    cl_client.init(&admin, &auth_id);

    let missing_product = String::from_str(&env, "MISSING-PROD");
    let missing_event_ids = cl_client.try_get_product_event_ids(&missing_product);
    assert_eq!(missing_event_ids, Err(Ok(Error::ProductNotFound)));

    let missing_count = cl_client.try_get_product_events(&missing_product, &0u64, &10u64);
    assert_eq!(missing_count, Err(Ok(Error::ProductNotFound)));

    let missing_event = cl_client.try_get_event(&999u64);
    assert_eq!(missing_event, Err(Ok(Error::EventNotFound)));
}

#[test]
fn test_authorization_core_error_paths() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let initializer = Address::generate(&env);
    let bad_initializer = Address::generate(&env);
    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let product_id = String::from_str(&env, "AUTH-ERR-001");
    let missing_id = String::from_str(&env, "AUTH-ERR-404");

    let uninitialized_owner_set =
        auth_client.try_init_product_owner(&initializer, &product_id, &owner);
    assert_eq!(uninitialized_owner_set, Err(Ok(Error::NotInitialized)));

    auth_client.configure_initializer(&initializer);

    let wrong_initializer_owner_set =
        auth_client.try_init_product_owner(&bad_initializer, &product_id, &owner);
    assert_eq!(wrong_initializer_owner_set, Err(Ok(Error::Unauthorized)));

    auth_client.init_product_owner(&initializer, &product_id, &owner);

    let duplicate_owner_set = auth_client.try_init_product_owner(&initializer, &product_id, &owner);
    assert_eq!(duplicate_owner_set, Err(Ok(Error::ProductAlreadyExists)));

    let missing_auth_check = auth_client.try_is_authorized(&missing_id, &actor);
    assert_eq!(missing_auth_check, Err(Ok(Error::ProductNotFound)));

    let missing_remove = auth_client.try_remove_authorized_actor(&owner, &missing_id, &actor);
    assert_eq!(missing_remove, Err(Ok(Error::ProductNotFound)));
}

#[test]
fn test_pause_contract_rejects_tracking_calls() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let cl_client = ChainLogisticsContractClient::new(&env, &cl_id);
    let auth_client = AuthorizationContractClient::new(&env, &auth_id);

    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    let product_id = String::from_str(&env, "P-NO-STORAGE");

    cl_client.init(&admin, &auth_id);
    auth_client.configure_initializer(&cl_id);
    auth_client.init_product_owner(&cl_id, &product_id, &actor);

    cl_client.pause(&admin);

    let res = cl_client.try_add_tracking_event(
        &actor,
        &product_id,
        &Symbol::new(&env, "SHIP"),
        &String::from_str(&env, "Lagos"),
        &soroban_sdk::BytesN::from_array(&env, &[0; 32]),
        &String::from_str(&env, "paused check"),
        &soroban_sdk::Map::new(&env),
    );
    assert_eq!(res, Err(Ok(Error::ContractPaused)));
}

#[test]
fn test_product_event_ids_pagination_and_count() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);

    let cl_client = ChainLogisticsContractClient::new(&env, &cl_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let product_id = String::from_str(&env, "P-PAGED-001");

    cl_client.init(&admin, &auth_id);

    env.as_contract(&cl_id, || {
        crate::storage::put_product(
            &env,
            &Product {
                id: product_id.clone(),
                name: String::from_str(&env, "Paged Product"),
                description: String::from_str(&env, "pagination coverage"),
                origin: Origin {
                    location: String::from_str(&env, "Accra"),
                },
                owner: owner.clone(),
                created_at: env.ledger().timestamp(),
                active: true,
                category: String::from_str(&env, "Coffee"),
                tags: Vec::new(&env),
                certifications: Vec::new(&env),
                media_hashes: Vec::new(&env),
                custom: soroban_sdk::Map::new(&env),
                deactivation_info: Vec::new(&env),
            },
        );

        let mut ids = Vec::new(&env);
        for i in 1..=120u64 {
            ids.push_back(i);
        }
        crate::storage::put_product_event_ids(&env, &product_id, &ids);
    });

    let count = cl_client.get_product_event_count(&product_id);
    assert_eq!(count, 120);

    let all_ids = cl_client.get_product_event_ids(&product_id);
    assert_eq!(all_ids.len(), 120);
    assert_eq!(all_ids.get(0), Some(1));
    assert_eq!(all_ids.get(119), Some(120));

    let first_page = cl_client.get_product_event_ids_paginated(&product_id, &0, &20);
    assert_eq!(first_page.len(), 20);
    assert_eq!(first_page.get(0), Some(1));
    assert_eq!(first_page.get(19), Some(20));

    let second_page = cl_client.get_product_event_ids_paginated(&product_id, &20, &20);
    assert_eq!(second_page.len(), 20);
    assert_eq!(second_page.get(0), Some(21));
    assert_eq!(second_page.get(19), Some(40));

    let capped_page = cl_client.get_product_event_ids_paginated(&product_id, &0, &500);
    assert_eq!(capped_page.len(), 100);
    assert_eq!(capped_page.get(0), Some(1));
    assert_eq!(capped_page.get(99), Some(100));

    let zero_limit = cl_client.get_product_event_ids_paginated(&product_id, &0, &0);
    assert_eq!(zero_limit.len(), 0);

    let out_of_range = cl_client.get_product_event_ids_paginated(&product_id, &120, &10);
    assert_eq!(out_of_range.len(), 0);
}

#[test]
fn test_product_event_ids_pagination_and_count_missing_product() {
    let env = Env::default();
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let cl_client = ChainLogisticsContractClient::new(&env, &cl_id);

    let admin = Address::generate(&env);
    let missing_product = String::from_str(&env, "MISSING-PAGED");

    cl_client.init(&admin, &auth_id);

    let paged = cl_client.try_get_product_event_ids_paginated(&missing_product, &0, &10);
    assert_eq!(paged, Err(Ok(Error::ProductNotFound)));

    let count = cl_client.try_get_product_event_count(&missing_product);
    assert_eq!(count, Err(Ok(Error::ProductNotFound)));
}

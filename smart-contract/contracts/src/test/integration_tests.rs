use soroban_sdk::{log, testutils::Address as _, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{
    AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
    ChainLogisticsContractClient, Error, EventQueryContract, EventQueryContractClient,
    ProductConfig, ProductRegistryContract, ProductRegistryContractClient, ProductTransferContract,
    ProductTransferContractClient, TrackingContract, TrackingContractClient,
};

fn setup(
    env: &Env,
) -> (
    ChainLogisticsContractClient,
    Address,
    ProductRegistryContractClient,
    AuthorizationContractClient,
    ProductTransferContractClient,
    TrackingContractClient,
    EventQueryContractClient,
) {
    env.mock_all_auths();

    let auth_id = env.register_contract(None, AuthorizationContract);
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let registry_id = env.register_contract(None, ProductRegistryContract);
    let transfer_id = env.register_contract(None, ProductTransferContract);
    let tracking_id = env.register_contract(None, TrackingContract);
    let query_id = env.register_contract(None, EventQueryContract);

    let auth = AuthorizationContractClient::new(env, &auth_id);
    let cl = ChainLogisticsContractClient::new(env, &cl_id);
    let registry = ProductRegistryContractClient::new(env, &registry_id);
    let transfer = ProductTransferContractClient::new(env, &transfer_id);
    let tracking = TrackingContractClient::new(env, &tracking_id);
    let query = EventQueryContractClient::new(env, &query_id);

    auth.configure_initializer(&registry_id);
    let admin = Address::generate(env);
    cl.init(&admin, &auth_id);
    registry.configure_auth_contract(&auth_id);
    transfer.pt_init(&registry_id, &auth_id);
    tracking.init(&cl_id);
    query.init(&registry_id, &tracking_id);

    (cl, admin, registry, auth, transfer, tracking, query)
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

fn add_tracking_event(
    env: &Env,
    tracking: &TrackingContractClient,
    actor: &Address,
    product_id: &String,
    event_type: &Symbol,
    location: &String,
    note: &String,
) -> u64 {
    tracking.tracking_add_event(
        actor,
        product_id,
        event_type,
        location,
        &BytesN::from_array(env, &[0; 32]),
        note,
        &Map::new(env),
    )
}

fn reset_and_print_budget(env: &Env, label: &str) {
    env.budget().reset_tracker();
    log!(env, "BUDGET_START {}", label);
}

fn print_budget(env: &Env, label: &str) {
    let cpu = env.budget().cpu_instruction_cost();
    let mem = env.budget().memory_bytes_cost();
    log!(
        env,
        "BUDGET_END {} cpu_instruction_cost={} memory_bytes_cost={}",
        label,
        cpu,
        mem
    );
    env.budget().print();
}

#[test]
fn e2e_product_registration_flow() {
    let env = Env::default();
    let (_cl, _admin, registry, auth, _transfer, _tracking, _query) = setup(&env);

    let owner = Address::generate(&env);

    reset_and_print_budget(&env, "register_product");
    let id = register_product(&env, &registry, &owner, "INTEG-REG-001");
    print_budget(&env, "register_product");

    let p = registry.get_product(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.owner, owner);
    assert!(p.active);

    assert!(auth.is_authorized(&id, &owner));
}

#[test]
fn multi_actor_event_tracking_flow() {
    let env = Env::default();
    let (_cl, _admin, registry, auth, _transfer, tracking, query) = setup(&env);

    let owner = Address::generate(&env);
    let actor_a = Address::generate(&env);
    let actor_b = Address::generate(&env);

    let id = register_product(&env, &registry, &owner, "INTEG-TRK-001");

    auth.add_authorized_actor(&owner, &id, &actor_a);
    auth.add_authorized_actor(&owner, &id, &actor_b);

    let shipped = Symbol::new(&env, "shipped");
    let received = Symbol::new(&env, "received");

    reset_and_print_budget(&env, "tracking_add_event_owner");
    let e1 = add_tracking_event(
        &env,
        &tracking,
        &owner,
        &id,
        &shipped,
        &String::from_str(&env, "Warehouse A"),
        &String::from_str(&env, "Packed"),
    );
    print_budget(&env, "tracking_add_event_owner");

    reset_and_print_budget(&env, "tracking_add_event_actor_a");
    let e2 = add_tracking_event(
        &env,
        &tracking,
        &actor_a,
        &id,
        &shipped,
        &String::from_str(&env, "Port"),
        &String::from_str(&env, "Departed"),
    );
    print_budget(&env, "tracking_add_event_actor_a");

    reset_and_print_budget(&env, "tracking_add_event_actor_b");
    let e3 = add_tracking_event(
        &env,
        &tracking,
        &actor_b,
        &id,
        &received,
        &String::from_str(&env, "Destination"),
        &String::from_str(&env, "Arrived"),
    );
    print_budget(&env, "tracking_add_event_actor_b");

    assert_eq!(e1, 1);
    assert_eq!(e2, 2);
    assert_eq!(e3, 3);

    let page = query.query_get_product_events(&id, &0u64, &10u64);
    assert_eq!(page.total_count, 3);
    assert_eq!(page.events.len(), 3);

    let first = page.events.get_unchecked(0);
    let last = page.events.get_unchecked(2);
    assert_eq!(first.product_id, id);
    assert_eq!(last.product_id, id);
}

#[test]
fn ownership_transfer_flow() {
    let env = Env::default();
    let (_cl, _admin, registry, auth, transfer, _tracking, _query) = setup(&env);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);

    let id = register_product(&env, &registry, &owner, "INTEG-XFER-001");
    auth.add_authorized_actor(&owner, &id, &actor);
    assert!(auth.is_authorized(&id, &actor));

    reset_and_print_budget(&env, "transfer_product");
    transfer.transfer_product(&owner, &id, &new_owner);
    print_budget(&env, "transfer_product");

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
fn error_cases_integration() {
    let env = Env::default();
    let (cl, admin, registry, auth, transfer, tracking, query) = setup(&env);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let new_owner = Address::generate(&env);

    let id = register_product(&env, &registry, &owner, "INTEG-ERR-001");

    let res = transfer.try_transfer_product(&attacker, &id, &new_owner);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));

    let page = query.query_get_product_events(&id, &0u64, &10u64);
    assert_eq!(page.total_count, 0);

    registry.deactivate_product(&owner, &id, &String::from_str(&env, "finalized"));
    let blocked = transfer.try_transfer_product(&owner, &id, &new_owner);
    assert_eq!(blocked, Err(Ok(Error::ProductDeactivated)));

    let missing =
        query.try_query_get_product_events(&String::from_str(&env, "MISSING"), &0u64, &10u64);
    assert_eq!(missing, Err(Ok(Error::ProductNotFound)));

    let unauthorized = auth.try_add_authorized_actor(&attacker, &id, &attacker);
    assert_eq!(unauthorized, Err(Ok(Error::Unauthorized)));

    let not_found = tracking.try_tracking_get_event(&999);
    assert_eq!(not_found, Err(Ok(Error::EventNotFound)));

    // Pause (via ChainLogisticsContract) should block tracking adds.
    cl.pause(&admin);

    let res = tracking.try_tracking_add_event(
        &owner,
        &id,
        &Symbol::new(&env, "shipped"),
        &String::from_str(&env, "X"),
        &BytesN::from_array(&env, &[0; 32]),
        &String::from_str(&env, "Y"),
        &Map::new(&env),
    );
    assert_eq!(res, Err(Ok(Error::ContractPaused)));
}

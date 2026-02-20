#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env, Map, String, Symbol, Vec};

fn default_register_args(env: &Env) -> (Vec<String>, Vec<BytesN<32>>, Vec<BytesN<32>>, Map<Symbol, String>) {
    (Vec::new(env), Vec::new(env), Vec::new(env), Map::new(env))
}

fn add_event(client: &ChainLogisticsContractClient, env: &Env, actor: &Address, product_id: &str, t: Symbol) -> u64 {
    let h = BytesN::from_array(env, &[0; 32]);
    client.add_tracking_event(
        actor,
        &String::from_str(env, product_id),
        &t,
        &h,
        &String::from_str(env, ""),
    )
}

fn id_for_i(i: u32) -> &'static str {
    match i {
        0 => "P-0",
        1 => "P-1",
        2 => "P-2",
        3 => "P-3",
        4 => "P-4",
        5 => "P-5",
        6 => "P-6",
        7 => "P-7",
        8 => "P-8",
        9 => "P-9",
        _ => "P-X",
    }
}

fn register_one(client: &ChainLogisticsContractClient, env: &Env, owner: &Address, id: &str) {
    let (tags, certs, media, custom) = default_register_args(env);
    let _ = client.register_product(
        owner,
        &String::from_str(env, id),
        &String::from_str(env, "Name"),
        &String::from_str(env, ""),
        &String::from_str(env, "Origin"),
        &String::from_str(env, "Category"),
        &tags,
        &certs,
        &media,
        &custom,
    );
}

#[test]
fn test_register_and_get_product() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let id = String::from_str(&env, "COFFEE-ETH-001");
    let (tags, certs, media, custom) = default_register_args(&env);

    let created = client.register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );

    assert_eq!(created.id, id);
    assert_eq!(created.owner, owner);
    assert!(created.active);

    let product = client.get_product(&id);
    assert_eq!(product.id, id);
    assert_eq!(product.owner, owner);
    assert!(product.active);
}

#[test]
fn test_duplicate_product_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    register_one(&client, &env, &owner, "COFFEE-ETH-001");

    let (tags, certs, media, custom) = default_register_args(&env);
    let res = client.try_register_product(
        &owner,
        &String::from_str(&env, "COFFEE-ETH-001"),
        &String::from_str(&env, "Duplicate"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Somewhere"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::ProductAlreadyExists),
        _ => panic!("expected ProductAlreadyExists"),
    }
}

#[test]
fn test_register_products_batch_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let (tags, certs, media, custom) = default_register_args(&env);

    let mut inputs: Vec<ProductRegistrationInput> = Vec::new(&env);
    for i in 0..10u32 {
        let id = String::from_str(&env, id_for_i(i));
        inputs.push_back(ProductRegistrationInput {
            id,
            name: String::from_str(&env, "Name"),
            description: String::from_str(&env, ""),
            origin_location: String::from_str(&env, "Origin"),
            category: String::from_str(&env, "Category"),
            tags: tags.clone(),
            certifications: certs.clone(),
            media_hashes: media.clone(),
            custom: custom.clone(),
        });
    }

    let res = client.register_products_batch(&owner, &inputs);
    assert_eq!(res.len(), 10);
    let p0 = client.get_product(&String::from_str(&env, "P-0"));
    assert_eq!(p0.owner, owner);
}

#[test]
fn test_register_products_batch_atomic_failure() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let (tags, certs, media, custom) = default_register_args(&env);

    let mut inputs: Vec<ProductRegistrationInput> = Vec::new(&env);
    inputs.push_back(ProductRegistrationInput {
        id: String::from_str(&env, "OK"),
        name: String::from_str(&env, "Name"),
        description: String::from_str(&env, ""),
        origin_location: String::from_str(&env, "Origin"),
        category: String::from_str(&env, "Category"),
        tags: tags.clone(),
        certifications: certs.clone(),
        media_hashes: media.clone(),
        custom: custom.clone(),
    });
    inputs.push_back(ProductRegistrationInput {
        id: String::from_str(&env, ""),
        name: String::from_str(&env, "Name"),
        description: String::from_str(&env, ""),
        origin_location: String::from_str(&env, "Origin"),
        category: String::from_str(&env, "Category"),
        tags: tags.clone(),
        certifications: certs.clone(),
        media_hashes: media.clone(),
        custom: custom.clone(),
    });

    let res = client.try_register_products_batch(&owner, &inputs);
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidProductId),
        _ => panic!("expected InvalidProductId"),
    }

    // Atomic: first product must not have been stored.
    let p = client.try_get_product(&String::from_str(&env, "OK"));
    assert!(p.is_err());
}

#[test]
fn test_add_tracking_events_batch_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);

    register_one(&client, &env, &owner, "P1");
    client.add_authorized_actor(&owner, &String::from_str(&env, "P1"), &actor);

    let h = BytesN::from_array(&env, &[0; 32]);
    let mut inputs: Vec<TrackingEventInput> = Vec::new(&env);
    for _ in 0..5u32 {
        inputs.push_back(TrackingEventInput {
            product_id: String::from_str(&env, "P1"),
            event_type: symbol_short!("PROC"),
            data_hash: h.clone(),
            note: String::from_str(&env, ""),
        });
    }

    let ids = client.add_tracking_events_batch(&actor, &inputs);
    assert_eq!(ids.len(), 5);

    let stored = client.get_product_event_ids(&String::from_str(&env, "P1"));
    assert_eq!(stored.len(), 5);
}

#[test]
fn test_event_query_indexes_and_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor1 = Address::generate(&env);
    let actor2 = Address::generate(&env);

    register_one(&client, &env, &owner, "P1");
    client.add_authorized_actor(&owner, &String::from_str(&env, "P1"), &actor1);
    client.add_authorized_actor(&owner, &String::from_str(&env, "P1"), &actor2);

    // Add 6 events: PROC, SHIP alternating, actor1 adds 4, actor2 adds 2
    let _e0 = add_event(&client, &env, &actor1, "P1", symbol_short!("PROC"));
    let _e1 = add_event(&client, &env, &actor1, "P1", symbol_short!("SHIP"));
    let _e2 = add_event(&client, &env, &actor2, "P1", symbol_short!("PROC"));
    let _e3 = add_event(&client, &env, &actor1, "P1", symbol_short!("SHIP"));
    let _e4 = add_event(&client, &env, &actor2, "P1", symbol_short!("SHIP"));
    let _e5 = add_event(&client, &env, &actor1, "P1", symbol_short!("PROC"));

    // All events paginated
    let p1 = client.get_product_event_ids_page(&String::from_str(&env, "P1"), &0u32, &2u32);
    assert_eq!(p1.ids.len(), 2);
    let p2 = client.get_product_event_ids_page(&String::from_str(&env, "P1"), &p1.next_cursor, &10u32);
    assert_eq!(p2.ids.len(), 4);

    // Recent (reverse) pagination
    let r1 = client.get_product_event_ids_rcnt_page(&String::from_str(&env, "P1"), &0u32, &3u32);
    assert_eq!(r1.ids.len(), 3);
    let r2 = client.get_product_event_ids_rcnt_page(&String::from_str(&env, "P1"), &r1.next_cursor, &10u32);
    assert_eq!(r2.ids.len(), 3);

    // By type
    let proc = client.get_evt_ids_type_page(&String::from_str(&env, "P1"), &symbol_short!("PROC"), &0u32, &10u32);
    assert_eq!(proc.ids.len(), 3);
    let ship = client.get_evt_ids_type_page(&String::from_str(&env, "P1"), &symbol_short!("SHIP"), &0u32, &10u32);
    assert_eq!(ship.ids.len(), 3);

    // By actor
    let a1 = client.get_evt_ids_actr_page(&String::from_str(&env, "P1"), &actor1, &0u32, &10u32);
    assert_eq!(a1.ids.len(), 4);
    let a2 = client.get_evt_ids_actr_page(&String::from_str(&env, "P1"), &actor2, &0u32, &10u32);
    assert_eq!(a2.ids.len(), 2);

    // Date range: since all events share the same ledger timestamp in this test env,
    // querying that exact timestamp should return all events.
    let ts = env.ledger().timestamp();
    let d = client.get_evt_ids_date_page(&String::from_str(&env, "P1"), &ts, &ts, &0u32, &100u32);
    assert_eq!(d.ids.len(), 6);
}

#[test]
fn test_event_query_1000_events_recent_page() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    register_one(&client, &env, &owner, "P1");
    client.add_authorized_actor(&owner, &String::from_str(&env, "P1"), &actor);

    for i in 0..1000u32 {
        let t = if i % 2 == 0 { symbol_short!("PROC") } else { symbol_short!("SHIP") };
        let _ = add_event(&client, &env, &actor, "P1", t);
    }

    let first = client.get_product_event_ids_rcnt_page(&String::from_str(&env, "P1"), &0u32, &50u32);
    assert_eq!(first.ids.len(), 50);
    let second = client.get_product_event_ids_rcnt_page(&String::from_str(&env, "P1"), &first.next_cursor, &50u32);
    assert_eq!(second.ids.len(), 50);

    let proc = client.get_evt_ids_type_page(&String::from_str(&env, "P1"), &symbol_short!("PROC"), &0u32, &1000u32);
    assert_eq!(proc.ids.len(), 500);
}

#[test]
fn test_authorize_add_event_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let farmer = Address::generate(&env);
    let processor = Address::generate(&env);
    let shipper = Address::generate(&env);

    let id = String::from_str(&env, "COFFEE-ETH-001");
    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    client.register_product(
        &farmer,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );

    client.add_authorized_actor(&farmer, &id, &processor);

    let h = BytesN::from_array(&env, &[0; 32]);
    let event_id = client.add_tracking_event(
        &processor,
        &id,
        &symbol_short!("PROC"),
        &h,
        &String::from_str(&env, ""),
    );
    let ids = client.get_product_event_ids(&id);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get_unchecked(0), event_id);

    client.transfer_product(&farmer, &id, &shipper);

    let p = client.get_product(&id);
    assert_eq!(p.owner, shipper);
}

#[test]
fn test_register_rejects_empty_id() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidProductId),
        _ => panic!("expected InvalidProductId"),
    }
}

#[test]
fn test_register_rejects_empty_origin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidOrigin),
        _ => panic!("expected InvalidOrigin"),
    }
}

#[test]
fn test_unauthorized_cannot_add_authorized_actor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let actor = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    client.register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );

    let res = client.try_add_authorized_actor(&attacker, &id, &actor);
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::Unauthorized),
        _ => panic!("expected Unauthorized"),
    }
}

#[test]
fn test_register_rejects_empty_name() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidProductName),
        _ => panic!("expected InvalidProductName"),
    }
}

#[test]
fn test_register_rejects_empty_category() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, ""),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidCategory),
        _ => panic!("expected InvalidCategory"),
    }
}

#[test]
fn test_register_rejects_too_long_description() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let long_desc = "a".repeat(3000);
    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, &long_desc),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::DescriptionTooLong),
        _ => panic!("expected DescriptionTooLong"),
    }
}

#[test]
fn test_register_rejects_too_many_tags() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let mut tags: Vec<String> = Vec::new(&env);
    for _ in 0..21 {
        tags.push_back(String::from_str(&env, "t"));
    }
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::TooManyTags),
        _ => panic!("expected TooManyTags"),
    }
}

#[test]
fn test_register_rejects_tag_too_long() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let mut tags: Vec<String> = Vec::new(&env);
    let long_tag = "t".repeat(100);
    tags.push_back(String::from_str(&env, &long_tag));

    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::TagTooLong),
        _ => panic!("expected TagTooLong"),
    }
}

#[test]
fn test_register_rejects_too_many_custom_fields() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);

    let mut custom: Map<Symbol, String> = Map::new(&env);
    let keys = [
        "k0", "k1", "k2", "k3", "k4", "k5", "k6", "k7", "k8", "k9", "k10",
        "k11", "k12", "k13", "k14", "k15", "k16", "k17", "k18", "k19", "k20",
    ];
    for i in 0..21u32 {
        let k = Symbol::new(&env, keys[i as usize]);
        custom.set(k, String::from_str(&env, "v"));
    }

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::TooManyCustomFields),
        _ => panic!("expected TooManyCustomFields"),
    }
}

#[test]
fn test_product_storage() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let product_id = String::from_str(&env, "TEST-PRODUCT-001");
    let (tags, certs, media, custom) = default_register_args(&env);

    // Register a product
    let created = client.register_product(
        &owner,
        &product_id,
        &String::from_str(&env, "Test Product"),
        &String::from_str(&env, "Test description"),
        &String::from_str(&env, "Test Origin"),
        &String::from_str(&env, "Test Category"),
        &tags,
        &certs,
        &media,
        &custom,
    );

    // Verify product was stored and can be retrieved
    let retrieved = client.get_product(&product_id);
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, created.name);
    assert_eq!(retrieved.owner, owner);
    assert_eq!(retrieved.description, created.description);
    assert_eq!(retrieved.origin.location, created.origin.location);
    assert_eq!(retrieved.category, created.category);
    assert!(retrieved.active);

    // Verify product persists across separate contract calls
    let retrieved_again = client.get_product(&product_id);
    assert_eq!(retrieved_again.id, product_id);
    assert_eq!(retrieved_again.owner, owner);

    // Verify duplicate product ID is rejected
    let duplicate_result = client.try_register_product(
        &owner,
        &product_id,
        &String::from_str(&env, "Duplicate Product"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Another Origin"),
        &String::from_str(&env, "Another Category"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match duplicate_result {
        Err(Ok(e)) => assert_eq!(e, Error::ProductAlreadyExists),
        _ => panic!("expected ProductAlreadyExists error for duplicate ID"),
    }

    // Verify non-existent product returns error
    let non_existent_id = String::from_str(&env, "NON-EXISTENT-001");
    let missing_result = client.try_get_product(&non_existent_id);
    match missing_result {
        Err(Ok(e)) => assert_eq!(e, Error::ProductNotFound),
        _ => panic!("expected ProductNotFound error for missing product"),
    }
}

#[test]
fn test_register_rejects_custom_field_value_too_long() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = String::from_str(&env, "COFFEE-ETH-001");

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);

    let mut custom: Map<Symbol, String> = Map::new(&env);
    let long_val = "v".repeat(600);
    custom.set(Symbol::new(&env, "k"), String::from_str(&env, &long_val));

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Organic Coffee Beans"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &String::from_str(&env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    match res {
        Err(Ok(e)) => assert_eq!(e, Error::CustomFieldValueTooLong),
        _ => panic!("expected CustomFieldValueTooLong"),
    }
}

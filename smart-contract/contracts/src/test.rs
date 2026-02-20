#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env, Map, String, Symbol, Vec};

fn default_register_args(env: &Env) -> (Vec<String>, Vec<BytesN<32>>, Vec<BytesN<32>>, Map<Symbol, String>) {
    (Vec::new(env), Vec::new(env), Vec::new(env), Map::new(env))
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

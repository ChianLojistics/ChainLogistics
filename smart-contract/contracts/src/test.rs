#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, Address, BytesN, Env, Map, String, Symbol, Vec};
use soroban_sdk::testutils::Address as _;

#[test]
fn test_register_and_get_product() {
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

    let p = client.get_product(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.owner, owner);
    assert!(p.active);
}

#[test]
fn test_duplicate_product_rejected() {
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

    let res = client.try_register_product(
        &owner,
        &id,
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

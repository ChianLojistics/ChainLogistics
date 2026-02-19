#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::{symbol_short, Address, BytesN, Env, Map, String, Symbol, Vec};
use soroban_sdk::testutils::Address as _;

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
    let origin = String::from_str(&env, "USA");
    let metadata = String::from_str(&env, "Metadata");

    // Register 10 products
    for _ in 0..10 {
        client.register_product(&owner, &origin, &metadata);
    }

    // Get first 5
    let page1 = client.get_all_products(&0, &5);
    assert_eq!(page1.len(), 5);
    assert_eq!(page1.get(0).unwrap().id, 1);
    assert_eq!(page1.get(4).unwrap().id, 5);

    // Get next 5
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

    // Filter by Owner 1
    let owner1_products = client.get_products_by_owner(&owner1, &0, &10);
    assert_eq!(owner1_products.len(), 2);
    // Note: Order depends on implementation details, but registration order is preserved in our logic
    assert_eq!(owner1_products.get(0).unwrap().id, 1);
    assert_eq!(owner1_products.get(1).unwrap().id, 3);

    // Filter by Origin 2 ("Germany")
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

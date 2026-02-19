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

    client.register_product(
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
        &String::from_str(&env, ""),
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
        &String::from_str(&env, ""),
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
        &String::from_str(&env, ""),
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

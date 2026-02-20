#![cfg(test)]

use crate::{ChainLogisticsContract, ChainLogisticsContractClient, Error};
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::{symbol_short, Address, BytesN, Env, Map, String, Symbol, Vec};
use soroban_sdk::testutils::Address as _;

use crate::*;

fn setup_product(env: &Env, client: &ChainLogisticsContractClient, owner: &Address) -> String {
    let id = String::from_str(env, "COFFEE-ETH-001");
    let tags: Vec<String> = Vec::new(env);
    let certs: Vec<BytesN<32>> = Vec::new(env);
    let media: Vec<BytesN<32>> = Vec::new(env);
    let custom: Map<Symbol, String> = Map::new(env);

    client.register_product(
        owner,
        &id,
        &String::from_str(env, "Organic Coffee Beans"),
        &String::from_str(env, "Premium single-origin coffee"),
        &String::from_str(env, "Yirgacheffe, Ethiopia"),
        &String::from_str(env, "Coffee"),
        &tags,
        &certs,
        &media,
        &custom,
    );
    id
}

#[test]
fn test_register_and_get_product() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    let p = client.get_product(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.owner, owner);
    assert!(p.active);
}

#[test]
fn test_add_tracking_event_with_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    let mut metadata: Map<Symbol, String> = Map::new(&env);
    metadata.set(Symbol::new(&env, "temperature"), String::from_str(&env, "22.5"));
    metadata.set(Symbol::new(&env, "humidity"), String::from_str(&env, "65"));
    metadata.set(Symbol::new(&env, "batch"), String::from_str(&env, "B2024-001"));

    let h = BytesN::from_array(&env, &[0; 32]);
    let event_id = client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Yirgacheffe Farm"),
        &h,
        &String::from_str(&env, "Coffee harvested at peak ripeness"),
        &metadata,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event.event_id, event_id);
    assert_eq!(event.product_id, id);
    assert_eq!(event.actor, owner);
    assert_eq!(event.event_type, symbol_short!("HARVEST"));
    assert_eq!(event.location, String::from_str(&env, "Yirgacheffe Farm"));
    
    let temp = event.metadata.get(Symbol::new(&env, "temperature"));
    assert_eq!(temp, Some(String::from_str(&env, "22.5")));
}

#[test]
fn test_event_pagination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    for _ in 0..10 {
        client.register_product(&owner, &origin, &metadata);
    }

    let page1 = client.get_all_products(&0, &5);
    assert_eq!(page1.len(), 5);
    assert_eq!(page1.get(0).unwrap().id, 1);
    assert_eq!(page1.get(4).unwrap().id, 5);

    let page2 = client.get_all_products(&5, &5);
    assert_eq!(page2.len(), 5);
    assert_eq!(page2.get(0).unwrap().id, 6);
    assert_eq!(page2.get(4).unwrap().id, 10);
    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    
    for i in 0..10 {
        let event_type = match i % 3 {
            0 => symbol_short!("HARVEST"),
            1 => symbol_short!("PROCESS"),
            _ => symbol_short!("SHIP"),
        };
        client.add_tracking_event(
            &owner,
            &id,
            &event_type,
            &String::from_str(&env, "Location"),
            &h,
            &String::from_str(&env, ""),
            &metadata,
        );
    }

    let page1 = client.get_product_events(&id, &0, &5);
    assert_eq!(page1.events.len(), 5);
    assert!(page1.has_more);
    assert_eq!(page1.total_count, 10);

    let page2 = client.get_product_events(&id, &5, &5);
    assert_eq!(page2.events.len(), 5);
    assert!(!page2.has_more);
    assert_eq!(page2.total_count, 10);

    let page3 = client.get_product_events(&id, &20, &5);
    assert_eq!(page3.events.len(), 0);
    assert!(!page3.has_more);
}

#[test]
fn test_filter_events_by_type() {
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

    let owner1_products = client.get_products_by_owner(&owner1, &0, &10);
    assert_eq!(owner1_products.len(), 2);
    assert_eq!(owner1_products.get(0).unwrap().id, 1);
    assert_eq!(owner1_products.get(1).unwrap().id, 3);

    let origin2_products = client.get_products_by_origin(&origin2, &0, &10);
    assert_eq!(origin2_products.len(), 2);
    assert_eq!(origin2_products.get(0).unwrap().id, 2);
    assert_eq!(origin2_products.get(1).unwrap().id, 3);
    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);
    
    for _ in 0..3 {
        client.add_tracking_event(
            &owner,
            &id,
            &symbol_short!("HARVEST"),
            &String::from_str(&env, "Farm"),
            &h,
            &String::from_str(&env, ""),
            &metadata,
        );
    }

    for _ in 0..2 {
        client.add_tracking_event(
            &owner,
            &id,
            &symbol_short!("SHIP"),
            &String::from_str(&env, "Port"),
            &h,
            &String::from_str(&env, ""),
            &metadata,
        );
    }

    let harvest_events = client.get_events_by_type(&id, &symbol_short!("HARVEST"), &0, &10);
    assert_eq!(harvest_events.total_count, 3);
    assert_eq!(harvest_events.events.len(), 3);

    let ship_events = client.get_events_by_type(&id, &symbol_short!("SHIP"), &0, &10);
    assert_eq!(ship_events.total_count, 2);
    assert_eq!(ship_events.events.len(), 2);

    let process_events = client.get_events_by_type(&id, &symbol_short!("PROCESS"), &0, &10);
    assert_eq!(process_events.total_count, 0);
}

#[test]
fn test_filter_events_by_time_range() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    // Add events at the current ledger timestamp
    let current_time = env.ledger().timestamp();
    
    let stats = client.get_stats();
    assert_eq!(stats.total_products, 2);
    assert_eq!(stats.active_products, 2);
}

#[test]
fn test_add_authorized_actor() {
    client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("SHIP"),
        &String::from_str(&env, "Port"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("RECEIVE"),
        &String::from_str(&env, "Warehouse"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    // Get all events (filter with wide time range)
    let events = client.get_events_by_time_range(&id, &0, &(current_time + 1000), &0, &10);
    assert_eq!(events.total_count, 3);
}

#[test]
fn test_flexible_filter() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm A"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm B"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    client.add_tracking_event(
        &owner,
        &id,
        &symbol_short!("PROCESS"),
        &String::from_str(&env, "Mill"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    let filter = EventFilter {
        event_type: Symbol::new(&env, ""),
        start_time: 0,
        end_time: u64::MAX,
        location: String::from_str(&env, "Farm A"),
    };
    let events = client.get_filtered_events(&id, &filter, &0, &10);
    assert_eq!(events.total_count, 1);
    assert_eq!(events.events.get_unchecked(0).location, String::from_str(&env, "Farm A"));

    let filter = EventFilter {
        event_type: symbol_short!("HARVEST"),
        start_time: 0,
        end_time: u64::MAX,
        location: String::from_str(&env, ""),
    };
    let events = client.get_filtered_events(&id, &filter, &0, &10);
    assert_eq!(events.total_count, 2);
}

#[test]
fn test_authorized_actor_can_add_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.add_authorized_actor(&owner, &id, &actor);
    assert!(client.is_authorized(&id, &actor));
    assert!(!client.is_authorized(&id, &Address::generate(&env)));
}

#[test]
fn test_non_owner_cannot_add_actor() {
    let processor = Address::generate(&env);

    let id = setup_product(&env, &client, &owner);
    client.add_authorized_actor(&owner, &id, &processor);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    let event_id = client.add_tracking_event(
        &processor,
        &id,
        &symbol_short!("PROCESS"),
        &String::from_str(&env, "Processing Mill"),
        &h,
        &String::from_str(&env, "Washed and dried"),
        &metadata,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event.actor, processor);
}

#[test]
fn test_unauthorized_actor_cannot_add_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);

    let id = setup_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    let res = client.try_add_tracking_event(
        &attacker,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    match res {
        Err(Ok(e)) => assert_eq!(e, Error::Unauthorized),
        _ => panic!("expected Unauthorized"),
    }
}

#[test]
fn test_inactive_product_cannot_add_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    // Call add_authorized_actor, passing non_owner as the required caller.
    // The implementation should verify: `if product.owner != caller.owner`
    let result = client.try_add_authorized_actor(&non_owner, &id, &actor);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_remove_authorized_actor() {
    let id = setup_product(&env, &client, &owner);

    client.set_product_active(&owner, &id, &false);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    let res = client.try_add_tracking_event(
        &owner,
        &id,
        &symbol_short!("HARVEST"),
        &String::from_str(&env, "Farm"),
        &h,
        &String::from_str(&env, ""),
        &metadata,
    );

    match res {
        Err(Ok(e)) => assert_eq!(e, Error::InvalidInput),
        _ => panic!("expected InvalidInput"),
    }
}

#[test]
fn test_event_count_functions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let id = setup_product(&env, &client, &owner);

    let h = BytesN::from_array(&env, &[0; 32]);
    let metadata: Map<Symbol, String> = Map::new(&env);

    for _ in 0..5 {
        client.add_tracking_event(
            &owner,
            &id,
            &symbol_short!("HARVEST"),
            &String::from_str(&env, "Farm"),
            &h,
            &String::from_str(&env, ""),
            &metadata,
        );
    }

    for _ in 0..3 {
        client.add_tracking_event(
            &owner,
            &id,
            &symbol_short!("SHIP"),
            &String::from_str(&env, "Port"),
            &h,
            &String::from_str(&env, ""),
            &metadata,
        );
    }

    assert_eq!(client.get_event_count(&id), 8);
    assert_eq!(client.get_event_count_by_type(&id, &symbol_short!("HARVEST")), 5);
    assert_eq!(client.get_event_count_by_type(&id, &symbol_short!("SHIP")), 3);
    assert_eq!(client.get_event_count_by_type(&id, &symbol_short!("PROCESS")), 0);
}

#[test]
fn test_coffee_supply_chain_scenario() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.add_authorized_actor(&owner, &id, &actor);
    assert!(client.is_authorized(&id, &actor));

    client.remove_authorized_actor(&owner, &id, &actor);
    assert!(!client.is_authorized(&id, &actor));
}

#[test]
fn test_transfer_ownership() {
    let farmer = Address::generate(&env);
    let processor = Address::generate(&env);
    let shipper = Address::generate(&env);

    let id = setup_product(&env, &client, &farmer);

    client.add_authorized_actor(&farmer, &id, &processor);
    client.add_authorized_actor(&farmer, &id, &shipper);

    let h = BytesN::from_array(&env, &[0; 32]);

    let mut metadata = Map::new(&env);
    metadata.set(Symbol::new(&env, "gps"), String::from_str(&env, "6.5244,38.4356"));
    metadata.set(Symbol::new(&env, "farmer_name"), String::from_str(&env, "Abebe Bekele"));

    client.add_tracking_event(
        &farmer,
        &id,
        &Symbol::new(&env, "HARVEST"),
        &String::from_str(&env, "Yirgacheffe, Ethiopia"),
        &h,
        &String::from_str(&env, "Hand-picked at peak ripeness"),
        &metadata,
    );

    let mut metadata = Map::new(&env);
    metadata.set(Symbol::new(&env, "method"), String::from_str(&env, "Washed"));
    metadata.set(Symbol::new(&env, "grade"), String::from_str(&env, "Grade 1"));

    client.add_tracking_event(
        &processor,
        &id,
        &Symbol::new(&env, "PROCESS"),
        &String::from_str(&env, "Addis Mill"),
        &h,
        &String::from_str(&env, "Fermented for 24 hours"),
        &metadata,
    );

    let mut metadata = Map::new(&env);
    metadata.set(Symbol::new(&env, "score"), String::from_str(&env, "87.5"));
    metadata.set(Symbol::new(&env, "defects"), String::from_str(&env, "0"));

    client.add_tracking_event(
        &processor,
        &id,
        &Symbol::new(&env, "QUALITY"),
        &String::from_str(&env, "Addis Mill QC Lab"),
        &h,
        &String::from_str(&env, "Cupping notes: floral, citrus, chocolate"),
        &metadata,
    );

    let mut metadata = Map::new(&env);
    metadata.set(Symbol::new(&env, "batch"), String::from_str(&env, "B2024-001"));
    metadata.set(Symbol::new(&env, "weight_kg"), String::from_str(&env, "60"));

    client.add_tracking_event(
        &processor,
        &id,
        &Symbol::new(&env, "PACKAGE"),
        &String::from_str(&env, "Addis Export Facility"),
        &h,
        &String::from_str(&env, "Vacuum sealed in GrainPro bags"),
        &metadata,
    );

    let mut metadata = Map::new(&env);
    metadata.set(Symbol::new(&env, "carrier"), String::from_str(&env, "Maersk"));
    metadata.set(Symbol::new(&env, "container"), String::from_str(&env, "MSKU1234567"));

    client.add_tracking_event(
        &shipper,
        &id,
        &Symbol::new(&env, "SHIP"),
        &String::from_str(&env, "Port of Djibouti"),
        &h,
        &String::from_str(&env, "Departed for Hamburg"),
        &metadata,
    );

    let events = client.get_product_events(&id, &0, &10);
    assert_eq!(events.total_count, 5);

    let harvest_events = client.get_events_by_type(&id, &Symbol::new(&env, "HARVEST"), &0, &10);
    assert_eq!(harvest_events.total_count, 1);
    let event = harvest_events.events.get_unchecked(0);
    assert_eq!(event.event_type, Symbol::new(&env, "HARVEST"));
    assert_eq!(event.location, String::from_str(&env, "Yirgacheffe, Ethiopia"));
    let gps = event.metadata.get(Symbol::new(&env, "gps"));
    assert_eq!(gps, Some(String::from_str(&env, "6.5244,38.4356")));
}

#[test]
fn test_pharma_cold_chain_scenario() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let id = String::from_str(&env, "VACCINE-2024-001");
    client.register_product(
        &manufacturer,
        &id,
        &String::from_str(&env, "COVID-19 Vaccine Batch A"),
        &String::from_str(&env, "mRNA vaccine requiring cold chain"),
        &String::from_str(&env, "Pfizer Manufacturing, Belgium"),
        &String::from_str(&env, "Pharmaceutical"),
        &tags,
        &certs,
        &media,
        &custom,
    );

    client.add_authorized_actor(&manufacturer, &id, &distributor);

    let h = BytesN::from_array(&env, &[0; 32]);

    let mut metadata = Map::new(&env);
    metadata.set(Symbol::new(&env, "batch"), String::from_str(&env, "LOT-2024-A-001"));
    metadata.set(Symbol::new(&env, "formula"), String::from_str(&env, "BNT162b2"));
    metadata.set(Symbol::new(&env, "doses"), String::from_str(&env, "10000"));

    client.add_tracking_event(
        &manufacturer,
        &id,
        &Symbol::new(&env, "MANUFACTURE"),
        &String::from_str(&env, "Puurs, Belgium"),
        &h,
        &String::from_str(&env, "Quality controlled batch"),
        &metadata,
    );

    let temps = ["-75.0", "-74.5", "-75.2", "-74.8"];
    let ts_strings = ["1000", "2000", "3000", "4000"];

    for i in 0..4 {
        let mut metadata = Map::new(&env);
        metadata.set(Symbol::new(&env, "temperature_c"), String::from_str(&env, temps[i]));
        metadata.set(Symbol::new(&env, "sensor_id"), String::from_str(&env, "TEMP-001"));
        metadata.set(Symbol::new(&env, "recorded_at"), String::from_str(&env, ts_strings[i]));

        client.add_tracking_event(
            &manufacturer,
            &id,
            &Symbol::new(&env, "TEMP_CHECK"),
            &String::from_str(&env, "Cold Storage A"),
            &h,
            &String::from_str(&env, "Automated temperature log"),
            &metadata,
        );
    }

    let temp_events = client.get_events_by_type(&id, &Symbol::new(&env, "TEMP_CHECK"), &0, &10);
    assert_eq!(temp_events.total_count, 4);

    // Get events using actual ledger timestamp range
    let current_time = env.ledger().timestamp();
    let temp_range = client.get_events_by_time_range(&id, &0, &(current_time + 1000), &0, &10);
    // Time range includes all 5 events (1 MANUFACTURE + 4 TEMP_CHECK)
    assert_eq!(temp_range.total_count, 5);
}

#[test]
fn test_duplicate_product_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.transfer_product(&owner, &id, &new_owner);

    let p = client.get_product(&id).unwrap();
    assert_eq!(p.owner, new_owner);

    // Old owner shouldn't be authorized anymore
    assert!(!client.is_authorized(&id, &owner));
    
    // Old owner shouldn't be able to add actors
    let actor = Address::generate(&env);
    let result = client.try_add_authorized_actor(&owner, &id, &actor);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_ownership_preserves_authorized_actors() {
    let id = setup_product(&env, &client, &owner);

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &id,
        &String::from_str(&env, "Duplicate"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Somewhere"),
        &String::from_str(&env, "Other"),
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
fn test_register_rejects_empty_id() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &String::from_str(&env, ""),
        &String::from_str(&env, "Name"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Origin"),
        &String::from_str(&env, "Category"),
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
    let new_owner = Address::generate(&env);
    let actor = Address::generate(&env);
    let origin = String::from_str(&env, "Nigeria");
    let metadata = String::from_str(&env, "Metadata");
    let id = client.register_product(&owner, &origin, &metadata);

    client.add_authorized_actor(&owner, &id, &actor);
    client.transfer_product(&owner, &id, &new_owner);

    // Actor should still be authorized
    assert!(client.is_authorized(&id, &actor));

    let tags: Vec<String> = Vec::new(&env);
    let certs: Vec<BytesN<32>> = Vec::new(&env);
    let media: Vec<BytesN<32>> = Vec::new(&env);
    let custom: Map<Symbol, String> = Map::new(&env);

    let res = client.try_register_product(
        &owner,
        &String::from_str(&env, "ID-001"),
        &String::from_str(&env, "Name"),
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Category"),
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
fn test_transfer_product() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ChainLogisticsContract);
    let client = ChainLogisticsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    let id = setup_product(&env, &client, &owner);

    client.transfer_product(&owner, &id, &new_owner);

    let p = client.get_product(&id);
    assert_eq!(p.owner, new_owner);
}

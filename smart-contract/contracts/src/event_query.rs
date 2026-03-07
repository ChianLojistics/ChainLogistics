use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{DataKey, TrackingEvent, TrackingEventFilter, TrackingEventPage};
use crate::ChainLogisticsContractClient;

// ─── Storage helpers for EventQueryContract ──────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::MainContract, address);
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct EventQueryContract;

#[contractimpl]
impl EventQueryContract {
    /// Initialize the EventQueryContract with the main contract address.
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_main_contract(&env, &main_contract);
        Ok(())
    }

    /// Get paginated events for a product.
    /// Returns events with pagination info (total_count, has_more).
    pub fn get_product_events(
        env: Env,
        product_id: String,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        // Get all event IDs for the product
        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let total_count = all_ids.len() as u64;

        // Get paginated event IDs
        let event_ids = storage::get_product_event_ids_paginated(&env, &product_id, offset, limit);

        // Fetch actual events
        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let eid = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (event_ids.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events filtered by type with pagination.
    pub fn get_events_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        // Get total count for this type
        let total_count = storage::get_event_count_by_type(&env, &product_id, &event_type);
        
        // Get event IDs by type
        let event_ids = storage::get_event_ids_by_type(&env, &product_id, &event_type, offset, limit);

        // Fetch actual events
        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let eid = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (event_ids.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events within a time range with pagination.
    pub fn get_events_by_time_range(
        env: Env,
        product_id: String,
        start_time: u64,
        end_time: u64,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        // Get all event IDs for the product
        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);

        // Filter by time range
        for i in 0..all_ids.len() {
            let eid = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                if event.timestamp >= start_time && event.timestamp <= end_time {
                    matching_ids.push_back(eid);
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        // Apply pagination
        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        for i in start..end {
            let eid = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (events.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events with composite filtering (type, time range, location).
    /// All filter criteria are optional - use empty values to skip a filter.
    pub fn get_filtered_events(
        env: Env,
        product_id: String,
        filter: TrackingEventFilter,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        // Get all event IDs for the product
        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);

        let empty_sym = Symbol::new(&env, "");
        let empty_loc = String::from_str(&env, "");

        // Apply composite filters
        for i in 0..all_ids.len() {
            let eid = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                let mut matches = true;

                if filter.event_type != empty_sym && event.event_type != filter.event_type {
                    matches = false;
                }
                if filter.start_time > 0 && event.timestamp < filter.start_time {
                    matches = false;
                }
                if filter.end_time < u64::MAX && event.timestamp > filter.end_time {
                    matches = false;
                }
                if filter.location != empty_loc && event.location != filter.location {
                    matches = false;
                }

                if matches {
                    matching_ids.push_back(eid);
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        // Apply pagination
        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        for i in start..end {
            let eid = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (events.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get total event count for a product.
    pub fn get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        let ids = storage::get_product_event_ids(&env, &product_id);
        Ok(ids.len() as u64)
    }

    /// Get event count by type for a product.
    pub fn get_event_count_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
    ) -> Result<u64, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        Ok(storage::get_event_count_by_type(&env, &product_id, &event_type))
    }
}

#[cfg(test)]
mod test_event_query {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map};
    use crate::{
        AuthorizationContract, ChainLogisticsContract, ChainLogisticsContractClient,
        ProductConfig, TrackingContract, TrackingContractClient,
    };

    fn setup(env: &Env) -> (ChainLogisticsContractClient, TrackingContractClient, super::EventQueryContractClient) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let tracking_id = env.register_contract(None, TrackingContract);
        let query_id = env.register_contract(None, super::EventQueryContract);

        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let tracking_client = TrackingContractClient::new(env, &tracking_id);
        let query_client = super::EventQueryContractClient::new(env, &query_id);

        let admin = Address::generate(env);
        cl_client.init(&admin, &auth_id);
        tracking_client.init(&cl_id);
        query_client.init(&cl_id);

        (cl_client, tracking_client, query_client)
    }

    fn register_test_product(
        env: &Env,
        client: &ChainLogisticsContractClient,
        owner: &Address,
        id: &str,
    ) -> String {
        let product_id = String::from_str(env, id);
        client.register_product(
            owner,
            &ProductConfig {
                id: product_id.clone(),
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
        product_id
    }

    fn add_test_event(
        env: &Env,
        tracking_client: &TrackingContractClient,
        owner: &Address,
        product_id: &String,
        event_type: &str,
    ) -> u64 {
        tracking_client.add_tracking_event(
            owner,
            product_id,
            &Symbol::new(env, event_type),
            &String::from_str(env, "Test Location"),
            &BytesN::from_array(env, &[0; 32]),
            &String::from_str(env, "Test note"),
            &Map::new(env),
        )
    }

    #[test]
    fn test_get_product_events_empty() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Get events for product with no events
        let result = query_client.get_product_events(&product_id, &0, &10);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_get_product_events_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _tracking_client, query_client) = setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = query_client.try_get_product_events(&fake_id, &0, &10);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_event_count() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Initially 0 events
        assert_eq!(query_client.get_event_count(&product_id), 0);

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Count should be 2
        assert_eq!(query_client.get_event_count(&product_id), 2);
    }

    #[test]
    fn test_get_event_count_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _tracking_client, query_client) = setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = query_client.try_get_event_count(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_event_count_by_type() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Add events of different types
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Check counts by type
        assert_eq!(
            query_client.get_event_count_by_type(&product_id, &Symbol::new(&env, "created")),
            1
        );
        assert_eq!(
            query_client.get_event_count_by_type(&product_id, &Symbol::new(&env, "shipped")),
            2
        );
        assert_eq!(
            query_client.get_event_count_by_type(&product_id, &Symbol::new(&env, "received")),
            0
        );
    }

    #[test]
    fn test_get_events_by_type() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");
        add_test_event(&env, &tracking_client, &owner, &product_id, "received");

        // Get only shipped events
        let result = query_client.get_events_by_type(&product_id, &Symbol::new(&env, "shipped"), &0, &10);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_get_events_by_time_range() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Get events in time range (all events)
        let result = query_client.get_events_by_time_range(&product_id, &0, &u64::MAX, &0, &10);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_get_filtered_events() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Filter by type
        let filter = TrackingEventFilter {
            event_type: Symbol::new(&env, "created"),
            start_time: 0,
            end_time: u64::MAX,
            location: String::from_str(&env, ""),
        };
        let result = query_client.get_filtered_events(&product_id, &filter, &0, &10);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_pagination() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, tracking_client, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Add 5 events
        for _ in 0..5 {
            add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        }

        // Get first 2
        let result = query_client.get_product_events(&product_id, &0, &2);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.total_count, 5);
        assert!(result.has_more);

        // Get next 2
        let result = query_client.get_product_events(&product_id, &2, &2);
        assert_eq!(result.events.len(), 2);
        assert!(result.has_more);

        // Get last 1
        let result = query_client.get_product_events(&product_id, &4, &2);
        assert_eq!(result.events.len(), 1);
        assert!(!result.has_more);
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _tracking_client, query_client) = setup(&env);
        let cl_id = env.register_contract(None, ChainLogisticsContract);

        // Second init should fail
        let res = query_client.try_init(&cl_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_query_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let query_id = env.register_contract(None, super::EventQueryContract);
        let query_client = super::EventQueryContractClient::new(&env, &query_id);

        let fake_id = String::from_str(&env, "FAKE-001");

        // Query without initialization should fail
        let res = query_client.try_get_event_count(&fake_id);
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

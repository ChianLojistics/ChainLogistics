use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::types::TrackingEventFilter;
use crate::types::TrackingEventPage;
use crate::validation_contract::ValidationContract;
use crate::{ProductRegistryContractClient, TrackingContractClient};

#[contracttype]
#[derive(Clone)]
enum QueryDataKey {
    RegistryContract,
    TrackingContract,
}

fn get_registry_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&QueryDataKey::RegistryContract)
}

fn set_registry_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&QueryDataKey::RegistryContract, address);
}

fn get_tracking_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&QueryDataKey::TrackingContract)
}

fn set_tracking_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&QueryDataKey::TrackingContract, address);
}

fn ensure_product_exists(env: &Env, product_id: &String) -> Result<(), Error> {
    ValidationContract::non_empty(product_id)?;
    ValidationContract::max_len(product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;
    let registry = get_registry_contract(env).ok_or(Error::NotInitialized)?;
    let registry_client = ProductRegistryContractClient::new(env, &registry);

    match registry_client.try_get_product(product_id) {
        Ok(Ok(_)) => Ok(()),
        _ => Err(Error::ProductNotFound),
    }
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct EventQueryContract;

#[contractimpl]
impl EventQueryContract {
    /// Initialize the EventQueryContract with the registry + tracking contract addresses.
    pub fn init(
        env: Env,
        registry_contract: Address,
        tracking_contract: Address,
    ) -> Result<(), Error> {
        if get_registry_contract(&env).is_some() || get_tracking_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        ValidationContract::validate_contract_address(&env, &registry_contract)?;
        ValidationContract::validate_contract_address(&env, &tracking_contract)?;
        set_registry_contract(&env, &registry_contract);
        set_tracking_contract(&env, &tracking_contract);
        Ok(())
    }

    /// Get paginated events for a product.
    /// Returns events with pagination info (total_count, has_more).
    pub fn query_get_product_events(
        env: Env,
        product_id: String,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        if ValidationContract::validate_pagination_limit(limit, ValidationContract::MAX_PAGE_LIMIT)
            .is_err()
        {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count: 0,
                has_more: false,
            });
        }

        ensure_product_exists(&env, &product_id)?;

        let tracking = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_client = TrackingContractClient::new(&env, &tracking);

        let all_ids = tracking_client.tracking_get_product_event_ids(&product_id);
        let total_count = all_ids.len() as u64;

        if offset >= total_count {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count,
                has_more: false,
            });
        }

        let start = offset as u32;
        let end = ((offset + limit) as u32).min(all_ids.len());

        // Fetch actual events
        let mut events = Vec::new(&env);
        for i in start..end {
            if let Some(eid) = all_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
                        events.push_back(event);
                    }
                }
            }
        }

        let has_more = offset
            .checked_add(events.len() as u64)
            .ok_or(Error::ArithmeticOverflow)?
            < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events filtered by type with pagination.
    pub fn query_get_events_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        if ValidationContract::validate_pagination_limit(limit, ValidationContract::MAX_PAGE_LIMIT)
            .is_err()
        {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count: 0,
                has_more: false,
            });
        }

        ensure_product_exists(&env, &product_id)?;

        let tracking = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_client = TrackingContractClient::new(&env, &tracking);

        let all_ids = tracking_client.tracking_get_product_event_ids(&product_id);
        let mut matching_ids: Vec<u64> = Vec::new(&env);

        for i in 0..all_ids.len() {
            if let Some(eid) = all_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
                        if event.event_type == event_type {
                            matching_ids.push_back(eid);
                        }
                    }
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        if offset >= total_count {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count,
                has_more: false,
            });
        }

        let start = offset as u32;
        let end_u64 = offset.checked_add(limit).ok_or(Error::ArithmeticOverflow)?;
        let end = (end_u64.min(u32::MAX as u64) as u32).min(matching_ids.len());

        let mut events = Vec::new(&env);
        for i in start..end {
            if let Some(eid) = matching_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
                        events.push_back(event);
                    }
                }
            }
        }

        let has_more = offset
            .checked_add(events.len() as u64)
            .ok_or(Error::ArithmeticOverflow)?
            < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events within a time range with pagination.
    pub fn query_get_events_by_time_range(
        env: Env,
        product_id: String,
        start_time: u64,
        end_time: u64,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        if ValidationContract::validate_pagination_limit(limit, ValidationContract::MAX_PAGE_LIMIT)
            .is_err()
        {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count: 0,
                has_more: false,
            });
        }
        ValidationContract::validate_time_range(&env, start_time, end_time)?;

        ensure_product_exists(&env, &product_id)?;

        let tracking = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_client = TrackingContractClient::new(&env, &tracking);

        let all_ids = tracking_client.tracking_get_product_event_ids(&product_id);
        let mut matching_ids: Vec<u64> = Vec::new(&env);

        for i in 0..all_ids.len() {
            if let Some(eid) = all_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
                        if event.timestamp >= start_time && event.timestamp <= end_time {
                            matching_ids.push_back(eid);
                        }
                    }
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        if offset >= total_count {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count,
                has_more: false,
            });
        }

        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        let mut events = Vec::new(&env);
        for i in start..end {
            if let Some(eid) = matching_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
                        events.push_back(event);
                    }
                }
            }
        }

        let has_more = offset
            .checked_add(events.len() as u64)
            .ok_or(Error::ArithmeticOverflow)?
            < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events with composite filtering (type, time range, location).
    /// All filter criteria are optional - use empty values to skip a filter.
    pub fn query_get_filtered_events(
        env: Env,
        product_id: String,
        filter: TrackingEventFilter,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        if ValidationContract::validate_pagination_limit(limit, ValidationContract::MAX_PAGE_LIMIT)
            .is_err()
        {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count: 0,
                has_more: false,
            });
        }
        if filter.start_time > 0 || filter.end_time < u64::MAX {
            ValidationContract::validate_time_range(&env, filter.start_time, filter.end_time)?;
        }

        ensure_product_exists(&env, &product_id)?;

        let tracking = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_client = TrackingContractClient::new(&env, &tracking);

        let all_ids = tracking_client.tracking_get_product_event_ids(&product_id);
        let mut matching_ids: Vec<u64> = Vec::new(&env);

        let empty_sym = Symbol::new(&env, "");
        let empty_loc = String::from_str(&env, "");

        for i in 0..all_ids.len() {
            if let Some(eid) = all_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
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
            }
        }

        let total_count = matching_ids.len() as u64;

        if offset >= total_count {
            return Ok(TrackingEventPage {
                events: Vec::new(&env),
                total_count,
                has_more: false,
            });
        }

        let start = offset as u32;
        let end_u64 = offset.checked_add(limit).ok_or(Error::ArithmeticOverflow)?;
        let end = (end_u64.min(u32::MAX as u64) as u32).min(matching_ids.len());

        let mut events = Vec::new(&env);
        for i in start..end {
            if let Some(eid) = matching_ids.get(i) {
                if let Ok(event) = tracking_client.try_tracking_get_event(&eid) {
                    if let Ok(event) = event {
                        events.push_back(event);
                    }
                }
            }
        }

        let has_more = offset
            .checked_add(events.len() as u64)
            .ok_or(Error::ArithmeticOverflow)?
            < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get total event count for a product.
    pub fn query_get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        ensure_product_exists(&env, &product_id)?;

        let tracking = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_client = TrackingContractClient::new(&env, &tracking);

        Ok(tracking_client.tracking_get_event_count(&product_id))
    }

    /// Get event count by type for a product.
    pub fn query_get_event_count_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
    ) -> Result<u64, Error> {
        ensure_product_exists(&env, &product_id)?;

        let tracking = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_client = TrackingContractClient::new(&env, &tracking);

        Ok(tracking_client.tracking_get_event_count_by_type(&product_id, &event_type))
    }
}

#[cfg(test)]
mod test_event_query {
    use super::*;
    use crate::{
        AuthorizationContract, AuthorizationContractClient, ChainLogisticsContract,
        ChainLogisticsContractClient, ProductConfig, ProductRegistryContract,
        ProductRegistryContractClient, TrackingContract, TrackingContractClient,
    };
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, Vec};

    fn setup(
        env: &Env,
    ) -> (
        ProductRegistryContractClient,
        TrackingContractClient,
        super::EventQueryContractClient,
        Address,
        Address,
    ) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let registry_id = env.register_contract(None, ProductRegistryContract);
        let tracking_id = env.register_contract(None, TrackingContract);
        let query_id = env.register_contract(None, super::EventQueryContract);

        let admin = Address::generate(env);
        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        cl_client.init(&admin, &auth_id);

        let registry_client = ProductRegistryContractClient::new(env, &registry_id);
        let auth_client = AuthorizationContractClient::new(env, &auth_id);
        let tracking_client = TrackingContractClient::new(env, &tracking_id);
        let query_client = super::EventQueryContractClient::new(env, &query_id);

        auth_client.configure_initializer(&registry_id);
        registry_client.configure_auth_contract(&auth_id);

        tracking_client.init(&cl_id);
        query_client.init(&registry_id, &tracking_id);

        (
            registry_client,
            tracking_client,
            query_client,
            registry_id,
            tracking_id,
        )
    }

    fn register_test_product(
        env: &Env,
        client: &ProductRegistryContractClient,
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
        tracking_client.tracking_add_event(
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

        let (registry_client, _tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Get events for product with no events
        let result = query_client.query_get_product_events(&product_id, &0, &10);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_get_product_events_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = query_client.try_query_get_product_events(&fake_id, &0, &10);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_event_count() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Initially 0 events
        assert_eq!(query_client.query_get_event_count(&product_id), 0);

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Count should be 2
        assert_eq!(query_client.query_get_event_count(&product_id), 2);
    }

    #[test]
    fn test_get_event_count_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = query_client.try_query_get_event_count(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_event_count_by_type() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add events of different types
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Check counts by type
        assert_eq!(
            query_client.query_get_event_count_by_type(&product_id, &Symbol::new(&env, "created")),
            1
        );
        assert_eq!(
            query_client.query_get_event_count_by_type(&product_id, &Symbol::new(&env, "shipped")),
            2
        );
        assert_eq!(
            query_client.query_get_event_count_by_type(&product_id, &Symbol::new(&env, "received")),
            0
        );
    }

    #[test]
    fn test_get_events_by_type() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");
        add_test_event(&env, &tracking_client, &owner, &product_id, "received");

        // Get only shipped events
        let result = query_client.query_get_events_by_type(
            &product_id,
            &Symbol::new(&env, "shipped"),
            &0,
            &10,
        );
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_get_events_by_time_range() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Get events in time range (all events)
        let result =
            query_client.query_get_events_by_time_range(&product_id, &0, &u64::MAX, &0, &10);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_get_filtered_events() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

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
        let result = query_client.query_get_filtered_events(&product_id, &filter, &0, &10);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_pagination() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add 5 events
        for _ in 0..5 {
            add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        }

        // Get first 2
        let result = query_client.query_get_product_events(&product_id, &0, &2);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.total_count, 5);
        assert!(result.has_more);

        // Get next 2
        let result = query_client.query_get_product_events(&product_id, &2, &2);
        assert_eq!(result.events.len(), 2);
        assert!(result.has_more);

        // Get last 1
        let result = query_client.query_get_product_events(&product_id, &4, &2);
        assert_eq!(result.events.len(), 1);
        assert!(!result.has_more);
    }

    #[test]
    fn test_pagination_boundary_conditions() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add 5 events for testing
        for _ in 0..5 {
            add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        }

        // Test limit == 0 (should return empty due to validation)
        let result = query_client.query_get_product_events(&product_id, &0, &0);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        // Test very large limit (should return empty due to validation)
        let result = query_client.query_get_product_events(&product_id, &0, &1001);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        // Test offset == total_count (should return empty)
        let result = query_client.query_get_product_events(&product_id, &5, &2);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 5);
        assert!(!result.has_more);

        // Test offset > total_count (should return empty)
        let result = query_client.query_get_product_events(&product_id, &10, &2);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 5);
        assert!(!result.has_more);

        // Test normal pagination still works
        let result = query_client.query_get_product_events(&product_id, &0, &3);
        assert_eq!(result.events.len(), 3);
        assert_eq!(result.total_count, 5);
        assert!(result.has_more);

        let result = query_client.query_get_product_events(&product_id, &3, &3);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.total_count, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn test_empty_product_boundary_conditions() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "EMPTY_PROD");

        // Test with empty product (no events)
        let result = query_client.query_get_product_events(&product_id, &0, &10);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        // Test offset > 0 with empty product
        let result = query_client.query_get_product_events(&product_id, &5, &10);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        // Test filtered queries with empty product
        let result = query_client.query_get_events_by_type(
            &product_id,
            &Symbol::new(&env, "created"),
            &0,
            &10,
        );
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        let filter = TrackingEventFilter {
            event_type: Symbol::new(&env, ""),
            start_time: 0,
            end_time: u64::MAX,
            location: String::from_str(&env, ""),
        };
        let result = query_client.query_get_filtered_events(&product_id, &filter, &0, &10);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_query_boundary_conditions_with_validation() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, tracking_client, query_client, _registry_id, _tracking_id) =
            setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Add some events
        add_test_event(&env, &tracking_client, &owner, &product_id, "created");
        add_test_event(&env, &tracking_client, &owner, &product_id, "shipped");

        // Test validation on different query types
        let result = query_client.query_get_events_by_type(
            &product_id,
            &Symbol::new(&env, "created"),
            &0,
            &0,
        );
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        let result =
            query_client.query_get_events_by_time_range(&product_id, &0, &u64::MAX, &0, &1001);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);

        let filter = TrackingEventFilter {
            event_type: Symbol::new(&env, "created"),
            start_time: 0,
            end_time: u64::MAX,
            location: String::from_str(&env, ""),
        };
        let result = query_client.query_get_filtered_events(&product_id, &filter, &0, &0);
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, query_client, registry_id, tracking_id) =
            setup(&env);

        // Second init should fail
        let res = query_client.try_init(&registry_id, &tracking_id);
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
        let res = query_client.try_query_get_event_count(&fake_id);
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

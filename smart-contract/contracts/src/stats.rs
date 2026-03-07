use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{DataKey, ProductStats};

// ─── Storage helpers for StatsContract ───────────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::MainContract, address);
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct StatsContract;

#[contractimpl]
impl StatsContract {
    /// Initialize the StatsContract with the main contract address.
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_main_contract(&env, &main_contract);
        Ok(())
    }

    /// Get global product statistics.
    /// Returns total and active product counts.
    pub fn get_stats(env: Env) -> Result<ProductStats, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        
        Ok(ProductStats {
            total_products: storage::get_total_products(&env),
            active_products: storage::get_active_products(&env),
        })
    }

    /// Get the total number of products registered.
    pub fn get_total_products(env: Env) -> Result<u64, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        Ok(storage::get_total_products(&env))
    }

    /// Get the number of active products.
    pub fn get_active_products(env: Env) -> Result<u64, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        Ok(storage::get_active_products(&env))
    }

    /// Get the number of inactive products.
    pub fn get_inactive_products(env: Env) -> Result<u64, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let total = storage::get_total_products(&env);
        let active = storage::get_active_products(&env);
        Ok(total.saturating_sub(active))
    }

    /// Get the total number of tracking events across all products.
    pub fn get_total_events(env: Env) -> Result<u64, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        
        // Get the last event ID which represents total events created
        let last_event_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::EventSeq)
            .unwrap_or(0);
        Ok(last_event_id)
    }

    /// Get product-specific statistics.
    /// Returns (event_count, is_active) for a given product.
    pub fn get_product_stats(env: Env, product_id: String) -> Result<(u64, bool), Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        
        // Check if product exists
        let product = storage::get_product(&env, &product_id)
            .ok_or(Error::ProductNotFound)?;
        
        let event_count = storage::get_product_event_ids(&env, &product_id).len() as u64;
        Ok((event_count, product.active))
    }

    /// Get the average number of events per product.
    pub fn get_average_events_per_product(env: Env) -> Result<u64, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        
        let total_products = storage::get_total_products(&env);
        if total_products == 0 {
            return Ok(0);
        }
        
        let total_events = Self::get_total_events(env)?;
        Ok(total_events / total_products)
    }

    /// Get event type distribution for a product.
    /// Returns a Vec of (event_type, count) tuples.
    pub fn get_event_type_distribution(
        env: Env,
        product_id: String,
    ) -> Result<Vec<(Symbol, u64)>, Error> {
        let _main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        
        // Verify product exists
        if !storage::has_product(&env, &product_id) {
            return Err(Error::ProductNotFound);
        }
        
        // Get all event IDs for the product
        let event_ids = storage::get_product_event_ids(&env, &product_id);
        
        // Count events by type
        let mut type_counts = Vec::new(&env);
        
        // Common event types to check
        let event_types = [
            Symbol::new(&env, "created"),
            Symbol::new(&env, "shipped"),
            Symbol::new(&env, "received"),
            Symbol::new(&env, "transferred"),
            Symbol::new(&env, "updated"),
        ];
        
        for event_type in event_types.iter() {
            let count = storage::get_event_count_by_type(&env, &product_id, event_type);
            if count > 0 {
                type_counts.push_back((event_type.clone(), count));
            }
        }
        
        Ok(type_counts)
    }
}

#[cfg(test)]
mod test_stats {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Map};
    use crate::{
        AuthorizationContract, ChainLogisticsContract, ChainLogisticsContractClient,
        ProductConfig,
    };

    fn setup(env: &Env) -> (ChainLogisticsContractClient, Address, Address, super::StatsContractClient) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let stats_id = env.register_contract(None, super::StatsContract);

        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let stats_client = super::StatsContractClient::new(env, &stats_id);

        let admin = Address::generate(env);
        cl_client.init(&admin, &auth_id);
        stats_client.init(&cl_id);

        (cl_client, admin, cl_id, stats_client)
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

    #[test]
    fn test_get_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);

        // Initial stats
        let stats = stats_client.get_stats();
        assert_eq!(stats.total_products, 0);
        assert_eq!(stats.active_products, 0);

        // Register products
        let owner = Address::generate(&env);
        register_test_product(&env, &cl_client, &owner, "PROD1");
        register_test_product(&env, &cl_client, &owner, "PROD2");

        // Updated stats
        let stats = stats_client.get_stats();
        assert_eq!(stats.total_products, 2);
        assert_eq!(stats.active_products, 2);
    }

    #[test]
    fn test_get_total_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);

        // Initial count
        assert_eq!(stats_client.get_total_products(), 0);

        // Register products
        let owner = Address::generate(&env);
        register_test_product(&env, &cl_client, &owner, "PROD1");
        register_test_product(&env, &cl_client, &owner, "PROD2");
        register_test_product(&env, &cl_client, &owner, "PROD3");

        // Updated count
        assert_eq!(stats_client.get_total_products(), 3);
    }

    #[test]
    fn test_get_active_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);
        let owner = Address::generate(&env);

        // Register and deactivate a product
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");
        register_test_product(&env, &cl_client, &owner, "PROD2");

        // Both active initially
        assert_eq!(stats_client.get_active_products(), 2);

        // Deactivate one
        cl_client.deactivate_product(&owner, &product_id, &String::from_str(&env, "Testing"));

        // One active now
        assert_eq!(stats_client.get_active_products(), 1);
    }

    #[test]
    fn test_get_inactive_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);
        let owner = Address::generate(&env);

        // Register products
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");
        register_test_product(&env, &cl_client, &owner, "PROD2");

        // No inactive initially
        assert_eq!(stats_client.get_inactive_products(), 0);

        // Deactivate one
        cl_client.deactivate_product(&owner, &product_id, &String::from_str(&env, "Testing"));

        // One inactive now
        assert_eq!(stats_client.get_inactive_products(), 1);
    }

    #[test]
    fn test_get_total_events() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _admin, _cl_id, stats_client) = setup(&env);

        // No events initially
        assert_eq!(stats_client.get_total_events(), 0);

        // Note: Events are added through TrackingContract
        // Since TrackingContract shares storage, we can verify stats work
    }

    #[test]
    fn test_get_product_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Get product stats
        let (event_count, is_active) = stats_client.get_product_stats(&product_id);
        assert_eq!(event_count, 0); // No events yet
        assert!(is_active); // Product is active

        // Deactivate product
        cl_client.deactivate_product(&owner, &product_id, &String::from_str(&env, "Testing"));

        // Check stats again
        let (event_count, is_active) = stats_client.get_product_stats(&product_id);
        assert_eq!(event_count, 0);
        assert!(!is_active); // Product is now inactive
    }

    #[test]
    fn test_get_product_stats_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _admin, _cl_id, stats_client) = setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = stats_client.try_get_product_stats(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_average_events_per_product() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);

        // No products - average should be 0
        assert_eq!(stats_client.get_average_events_per_product(), 0);

        // Register products
        let owner = Address::generate(&env);
        register_test_product(&env, &cl_client, &owner, "PROD1");
        register_test_product(&env, &cl_client, &owner, "PROD2");

        // Still 0 events - average should be 0
        assert_eq!(stats_client.get_average_events_per_product(), 0);
    }

    #[test]
    fn test_get_event_type_distribution() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, stats_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // No events yet - distribution should be empty
        let distribution = stats_client.get_event_type_distribution(&product_id);
        assert_eq!(distribution.len(), 0);
    }

    #[test]
    fn test_get_event_type_distribution_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _admin, _cl_id, stats_client) = setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = stats_client.try_get_event_type_distribution(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_cl_client, _admin, cl_id, stats_client) = setup(&env);

        // Second init should fail
        let res = stats_client.try_init(&cl_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_stats_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let stats_id = env.register_contract(None, super::StatsContract);
        let stats_client = super::StatsContractClient::new(&env, &stats_id);

        // Get stats without initialization should fail
        let res = stats_client.try_get_stats();
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::types::{DataKey, ProductStats};
use crate::{ProductRegistryContractClient, TrackingContractClient};

#[contracttype]
#[derive(Clone)]
enum StatsDataKey {
    RegistryContract,
    TrackingContract,
}

// ─── Storage helpers for StatsContract ───────────────────────────────────────

fn get_registry_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&StatsDataKey::RegistryContract)
}

fn set_registry_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StatsDataKey::RegistryContract, address);
}

fn get_tracking_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&StatsDataKey::TrackingContract)
}

fn set_tracking_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StatsDataKey::TrackingContract, address);
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct StatsContract;

#[contractimpl]
impl StatsContract {
    /// Initialize the StatsContract with the main contract address.
    pub fn init(
        env: Env,
        registry_contract: Address,
        tracking_contract: Address,
    ) -> Result<(), Error> {
        if get_registry_contract(&env).is_some() || get_tracking_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_registry_contract(&env, &registry_contract);
        set_tracking_contract(&env, &tracking_contract);
        Ok(())
    }

    /// Get global product statistics.
    /// Returns total and active product counts.
    pub fn stats_get_stats(env: Env) -> Result<ProductStats, Error> {
        let registry_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;
        let registry_client = ProductRegistryContractClient::new(&env, &registry_contract);
        Ok(registry_client.get_stats())
    }

    /// Get the total number of products registered.
    pub fn get_total_products(env: Env) -> Result<u64, Error> {
        let registry_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;
        let registry_client = ProductRegistryContractClient::new(&env, &registry_contract);

        let stats = registry_client.get_stats();
        Ok(stats.total_products)
    }

    /// Get the number of active products.
    pub fn get_active_products(env: Env) -> Result<u64, Error> {
        let registry_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;
        let registry_client = ProductRegistryContractClient::new(&env, &registry_contract);

        let stats = registry_client.get_stats();
        Ok(stats.active_products)
    }

    /// Get the number of inactive products.
    pub fn get_inactive_products(env: Env) -> Result<u64, Error> {
        let registry_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;
        let registry_client = ProductRegistryContractClient::new(&env, &registry_contract);

        let stats = registry_client.get_stats();
        let total = stats.total_products;
        let active = stats.active_products;
        Ok(total.saturating_sub(active))
    }

    /// Get the total number of tracking events across all products.
    pub fn get_total_events(env: Env) -> Result<u64, Error> {
        let _ = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
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
        let registry_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_contract = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;

        let registry_client = ProductRegistryContractClient::new(&env, &registry_contract);
        let tracking_client = TrackingContractClient::new(&env, &tracking_contract);

        let is_active = match registry_client.try_get_product(&product_id) {
            Ok(Ok(product)) => product.active,
            _ => return Err(Error::ProductNotFound),
        };

        let event_count = tracking_client.tracking_get_event_count(&product_id);

        Ok((event_count, is_active))
    }

    /// Get the average number of events per product.
    pub fn get_average_events_per_product(env: Env) -> Result<u64, Error> {
        let _main_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;

        let total_products = Self::get_total_products(env.clone())?;
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
        let registry_contract = get_registry_contract(&env).ok_or(Error::NotInitialized)?;
        let tracking_contract = get_tracking_contract(&env).ok_or(Error::NotInitialized)?;
        let registry_client = ProductRegistryContractClient::new(&env, &registry_contract);
        let tracking_client = TrackingContractClient::new(&env, &tracking_contract);

        match registry_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {}
            _ => return Err(Error::ProductNotFound),
        };

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
            let count = tracking_client.tracking_get_event_count_by_type(&product_id, event_type);
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
    use crate::{
        AuthorizationContract, AuthorizationContractClient, ProductConfig, ProductRegistryContract,
        ProductRegistryContractClient, TrackingContract, TrackingContractClient,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Map};

    fn setup(
        env: &Env,
    ) -> (
        ProductRegistryContractClient<'_>,
        TrackingContractClient<'_>,
        super::StatsContractClient<'_>,
    ) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let registry_id = env.register_contract(None, ProductRegistryContract);
        let tracking_id = env.register_contract(None, TrackingContract);
        let stats_id = env.register_contract(None, super::StatsContract);

        let registry_client = ProductRegistryContractClient::new(env, &registry_id);
        let tracking_client = TrackingContractClient::new(env, &tracking_id);
        let stats_client = super::StatsContractClient::new(env, &stats_id);

        let auth_client = AuthorizationContractClient::new(env, &auth_id);
        auth_client.configure_initializer(&registry_id);
        registry_client.configure_auth_contract(&auth_id);

        tracking_client.init(&registry_id);
        stats_client.init(&registry_id, &tracking_id);

        (registry_client, tracking_client, stats_client)
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

    #[test]
    fn test_get_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);

        // Initial stats
        let stats = stats_client.stats_get_stats();
        assert_eq!(stats.total_products, 0);
        assert_eq!(stats.active_products, 0);

        // Register products
        let owner = Address::generate(&env);
        register_test_product(&env, &registry_client, &owner, "PROD1");
        register_test_product(&env, &registry_client, &owner, "PROD2");

        // Updated stats
        let stats = stats_client.stats_get_stats();
        assert_eq!(stats.total_products, 2);
        assert_eq!(stats.active_products, 2);
    }

    #[test]
    fn test_get_total_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);

        // Initial count
        assert_eq!(stats_client.get_total_products(), 0);

        // Register products
        let owner = Address::generate(&env);
        register_test_product(&env, &registry_client, &owner, "PROD1");
        register_test_product(&env, &registry_client, &owner, "PROD2");
        register_test_product(&env, &registry_client, &owner, "PROD3");

        // Updated count
        assert_eq!(stats_client.get_total_products(), 3);
    }

    #[test]
    fn test_get_active_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);
        let owner = Address::generate(&env);

        // Register and deactivate a product
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");
        register_test_product(&env, &registry_client, &owner, "PROD2");

        // Both active initially
        assert_eq!(stats_client.get_active_products(), 2);

        // Deactivate one
        registry_client.deactivate_product(&owner, &product_id, &String::from_str(&env, "Testing"));

        // One active now
        assert_eq!(stats_client.get_active_products(), 1);
    }

    #[test]
    fn test_get_inactive_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);
        let owner = Address::generate(&env);

        // Register products
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");
        register_test_product(&env, &registry_client, &owner, "PROD2");

        // No inactive initially
        assert_eq!(stats_client.get_inactive_products(), 0);

        // Deactivate one
        registry_client.deactivate_product(&owner, &product_id, &String::from_str(&env, "Testing"));

        // One inactive now
        assert_eq!(stats_client.get_inactive_products(), 1);
    }

    #[test]
    fn test_get_total_events() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, stats_client) = setup(&env);

        // No events initially
        assert_eq!(stats_client.get_total_events(), 0);
    }

    #[test]
    fn test_get_product_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // Get product stats
        let (event_count, is_active) = stats_client.get_product_stats(&product_id);
        assert_eq!(event_count, 0); // No events yet
        assert!(is_active); // Product is active

        // Deactivate product
        registry_client.deactivate_product(&owner, &product_id, &String::from_str(&env, "Testing"));

        // Check stats again
        let (event_count, is_active) = stats_client.get_product_stats(&product_id);
        assert_eq!(event_count, 0);
        assert!(!is_active); // Product is now inactive
    }

    #[test]
    fn test_get_product_stats_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, stats_client) = setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = stats_client.try_get_product_stats(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_average_events_per_product() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);

        // No products - average should be 0
        assert_eq!(stats_client.get_average_events_per_product(), 0);

        // Register products
        let owner = Address::generate(&env);
        register_test_product(&env, &registry_client, &owner, "PROD1");
        register_test_product(&env, &registry_client, &owner, "PROD2");

        // Still 0 events - average should be 0
        assert_eq!(stats_client.get_average_events_per_product(), 0);
    }

    #[test]
    fn test_get_event_type_distribution() {
        let env = Env::default();
        env.mock_all_auths();

        let (registry_client, _tracking_client, stats_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &registry_client, &owner, "PROD1");

        // No events yet - distribution should be empty
        let distribution = stats_client.get_event_type_distribution(&product_id);
        assert_eq!(distribution.len(), 0);
    }

    #[test]
    fn test_get_event_type_distribution_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, stats_client) = setup(&env);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = stats_client.try_get_event_type_distribution(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_registry_client, _tracking_client, stats_client) = setup(&env);
        let pr_id = env.register_contract(None, ProductRegistryContract);
        let tracking_id = env.register_contract(None, TrackingContract);

        // Second init should fail
        let res = stats_client.try_init(&pr_id, &tracking_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_stats_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let stats_id = env.register_contract(None, super::StatsContract);
        let stats_client = super::StatsContractClient::new(&env, &stats_id);

        // Get stats without initialization should fail
        let res = stats_client.try_stats_get_stats();
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

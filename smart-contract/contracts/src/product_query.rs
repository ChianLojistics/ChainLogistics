use soroban_sdk::{contract, contractimpl, Env, String, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{Product, ProductStats};

// ─── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ProductQueryContract;

#[contractimpl]
impl ProductQueryContract {
    /// Retrieve a single product by its ID.
    /// Returns ProductNotFound error if the product doesn't exist.
    pub fn get_product(env: Env, product_id: String) -> Result<Product, Error> {
        storage::get_product(&env, &product_id).ok_or(Error::ProductNotFound)
    }

    /// Get all event IDs associated with a product.
    /// Returns ProductNotFound error if the product doesn't exist.
    pub fn get_product_event_ids(env: Env, product_id: String) -> Result<Vec<u64>, Error> {
        // Verify product exists
        if !storage::has_product(&env, &product_id) {
            return Err(Error::ProductNotFound);
        }
        Ok(storage::get_product_event_ids(&env, &product_id))
    }

    /// Get global product statistics.
    /// Returns total and active product counts.
    pub fn get_stats(env: Env) -> ProductStats {
        ProductStats {
            total_products: storage::get_total_products(&env),
            active_products: storage::get_active_products(&env),
        }
    }

    /// Check if a product exists in the system.
    pub fn product_exists(env: Env, product_id: String) -> bool {
        storage::has_product(&env, &product_id)
    }

    /// Get the total number of events for a product.
    /// Returns ProductNotFound error if the product doesn't exist.
    pub fn get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        // Verify product exists
        if !storage::has_product(&env, &product_id) {
            return Err(Error::ProductNotFound);
        }
        let ids = storage::get_product_event_ids(&env, &product_id);
        Ok(ids.len() as u64)
    }
}

#[cfg(test)]
mod test_product_query {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Map};
    use crate::{
        AuthorizationContract, ChainLogisticsContract, ChainLogisticsContractClient,
        ProductConfig,
    };

    fn setup(env: &Env) -> (ChainLogisticsContractClient, Address) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);

        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);

        let admin = Address::generate(env);
        cl_client.init(&admin, &auth_id);

        (cl_client, admin)
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
    fn test_get_product() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Create query contract client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        // Get product
        let product = query_client.get_product(&product_id);
        assert_eq!(product.id, product_id);
        assert_eq!(product.owner, owner);
    }

    #[test]
    fn test_get_product_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        // Create query contract client without main contract setup
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = query_client.try_get_product(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_product_event_ids() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Create query contract client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        // Get event IDs (should be empty for new product)
        let event_ids = query_client.get_product_event_ids(&product_id);
        assert_eq!(event_ids.len(), 0);
    }

    #[test]
    fn test_get_product_event_ids_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        // Create query contract client without main contract setup
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = query_client.try_get_product_event_ids(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin) = setup(&env);

        // Create query contract client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        // Initial stats
        let stats = query_client.get_stats();
        assert_eq!(stats.total_products, 0);
        assert_eq!(stats.active_products, 0);

        // Register a product
        let owner = Address::generate(&env);
        register_test_product(&env, &cl_client, &owner, "PROD1");

        // Updated stats
        let stats = query_client.get_stats();
        assert_eq!(stats.total_products, 1);
        assert_eq!(stats.active_products, 1);
    }

    #[test]
    fn test_product_exists() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Create query contract client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        // Existing product
        assert!(query_client.product_exists(&product_id));

        // Non-existing product
        let fake_id = String::from_str(&env, "NONEXISTENT");
        assert!(!query_client.product_exists(&fake_id));
    }

    #[test]
    fn test_get_event_count() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Create query contract client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        // Get event count (should be 0 for new product)
        let count = query_client.get_event_count(&product_id);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_event_count_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        // Create query contract client without main contract setup
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = query_client.try_get_event_count(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }
}

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

use crate::error::Error;
use crate::types::{DataKey, Product, ProductStats};
use crate::ChainLogisticsContractClient;

// ─── Storage helpers ─────────────────────────────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::MainContract, address);
}

// ─── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ProductQueryContract;

#[contractimpl]
impl ProductQueryContract {
    /// Initialize the ProductQueryContract with the main contract address.
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_main_contract(&env, &main_contract);
        Ok(())
    }

    /// Retrieve a single product by its ID.
    /// Returns ProductNotFound error if the product doesn't exist.
    pub fn get_product(env: Env, product_id: String) -> Result<Product, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Call main contract to get product
        match main_client.try_get_product(&product_id) {
            Ok(Ok(product)) => Ok(product),
            Ok(Err(_)) => Err(Error::ProductNotFound),
            Err(_) => Err(Error::ProductNotFound),
        }
    }

    /// Get all event IDs associated with a product.
    /// Returns ProductNotFound error if the product doesn't exist.
    pub fn get_product_event_ids(env: Env, product_id: String) -> Result<Vec<u64>, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists first
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        // Get event IDs from main contract
        match main_client.try_get_product_event_ids(&product_id) {
            Ok(Ok(ids)) => Ok(ids),
            Ok(Err(_)) | Err(_) => Err(Error::ProductNotFound),
        }
    }

    /// Get global product statistics.
    /// Returns total and active product counts.
    pub fn get_stats(env: Env) -> Result<ProductStats, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        Ok(main_client.get_stats())
    }

    /// Check if a product exists in the system.
    pub fn product_exists(env: Env, product_id: String) -> Result<bool, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Get the total number of events for a product.
    /// Returns ProductNotFound error if the product doesn't exist.
    pub fn get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        
        // Verify product exists first
        match main_client.try_get_product(&product_id) {
            Ok(Ok(_)) => {},
            _ => return Err(Error::ProductNotFound),
        }
        
        match main_client.try_get_event_count(&product_id) {
            Ok(Ok(count)) => Ok(count),
            Ok(Err(_)) | Err(_) => Err(Error::ProductNotFound),
        }
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

    fn setup(env: &Env) -> (ChainLogisticsContractClient, Address, Address, super::ProductQueryContractClient) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let query_id = env.register_contract(None, super::ProductQueryContract);

        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let query_client = super::ProductQueryContractClient::new(env, &query_id);

        let admin = Address::generate(env);
        cl_client.init(&admin, &auth_id);
        query_client.init(&cl_id);

        (cl_client, admin, cl_id, query_client)
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

        let (cl_client, _admin, _cl_id, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Get product
        let product = query_client.get_product(&product_id);
        assert_eq!(product.id, product_id);
        assert_eq!(product.owner, owner);
    }

    #[test]
    fn test_get_product_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, cl_id, _query_client) = setup(&env);
        
        // Create a new query client that's not initialized
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        // Initialize with the main contract
        query_client.init(&cl_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = query_client.try_get_product(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_product_event_ids() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Get event IDs (should be empty for new product)
        let event_ids = query_client.get_product_event_ids(&product_id);
        assert_eq!(event_ids.len(), 0);
    }

    #[test]
    fn test_get_product_event_ids_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, cl_id, _query_client) = setup(&env);
        
        // Create query client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.init(&cl_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = query_client.try_get_product_event_ids(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_get_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, _cl_id, query_client) = setup(&env);

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

        let (cl_client, _admin, _cl_id, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

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

        let (cl_client, _admin, _cl_id, query_client) = setup(&env);
        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &cl_client, &owner, "PROD1");

        // Get event count (should be 0 for new product)
        let count = query_client.get_event_count(&product_id);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_event_count_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, cl_id, _query_client) = setup(&env);
        
        // Create query client
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.init(&cl_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = query_client.try_get_event_count(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _admin, cl_id, query_client) = setup(&env);

        // Second init should fail
        let res = query_client.try_init(&cl_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_query_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        let fake_id = String::from_str(&env, "FAKE-001");

        // Query without initialization should fail
        let res = query_client.try_get_product(&fake_id);
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

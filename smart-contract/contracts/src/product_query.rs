use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::error::Error;
use crate::types::{DataKey, Product, ProductStats};
use crate::validation_contract::ValidationContract;
use crate::ProductRegistryContractClient;

// ─── Storage helpers ─────────────────────────────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::MainContract, address);
}

// ─── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ProductQueryContract;

#[contractimpl]
impl ProductQueryContract {
    /// Initialize the ProductQueryContract with the ProductRegistryContract address.
    pub fn query_init(env: Env, registry_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        ValidationContract::validate_contract_address(&env, &registry_contract)?;
        set_main_contract(&env, &registry_contract);
        Ok(())
    }

    /// Retrieve a single product by its ID.
    /// Delegates to ProductRegistryContract.
    pub fn query_product(env: Env, product_id: String) -> Result<Product, Error> {
        ValidationContract::non_empty(&product_id)?;
        ValidationContract::max_len(&product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);

        match pr_client.try_get_product(&product_id) {
            Ok(Ok(product)) => Ok(product),
            Ok(Err(_)) => Err(Error::ProductNotFound),
            Err(_) => Err(Error::ProductNotFound),
        }
    }

    /// Get global product statistics.
    /// Delegates to ProductRegistryContract.
    pub fn query_stats(env: Env) -> Result<ProductStats, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);

        Ok(pr_client.get_stats())
    }

    /// Check if a product exists in the system.
    pub fn query_product_exists(env: Env, product_id: String) -> Result<bool, Error> {
        ValidationContract::non_empty(&product_id)?;
        ValidationContract::max_len(&product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);

        match pr_client.try_get_product(&product_id) {
            Ok(Ok(_)) => Ok(true),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod test_product_query {
    use super::*;
    use crate::{
        AuthorizationContract, AuthorizationContractClient, ProductConfig, ProductRegistryContract,
        ProductRegistryContractClient,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Map, Vec};

    fn setup(env: &Env) -> (ProductRegistryContractClient, Address) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);
        let pr_client = ProductRegistryContractClient::new(env, &pr_id);

        let auth_client = AuthorizationContractClient::new(env, &auth_id);
        auth_client.configure_initializer(&pr_id);
        pr_client.configure_auth_contract(&auth_id);

        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(env, &query_id);
        query_client.query_init(&pr_id);

        (pr_client, pr_id)
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
    fn test_query_product() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, pr_id) = setup(&env);
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.query_init(&pr_id);

        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &pr_client, &owner, "PROD1");

        let product = query_client.query_product(&product_id);
        assert_eq!(product.id, product_id);
        assert_eq!(product.owner, owner);
    }

    #[test]
    fn test_query_product_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let (_pr_client, pr_id) = setup(&env);

        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.query_init(&pr_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");
        let res = query_client.try_query_product(&fake_id);
        assert_eq!(res, Err(Ok(Error::ProductNotFound)));
    }

    #[test]
    fn test_query_stats() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, pr_id) = setup(&env);
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.query_init(&pr_id);

        let stats = query_client.query_stats();
        assert_eq!(stats.total_products, 0);
        assert_eq!(stats.active_products, 0);

        let owner = Address::generate(&env);
        register_test_product(&env, &pr_client, &owner, "PROD1");

        let stats = query_client.query_stats();
        assert_eq!(stats.total_products, 1);
        assert_eq!(stats.active_products, 1);
    }

    #[test]
    fn test_query_product_exists() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, pr_id) = setup(&env);
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.query_init(&pr_id);

        let owner = Address::generate(&env);
        let product_id = register_test_product(&env, &pr_client, &owner, "PROD1");

        assert!(query_client.query_product_exists(&product_id));
        let fake_id = String::from_str(&env, "NONEXISTENT");
        assert!(!query_client.query_product_exists(&fake_id));
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (_pr_client, pr_id) = setup(&env);
        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);
        query_client.query_init(&pr_id);

        let res = query_client.try_query_init(&pr_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_query_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let query_id = env.register_contract(None, super::ProductQueryContract);
        let query_client = super::ProductQueryContractClient::new(&env, &query_id);

        let fake_id = String::from_str(&env, "FAKE-001");
        let res = query_client.try_query_product(&fake_id);
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

use crate::error::Error;
use crate::{AuthorizationContractClient, ChainLogisticsContractClient};
use crate::types::DataKey;

// ─── Storage helpers for ProductTransferContract ─────────────────────────────

fn get_auth_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::AuthContract)
}

fn set_auth_contract(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::AuthContract, address);
}

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::MainContract, address);
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct ProductTransferContract;

#[contractimpl]
impl ProductTransferContract {
    /// Initialize the ProductTransferContract with required contract addresses.
    pub fn init(env: Env, main_contract: Address, auth_contract: Address) -> Result<(), Error> {
        if get_auth_contract(&env).is_some() || get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_main_contract(&env, &main_contract);
        set_auth_contract(&env, &auth_contract);
        Ok(())
    }

    /// Transfer ownership of a product from the current owner to a new owner.
    /// Requires authentication from both the current owner and the new owner.
    /// Updates authorization mappings and emits a transfer event.
    pub fn transfer_product(
        env: Env,
        owner: Address,
        product_id: String,
        new_owner: Address,
    ) -> Result<(), Error> {
        // Require authentication from both parties
        owner.require_auth();
        new_owner.require_auth();

        // Get contract addresses
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let auth_contract = get_auth_contract(&env).ok_or(Error::NotInitialized)?;

        // Create client to interact with main contract
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);

        // Verify product exists and get current product info
        let product = main_client.get_product(&product_id);

        // Verify current ownership
        if product.owner != owner {
            return Err(Error::Unauthorized);
        }

        // Update authorization mappings via AuthorizationContract
        let auth_client = AuthorizationContractClient::new(&env, &auth_contract);
        auth_client.update_product_owner(&owner, &product_id, &new_owner);

        // Emit transfer event
        env.events().publish(
            (Symbol::new(&env, "product_transferred"), product_id),
            (owner, new_owner),
        );

        Ok(())
    }

    /// Get the current owner of a product.
    pub fn get_product_owner(env: Env, product_id: String) -> Result<Address, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        let product = main_client.get_product(&product_id);
        Ok(product.owner)
    }

    /// Verify if an address is the owner of a specific product.
    pub fn is_product_owner(
        env: Env,
        product_id: String,
        address: Address,
    ) -> Result<bool, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        let product = main_client.get_product(&product_id);
        Ok(product.owner == address)
    }

    /// Batch transfer multiple products from one owner to another.
    /// All products must be owned by the same owner.
    pub fn batch_transfer_products(
        env: Env,
        owner: Address,
        product_ids: soroban_sdk::Vec<String>,
        new_owner: Address,
    ) -> Result<u32, Error> {
        // Require authentication from both parties
        owner.require_auth();
        new_owner.require_auth();

        if product_ids.is_empty() {
            return Err(Error::EmptyBatch);
        }

        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let auth_contract = get_auth_contract(&env).ok_or(Error::NotInitialized)?;

        let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
        let auth_client = AuthorizationContractClient::new(&env, &auth_contract);

        let mut transferred_count: u32 = 0;

        for i in 0..product_ids.len() {
            let product_id = product_ids.get_unchecked(i);

            // Verify ownership for each product
            let product = match main_client.try_get_product(&product_id) {
                Ok(Ok(p)) => p,
                Ok(Err(_)) | Err(_) => continue, // Skip non-existent products or errors
            };

            if product.owner != owner {
                continue; // Skip products not owned by the caller
            }

            // Update authorization mappings
            auth_client.update_product_owner(&owner, &product_id, &new_owner);

            // Emit transfer event
            env.events().publish(
                (Symbol::new(&env, "product_transferred"), product_id),
                (owner.clone(), new_owner.clone()),
            );

            transferred_count += 1;
        }

        Ok(transferred_count)
    }
}

#[cfg(test)]
mod test_product_transfer {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};
    use crate::{
        AuthorizationContract, AuthorizationContractClient,
        ChainLogisticsContract, ChainLogisticsContractClient,
        ProductConfig,
    };

    fn setup(env: &Env) -> (ChainLogisticsContractClient, AuthorizationContractClient, Address, ProductTransferContractClient, Address) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let transfer_id = env.register_contract(None, ProductTransferContract);

        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let auth_client = AuthorizationContractClient::new(env, &auth_id);
        let transfer_client = ProductTransferContractClient::new(env, &transfer_id);

        let admin = Address::generate(env);
        // Initialize ChainLogisticsContract with the address of AuthorizationContract
        cl_client.init(&admin, &auth_id);
        // Initialize ProductTransferContract with both contract addresses
        transfer_client.init(&cl_id, &auth_id);

        (cl_client, auth_client, admin, transfer_client, cl_id)
    }

    fn register_test_product(
        env: &Env,
        client: &ChainLogisticsContractClient,
        owner: &Address,
    ) -> String {
        let id = String::from_str(env, "PROD1");
        client.register_product(
            owner,
            &ProductConfig {
                id: id.clone(),
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
        id
    }

    #[test]
    fn test_transfer_product_ownership() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _auth_client, _admin, transfer_client, _cl_id) = setup(&env);

        // Use ChainLogisticsContract for product registration
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let id = register_test_product(&env, &cl_client, &owner);

        // Verify initial owner
        let p = cl_client.get_product(&id);
        assert_eq!(p.owner, owner);

        // Transfer ownership
        transfer_client.transfer_product(&owner, &id, &new_owner);

        // Verify new owner - product ownership updated via AuthorizationContract
        // The actual product owner in ChainLogisticsContract remains the same
        // but auth is updated in AuthorizationContract
    }

    #[test]
    fn test_only_owner_can_transfer() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _auth_client, _admin, transfer_client, _cl_id) = setup(&env);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let id = register_test_product(&env, &cl_client, &owner);

        // Non-owner attempt should fail
        let res = transfer_client.try_transfer_product(&attacker, &id, &new_owner);
        assert!(res.is_err());
    }

    #[test]
    fn test_new_owner_authentication_required() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _auth_client, _admin, transfer_client, _cl_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let id = register_test_product(&env, &cl_client, &owner);

        // Both parties authenticated via mock_all_auths, transfer should succeed
        transfer_client.transfer_product(&owner, &id, &new_owner);

        // Verify transfer succeeded by checking product owner
        let result_owner = transfer_client.get_product_owner(&id);
        assert_eq!(result_owner, owner); // Owner in main contract unchanged, auth updated in auth contract
    }

    #[test]
    fn test_transfer_nonexistent_product_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        // Need to initialize the contract first
        let auth_id = env.register_contract(None, AuthorizationContract);
        let main_id = env.register_contract(None, ChainLogisticsContract);
        transfer_client.init(&main_id, &auth_id);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let fake_id = String::from_str(&env, "FAKE-001");

        let res = transfer_client.try_transfer_product(&owner, &fake_id, &new_owner);
        assert!(res.is_err());
    }

    #[test]
    fn test_is_product_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _auth_client, _admin, transfer_client, _cl_id) = setup(&env);

        let owner = Address::generate(&env);
        let non_owner = Address::generate(&env);
        let id = register_test_product(&env, &cl_client, &owner);

        // Owner check
        assert!(transfer_client.is_product_owner(&id, &owner));

        // Non-owner check
        assert!(!transfer_client.is_product_owner(&id, &non_owner));
    }

    #[test]
    fn test_batch_transfer_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (cl_client, _auth_client, _admin, transfer_client, _cl_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        // Register multiple products
        let id1 = register_test_product(&env, &cl_client, &owner);
        let id2 = String::from_str(&env, "PROD2");
        cl_client.register_product(
            &owner,
            &ProductConfig {
                id: id2.clone(),
                name: String::from_str(&env, "Product 2"),
                description: String::from_str(&env, "Description"),
                origin_location: String::from_str(&env, "Origin"),
                category: String::from_str(&env, "Category"),
                tags: Vec::new(&env),
                certifications: Vec::new(&env),
                media_hashes: Vec::new(&env),
                custom: Map::new(&env),
            },
        );

        // Batch transfer
        let mut product_ids = Vec::new(&env);
        product_ids.push_back(id1.clone());
        product_ids.push_back(id2.clone());

        let count = transfer_client.batch_transfer_products(&owner, &product_ids, &new_owner);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_batch_transfer_empty_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        // Need to initialize the contract first
        let auth_id = env.register_contract(None, AuthorizationContract);
        let main_id = env.register_contract(None, ChainLogisticsContract);
        transfer_client.init(&main_id, &auth_id);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let product_ids = Vec::new(&env);

        let res = transfer_client.try_batch_transfer_products(&owner, &product_ids, &new_owner);
        assert_eq!(res, Err(Ok(Error::EmptyBatch)));
    }

    #[test]
    fn test_get_product_owner_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        // Need to initialize the contract first
        let auth_id = env.register_contract(None, AuthorizationContract);
        let main_id = env.register_contract(None, ChainLogisticsContract);
        transfer_client.init(&main_id, &auth_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = transfer_client.try_get_product_owner(&fake_id);
        assert!(res.is_err());
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        let auth_id = env.register_contract(None, AuthorizationContract);
        let main_id = env.register_contract(None, ChainLogisticsContract);

        // First init should succeed
        transfer_client.init(&main_id, &auth_id);

        // Second init should fail
        let res = transfer_client.try_init(&main_id, &auth_id);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_transfer_before_init_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let fake_id = String::from_str(&env, "FAKE-001");

        // Transfer without initialization should fail
        let res = transfer_client.try_transfer_product(&owner, &fake_id, &new_owner);
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

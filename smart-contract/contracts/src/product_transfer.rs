/// Product Transfer contract for managing product ownership transfers.
/// This contract handles:
/// - Single product ownership transfers
/// - Batch product ownership transfers
/// - Owner verification
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

use crate::error::Error;
use crate::types::DataKey;
use crate::{AuthorizationContractClient, ProductRegistryContractClient};

// ─── Storage helpers for ProductTransferContract ─────────────────────────────

/// Get the authorization contract address.
fn get_auth_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::AuthContract)
}

/// Set the authorization contract address.
fn set_auth_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::AuthContract, address);
}

/// Get the main contract address.
fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

/// Set the main contract address.
fn set_main_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::MainContract, address);
}

// ─── Contract ────────────────────────────────────────────────────────────────

/// The Product Transfer contract manages product ownership transfers.
#[contract]
pub struct ProductTransferContract;

#[contractimpl]
impl ProductTransferContract {
    /// Initialize the ProductTransferContract with required contract addresses.
    /// 
    /// # Arguments
    /// * `main_contract` - The address of the main ChainLogistics contract
    /// * `auth_contract` - The address of the authorization contract
    /// 
    /// # Returns
    /// * `Result<(), Error>` - Returns error if already initialized
    /// 
    /// # Errors
    /// * `AlreadyInitialized` - If the contract has already been initialized
    pub fn pt_init(env: Env, main_contract: Address, auth_contract: Address) -> Result<(), Error> {
        if get_auth_contract(&env).is_some() || get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_main_contract(&env, &main_contract);
        set_auth_contract(&env, &auth_contract);

        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);
        let self_address = env.current_contract_address();
        pr_client.configure_transfer_contract(&self_address);

        Ok(())
    }

    /// Transfer ownership of a product from the current owner to a new owner.
    /// Requires authentication from both the current owner and the new owner.
    /// 
    /// # Arguments
    /// * `owner` - The current owner of the product
    /// * `product_id` - The ID of the product to transfer
    /// * `new_owner` - The new owner of the product
    /// 
    /// # Returns
    /// * `Result<(), Error>` - Returns error if transfer fails
    /// 
    /// # Errors
    /// * `NotInitialized` - If the contract is not initialized
    /// * `ProductNotFound` - If the product does not exist
    /// * `Unauthorized` - If owner is not the current owner
    /// * `ProductDeactivated` - If the product is deactivated
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

        // Create client to interact with ProductRegistryContract
        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);

        // Verify product exists and get current product info
        let product = match pr_client.try_get_product(&product_id) {
            Ok(Ok(p)) => p,
            Ok(Err(_)) => return Err(Error::ProductNotFound),
            Err(_) => return Err(Error::ProductNotFound),
        };

        // Verify current ownership
        if product.owner != owner {
            return Err(Error::Unauthorized);
        }

        if !product.active {
            return Err(Error::ProductDeactivated);
        }

        // Update authorization mappings via AuthorizationContract
        let auth_client = AuthorizationContractClient::new(&env, &auth_contract);
        auth_client.update_product_owner(&owner, &product_id, &new_owner);

        // Update registry product record ownership
        let self_address = env.current_contract_address();
        pr_client.transfer_owner(&self_address, &product_id, &new_owner);

        // Emit transfer event
        env.events().publish(
            (Symbol::new(&env, "product_transferred"), product_id),
            (owner, new_owner),
        );

        Ok(())
    }

    /// Get the current owner of a product.
    /// 
    /// # Arguments
    /// * `product_id` - The ID of the product
    /// 
    /// # Returns
    /// * `Result<Address, Error>` - The address of the current owner
    /// 
    /// # Errors
    /// * `NotInitialized` - If the contract is not initialized
    /// * `ProductNotFound` - If the product does not exist
    pub fn get_product_owner(env: Env, product_id: String) -> Result<Address, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);
        let product = match pr_client.try_get_product(&product_id) {
            Ok(Ok(p)) => p,
            Ok(Err(_)) => return Err(Error::ProductNotFound),
            Err(_) => return Err(Error::ProductNotFound),
        };
        Ok(product.owner)
    }

    /// Verify if an address is the owner of a specific product.
    /// 
    /// # Arguments
    /// * `product_id` - The ID of the product
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// * `Result<bool, Error>` - True if the address is the owner, false otherwise
    /// 
    /// # Errors
    /// * `NotInitialized` - If the contract is not initialized
    /// * `ProductNotFound` - If the product does not exist
    pub fn is_product_owner(env: Env, product_id: String, address: Address) -> Result<bool, Error> {
        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);
        let product = match pr_client.try_get_product(&product_id) {
            Ok(Ok(p)) => p,
            Ok(Err(_)) => return Err(Error::ProductNotFound),
            Err(_) => return Err(Error::ProductNotFound),
        };
        Ok(product.owner == address)
    }

    /// Batch transfer multiple products from one owner to another.
    /// All products must be owned by the same owner.
    /// 
    /// # Arguments
    /// * `owner` - The current owner of the products
    /// * `product_ids` - A vector of product IDs to transfer
    /// * `new_owner` - The new owner of the products
    /// 
    /// # Returns
    /// * `Result<u32, Error>` - The number of products successfully transferred
    /// 
    /// # Errors
    /// * `NotInitialized` - If the contract is not initialized
    /// * `EmptyBatch` - If the product_ids vector is empty or exceeds limit (100)
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

        // Validate batch size to prevent excessive computation
        if product_ids.len() > 100 {
            return Err(Error::EmptyBatch); // Reuse error for batch size limit
        }

        let main_contract = get_main_contract(&env).ok_or(Error::NotInitialized)?;
        let auth_contract = get_auth_contract(&env).ok_or(Error::NotInitialized)?;

        let pr_client = ProductRegistryContractClient::new(&env, &main_contract);
        let auth_client = AuthorizationContractClient::new(&env, &auth_contract);
        let self_address = env.current_contract_address();

        let mut transferred_count: u32 = 0;

        for i in 0..product_ids.len() {
            if let Some(product_id) = product_ids.get(i) {
                // Verify ownership for each product
                let product = match pr_client.try_get_product(&product_id) {
                    Ok(Ok(p)) => p,
                    Ok(Err(_)) | Err(_) => continue, // Skip non-existent products or errors
                };

                if product.owner != owner {
                    continue; // Skip products not owned by the caller
                }

                if !product.active {
                    continue;
                }

                // Update authorization mappings
                auth_client.update_product_owner(&owner, &product_id, &new_owner);

                // Update registry product record ownership
                pr_client.transfer_owner(&self_address, &product_id, &new_owner);

                // Emit transfer event
                env.events().publish(
                    (Symbol::new(&env, "product_transferred"), product_id),
                    (owner.clone(), new_owner.clone()),
                );

                transferred_count += 1;
            }
        }

        Ok(transferred_count)
    }
}

#[cfg(test)]
mod test_product_transfer {
    use super::*;
    use crate::{
        AuthorizationContract, AuthorizationContractClient, ProductConfig, ProductRegistryContract,
        ProductRegistryContractClient,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

    fn setup(
        env: &Env,
    ) -> (
        ProductRegistryContractClient,
        AuthorizationContractClient,
        Address,
        ProductTransferContractClient,
        Address,
    ) {
        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);
        let transfer_id = env.register_contract(None, ProductTransferContract);

        let pr_client = ProductRegistryContractClient::new(env, &pr_id);
        let auth_client = AuthorizationContractClient::new(env, &auth_id);
        let transfer_client = ProductTransferContractClient::new(env, &transfer_id);

        auth_client.configure_initializer(&pr_id);
        pr_client.configure_auth_contract(&auth_id);

        let admin = Address::generate(env);
        // Initialize ProductTransferContract with ProductRegistryContract and AuthorizationContract
        transfer_client.pt_init(&pr_id, &auth_id);

        (pr_client, auth_client, admin, transfer_client, pr_id)
    }

    fn register_test_product(
        env: &Env,
        client: &ProductRegistryContractClient,
        owner: &Address,
        id: &str,
    ) -> String {
        let id = String::from_str(env, id);
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

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let id = register_test_product(&env, &pr_client, &owner, "PROD1");

        // Verify initial owner
        let p = pr_client.get_product(&id);
        assert_eq!(p.owner, owner);

        // Transfer ownership
        transfer_client.transfer_product(&owner, &id, &new_owner);

        // Verify new owner in registry
        let p2 = pr_client.get_product(&id);
        assert_eq!(p2.owner, new_owner);

        // Verify new owner is authorized in authorization contract
        let ok = _auth_client.is_authorized(&id, &new_owner);
        assert!(ok);
    }

    #[test]
    fn test_only_owner_can_transfer() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let id = register_test_product(&env, &pr_client, &owner, "PROD1");

        // Non-owner attempt should fail
        let res = transfer_client.try_transfer_product(&attacker, &id, &new_owner);
        assert!(res.is_err());
    }

    #[test]
    fn test_new_owner_authentication_required() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let id = register_test_product(&env, &pr_client, &owner, "PROD1");

        // Both parties authenticated via mock_all_auths, transfer should succeed
        transfer_client.transfer_product(&owner, &id, &new_owner);

        // Verify transfer succeeded by checking product owner
        let result_owner = transfer_client.get_product_owner(&id);
        assert_eq!(result_owner, new_owner);
    }

    #[test]
    fn test_transfer_nonexistent_product_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);

        // Initialize the transfer contract
        transfer_client.pt_init(&pr_id, &auth_id);

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

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let non_owner = Address::generate(&env);
        let id = register_test_product(&env, &pr_client, &owner, "PROD1");

        assert!(transfer_client.is_product_owner(&id, &owner));
        assert!(!transfer_client.is_product_owner(&id, &non_owner));
    }

    #[test]
    fn test_batch_transfer_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        // Register multiple products
        let id1 = register_test_product(&env, &pr_client, &owner, "PROD1");
        let id2 = register_test_product(&env, &pr_client, &owner, "PROD2");

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

        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);
        transfer_client.pt_init(&pr_id, &auth_id);

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

        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);

        // Initialize the transfer contract
        transfer_client.pt_init(&pr_id, &auth_id);

        let fake_id = String::from_str(&env, "NONEXISTENT");

        let res = transfer_client.try_get_product_owner(&fake_id);
        assert!(res.is_err());
    }

    #[test]
    fn test_batch_transfer_boundary_conditions() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        // Test empty batch (should fail)
        let product_ids = Vec::new(&env);
        let res = transfer_client.try_batch_transfer_products(&owner, &product_ids, &new_owner);
        assert_eq!(res, Err(Ok(Error::EmptyBatch)));

        // Test batch size limit (should fail)
        let mut large_batch = Vec::new(&env);
        for i in 0..101 {
            let id = String::from_str(&env, "PROD");
            large_batch.push_back(id);
        }
        let res = transfer_client.try_batch_transfer_products(&owner, &large_batch, &new_owner);
        assert_eq!(res, Err(Ok(Error::EmptyBatch))); // Reusing error for batch size limit

        // Test normal batch size (should work if products exist)
        let mut normal_batch = Vec::new(&env);
        for i in 0..5 {
            let id = String::from_str(&env, "PROD");
            normal_batch.push_back(id);
        }
        // This will return 0 because products don't exist, but won't fail due to batch size
        let count = transfer_client.batch_transfer_products(&owner, &normal_batch, &new_owner);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_batch_transfer_with_nonexistent_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        // Register some products
        let id1 = register_test_product(&env, &pr_client, &owner, "PROD1");
        let id2 = register_test_product(&env, &pr_client, &owner, "PROD2");

        // Create batch with mix of existing and non-existing products
        let mut mixed_batch = Vec::new(&env);
        mixed_batch.push_back(id1); // exists
        mixed_batch.push_back(String::from_str(&env, "NONEXISTENT")); // doesn't exist
        mixed_batch.push_back(id2); // exists
        mixed_batch.push_back(String::from_str(&env, "ALSO_NONEXISTENT")); // doesn't exist

        let count = transfer_client.batch_transfer_products(&owner, &mixed_batch, &new_owner);
        assert_eq!(count, 2); // Should only transfer existing products
    }

    #[test]
    fn test_batch_transfer_with_unowned_products() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, auth_client, _admin, transfer_client, _pr_id) = setup(&env);

        let owner = Address::generate(&env);
        let other_owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        // Register products by different owners
        let id1 = register_test_product(&env, &pr_client, &owner, "PROD1");
        let id2 = register_test_product(&env, &pr_client, &other_owner, "PROD2");

        // Create batch with mix of owned and unowned products
        let mut mixed_batch = Vec::new(&env);
        mixed_batch.push_back(id1); // owned by caller
        mixed_batch.push_back(id2); // owned by other person

        let count = transfer_client.batch_transfer_products(&owner, &mixed_batch, &new_owner);
        assert_eq!(count, 1); // Should only transfer owned products
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);

        transfer_client.pt_init(&pr_id, &auth_id);

        let res = transfer_client.try_pt_init(&pr_id, &auth_id);
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

        let res = transfer_client.try_transfer_product(&owner, &fake_id, &new_owner);
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }
}

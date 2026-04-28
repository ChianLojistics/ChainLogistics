/// Product Transfer contract for managing product ownership transfers.
/// This contract handles:
/// - Single product ownership transfers
/// - Batch product ownership transfers
/// - Owner verification
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

use crate::error::Error;
use crate::types::{BatchProgress, DataKey, GasEstimate, GasPolicy};
use crate::{storage, validation_contract::ValidationContract};
use crate::{AuthorizationContractClient, ProductRegistryContractClient};

const MAX_BATCH_SIZE: u32 = 100;
const RECOMMENDED_CHUNK_SIZE: u32 = 25;
const BASE_COST_UNITS: u64 = 15;
const PER_ITEM_COST_UNITS: u64 = 8;

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

fn gas_policy() -> GasPolicy {
    GasPolicy {
        max_batch_size: MAX_BATCH_SIZE,
        recommended_chunk_size: RECOMMENDED_CHUNK_SIZE,
        base_cost_units: BASE_COST_UNITS,
        per_item_cost_units: PER_ITEM_COST_UNITS,
    }
}

fn estimate_batch(item_count: u32) -> GasEstimate {
    let policy = gas_policy();
    let chunk_count = if item_count == 0 {
        0
    } else {
        item_count.div_ceil(policy.recommended_chunk_size)
    };

    let estimated_cost_units = policy
        .per_item_cost_units
        .checked_mul(item_count as u64)
        .and_then(|x| x.checked_add(policy.base_cost_units))
        .unwrap_or(u64::MAX);

    GasEstimate {
        item_count,
        estimated_cost_units,
        recommended_chunk_size: policy.recommended_chunk_size,
        recommended_chunk_count: chunk_count,
        fits_single_transaction: item_count <= policy.max_batch_size,
    }
}

fn transfer_batch_chunk(
    env: &Env,
    owner: &Address,
    product_ids: &soroban_sdk::Vec<String>,
    new_owner: &Address,
    cursor: u32,
    chunk_size: u32,
) -> Result<BatchProgress, Error> {
    ValidationContract::validate_distinct_addresses(owner, new_owner)?;

    if product_ids.is_empty() {
        return Err(Error::EmptyBatch);
    }

    if product_ids.len() > MAX_BATCH_SIZE {
        return Err(Error::BatchTooLarge);
    }

    if chunk_size == 0 {
        return Err(Error::InvalidInput);
    }

    if chunk_size > MAX_BATCH_SIZE {
        return Err(Error::BatchTooLarge);
    }

    let main_contract = get_main_contract(env).ok_or(Error::NotInitialized)?;
    let auth_contract = get_auth_contract(env).ok_or(Error::NotInitialized)?;

    let pr_client = ProductRegistryContractClient::new(env, &main_contract);
    let auth_client = AuthorizationContractClient::new(env, &auth_contract);
    let self_address = env.current_contract_address();

    if cursor >= product_ids.len() {
        return Ok(BatchProgress {
            requested: product_ids.len(),
            processed: 0,
            succeeded: 0,
            next_cursor: cursor,
            complete: true,
        });
    }

    let end = cursor
        .checked_add(chunk_size)
        .ok_or(Error::ArithmeticOverflow)?
        .min(product_ids.len());
    let mut transferred_count: u32 = 0;

    let transfer_scope = Symbol::new(env, "batch_transfer");
    storage::acquire_reentrancy_lock(env, &transfer_scope)?;

    for i in cursor..end {
        if let Some(product_id) = product_ids.get(i) {
            let product = match pr_client.try_get_product(&product_id) {
                Ok(Ok(p)) => p,
                Ok(Err(_)) | Err(_) => continue,
            };

            if product.owner != *owner || !product.active {
                continue;
            }

            auth_client.update_product_owner(owner, &product_id, new_owner);
            pr_client.transfer_owner(&self_address, &product_id, new_owner);

            env.events().publish(
                (Symbol::new(env, "product_transferred"), product_id),
                (owner.clone(), new_owner.clone()),
            );

            transferred_count = transferred_count
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
        }
    }

    storage::release_reentrancy_lock(env, &transfer_scope);

    Ok(BatchProgress {
        requested: product_ids.len(),
        processed: end - cursor,
        succeeded: transferred_count,
        next_cursor: end,
        complete: end == product_ids.len(),
    })
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
        ValidationContract::validate_contract_address(&env, &main_contract)?;
        ValidationContract::validate_contract_address(&env, &auth_contract)?;
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
        ValidationContract::validate_distinct_addresses(&owner, &new_owner)?;
        ValidationContract::non_empty(&product_id)?;
        ValidationContract::max_len(&product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;

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

        let transfer_scope = Symbol::new(&env, "transfer_product");
        storage::acquire_reentrancy_lock(&env, &transfer_scope)?;
        auth_client.update_product_owner(&owner, &product_id, &new_owner);

        // Update registry product record ownership
        let self_address = env.current_contract_address();
        pr_client.transfer_owner(&self_address, &product_id, &new_owner);
        storage::release_reentrancy_lock(&env, &transfer_scope);

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
        ValidationContract::non_empty(&product_id)?;
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
        ValidationContract::non_empty(&product_id)?;
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

        let progress =
            transfer_batch_chunk(&env, &owner, &product_ids, &new_owner, 0, product_ids.len())?;

        Ok(progress.succeeded)
    }

    pub fn get_batch_transfer_gas_policy(_env: Env) -> GasPolicy {
        gas_policy()
    }

    pub fn estimate_batch_transfer(_env: Env, item_count: u32) -> GasEstimate {
        estimate_batch(item_count)
    }

    pub fn batch_transfer_products_chunk(
        env: Env,
        owner: Address,
        product_ids: soroban_sdk::Vec<String>,
        new_owner: Address,
        cursor: u32,
        chunk_size: u32,
    ) -> Result<BatchProgress, Error> {
        owner.require_auth();
        new_owner.require_auth();
        transfer_batch_chunk(&env, &owner, &product_ids, &new_owner, cursor, chunk_size)
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
        assert_eq!(res, Err(Ok(Error::BatchTooLarge)));

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
    fn test_batch_transfer_chunk_progress_and_policy() {
        let env = Env::default();
        env.mock_all_auths();

        let (pr_client, _auth_client, _admin, transfer_client, _pr_id) = setup(&env);
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        let mut product_ids = Vec::new(&env);
        for id_str in ["PROD0", "PROD1", "PROD2", "PROD3", "PROD4"] {
            let id = register_test_product(&env, &pr_client, &owner, id_str);
            product_ids.push_back(id);
        }

        let policy = transfer_client.get_batch_transfer_gas_policy();
        assert_eq!(policy.max_batch_size, MAX_BATCH_SIZE);
        assert_eq!(policy.recommended_chunk_size, RECOMMENDED_CHUNK_SIZE);

        let estimate = transfer_client.estimate_batch_transfer(&60);
        assert_eq!(estimate.recommended_chunk_count, 3);
        assert!(estimate.fits_single_transaction);

        let first =
            transfer_client.batch_transfer_products_chunk(&owner, &product_ids, &new_owner, &0, &2);
        assert_eq!(first.processed, 2);
        assert_eq!(first.succeeded, 2);
        assert_eq!(first.next_cursor, 2);
        assert!(!first.complete);

        let second =
            transfer_client.batch_transfer_products_chunk(&owner, &product_ids, &new_owner, &2, &2);
        assert_eq!(second.processed, 2);
        assert_eq!(second.succeeded, 2);
        assert_eq!(second.next_cursor, 4);
        assert!(!second.complete);

        let third =
            transfer_client.batch_transfer_products_chunk(&owner, &product_ids, &new_owner, &4, &2);
        assert_eq!(third.processed, 1);
        assert_eq!(third.succeeded, 1);
        assert!(third.complete);
    }

    #[test]
    fn test_batch_transfer_estimates_are_monotonic() {
        let env = Env::default();
        env.mock_all_auths();

        let transfer_id = env.register_contract(None, ProductTransferContract);
        let transfer_client = ProductTransferContractClient::new(&env, &transfer_id);

        let auth_id = env.register_contract(None, AuthorizationContract);
        let pr_id = env.register_contract(None, ProductRegistryContract);
        transfer_client.pt_init(&pr_id, &auth_id);

        let mut last_cost = 0u64;
        for item_count in 1..=MAX_BATCH_SIZE {
            let estimate = transfer_client.estimate_batch_transfer(&item_count);
            assert!(estimate.estimated_cost_units > last_cost);
            assert!(estimate.recommended_chunk_count >= 1);
            last_cost = estimate.estimated_cost_units;
        }
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

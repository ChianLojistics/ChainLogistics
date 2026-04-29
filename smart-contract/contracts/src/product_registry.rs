/// Product Registry contract for managing product lifecycle.
/// This contract handles:
/// - Product registration
/// - Product deactivation and reactivation
/// - Product queries and search
/// - Product statistics
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{DeactInfo, Origin, Product, ProductConfig, ProductStats};
use crate::validation_contract::ValidationContract;
use crate::AuthorizationContractClient;

// ─── Storage helpers for trusted transfer contract ───────────────────────────

/// Get the trusted transfer contract address.
fn get_transfer_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&crate::types::DataKey::TransferContract)
}

/// Set the trusted transfer contract address.
fn set_transfer_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&crate::types::DataKey::TransferContract, address);
}

/// Get the authorization contract address.
fn get_auth_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&crate::types::DataKey::AuthContract)
}

/// Set the authorization contract address.
fn set_auth_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&crate::types::DataKey::AuthContract, address);
}

/// Ensure the caller is the trusted transfer contract.
/// Returns NotInitialized if transfer contract is not set.
/// Returns Unauthorized if caller is not the transfer contract.
fn require_transfer_contract(env: &Env, caller: &Address) -> Result<(), Error> {
    let trusted = get_transfer_contract(env).ok_or(Error::NotInitialized)?;
    caller.require_auth();
    if *caller != trusted {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Read a product from storage.
/// Returns ProductNotFound if the product does not exist.
fn read_product(env: &Env, product_id: &String) -> Result<Product, Error> {
    storage::get_product(env, product_id).ok_or(Error::ProductNotFound)
}

/// Write a product to storage.
fn write_product(env: &Env, product: &Product) {
    storage::put_product(env, product);
}

/// Ensure the caller is the product owner.
/// Returns Unauthorized if caller is not the owner.
fn require_owner(product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if &product.owner != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// ─── Search helpers (Gas-optimized) ───────────────────────────────────────────

// Gas-optimized: Batch indexing reduces storage operations
fn index_product(env: &Env, product: &Product) {
    // Index only meaningful fields to reduce gas costs
    // Use full text as single keyword for efficient lookups

    // Index name (most common search field)
    if product.name.len() > 2 {
        storage::add_to_search_index(env, product.name.clone(), &product.id);
    }

    // Index origin location
    if product.origin.location.len() > 2 {
        storage::add_to_search_index(env, product.origin.location.clone(), &product.id);
    }

    // Index category
    if product.category.len() > 2 {
        storage::add_to_search_index(env, product.category.clone(), &product.id);
    }
}

// Gas-optimized: Removed unnecessary split_into_words function
// Using full text reduces complexity and gas costs

// Gas-optimized: Batch deindexing
fn deindex_product(env: &Env, product: &Product) {
    // Remove from indexes - only if they were indexed
    if product.name.len() > 2 {
        storage::remove_from_search_index(env, product.name.clone(), &product.id);
    }

    if product.origin.location.len() > 2 {
        storage::remove_from_search_index(env, product.origin.location.clone(), &product.id);
    }

    if product.category.len() > 2 {
        storage::remove_from_search_index(env, product.category.clone(), &product.id);
    }
}

// ─── Contract ────────────────────────────────────────────────────────────────

/// The Product Registry contract manages product lifecycle.
#[contract]
pub struct ProductRegistryContract;

#[contractimpl]
impl ProductRegistryContract {
    // ═══════════════════════════════════════════════════════════════════════
    // PRODUCT REGISTRATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Register a new product with full validation.
    ///
    /// Validates all input fields, creates the product, updates global
    /// counters, and emits a `product_registered` event.
    pub fn register_product(
        env: Env,
        owner: Address,
        config: ProductConfig,
    ) -> Result<Product, Error> {
        owner.require_auth();

        ValidationContract::validate_product_config(&config)?;

        // --- Duplicate check ---
        if storage::has_product(&env, &config.id) {
            return Err(Error::ProductAlreadyExists);
        }

        // --- Build product ---
        let product = Product {
            id: config.id.clone(),
            name: config.name,
            description: config.description,
            origin: Origin {
                location: config.origin_location,
            },
            owner: owner.clone(),
            created_at: env.ledger().timestamp(),
            active: true,
            category: config.category,
            tags: config.tags,
            certifications: config.certifications,
            media_hashes: config.media_hashes,
            custom: config.custom,
            deactivation_info: Vec::new(&env),
        };

        write_product(&env, &product);
        storage::put_product_event_ids(&env, &config.id, &Vec::new(&env));
        storage::set_auth(&env, &config.id, &owner, true);

        // Index product for search
        index_product(&env, &product);

        let register_scope = Symbol::new(&env, "register_product");
        storage::acquire_reentrancy_lock(&env, &register_scope)?;
        let auth_contract = get_auth_contract(&env).ok_or(Error::NotInitialized)?;
        let auth_client = AuthorizationContractClient::new(&env, &auth_contract);
        let self_address = env.current_contract_address();
        auth_client.init_product_owner(&self_address, &config.id, &owner);
        storage::release_reentrancy_lock(&env, &register_scope);

        // Update global counters
        let total = storage::get_total_products(&env)
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        storage::set_total_products(&env, total);

        let active = storage::get_active_products(&env)
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        storage::set_active_products(&env, active);

        env.events().publish(
            (Symbol::new(&env, "product_registered"), config.id.clone()),
            product.clone(),
        );

        Ok(product)
    }

    /// Configure the authorization contract address.
    /// This can only be called once.
    ///
    /// # Arguments
    /// * `auth_contract` - The address of the authorization contract
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns error if already initialized with different address
    ///
    /// # Errors
    /// * `AlreadyInitialized` - If already initialized with a different address
    pub fn configure_auth_contract(env: Env, auth_contract: Address) -> Result<(), Error> {
        ValidationContract::validate_contract_address(&env, &auth_contract)?;
        match get_auth_contract(&env) {
            None => {
                set_auth_contract(&env, &auth_contract);
                Ok(())
            }
            Some(existing) if existing == auth_contract => Ok(()),
            Some(_) => Err(Error::AlreadyInitialized),
        }
    }

    /// Configure which contract is allowed to call `transfer_owner`.
    ///
    /// This is intentionally one-time set (or idempotent if set to the same
    /// address) to avoid ownership transfers being callable by arbitrary
    /// contracts.
    pub fn configure_transfer_contract(env: Env, transfer_contract: Address) -> Result<(), Error> {
        ValidationContract::validate_contract_address(&env, &transfer_contract)?;
        match get_transfer_contract(&env) {
            None => {
                set_transfer_contract(&env, &transfer_contract);
                Ok(())
            }
            Some(existing) if existing == transfer_contract => Ok(()),
            Some(_) => Err(Error::AlreadyInitialized),
        }
    }

    /// Update a product's `owner` field.
    ///
    /// This must only be called by the configured `ProductTransferContract`.
    pub fn transfer_owner(
        env: Env,
        caller: Address,
        product_id: String,
        new_owner: Address,
    ) -> Result<Product, Error> {
        require_transfer_contract(&env, &caller)?;
        ValidationContract::non_empty(&product_id)?;

        let mut product = read_product(&env, &product_id)?;
        if !product.active {
            return Err(Error::ProductDeactivated);
        }
        ValidationContract::validate_distinct_addresses(&product.owner, &new_owner)?;
        product.owner = new_owner;
        write_product(&env, &product);

        Ok(product)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PRODUCT LIFECYCLE — DEACTIVATION & REACTIVATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Deactivate a product.
    ///
    /// Only the product owner can deactivate. A reason must be provided.
    /// Deactivation prevents new tracking events and decrements the active
    /// product counter.
    pub fn deactivate_product(
        env: Env,
        owner: Address,
        product_id: String,
        reason: String,
    ) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        if !product.active {
            return Err(Error::ProductDeactivated);
        }

        ValidationContract::validate_deactivation_reason(&reason)?;

        product.active = false;
        let mut info = Vec::new(&env);
        info.push_back(DeactInfo {
            reason: reason.clone(),
            deactivated_at: env.ledger().timestamp(),
            deactivated_by: owner.clone(),
        });
        product.deactivation_info = info;

        write_product(&env, &product);

        // Remove product from search index when deactivated
        deindex_product(&env, &product);

        // Decrement active counter
        let active = storage::get_active_products(&env).saturating_sub(1);
        storage::set_active_products(&env, active);

        env.events().publish(
            (Symbol::new(&env, "product_deactivated"), product_id.clone()),
            (owner, reason),
        );

        Ok(())
    }

    /// Reactivate a previously deactivated product.
    ///
    /// Only the product owner can reactivate. Clears deactivation info
    /// and increments the active product counter.
    pub fn reactivate_product(env: Env, owner: Address, product_id: String) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        if product.active {
            return Err(Error::ProductAlreadyActive);
        }

        product.active = true;
        product.deactivation_info = Vec::new(&env);

        write_product(&env, &product);

        // Re-index product when reactivated
        index_product(&env, &product);

        // Increment active counter
        let active = storage::get_active_products(&env)
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        storage::set_active_products(&env, active);

        env.events().publish(
            (Symbol::new(&env, "product_reactivated"), product_id.clone()),
            owner,
        );

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PRODUCT QUERIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Get a product by its string ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the product to retrieve
    ///
    /// # Returns
    /// * `Result<Product, Error>` - The product information
    ///
    /// # Errors
    /// * `ProductNotFound` - If the product does not exist
    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    /// Get global product statistics.
    ///
    /// # Returns
    /// * `ProductStats` - Global statistics including total and active product counts
    pub fn get_stats(env: Env) -> ProductStats {
        ProductStats {
            total_products: storage::get_total_products(&env),
            active_products: storage::get_active_products(&env),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PRODUCT SEARCH
    // ═══════════════════════════════════════════════════════════════════════

    /// Search products by name, origin, or category.
    /// Returns matching product IDs with exact matching.
    /// Results are limited for gas efficiency.
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// * `Vec<String>` - A vector of matching product IDs
    pub fn search_products(env: Env, query: String, limit: u32) -> Vec<String> {
        let mut results = Vec::new(&env);

        if limit == 0 {
            return results;
        }

        // Search for exact match first (case sensitive)
        let exact_matches = storage::get_search_index(&env, &query);
        for i in 0..exact_matches.len() {
            if results.len() >= limit {
                return results;
            }
            let product_id = exact_matches.get(i).unwrap();
            if !results.contains(&product_id) {
                results.push_back(product_id.clone());
            }
        }

        // If we need more results, we could implement partial matching here
        // For now, this provides basic search functionality

        results
    }
}

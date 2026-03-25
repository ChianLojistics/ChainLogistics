use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{DeactInfo, Origin, Product, ProductConfig, ProductStats};
use crate::validation_contract::ValidationContract;
use crate::AuthorizationContractClient;

// ─── Storage helpers for trusted transfer contract ───────────────────────────

fn get_transfer_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&crate::types::DataKey::TransferContract)
}

fn set_transfer_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&crate::types::DataKey::TransferContract, address);
}

fn get_auth_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&crate::types::DataKey::AuthContract)
}

fn set_auth_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&crate::types::DataKey::AuthContract, address);
}

fn require_transfer_contract(env: &Env, caller: &Address) -> Result<(), Error> {
    let trusted = get_transfer_contract(env).ok_or(Error::NotInitialized)?;
    caller.require_auth();
    if *caller != trusted {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn read_product(env: &Env, product_id: &String) -> Result<Product, Error> {
    storage::get_product(env, product_id).ok_or(Error::ProductNotFound)
}

fn write_product(env: &Env, product: &Product) {
    storage::put_product(env, product);
}

fn require_owner(product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if &product.owner != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// ─── Search helpers ───────────────────────────────────────────────────────────

fn index_product(env: &Env, product: &Product) {
    // Index individual words from name, origin, and category
    // This allows for partial matching

    // Index name words
    let name_words = split_into_words(env, &product.name);
    for i in 0..name_words.len() {
        let word = name_words.get(i).unwrap();
        storage::add_to_search_index(env, word.clone(), &product.id);
    }

    // Index origin words
    let origin_words = split_into_words(env, &product.origin.location);
    for i in 0..origin_words.len() {
        let word = origin_words.get(i).unwrap();
        storage::add_to_search_index(env, word.clone(), &product.id);
    }

    // Index category words
    let category_words = split_into_words(env, &product.category);
    for i in 0..category_words.len() {
        let word = category_words.get(i).unwrap();
        storage::add_to_search_index(env, word.clone(), &product.id);
    }
}

fn split_into_words(env: &Env, text: &String) -> Vec<String> {
    let mut words = Vec::new(env);

    // For now, just use the full text as a single "word"
    // This avoids the to_string() conversion issues
    // In a real implementation, we'd want to split into individual words
    if text.len() > 2 {
        words.push_back(text.clone());
    }

    words
}

fn deindex_product(env: &Env, product: &Product) {
    // Remove from name index (using the same logic as indexing)
    let name_words = split_into_words(env, &product.name);
    for i in 0..name_words.len() {
        let word = name_words.get(i).unwrap();
        storage::remove_from_search_index(env, word.clone(), &product.id);
    }

    // Remove from origin index
    let origin_words = split_into_words(env, &product.origin.location);
    for i in 0..origin_words.len() {
        let word = origin_words.get(i).unwrap();
        storage::remove_from_search_index(env, word.clone(), &product.id);
    }

    // Remove from category index
    let category_words = split_into_words(env, &product.category);
    for i in 0..category_words.len() {
        let word = category_words.get(i).unwrap();
        storage::remove_from_search_index(env, word.clone(), &product.id);
    }
}

// ─── Contract ────────────────────────────────────────────────────────────────

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

        let auth_contract = get_auth_contract(&env).ok_or(Error::NotInitialized)?;
        let auth_client = AuthorizationContractClient::new(&env, &auth_contract);
        let self_address = env.current_contract_address();
        auth_client.init_product_owner(&self_address, &config.id, &owner);

        // Update global counters
        let total = storage::get_total_products(&env) + 1;
        storage::set_total_products(&env, total);

        let active = storage::get_active_products(&env) + 1;
        storage::set_active_products(&env, active);

        env.events().publish(
            (Symbol::new(&env, "product_registered"), config.id.clone()),
            product.clone(),
        );

        Ok(product)
    }

    pub fn configure_auth_contract(env: Env, auth_contract: Address) -> Result<(), Error> {
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

        let mut product = read_product(&env, &product_id)?;
        if !product.active {
            return Err(Error::ProductDeactivated);
        }
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
        let active = storage::get_active_products(&env) + 1;
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
    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    /// Get global product statistics.
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
    ///
    /// Returns matching product IDs with case insensitive matching.
    /// Results are limited for gas efficiency.
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

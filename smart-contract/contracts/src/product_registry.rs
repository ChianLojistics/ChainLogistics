use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::types::{DeactInfo, Origin, Product, ProductConfig, ProductStats};
use crate::{storage, validation};

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

        // --- Validation ---
        const MAX_ID_LEN: u32 = 64;
        const MAX_NAME_LEN: u32 = 128;
        const MAX_ORIGIN_LEN: u32 = 128;
        const MAX_CATEGORY_LEN: u32 = 64;
        const MAX_DESC_LEN: u32 = 512;
        const MAX_TAGS: u32 = 20;
        const MAX_TAG_LEN: u32 = 64;
        const MAX_CERTS: u32 = 50;
        const MAX_MEDIA: u32 = 50;
        const MAX_CUSTOM: u32 = 20;
        const MAX_CUSTOM_VAL_LEN: u32 = 256;

        if !validation::non_empty(&config.id) {
            return Err(Error::InvalidProductId);
        }
        if !validation::max_len(&config.id, MAX_ID_LEN) {
            return Err(Error::ProductIdTooLong);
        }
        if !validation::non_empty(&config.name) {
            return Err(Error::InvalidProductName);
        }
        if !validation::max_len(&config.name, MAX_NAME_LEN) {
            return Err(Error::ProductNameTooLong);
        }
        if !validation::non_empty(&config.origin_location) {
            return Err(Error::InvalidOrigin);
        }
        if !validation::max_len(&config.origin_location, MAX_ORIGIN_LEN) {
            return Err(Error::OriginTooLong);
        }
        if !validation::non_empty(&config.category) {
            return Err(Error::InvalidCategory);
        }
        if !validation::max_len(&config.category, MAX_CATEGORY_LEN) {
            return Err(Error::CategoryTooLong);
        }
        if !validation::max_len(&config.description, MAX_DESC_LEN) {
            return Err(Error::DescriptionTooLong);
        }
        if config.tags.len() > MAX_TAGS {
            return Err(Error::TooManyTags);
        }
        for i in 0..config.tags.len() {
            if !validation::max_len(&config.tags.get_unchecked(i), MAX_TAG_LEN) {
                return Err(Error::TagTooLong);
            }
        }
        if config.certifications.len() > MAX_CERTS {
            return Err(Error::TooManyCertifications);
        }
        if config.media_hashes.len() > MAX_MEDIA {
            return Err(Error::TooManyMediaHashes);
        }
        if config.custom.len() > MAX_CUSTOM {
            return Err(Error::TooManyCustomFields);
        }
        let custom_keys = config.custom.keys();
        for i in 0..custom_keys.len() {
            let k = custom_keys.get_unchecked(i);
            let v = config.custom.get_unchecked(k);
            if !validation::max_len(&v, MAX_CUSTOM_VAL_LEN) {
                return Err(Error::CustomFieldValueTooLong);
            }
        }

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

        if !validation::non_empty(&reason) {
            return Err(Error::DeactivationReasonRequired);
        }

        product.active = false;
        let mut info = Vec::new(&env);
        info.push_back(DeactInfo {
            reason: reason.clone(),
            deactivated_at: env.ledger().timestamp(),
            deactivated_by: owner.clone(),
        });
        product.deactivation_info = info;

        write_product(&env, &product);

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
    pub fn reactivate_product(
        env: Env,
        owner: Address,
        product_id: String,
    ) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        if product.active {
            return Err(Error::ProductAlreadyActive);
        }

        product.active = true;
        product.deactivation_info = Vec::new(&env);

        write_product(&env, &product);

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
}

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::types::{DeactInfo, Origin, Product, ProductConfig, ProductStats};
use crate::storage;
use crate::validation_contract::ValidationContract;

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

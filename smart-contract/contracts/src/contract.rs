use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::types::{
    DeactInfo, Origin, Product, ProductConfig, ProductStats, TrackingEvent,
    TrackingEventFilter, TrackingEventPage,
};
use crate::error::Error;
use crate::{storage, validation};

// ─── Internal helpers ────────────────────────────────────────────────────────

<<<<<<< nnennaokoye/chainlogistics
fn require_not_paused(env: &Env) -> Result<(), Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    caller.require_auth();
    if &admin != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

/// Reads a product from persistent storage.
/// 
/// Returns an error if the product doesn't exist.
=======
>>>>>>> main
fn read_product(env: &Env, product_id: &String) -> Result<Product, Error> {
    storage::get_product(env, product_id).ok_or(Error::ProductNotFound)
}

/// Writes a product to persistent storage.
/// 
/// Products are stored using Soroban's persistent storage API,
/// ensuring they persist across contract calls and ledger entries.
fn write_product(env: &Env, product: &Product) {
    storage::put_product(env, product);
}

/// Require that `caller` is the product owner (also calls `require_auth()`).
fn require_owner(product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if &product.owner != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

/// Verify that an actor is allowed to add events:
///  - product must be active
///  - caller must be the owner or an explicitly authorized actor
fn require_can_add_event(
    env: &Env,
    product_id: &String,
    product: &Product,
    caller: &Address,
) -> Result<(), Error> {
    caller.require_auth();

    // Deactivated products reject all new events
    if !product.active {
        return Err(Error::ProductDeactivated);
    }

    // Owner always has permission
    if &product.owner == caller {
        return Ok(());
    }

    // Check explicit authorization
    if !storage::is_authorized(env, product_id, caller) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn require_can_add_event(env: &Env, product_id: &String, product: &Product, caller: &Address) -> Result<(), Error> {
    require_can_add_event_internal(env, product_id, product, caller, true)
}

fn page_from_vec(env: &Env, ids: &Vec<u64>, cursor: u32, limit: u32) -> EventIdPage {
    if limit == 0 {
        return EventIdPage {
            ids: Vec::new(env),
            next_cursor: cursor,
        };
    }

    Ok(())
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct ChainLogisticsContract;

#[contractimpl]
impl ChainLogisticsContract {
<<<<<<< nnennaokoye/chainlogistics
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        storage::set_paused(&env, true);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        storage::set_paused(&env, false);
        Ok(())
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        new_admin.require_auth();
        storage::set_admin(&env, &new_admin);
        Ok(())
    }

    /// Registers a new product and stores it in persistent storage.
    /// Products persist across contract calls using Soroban's persistent storage API.
    /// Returns ProductAlreadyExists if the product ID already exists.
=======
    // --- Lifecycle Management ---
    // ═══════════════════════════════════════════════════════════════════════
    // PRODUCT REGISTRATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Register a new product.
>>>>>>> main
    pub fn register_product(
        env: Env,
        owner: Address,
        config: ProductConfig,
    ) -> Result<Product, Error> {
<<<<<<< nnennaokoye/chainlogistics
        require_not_paused(&env)?;
=======
        owner.require_auth();

        // --- Validation ---
>>>>>>> main
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
        if !validation::max_len(&description, MAX_DESCRIPTION_LEN) {
            return Err(Error::DescriptionTooLong);
        }

        if tags.len() > MAX_TAGS {
            return Err(Error::TooManyTags);
        }
        for i in 0..tags.len() {
            let t = tags.get_unchecked(i);
            if !validation::max_len(&t, MAX_TAG_LEN) {
                return Err(Error::TagTooLong);
            }
        }

        if certifications.len() > MAX_CERTIFICATIONS {
            return Err(Error::TooManyCertifications);
        }
        if media_hashes.len() > MAX_MEDIA_HASHES {
            return Err(Error::TooManyMediaHashes);
        }

        if custom.len() > MAX_CUSTOM_FIELDS {
            return Err(Error::TooManyCustomFields);
        }
        let custom_keys = custom.keys();
        for i in 0..custom_keys.len() {
            let k = custom_keys.get_unchecked(i);
            let v = custom.get_unchecked(k);
            if !validation::max_len(&v, MAX_CUSTOM_VALUE_LEN) {
                return Err(Error::CustomFieldValueTooLong);
            }
        }

        if storage::has_product(&env, &id) {
            return Err(Error::ProductAlreadyExists);
        }

        owner.require_auth();

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
        // Owner is implicitly authorized — store explicit auth entry for lookup convenience
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

    /// Reactivate a product.
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
}

#[contractimpl]
impl ChainLogisticsContract {
    // --- Product Queries ---

    /// Get a product by its string ID.
    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    /// Returns all event IDs associated with a product.
    pub fn get_product_event_ids(env: Env, id: String) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &id)?;
        Ok(storage::get_product_event_ids(&env, &id))
    }

    /// Get global product statistics.
    pub fn get_stats(env: Env) -> ProductStats {
        ProductStats {
            total_products: storage::get_total_products(&env),
            active_products: storage::get_active_products(&env),
        }
    }

<<<<<<< nnennaokoye/chainlogistics
    pub fn add_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        require_not_paused(&env)?;
=======
    // ═══════════════════════════════════════════════════════════════════════
    // AUTHORIZATION MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════

    /// Grant an actor the right to add tracking events to a product.
    pub fn add_authorized_actor(
        env: Env,
        owner: Address,
        product_id: String,
        actor: Address,
    ) -> Result<(), Error> {
>>>>>>> main
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, true);
        Ok(())
    }

<<<<<<< nnennaokoye/chainlogistics
    pub fn remove_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        require_not_paused(&env)?;
=======
    /// Revoke an actor's authorization to add events to a product.
    pub fn remove_authorized_actor(
        env: Env,
        owner: Address,
        product_id: String,
        actor: Address,
    ) -> Result<(), Error> {
>>>>>>> main
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, false);
        Ok(())
    }

<<<<<<< nnennaokoye/chainlogistics
    pub fn transfer_product(env: Env, owner: Address, product_id: String, new_owner: Address) -> Result<(), Error> {
        require_not_paused(&env)?;
=======
    /// Check whether an actor is authorized to add events to a product.
    pub fn is_authorized(env: Env, product_id: String, actor: Address) -> Result<bool, Error> {
        let product = read_product(&env, &product_id)?;
        if product.owner == actor {
            return Ok(true);
        }
        Ok(storage::is_authorized(&env, &product_id, &actor))
    }
}

#[contractimpl]
impl ChainLogisticsContract {
    // --- Authorization & Transfer ---

    /// Transfer product ownership to a new address.
    pub fn transfer_product(
        env: Env,
        owner: Address,
        product_id: String,
        new_owner: Address,
    ) -> Result<(), Error> {
>>>>>>> main
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        new_owner.require_auth();

        // Remove old owner's explicit auth entry, add new owner's
        storage::set_auth(&env, &product_id, &owner, false);
        storage::set_auth(&env, &product_id, &new_owner, true);

<<<<<<< nnennaokoye/chainlogistics
    pub fn set_product_active(env: Env, owner: Address, product_id: String, active: bool) -> Result<(), Error> {
        require_not_paused(&env)?;
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        product.active = active;
=======
        product.owner = new_owner.clone();
>>>>>>> main
        write_product(&env, &product);

        env.events().publish(
            (Symbol::new(&env, "product_transferred"), product_id),
            (owner, new_owner),
        );

        Ok(())
    }
}

#[contractimpl]
impl ChainLogisticsContract {
    // --- Tracking Events ---

<<<<<<< nnennaokoye/chainlogistics
    pub fn add_tracking_event(env: Env, actor: Address, product_id: String, event_type: Symbol, data_hash: BytesN<32>, note: String) -> Result<u64, Error> {
        require_not_paused(&env)?;
=======
    /// Add a tracking event to a product.
    pub fn add_tracking_event(
        env: Env,
        actor: Address,
        product_id: String,
        event_type: Symbol,
        location: String,
        data_hash: BytesN<32>,
        note: String,
        metadata: Map<Symbol, String>,
    ) -> Result<u64, Error> {
>>>>>>> main
        let product = read_product(&env, &product_id)?;
        require_can_add_event(&env, &product_id, &product, &actor)?;

        // Validate metadata limits
        const MAX_METADATA_FIELDS: u32 = 20;
        const MAX_METADATA_VALUE_LEN: u32 = 256;

        if metadata.len() > MAX_METADATA_FIELDS {
            return Err(Error::TooManyCustomFields);
        }

        let meta_keys = metadata.keys();
        for i in 0..meta_keys.len() {
            let k = meta_keys.get_unchecked(i);
            let v = metadata.get_unchecked(k);
            if !validation::max_len(&v, MAX_METADATA_VALUE_LEN) {
                return Err(Error::CustomFieldValueTooLong);
            }
        }

        let event_id = storage::next_event_id(&env);
        let event = TrackingEvent {
            event_id,
            product_id: product_id.clone(),
            actor,
            timestamp: env.ledger().timestamp(),
            event_type,
            data_hash,
            note,
        };

        storage::put_event(&env, &event);

        // Append to the product's ordered event list
        let mut ids = storage::get_product_event_ids(&env, &product_id);
        ids.push_back(event_id);
        storage::put_product_event_ids(&env, &product_id, &ids);

        // Index by event type for efficient type-based filtering
        storage::index_event_by_type(&env, &product_id, &event_type, event_id);

        env.events().publish(
            (
                Symbol::new(&env, "tracking_event"),
                product_id.clone(),
                event_id,
            ),
            event.clone(),
        );

        Ok(event_id)
    }

<<<<<<< nnennaokoye/chainlogistics
    pub fn register_products_batch(env: Env, owner: Address, inputs: Vec<ProductRegistrationInput>) -> Result<Vec<Product>, Error> {
        require_not_paused(&env)?;
        const MAX_BATCH: u32 = 100;
=======
    /// Get a single tracking event by its numeric ID.
    pub fn get_event(env: Env, event_id: u64) -> Result<TrackingEvent, Error> {
        storage::get_event(&env, event_id).ok_or(Error::EventNotFound)
    }
>>>>>>> main

    /// Get all events for a product with cursor-based pagination.
    pub fn get_product_events(
        env: Env,
        product_id: String,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let total_count = all_ids.len() as u64;

        let event_ids =
            storage::get_product_event_ids_paginated(&env, &product_id, offset, limit);

        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let eid = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (event_ids.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events for a product filtered by event type, with pagination.
    pub fn get_events_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let total_count =
            storage::get_event_count_by_type(&env, &product_id, &event_type);
        let event_ids =
            storage::get_event_ids_by_type(&env, &product_id, &event_type, offset, limit);

        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let eid = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (event_ids.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events filtered by time range with pagination.
    pub fn get_events_by_time_range(
        env: Env,
        product_id: String,
        start_time: u64,
        end_time: u64,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);

        for i in 0..all_ids.len() {
            let eid = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                if event.timestamp >= start_time && event.timestamp <= end_time {
                    matching_ids.push_back(eid);
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        for i in start..end {
            let eid = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (events.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

<<<<<<< nnennaokoye/chainlogistics
    pub fn add_tracking_events_batch(env: Env, actor: Address, inputs: Vec<TrackingEventInput>) -> Result<Vec<u64>, Error> {
        require_not_paused(&env)?;
        const MAX_BATCH: u32 = 200;
=======
    /// Get events with composite filter (type + time range + location).
    pub fn get_filtered_events(
        env: Env,
        product_id: String,
        filter: TrackingEventFilter,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);

        let empty_sym = Symbol::new(&env, "");
        let empty_loc = String::from_str(&env, "");

        for i in 0..all_ids.len() {
            let eid = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                let mut matches = true;

                // Event type filter
                if filter.event_type != empty_sym && event.event_type != filter.event_type {
                    matches = false;
                }
                // Time lower bound
                if filter.start_time > 0 && event.timestamp < filter.start_time {
                    matches = false;
                }
                // Time upper bound
                if filter.end_time < u64::MAX && event.timestamp > filter.end_time {
                    matches = false;
                }
                // Location filter
                if filter.location != empty_loc && event.location != filter.location {
                    matches = false;
                }
>>>>>>> main

                if matches {
                    matching_ids.push_back(eid);
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        for i in start..end {
            let eid = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (events.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EVENT COUNT HELPERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get total event count for a product.
    pub fn get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        let _ = read_product(&env, &product_id)?;
        let ids = storage::get_product_event_ids(&env, &product_id);
        Ok(ids.len() as u64)
    }

    /// Get event count for a specific event type on a product.
    pub fn get_event_count_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
    ) -> Result<u64, Error> {
        let _ = read_product(&env, &product_id)?;
        Ok(storage::get_event_count_by_type(&env, &product_id, &event_type))
    }
}

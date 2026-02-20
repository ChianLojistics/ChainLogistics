use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

use crate::storage::DataKey;
use crate::types::{Product, ProductStats};
use crate::error::Error;

#[contract]
pub struct ChainLogisticsContract;

#[contractimpl]
impl ChainLogisticsContract {
    /// Register a new product
    pub fn register_product(
        env: Env,
        owner: Address,
        origin: String,
        metadata: String,
    ) -> Result<u64, Error> {
        owner.require_auth();

        let mut total_products: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalProducts)
            .unwrap_or(0);
        total_products += 1;

        let product = Product {
            id: total_products,
            owner: owner.clone(),
            origin: origin.clone(),
            active: true,
            metadata,
            created_at: env.ledger().timestamp(),
        };

        // 1. Store Product
        env.storage()
            .persistent()
            .set(&DataKey::Product(total_products), &product);

        // 2. Global Index (Index -> ID)
        env.storage()
            .persistent()
            .set(&DataKey::AllProductsIndex(total_products), &total_products);

        // 3. Owner Index
        let mut owner_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerProductCount(owner.clone()))
            .unwrap_or(0);
        owner_count += 1;
        env.storage().persistent().set(
            &DataKey::OwnerProductIndex(owner.clone(), owner_count),
            &total_products,
        );
        env.storage()
            .persistent()
            .set(&DataKey::OwnerProductCount(owner.clone()), &owner_count);

        // 4. Origin Index
        let mut origin_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::OriginProductCount(origin.clone()))
            .unwrap_or(0);
        origin_count += 1;
        env.storage().persistent().set(
            &DataKey::OriginProductIndex(origin.clone(), origin_count),
            &total_products,
        );
        env.storage()
            .persistent()
            .set(&DataKey::OriginProductCount(origin.clone()), &origin_count);

        // Update global counters
        env.storage()
            .instance()
            .set(&DataKey::TotalProducts, &total_products);

        let mut active_products: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ActiveProducts)
            .unwrap_or(0);
        active_products += 1;
        env.storage()
            .instance()
            .set(&DataKey::ActiveProducts, &active_products);

        Ok(total_products)
    }

    /// Get a product by ID
    pub fn get_product(env: Env, id: u64) -> Option<Product> {
        env.storage().persistent().get(&DataKey::Product(id))
    }

    /// Get all products with pagination (start is 0-based)
    pub fn get_all_products(env: Env, start: u64, limit: u64) -> Vec<Product> {
        let total: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalProducts)
            .unwrap_or(0);
        let mut products = Vec::new(&env);

        let start_index = start + 1;
        let end_index = start + limit + 1;

        for i in start_index..end_index {
            if i > total {
                break;
            }
            if let Some(product_id) = env
                .storage()
                .persistent()
                .get::<DataKey, u64>(&DataKey::AllProductsIndex(i))
            {
                if let Some(product) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, Product>(&DataKey::Product(product_id))
                {
                    products.push_back(product);
                }
            }
        }
        products
    }

    /// Get products by owner with pagination
    pub fn get_products_by_owner(env: Env, owner: Address, start: u64, limit: u64) -> Vec<Product> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerProductCount(owner.clone()))
            .unwrap_or(0);
        let mut products = Vec::new(&env);

        let start_index = start + 1;
        let end_index = start + limit + 1;

        for i in start_index..end_index {
            if i > count {
                break;
            }
            if let Some(product_id) = env
                .storage()
                .persistent()
                .get::<DataKey, u64>(&DataKey::OwnerProductIndex(owner.clone(), i))
            {
                if let Some(product) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, Product>(&DataKey::Product(product_id))
                {
                    products.push_back(product);
                }
            }
        }
        products
    }

    /// Get products by origin with pagination
    pub fn get_products_by_origin(env: Env, origin: String, start: u64, limit: u64) -> Vec<Product> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::OriginProductCount(origin.clone()))
            .unwrap_or(0);
        let mut products = Vec::new(&env);

        let start_index = start + 1;
        let end_index = start + limit + 1;

        for i in start_index..end_index {
            if i > count {
                break;
            }
            if let Some(product_id) = env
                .storage()
                .persistent()
                .get::<DataKey, u64>(&DataKey::OriginProductIndex(origin.clone(), i))
            {
                if let Some(product) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, Product>(&DataKey::Product(product_id))
                {
                    products.push_back(product);
                }
            }
        }
        products
    }

    /// Get product stats
    pub fn get_stats(env: Env) -> ProductStats {
        ProductStats {
            total_products: env
                .storage()
                .instance()
                .get(&DataKey::TotalProducts)
                .unwrap_or(0),
            active_products: env
                .storage()
                .instance()
                .get(&DataKey::ActiveProducts)
                .unwrap_or(0),
        }
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{storage, validation, Error, Origin, Product, TrackingEvent, EventPage, EventFilter};

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

fn require_can_add_event(env: &Env, product_id: &String, product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if !product.active {
        return Err(Error::InvalidInput);
    }
    if &product.owner == caller {
        return Ok(());
    }
    if !storage::is_authorized(env, product_id, caller) {
        return Err(Error::Unauthorized);
    }

    /// Transfer ownership of a product
    pub fn transfer_product(
#[contract]
pub struct ChainLogisticsContract;

#[contractimpl]
impl ChainLogisticsContract {
    /// Register a new product with rich metadata
    pub fn register_product(
        env: Env,
        owner: Address,
        product_id: u64,
        new_owner: Address,
    ) -> Result<(), Error> {
        let mut product: Product = env
            .storage()
            .persistent()
            .get(&DataKey::Product(product_id))
            .ok_or(Error::ProductNotFound)?;
        
        owner.require_auth();
        if product.owner != owner {
            return Err(Error::Unauthorized);
        }

        let product = Product {
            id: id.clone(),
            name,
            description,
            origin: Origin {
                location: origin_location,
            },
            owner: owner.clone(),
            created_at: env.ledger().timestamp(),
            active: true,
            category,
            tags,
            certifications,
            media_hashes,
            custom,
        };

        write_product(&env, &product);
        storage::put_product_event_ids(&env, &id, &Vec::new(&env));
        storage::set_auth(&env, &id, &owner, true);

        env.events().publish((Symbol::new(&env, "product_registered"), id.clone()), product.clone());
        Ok(product)
    }

    /// Get a product by ID
    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    /// Get all event IDs for a product (for backwards compatibility)
    pub fn get_product_event_ids(env: Env, id: String) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &id)?;
        Ok(storage::get_product_event_ids(&env, &id))
    }

    /// Add an authorized actor who can add events to a product
    pub fn add_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, true);
        Ok(())
    }

    /// Remove an authorized actor
    pub fn remove_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, false);
        Ok(())
    }

    /// Transfer product ownership
    pub fn transfer_product(env: Env, owner: Address, product_id: String, new_owner: Address) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        new_owner.require_auth();

        // Transfer authorization
        env.storage()
            .persistent()
            .remove(&DataKey::Auth(product_id, product.owner.clone()));
            
        product.owner = new_owner.clone();
        
        env.storage()
            .persistent()
            .set(&DataKey::Product(product_id), &product);
            
        env.storage()
            .persistent()
            .set(&DataKey::Auth(product_id, new_owner), &true);
            
        Ok(())
    }

    /// Add an authorized actor
    pub fn add_authorized_actor(
        env: Env,
        owner: Address,
        product_id: u64,
        actor: Address,
    ) -> Result<(), Error> {
        let product: Product = env
            .storage()
            .persistent()
            .get(&DataKey::Product(product_id))
            .ok_or(Error::ProductNotFound)?;

        owner.require_auth();
        if product.owner != owner {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Auth(product_id, actor), &true);
            
        Ok(())
    }

    /// Remove an authorized actor
    pub fn remove_authorized_actor(
        env: Env,
        owner: Address,
        product_id: u64,
        actor: Address,
    ) -> Result<(), Error> {
        let product: Product = env
            .storage()
            .persistent()
            .get(&DataKey::Product(product_id))
            .ok_or(Error::ProductNotFound)?;

        owner.require_auth();
        if product.owner != owner {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::Auth(product_id, actor));
            
        Ok(())
    }

    /// Check if an actor is authorized
    pub fn is_authorized(env: Env, product_id: u64, actor: Address) -> bool {
        if let Some(product) = env.storage().persistent().get::<DataKey, Product>(&DataKey::Product(product_id)) {
            if product.owner == actor {
                return true;
            }
    /// Set product active/inactive
    pub fn set_product_active(env: Env, owner: Address, product_id: String, active: bool) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        product.active = active;
        write_product(&env, &product);
        Ok(())
    }

    /// Add a tracking event to a product
    /// 
    /// # Arguments
    /// * `actor` - Address adding the event (must be owner or authorized)
    /// * `product_id` - Product identifier
    /// * `event_type` - Type of event (e.g., "HARVEST", "SHIP", "RECEIVE")
    /// * `location` - Where the event occurred
    /// * `data_hash` - Hash of off-chain data (IPFS, etc.)
    /// * `note` - Human-readable note
    /// * `metadata` - Flexible key-value metadata
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
            actor: actor.clone(),
            timestamp: env.ledger().timestamp(),
            event_type: event_type.clone(),
            location: location.clone(),
            data_hash,
            note: note.clone(),
            metadata: metadata.clone(),
        };

        storage::put_event(&env, &event);
        
        // Update product event list
        let mut ids = storage::get_product_event_ids(&env, &product_id);
        ids.push_back(event_id);
        storage::put_product_event_ids(&env, &product_id, &ids);
        
        // Index by event type for efficient filtering
        storage::index_event_by_type(&env, &product_id, &event_type, event_id);

        env.events().publish(
            (Symbol::new(&env, "tracking_event"), product_id.clone(), event_id),
            event.clone(),
        );
        
        Ok(event_id)
    }

    /// Get a single event by ID
    pub fn get_event(env: Env, event_id: u64) -> Result<TrackingEvent, Error> {
        storage::get_event(&env, event_id).ok_or(Error::EventNotFound)
    }

    /// Get all events for a product with pagination
    /// 
    /// # Arguments
    /// * `product_id` - Product identifier
    /// * `offset` - Number of events to skip (for pagination)
    /// * `limit` - Maximum events to return
    pub fn get_product_events(
        env: Env,
        product_id: String,
        offset: u64,
        limit: u64,
    ) -> Result<EventPage, Error> {
        let _ = read_product(&env, &product_id)?;
        
        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let total_count = all_ids.len() as u64;
        
        let event_ids = storage::get_product_event_ids_paginated(&env, &product_id, offset, limit);
        
        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let event_id = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, event_id) {
                events.push_back(event);
            }
        }
        
        let has_more = offset + (event_ids.len() as u64) < total_count;
        
        Ok(EventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events filtered by type with pagination
    pub fn get_events_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
        offset: u64,
        limit: u64,
    ) -> Result<EventPage, Error> {
        let _ = read_product(&env, &product_id)?;
        
        let total_count = storage::get_event_count_by_type(&env, &product_id, &event_type);
        let event_ids = storage::get_event_ids_by_type(&env, &product_id, &event_type, offset, limit);
        
        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let event_id = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, event_id) {
                events.push_back(event);
            }
        }
        
        let has_more = offset + (event_ids.len() as u64) < total_count;
        
        Ok(EventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events filtered by time range with pagination
    /// 
    /// Note: This performs a linear scan through events. For products with
    /// many events, prefer using type-based filtering when possible.
    pub fn get_events_by_time_range(
        env: Env,
        product_id: String,
        start_time: u64,
        end_time: u64,
        offset: u64,
        limit: u64,
    ) -> Result<EventPage, Error> {
        let _ = read_product(&env, &product_id)?;
        
        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);
        
        // Collect matching event IDs
        for i in 0..all_ids.len() {
            let event_id = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, event_id) {
                if event.timestamp >= start_time && event.timestamp <= end_time {
                    matching_ids.push_back(event_id);
                }
            }
        }
        
        let total_count = matching_ids.len() as u64;
        
        // Apply pagination
        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());
        
        for i in start..end {
            let event_id = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, event_id) {
                events.push_back(event);
            }
        }
        
        let has_more = offset + (events.len() as u64) < total_count;
        
        Ok(EventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Get events with flexible filtering
    /// 
    /// Filter supports: event_type, time range, location
    pub fn get_filtered_events(
        env: Env,
        product_id: String,
        filter: EventFilter,
        offset: u64,
        limit: u64,
    ) -> Result<EventPage, Error> {
        let _ = read_product(&env, &product_id)?;
        
        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);
        
        // Collect matching event IDs based on filter criteria
        for i in 0..all_ids.len() {
            let event_id = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, event_id) {
                let mut matches = true;
                
                // Check event type filter (non-empty symbol means filter active)
                let empty_sym = Symbol::new(&env, "");
                if filter.event_type != empty_sym {
                    if event.event_type != filter.event_type {
                        matches = false;
                    }
                }
                
                // Check time range filters
                // start_time > 0 means filter active
                if filter.start_time > 0 {
                    if event.timestamp < filter.start_time {
                        matches = false;
                    }
                }
                
                // end_time < u64::MAX means filter active
                if filter.end_time < u64::MAX {
                    if event.timestamp > filter.end_time {
                        matches = false;
                    }
                }
                
                // Check location filter (non-empty string means filter active)
                let empty_loc = String::from_str(&env, "");
                if filter.location != empty_loc {
                    if event.location != filter.location {
                        matches = false;
                    }
                }
                
                if matches {
                    matching_ids.push_back(event_id);
                }
            }
        }
        
        let total_count = matching_ids.len() as u64;
        
        // Apply pagination
        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());
        
        for i in start..end {
            let event_id = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, event_id) {
                events.push_back(event);
            }
        }
        
        let has_more = offset + (events.len() as u64) < total_count;
        
        Ok(EventPage {
            events,
            total_count,
            has_more,
        })
    }

    /// Check if an actor is authorized to add events to a product
    pub fn is_authorized(env: Env, product_id: String, actor: Address) -> Result<bool, Error> {
        let product = read_product(&env, &product_id)?;
        if product.owner == actor {
            return Ok(true);
        }
        
        env.storage()
            .persistent()
            .get(&DataKey::Auth(product_id, actor))
            .unwrap_or(false)
    }

    /// Get event count for a product
    pub fn get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        let _ = read_product(&env, &product_id)?;
        let ids = storage::get_product_event_ids(&env, &product_id);
        Ok(ids.len() as u64)
    }

    /// Get event count by type for a product
    pub fn get_event_count_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
    ) -> Result<u64, Error> {
        let _ = read_product(&env, &product_id)?;
        Ok(storage::get_event_count_by_type(&env, &product_id, &event_type))
    }
}


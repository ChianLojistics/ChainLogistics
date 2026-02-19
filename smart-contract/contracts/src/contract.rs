use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec, vec};
use crate::storage::DataKey;
use crate::types::{Product, ProductStats};
use crate::error::Error;
 use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::{storage, validation, Error, Origin, Product, TrackingEvent};

#[contract]
pub struct ChainLogisticsContract;

#[contractimpl]
impl ChainLogisticsContract {
    
    /// Register a new product
    pub fn register_product(
        env: Env, 
        owner: Address, 
        origin: String, 
        metadata: String
    ) -> Result<u64, Error> {
        owner.require_auth();

        // increments product count
        let mut total_products: u64 = env.storage().instance().get(&DataKey::TotalProducts).unwrap_or(0);
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
        env.storage().persistent().set(&DataKey::Product(total_products), &product);
        
        // 2. Global Index (Index -> ID)
        // Since ID is sequential and starts at 1, we can just use ID as the index for "All Products" 
        // if we assume we iterate by ID. 
        // But if we want to support deleting or non-sequential IDs later, an explicit index is better.
        // For now, let's map Index (1-based) to ProductID.
        env.storage().persistent().set(&DataKey::AllProductsIndex(total_products), &total_products);

        // 3. Owner Index
        let mut owner_count: u64 = env.storage().persistent().get(&DataKey::OwnerProductCount(owner.clone())).unwrap_or(0);
        owner_count += 1;
        env.storage().persistent().set(&DataKey::OwnerProductIndex(owner.clone(), owner_count), &total_products);
        env.storage().persistent().set(&DataKey::OwnerProductCount(owner.clone()), &owner_count);

        // 4. Origin Index
        let mut origin_count: u64 = env.storage().persistent().get(&DataKey::OriginProductCount(origin.clone())).unwrap_or(0);
        origin_count += 1;
        env.storage().persistent().set(&DataKey::OriginProductIndex(origin.clone(), origin_count), &total_products);
        env.storage().persistent().set(&DataKey::OriginProductCount(origin.clone()), &origin_count);
        
        // Update global counters
        env.storage().instance().set(&DataKey::TotalProducts, &total_products);
        
        // Update active count
        let mut active_products: u64 = env.storage().instance().get(&DataKey::ActiveProducts).unwrap_or(0);
        active_products += 1;
        env.storage().instance().set(&DataKey::ActiveProducts, &active_products);

        Ok(total_products)
    }

    /// Get a product by ID
    pub fn get_product(env: Env, id: u64) -> Option<Product> {
        env.storage().persistent().get(&DataKey::Product(id))
    }

    /// Get all products with pagination
    pub fn get_all_products(env: Env, start: u64, limit: u64) -> Vec<Product> {
        let total = env.storage().instance().get(&DataKey::TotalProducts).unwrap_or(0);
        let mut products = Vec::new(&env);
        
        // start is 1-based index for our logic context if we want to be consistent,
        // or 0-based index. Let's assume start is 0-based offset, so we request index like array.
        // But our indices (TotalProducts) act like count. 
        // DataKey::AllProductsIndex(i) where i is 1..Total.
        
        // If start=0, limit=10. We want indices 1, 2, ..., 10.
        
        let start_index = start + 1;
        let end_index = start + limit + 1; // exclusive in loop, so effectively start+1 to start+limit

        for i in start_index..end_index {
            if i > total {
                break;
            }
            // In our simple case, Index i maps to Product ID i. 
            // access key: AllProductsIndex(i)
            if let Some(product_id) = env.storage().persistent().get::<DataKey, u64>(&DataKey::AllProductsIndex(i)) {
                 if let Some(product) = env.storage().persistent().get::<DataKey, Product>(&DataKey::Product(product_id)) {
                    products.push_back(product);
                 }
            }
        }
        
        products
    }

    /// Get products by owner with pagination
    pub fn get_products_by_owner(env: Env, owner: Address, start: u64, limit: u64) -> Vec<Product> {
        let count: u64 = env.storage().persistent().get(&DataKey::OwnerProductCount(owner.clone())).unwrap_or(0);
        let mut products = Vec::new(&env);
        
        let start_index = start + 1;
        let end_index = start + limit + 1;

        for i in start_index..end_index {
            if i > count {
                break;
            }
            if let Some(product_id) = env.storage().persistent().get::<DataKey, u64>(&DataKey::OwnerProductIndex(owner.clone(), i)) {
                 if let Some(product) = env.storage().persistent().get::<DataKey, Product>(&DataKey::Product(product_id)) {
                    products.push_back(product);
                 }
            }
        }
        products
    }

    /// Get products by origin with pagination
    pub fn get_products_by_origin(env: Env, origin: String, start: u64, limit: u64) -> Vec<Product> {
        let count: u64 = env.storage().persistent().get(&DataKey::OriginProductCount(origin.clone())).unwrap_or(0);
        let mut products = Vec::new(&env);
        
        let start_index = start + 1;
        let end_index = start + limit + 1;

        for i in start_index..end_index {
            if i > count {
                break;
            }
             if let Some(product_id) = env.storage().persistent().get::<DataKey, u64>(&DataKey::OriginProductIndex(origin.clone(), i)) {
                 if let Some(product) = env.storage().persistent().get::<DataKey, Product>(&DataKey::Product(product_id)) {
                    products.push_back(product);
                 }
            }
        }
        products
    }
    
    /// Get product stats
    pub fn get_stats(env: Env) -> ProductStats {
        ProductStats {
            total_products: env.storage().instance().get(&DataKey::TotalProducts).unwrap_or(0),
            active_products: env.storage().instance().get(&DataKey::ActiveProducts).unwrap_or(0),
        }
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
    Ok(())
}

#[contractimpl]
impl ChainLogisticsContract {
    pub fn register_product(
        env: Env,
        owner: Address,
        id: String,
        name: String,
        description: String,
        origin_location: String,
        category: String,
        tags: Vec<String>,
        certifications: Vec<BytesN<32>>,
        media_hashes: Vec<BytesN<32>>,
        custom: Map<Symbol, String>,
    ) -> Result<Product, Error> {
        const MAX_ID_LEN: u32 = 64;
        const MAX_NAME_LEN: u32 = 128;
        const MAX_ORIGIN_LEN: u32 = 256;
        const MAX_CATEGORY_LEN: u32 = 64;
        const MAX_DESCRIPTION_LEN: u32 = 2048;
        const MAX_TAG_LEN: u32 = 64;
        const MAX_CUSTOM_VALUE_LEN: u32 = 512;

        const MAX_TAGS: u32 = 20;
        const MAX_CERTIFICATIONS: u32 = 50;
        const MAX_MEDIA_HASHES: u32 = 50;
        const MAX_CUSTOM_FIELDS: u32 = 20;

        if !validation::non_empty(&id) {
            return Err(Error::InvalidProductId);
        }
        if !validation::max_len(&id, MAX_ID_LEN) {
            return Err(Error::ProductIdTooLong);
        }
        if !validation::non_empty(&name) {
            return Err(Error::InvalidProductName);
        }
        if !validation::max_len(&name, MAX_NAME_LEN) {
            return Err(Error::ProductNameTooLong);
        }
        if !validation::non_empty(&origin_location) {
            return Err(Error::InvalidOrigin);
        }
        if !validation::max_len(&origin_location, MAX_ORIGIN_LEN) {
            return Err(Error::OriginTooLong);
        }
        if !validation::non_empty(&category) {
            return Err(Error::InvalidCategory);
        }
        if !validation::max_len(&category, MAX_CATEGORY_LEN) {
            return Err(Error::CategoryTooLong);
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

    pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
        read_product(&env, &id)
    }

    pub fn get_product_event_ids(env: Env, id: String) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &id)?;
        Ok(storage::get_product_event_ids(&env, &id))
    }

    pub fn add_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, true);
        Ok(())
    }

    pub fn remove_authorized_actor(env: Env, owner: Address, product_id: String, actor: Address) -> Result<(), Error> {
        let product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        storage::set_auth(&env, &product_id, &actor, false);
        Ok(())
    }

    pub fn transfer_product(env: Env, owner: Address, product_id: String, new_owner: Address) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;

        new_owner.require_auth();

        storage::set_auth(&env, &product_id, &product.owner, false);
        product.owner = new_owner.clone();
        write_product(&env, &product);
        storage::set_auth(&env, &product_id, &new_owner, true);
        Ok(())
    }

    pub fn set_product_active(env: Env, owner: Address, product_id: String, active: bool) -> Result<(), Error> {
        let mut product = read_product(&env, &product_id)?;
        require_owner(&product, &owner)?;
        product.active = active;
        write_product(&env, &product);
        Ok(())
    }

    pub fn add_tracking_event(env: Env, actor: Address, product_id: String, event_type: Symbol, data_hash: BytesN<32>, note: String) -> Result<u64, Error> {
        let product = read_product(&env, &product_id)?;
        require_can_add_event(&env, &product_id, &product, &actor)?;

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
        let mut ids = storage::get_product_event_ids(&env, &product_id);
        ids.push_back(event_id);
        storage::put_product_event_ids(&env, &product_id, &ids);
        Ok(event_id)
    }

    pub fn get_event(env: Env, event_id: u64) -> Result<TrackingEvent, Error> {
        storage::get_event(&env, event_id).ok_or(Error::EventNotFound)
    }

    pub fn is_authorized(env: Env, product_id: String, actor: Address) -> Result<bool, Error> {
        let product = read_product(&env, &product_id)?;
        if product.owner == actor {
            return Ok(true);
        }
        Ok(storage::is_authorized(&env, &product_id, &actor))
    }
}

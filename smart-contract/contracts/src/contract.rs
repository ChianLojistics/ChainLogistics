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
    }

    /// Transfer ownership of a product
    pub fn transfer_product(
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
        }
        
        env.storage()
            .persistent()
            .get(&DataKey::Auth(product_id, actor))
            .unwrap_or(false)
    }
}


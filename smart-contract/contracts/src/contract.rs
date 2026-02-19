use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec, vec};
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
    }
}

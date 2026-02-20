use soroban_sdk::{Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    // Global counters
    TotalProducts,
    ActiveProducts,

    // Product storage: ProductId -> Product
    Product(u64),

    // Global Index: Index -> ProductId
    AllProductsIndex(u64),

    // Owner Index: (Owner, Index) -> ProductId
    OwnerProductIndex(Address, u64),
    OwnerProductCount(Address),

    // Origin Index: (Origin, Index) -> ProductId
    OriginProductIndex(String, u64),
    OriginProductCount(String),

    // Authorization: (ProductId, Actor) -> bool
    Auth(u64, Address),
}

use crate::{DataKey, Product, TrackingEvent};

pub fn has_product(env: &Env, product_id: &soroban_sdk::String) -> bool {
    env.storage().persistent()
        .has(&DataKey::Product(product_id.clone()))
}

pub fn put_product(env: &Env, product: &Product) {
    env.storage()
        .persistent()
        .set(&DataKey::Product(product.id.clone()), product);
}

pub fn get_product(env: &Env, product_id: &soroban_sdk::String) -> Option<Product> {
    env.storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
}

pub fn put_product_event_ids(env: &Env, product_id: &soroban_sdk::String, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductEventIds(product_id.clone()), ids);
}

pub fn get_product_event_ids(env: &Env, product_id: &soroban_sdk::String) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductEventIds(product_id.clone()))
        .unwrap_or(Vec::new(env))
}

/// Get paginated event IDs for a product
pub fn get_product_event_ids_paginated(
    env: &Env,
    product_id: &soroban_sdk::String,
    offset: u64,
    limit: u64,
) -> Vec<u64> {
    let all_ids = get_product_event_ids(env, product_id);
    let total = all_ids.len() as u64;
    
    let mut result = Vec::new(env);
    
    if offset >= total {
        return result;
    }
    
    let end = ((offset + limit) as u32).min(all_ids.len());
    let start = offset as u32;
    
    for i in start..end {
        result.push_back(all_ids.get_unchecked(i));
    }
    
    result
}

pub fn put_event(env: &Env, event: &TrackingEvent) {
    env.storage()
        .persistent()
        .set(&DataKey::Event(event.event_id), event);
}

pub fn get_event(env: &Env, event_id: u64) -> Option<TrackingEvent> {
    env.storage().persistent().get(&DataKey::Event(event_id))
}

pub fn next_event_id(env: &Env) -> u64 {
    let mut seq: u64 = env.storage().persistent().get(&DataKey::EventSeq).unwrap_or(0);
    seq += 1;
    env.storage().persistent().set(&DataKey::EventSeq, &seq);
    seq
}

/// Index event by type for efficient filtering
pub fn index_event_by_type(
    env: &Env,
    product_id: &soroban_sdk::String,
    event_type: &Symbol,
    event_id: u64,
) {
    let count_key = DataKey::EventTypeCount(product_id.clone(), event_type.clone());
    let mut count: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
    count += 1;
    env.storage().persistent().set(&count_key, &count);
    
    let index_key = DataKey::EventTypeIndex(product_id.clone(), event_type.clone(), count);
    env.storage().persistent().set(&index_key, &event_id);
}

/// Get event IDs by type with pagination
pub fn get_event_ids_by_type(
    env: &Env,
    product_id: &soroban_sdk::String,
    event_type: &Symbol,
    offset: u64,
    limit: u64,
) -> Vec<u64> {
    let count_key = DataKey::EventTypeCount(product_id.clone(), event_type.clone());
    let total: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
    
    let mut result = Vec::new(env);
    
    if offset >= total {
        return result;
    }
    
    let start = offset + 1; // 1-based index for storage
    let end = (start + limit).min(total + 1);
    
    for i in start..end {
        let index_key = DataKey::EventTypeIndex(product_id.clone(), event_type.clone(), i);
        if let Some(event_id) = env.storage().persistent().get::<DataKey, u64>(&index_key) {
            result.push_back(event_id);
        }
    }
    
    result
}

/// Get count of events by type
pub fn get_event_count_by_type(
    env: &Env,
    product_id: &soroban_sdk::String,
    event_type: &Symbol,
) -> u64 {
    let count_key = DataKey::EventTypeCount(product_id.clone(), event_type.clone());
    env.storage().persistent().get(&count_key).unwrap_or(0)
}

pub fn set_auth(env: &Env, product_id: &soroban_sdk::String, actor: &Address, value: bool) {
    if value {
        env.storage()
            .persistent()
            .set(&DataKey::Auth(product_id.clone(), actor.clone()), &true);
    } else {
        env.storage()
            .persistent()
            .remove(&DataKey::Auth(product_id.clone(), actor.clone()));
    }
}

pub fn is_authorized(env: &Env, product_id: &soroban_sdk::String, actor: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Auth(product_id.clone(), actor.clone()))
        .unwrap_or(false)
}

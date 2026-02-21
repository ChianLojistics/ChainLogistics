use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::types::{DataKey, Product, TrackingEvent};

// ─── Product ────────────────────────────────────────────────────────────────

pub fn has_product(env: &Env, product_id: &String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Product(product_id.clone()))
}

pub fn put_product(env: &Env, product: &Product) {
    env.storage()
        .persistent()
        .set(&DataKey::Product(product.id.clone()), product);
}

pub fn get_product(env: &Env, product_id: &String) -> Option<Product> {
    env.storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
}

// ─── Event IDs per product ───────────────────────────────────────────────────

pub fn put_product_event_ids(env: &Env, product_id: &String, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductEventIds(product_id.clone()), ids);
}

pub fn get_product_event_ids(env: &Env, product_id: &String) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductEventIds(product_id.clone()))
        .unwrap_or(Vec::new(env))
}

/// Returns a paginated slice of event IDs for a product.
pub fn get_product_event_ids_paginated(
    env: &Env,
    product_id: &String,
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

// ─── Events ─────────────────────────────────────────────────────────────────

pub fn put_event(env: &Env, event: &TrackingEvent) {
    env.storage()
        .persistent()
        .set(&DataKey::Event(event.event_id), event);
}

pub fn get_event(env: &Env, event_id: u64) -> Option<TrackingEvent> {
    env.storage().persistent().get(&DataKey::Event(event_id))
}

pub fn next_event_id(env: &Env) -> u64 {
    let mut seq: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::EventSeq)
        .unwrap_or(0);
    seq += 1;
    env.storage().persistent().set(&DataKey::EventSeq, &seq);
    seq
}

// ─── Event type index ────────────────────────────────────────────────────────

/// Index an event ID under its event_type for efficient filtering.
pub fn index_event_by_type(
    env: &Env,
    product_id: &String,
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

/// Returns paginated event IDs for a given product + event_type.
pub fn get_event_ids_by_type(
    env: &Env,
    product_id: &String,
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

    let start = offset + 1; // 1-based
    let end = (start + limit).min(total + 1);

    for i in start..end {
        let index_key = DataKey::EventTypeIndex(product_id.clone(), event_type.clone(), i);
        if let Some(event_id) = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&index_key)
        {
            result.push_back(event_id);
        }
    }

    result
}

/// Returns the total number of events of a given type for a product.
pub fn get_event_count_by_type(env: &Env, product_id: &String, event_type: &Symbol) -> u64 {
    let count_key = DataKey::EventTypeCount(product_id.clone(), event_type.clone());
    env.storage().persistent().get(&count_key).unwrap_or(0)
}

// ─── Authorization ───────────────────────────────────────────────────────────

pub fn set_auth(env: &Env, product_id: &String, actor: &Address, value: bool) {
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

pub fn is_authorized(env: &Env, product_id: &String, actor: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Auth(product_id.clone(), actor.clone()))
        .unwrap_or(false)
}

// ─── Global counters ─────────────────────────────────────────────────────────

pub fn get_total_products(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::TotalProducts)
        .unwrap_or(0)
}

pub fn set_total_products(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::TotalProducts, &count);
}

pub fn get_active_products(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ActiveProducts)
        .unwrap_or(0)
}

pub fn set_active_products(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::ActiveProducts, &count);
}

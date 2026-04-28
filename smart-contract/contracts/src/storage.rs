use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::storage_contract::StorageContract;
use crate::types::{Product, TrackingEvent};

pub fn get_auth_contract(env: &Env) -> Option<Address> {
    StorageContract::get_auth_contract(env)
}

pub fn set_auth_contract(env: &Env, address: &Address) {
    StorageContract::set_auth_contract(env, address)
}

pub fn get_multisig_contract(env: &Env) -> Option<Address> {
    StorageContract::get_multisig_contract(env)
}

pub fn set_multisig_contract(env: &Env, address: &Address) {
    StorageContract::set_multisig_contract(env, address)
}

pub fn get_timelock_contract(env: &Env) -> Option<Address> {
    StorageContract::get_timelock_contract(env)
}

pub fn set_timelock_contract(env: &Env, address: &Address) {
    StorageContract::set_timelock_contract(env, address)
}

// ─── Product ────────────────────────────────────────────────────────────────

pub fn has_product(env: &Env, product_id: &String) -> bool {
    StorageContract::has_product(env, product_id)
}

pub fn put_product(env: &Env, product: &Product) {
    StorageContract::put_product(env, product)
}

pub fn get_product(env: &Env, product_id: &String) -> Option<Product> {
    StorageContract::get_product(env, product_id)
}

// ─── Event IDs per product ───────────────────────────────────────────────────

pub fn put_product_event_ids(env: &Env, product_id: &String, ids: &Vec<u64>) {
    StorageContract::put_product_event_ids(env, product_id, ids)
}

pub fn get_product_event_ids(env: &Env, product_id: &String) -> Vec<u64> {
    StorageContract::get_product_event_ids(env, product_id)
}

pub fn get_product_event_ids_paginated(
    env: &Env,
    product_id: &String,
    offset: u64,
    limit: u64,
) -> Vec<u64> {
    StorageContract::get_product_event_ids_paginated(env, product_id, offset, limit)
}

// ─── Events ─────────────────────────────────────────────────────────────────

pub fn put_event(env: &Env, event: &TrackingEvent) {
    StorageContract::put_event(env, event)
}

pub fn get_event(env: &Env, event_id: u64) -> Option<TrackingEvent> {
    StorageContract::get_event(env, event_id)
}

pub fn next_event_id(env: &Env) -> Result<u64, Error> {
    StorageContract::next_event_id(env)
}

// ─── Event type index ────────────────────────────────────────────────────────

pub fn index_event_by_type(
    env: &Env,
    product_id: &String,
    event_type: &Symbol,
    event_id: u64,
) -> Result<(), Error> {
    StorageContract::index_event_by_type(env, product_id, event_type, event_id)
}

pub fn acquire_reentrancy_lock(env: &Env, scope: &Symbol) -> Result<(), Error> {
    StorageContract::acquire_reentrancy_lock(env, scope)
}

pub fn release_reentrancy_lock(env: &Env, scope: &Symbol) {
    StorageContract::release_reentrancy_lock(env, scope)
}

pub fn get_event_ids_by_type(
    env: &Env,
    product_id: &String,
    event_type: &Symbol,
    offset: u64,
    limit: u64,
) -> Vec<u64> {
    StorageContract::get_event_ids_by_type(env, product_id, event_type, offset, limit)
}

pub fn get_event_count_by_type(env: &Env, product_id: &String, event_type: &Symbol) -> u64 {
    StorageContract::get_event_count_by_type(env, product_id, event_type)
}

// ─── Authorization ───────────────────────────────────────────────────────────

pub fn set_auth(env: &Env, product_id: &String, actor: &Address, value: bool) {
    StorageContract::set_auth(env, product_id, actor, value)
}

pub fn is_authorized(env: &Env, product_id: &String, actor: &Address) -> bool {
    StorageContract::is_authorized(env, product_id, actor)
}

// ─── Global Management ───────────────────────────────────────────────────────

pub fn has_admin(env: &Env) -> bool {
    StorageContract::has_admin(env)
}

pub fn get_admin(env: &Env) -> Option<Address> {
    StorageContract::get_admin(env)
}

pub fn set_admin(env: &Env, admin: &Address) {
    StorageContract::set_admin(env, admin)
}

pub fn is_paused(env: &Env) -> bool {
    StorageContract::is_paused(env)
}

pub fn set_paused(env: &Env, paused: bool) {
    StorageContract::set_paused(env, paused)
}

// ─── Global counters ─────────────────────────────────────────────────────────

pub fn get_total_products(env: &Env) -> u64 {
    StorageContract::get_total_products(env)
}

pub fn set_total_products(env: &Env, count: u64) {
    StorageContract::set_total_products(env, count)
}

pub fn get_active_products(env: &Env) -> u64 {
    StorageContract::get_active_products(env)
}

pub fn set_active_products(env: &Env, count: u64) {
    StorageContract::set_active_products(env, count)
}

// ─── Search Index ───────────────────────────────────────────────────────────

pub fn put_search_index(env: &Env, keyword: &String, product_ids: &Vec<String>) {
    env.storage().persistent().set(
        &crate::types::DataKey::SearchIndex(crate::types::IndexKey::Keyword(keyword.clone())),
        product_ids,
    );
}

pub fn get_search_index(env: &Env, keyword: &String) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&crate::types::DataKey::SearchIndex(
            crate::types::IndexKey::Keyword(keyword.clone()),
        ))
        .unwrap_or_else(|| Vec::new(env))
}

// Gas-optimized: Check existence before loading full vector
pub fn add_to_search_index(env: &Env, keyword: String, product_id: &String) {
    let mut ids = get_search_index(env, &keyword);
    // Early exit if already indexed - saves gas
    if ids.contains(product_id) {
        return;
    }
    ids.push_back(product_id.clone());
    put_search_index(env, &keyword, &ids);
}

// Gas-optimized: Use iterator pattern and early exit
pub fn remove_from_search_index(env: &Env, keyword: String, product_id: &String) {
    let mut ids = get_search_index(env, &keyword);
    // Find and remove in single pass
    for i in 0..ids.len() {
        if ids.get(i).unwrap() == product_id.clone() {
            ids.remove(i);
            put_search_index(env, &keyword, &ids);
            return; // Early exit saves gas
        }
    }
    // No write if not found - saves gas
}

use soroban_sdk::{contracttype, Env, Vec};

use crate::types::{Product, SupplyChainEvent};

#[contracttype]
pub enum DataKey {
    Product(u64),
    Events(u64),
    ProductCounter,
}

pub fn get_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ProductCounter)
        .unwrap_or(0u64)
}

pub fn save_counter(env: &Env, counter: u64) {
    env.storage()
        .instance()
        .set(&DataKey::ProductCounter, &counter);
}

pub fn product_exists(env: &Env, product_id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Product(product_id))
}

pub fn get_product(env: &Env, product_id: u64) -> Option<Product> {
    env.storage()
        .persistent()
        .get(&DataKey::Product(product_id))
}

pub fn save_product(env: &Env, product: &Product) {
    env.storage()
        .persistent()
        .set(&DataKey::Product(product.id), product);
}

pub fn get_events(env: &Env, product_id: u64) -> Vec<SupplyChainEvent> {
    env.storage()
        .persistent()
        .get(&DataKey::Events(product_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn save_events(env: &Env, product_id: u64, events: &Vec<SupplyChainEvent>) {
    env.storage()
        .persistent()
        .set(&DataKey::Events(product_id), events);
}

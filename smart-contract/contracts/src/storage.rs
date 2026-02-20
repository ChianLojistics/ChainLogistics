use soroban_sdk::{contracttype, Address, Env, String, Symbol, Vec};

use crate::{Product, TrackingEvent};

/// Storage keys for persistent data on the blockchain.
/// 
/// Uses Soroban's persistent storage API which ensures data persists
/// across contract invocations and ledger entries.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Stores a Product struct by product ID
    Product(String),
    /// Stores a vector of event IDs associated with a product
    ProductEventIds(String),
    ProductEventTimestamps(String),
    ProductEventIdsByType(String, Symbol),
    ProductEventIdsByActor(String, Address),

    /// Stores a TrackingEvent by event ID
    Event(u64),
    /// Sequence counter for generating unique event IDs
    EventSeq,

    /// Authorization mapping: (product_id, actor_address) -> bool
    Auth(String, Address),
}

/// Checks if a product exists in persistent storage.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product_id` - The unique identifier for the product
/// 
/// # Returns
/// `true` if the product exists, `false` otherwise
pub fn has_product(env: &Env, product_id: &String) -> bool {
    env.storage().persistent().has(&DataKey::Product(product_id.clone()))
}

/// Stores a product in persistent storage.
/// 
/// Products are stored using persistent storage, which means they will
/// remain on the blockchain across contract calls and ledger entries.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product` - The Product struct to store
/// 
/// # Note
/// This will overwrite any existing product with the same ID.
pub fn put_product(env: &Env, product: &Product) {
    env.storage()
        .persistent()
        .set(&DataKey::Product(product.id.clone()), product);
}

/// Retrieves a product from persistent storage by ID.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product_id` - The unique identifier for the product
/// 
/// # Returns
/// `Some(Product)` if the product exists, `None` otherwise
pub fn get_product(env: &Env, product_id: &String) -> Option<Product> {
    env.storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
}

/// Stores the list of event IDs associated with a product.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product_id` - The product identifier
/// * `ids` - Vector of event IDs to store
pub fn put_product_event_ids(env: &Env, product_id: &String, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductEventIds(product_id.clone()), ids);
}

/// Retrieves the list of event IDs for a product.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product_id` - The product identifier
/// 
/// # Returns
/// Vector of event IDs, or empty vector if none exist
pub fn get_product_event_ids(env: &Env, product_id: &String) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductEventIds(product_id.clone()))
        .unwrap_or(Vec::new(env))
}

<<<<<<< feature/issue-8-event-querying
pub fn put_product_event_timestamps(env: &Env, product_id: &String, ts: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductEventTimestamps(product_id.clone()), ts);
}

pub fn get_product_event_timestamps(env: &Env, product_id: &String) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductEventTimestamps(product_id.clone()))
        .unwrap_or(Vec::new(env))
}

pub fn put_product_event_ids_by_type(env: &Env, product_id: &String, event_type: &Symbol, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductEventIdsByType(product_id.clone(), event_type.clone()), ids);
}

pub fn get_product_event_ids_by_type(env: &Env, product_id: &String, event_type: &Symbol) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductEventIdsByType(product_id.clone(), event_type.clone()))
        .unwrap_or(Vec::new(env))
}

pub fn put_product_event_ids_by_actor(env: &Env, product_id: &String, actor: &Address, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductEventIdsByActor(product_id.clone(), actor.clone()), ids);
}

pub fn get_product_event_ids_by_actor(env: &Env, product_id: &String, actor: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductEventIdsByActor(product_id.clone(), actor.clone()))
        .unwrap_or(Vec::new(env))
}

=======
/// Stores a tracking event in persistent storage.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `event` - The TrackingEvent to store
>>>>>>> main
pub fn put_event(env: &Env, event: &TrackingEvent) {
    env.storage()
        .persistent()
        .set(&DataKey::Event(event.event_id), event);
}

/// Retrieves a tracking event by ID.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `event_id` - The unique event identifier
/// 
/// # Returns
/// `Some(TrackingEvent)` if found, `None` otherwise
pub fn get_event(env: &Env, event_id: u64) -> Option<TrackingEvent> {
    env.storage().persistent().get(&DataKey::Event(event_id))
}

/// Generates and returns the next sequential event ID.
/// 
/// Uses a persistent counter to ensure unique event IDs across
/// all contract invocations.
/// 
/// # Arguments
/// * `env` - The contract environment
/// 
/// # Returns
/// The next available event ID
pub fn next_event_id(env: &Env) -> u64 {
    let mut seq: u64 = env.storage().persistent().get(&DataKey::EventSeq).unwrap_or(0);
    seq += 1;
    env.storage().persistent().set(&DataKey::EventSeq, &seq);
    seq
}

/// Sets or removes authorization for an actor on a product.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product_id` - The product identifier
/// * `actor` - The address to authorize or deauthorize
/// * `value` - `true` to authorize, `false` to remove authorization
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

/// Checks if an actor is authorized for a product.
/// 
/// # Arguments
/// * `env` - The contract environment
/// * `product_id` - The product identifier
/// * `actor` - The address to check
/// 
/// # Returns
/// `true` if authorized, `false` otherwise
pub fn is_authorized(env: &Env, product_id: &String, actor: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Auth(product_id.clone(), actor.clone()))
        .unwrap_or(false)
}

 use soroban_sdk::{Address, Env, Vec};
 
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

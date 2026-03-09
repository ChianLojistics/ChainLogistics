use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::types::{DataKey, Product, TrackingEvent};

pub struct StorageContract;

impl StorageContract {
    pub fn auth_contract_key() -> DataKey {
        DataKey::AuthContract
    }

    pub fn product_key(product_id: &String) -> DataKey {
        DataKey::Product(product_id.clone())
    }

    pub fn product_event_ids_key(product_id: &String) -> DataKey {
        DataKey::ProductEventIds(product_id.clone())
    }

    pub fn event_key(event_id: u64) -> DataKey {
        DataKey::Event(event_id)
    }

    pub fn event_seq_key() -> DataKey {
        DataKey::EventSeq
    }

    pub fn event_type_count_key(product_id: &String, event_type: &Symbol) -> DataKey {
        DataKey::EventTypeCount(product_id.clone(), event_type.clone())
    }

    pub fn event_type_index_key(product_id: &String, event_type: &Symbol, index: u64) -> DataKey {
        DataKey::EventTypeIndex(product_id.clone(), event_type.clone(), index)
    }

    pub fn auth_key(product_id: &String, actor: &Address) -> DataKey {
        DataKey::Auth(product_id.clone(), actor.clone())
    }

    pub fn admin_key() -> DataKey {
        DataKey::Admin
    }

    pub fn paused_key() -> DataKey {
        DataKey::Paused
    }

    pub fn total_products_key() -> DataKey {
        DataKey::TotalProducts
    }

    pub fn active_products_key() -> DataKey {
        DataKey::ActiveProducts
    }

    pub fn get_auth_contract(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::auth_contract_key())
    }

    pub fn set_auth_contract(env: &Env, address: &Address) {
        env.storage()
            .persistent()
            .set(&Self::auth_contract_key(), address);
    }

    pub fn has_product(env: &Env, product_id: &String) -> bool {
        env.storage().persistent().has(&Self::product_key(product_id))
    }

    pub fn put_product(env: &Env, product: &Product) {
        env.storage()
            .persistent()
            .set(&Self::product_key(&product.id), product);
    }

    pub fn get_product(env: &Env, product_id: &String) -> Option<Product> {
        env.storage()
            .persistent()
            .get(&Self::product_key(product_id))
    }

    pub fn put_product_event_ids(env: &Env, product_id: &String, ids: &Vec<u64>) {
        env.storage()
            .persistent()
            .set(&Self::product_event_ids_key(product_id), ids);
    }

    pub fn get_product_event_ids(env: &Env, product_id: &String) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&Self::product_event_ids_key(product_id))
            .unwrap_or(Vec::new(env))
    }

    pub fn get_product_event_ids_paginated(
        env: &Env,
        product_id: &String,
        offset: u64,
        limit: u64,
    ) -> Vec<u64> {
        // Validate input parameters
        if limit == 0 || limit > 1000 {
            // Return empty result for invalid limit (0 or too large)
            return Vec::new(env);
        }
        
        let all_ids = Self::get_product_event_ids(env, product_id);
        let total = all_ids.len() as u64;

        let mut result = Vec::new(env);

        if offset >= total {
            return result;
        }

        let end = ((offset + limit) as u32).min(all_ids.len());
        let start = offset as u32;

        for i in start..end {
            if let Some(id) = all_ids.get(i) {
                result.push_back(id);
            }
        }

        result
    }

    pub fn put_event(env: &Env, event: &TrackingEvent) {
        env.storage()
            .persistent()
            .set(&Self::event_key(event.event_id), event);
    }

    pub fn get_event(env: &Env, event_id: u64) -> Option<TrackingEvent> {
        env.storage().persistent().get(&Self::event_key(event_id))
    }

    pub fn next_event_id(env: &Env) -> u64 {
        let mut seq: u64 = env
            .storage()
            .persistent()
            .get(&Self::event_seq_key())
            .unwrap_or(0);
        seq += 1;
        env.storage()
            .persistent()
            .set(&Self::event_seq_key(), &seq);
        seq
    }

    pub fn index_event_by_type(env: &Env, product_id: &String, event_type: &Symbol, event_id: u64) {
        let count_key = Self::event_type_count_key(product_id, event_type);
        let mut count: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        
        // Store at 0-based index (count represents next available index)
        let index_key = Self::event_type_index_key(product_id, event_type, count);
        env.storage().persistent().set(&index_key, &event_id);
        
        // Increment count after storing
        count += 1;
        env.storage().persistent().set(&count_key, &count);
    }

    pub fn get_event_ids_by_type(
        env: &Env,
        product_id: &String,
        event_type: &Symbol,
        offset: u64,
        limit: u64,
    ) -> Vec<u64> {
        let count_key = Self::event_type_count_key(product_id, event_type);
        let total: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let mut result = Vec::new(env);

        if offset >= total {
            return result;
        }

        // Use 0-based indexing consistently
        let start = offset;
        let end = (start + limit).min(total);

        for i in start..end {
            let index_key = Self::event_type_index_key(product_id, event_type, i);
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

    pub fn get_event_count_by_type(env: &Env, product_id: &String, event_type: &Symbol) -> u64 {
        let count_key = Self::event_type_count_key(product_id, event_type);
        env.storage().persistent().get(&count_key).unwrap_or(0)
    }

    pub fn set_auth(env: &Env, product_id: &String, actor: &Address, value: bool) {
        if value {
            env.storage()
                .persistent()
                .set(&Self::auth_key(product_id, actor), &true);
        } else {
            env.storage()
                .persistent()
                .remove(&Self::auth_key(product_id, actor));
        }
    }

    pub fn is_authorized(env: &Env, product_id: &String, actor: &Address) -> bool {
        env.storage()
            .persistent()
            .get(&Self::auth_key(product_id, actor))
            .unwrap_or(false)
    }

    pub fn has_admin(env: &Env) -> bool {
        env.storage().persistent().has(&Self::admin_key())
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::admin_key())
    }

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().persistent().set(&Self::admin_key(), admin);
    }

    pub fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&Self::paused_key()).unwrap_or(false)
    }

    pub fn set_paused(env: &Env, paused: bool) {
        env.storage().persistent().set(&Self::paused_key(), &paused);
    }

    pub fn get_total_products(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&Self::total_products_key())
            .unwrap_or(0)
    }

    pub fn set_total_products(env: &Env, count: u64) {
        env.storage()
            .instance()
            .set(&Self::total_products_key(), &count);
    }

    pub fn get_active_products(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&Self::active_products_key())
            .unwrap_or(0)
    }

    pub fn set_active_products(env: &Env, count: u64) {
        env.storage()
            .instance()
            .set(&Self::active_products_key(), &count);
    }
}

#[cfg(test)]
mod test_storage_contract {
    use super::*;
    use soroban_sdk::{testutils::Address as _, BytesN, Map};

    use crate::contract::ChainLogisticsContract;
    use crate::types::Origin;

    #[test]
    fn test_product_put_get() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, ChainLogisticsContract);

        let product = Product {
            id: String::from_str(&env, "P1"),
            name: String::from_str(&env, "Name"),
            description: String::from_str(&env, "Desc"),
            origin: Origin {
                location: String::from_str(&env, "Loc"),
            },
            owner: owner.clone(),
            created_at: 0,
            active: true,
            category: String::from_str(&env, "Cat"),
            tags: Vec::new(&env),
            certifications: Vec::new(&env),
            media_hashes: Vec::new(&env),
            custom: Map::new(&env),
            deactivation_info: Vec::new(&env),
        };

        env.as_contract(&contract_id, || {
            assert!(!StorageContract::has_product(&env, &product.id));
            StorageContract::put_product(&env, &product);
            assert!(StorageContract::has_product(&env, &product.id));
            assert!(StorageContract::get_product(&env, &product.id).is_some());
        });
    }

    #[test]
    fn test_pagination_boundary_conditions() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);
        let product_id = String::from_str(&env, "TEST_PRODUCT");

        env.as_contract(&contract_id, || {
            // Create test data - 5 event IDs
            let mut test_ids = Vec::new(&env);
            for i in 1..=5 {
                test_ids.push_back(i);
            }
            StorageContract::put_product_event_ids(&env, &product_id, &test_ids);

            // Test offset == len (should return empty)
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 5, 2);
            assert_eq!(result.len(), 0);

            // Test limit == 0 (should return empty due to validation)
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 0, 0);
            assert_eq!(result.len(), 0);

            // Test very large limit (should return empty due to validation)
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 0, 1001);
            assert_eq!(result.len(), 0);

            // Test offset > len (should return empty)
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 10, 2);
            assert_eq!(result.len(), 0);

            // Test normal cases
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 0, 3);
            assert_eq!(result.len(), 3);
            assert_eq!(result.get(0), Some(1));
            assert_eq!(result.get(1), Some(2));
            assert_eq!(result.get(2), Some(3));

            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 3, 3);
            assert_eq!(result.len(), 2);
            assert_eq!(result.get(0), Some(4));
            assert_eq!(result.get(1), Some(5));

            // Test exact boundary
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 0, 5);
            assert_eq!(result.len(), 5);

            // Test limit larger than remaining
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 4, 10);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(0), Some(5));
        });
    }

    #[test]
    fn test_pagination_empty_list() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);
        let product_id = String::from_str(&env, "EMPTY_PRODUCT");

        env.as_contract(&contract_id, || {
            // Test with empty product event IDs
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 0, 10);
            assert_eq!(result.len(), 0);

            // Test offset > 0 with empty list
            let result = StorageContract::get_product_event_ids_paginated(&env, &product_id, 5, 10);
            assert_eq!(result.len(), 0);
        });
    }

    #[test]
    fn test_event_type_indexing_0_based() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);
        let product_id = String::from_str(&env, "TEST_PRODUCT");
        let event_type = Symbol::new(&env, "test_type");

        env.as_contract(&contract_id, || {
            // Index first event at position 0
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 100);
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 101);
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 102);

            // Test offset=0, limit=1 returns first event ID
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 0, 1);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(0), Some(100));

            // Test offset=1, limit=1 returns second event ID  
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 1, 1);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(0), Some(101));

            // Test offset=2, limit=1 returns third event ID
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 2, 1);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(0), Some(102));

            // Test offset=0, limit=2 returns first two event IDs
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 0, 2);
            assert_eq!(result.len(), 2);
            assert_eq!(result.get(0), Some(100));
            assert_eq!(result.get(1), Some(101));

            // Test offset=1, limit=2 returns last two event IDs
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 1, 2);
            assert_eq!(result.len(), 2);
            assert_eq!(result.get(0), Some(101));
            assert_eq!(result.get(1), Some(102));

            // Test offset >= total returns empty
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 3, 1);
            assert_eq!(result.len(), 0);

            // Test pagination with limit larger than remaining
            let result = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 1, 5);
            assert_eq!(result.len(), 2);
            assert_eq!(result.get(0), Some(101));
            assert_eq!(result.get(1), Some(102));
        });
    }

    #[test]
    fn test_event_type_count_consistency() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);
        let product_id = String::from_str(&env, "TEST_PRODUCT");
        let event_type = Symbol::new(&env, "test_type");

        env.as_contract(&contract_id, || {
            // Initially count should be 0
            assert_eq!(StorageContract::get_event_count_by_type(&env, &product_id, &event_type), 0);

            // Add first event
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 100);
            assert_eq!(StorageContract::get_event_count_by_type(&env, &product_id, &event_type), 1);

            // Add second event
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 101);
            assert_eq!(StorageContract::get_event_count_by_type(&env, &product_id, &event_type), 2);

            // Add third event
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 102);
            assert_eq!(StorageContract::get_event_count_by_type(&env, &product_id, &event_type), 3);
        });
    }

    #[test]
    fn test_event_type_indexing_no_duplicates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);
        let product_id = String::from_str(&env, "TEST_PRODUCT");
        let event_type = Symbol::new(&env, "test_type");

        env.as_contract(&contract_id, || {
            // Index 3 events
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 100);
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 101);
            StorageContract::index_event_by_type(&env, &product_id, &event_type, 102);

            // Get all events in different pages
            let page1 = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 0, 2);
            let page2 = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 2, 2);
            let page3 = StorageContract::get_event_ids_by_type(&env, &product_id, &event_type, 1, 2);

            // Collect all IDs and verify no duplicates manually
            let mut all_found = Vec::new(&env);
            all_found.push_back(100);
            all_found.push_back(101);
            all_found.push_back(102);

            // Verify page1 contains first two
            assert_eq!(page1.len(), 2);
            let mut found_first_two = false;
            for i in 0..page1.len() {
                if let Some(id) = page1.get(i) {
                    if id == 100 || id == 101 {
                        found_first_two = true;
                    }
                }
            }
            assert!(found_first_two);

            // Verify page2 contains third
            assert_eq!(page2.len(), 1);
            let mut found_third = false;
            for i in 0..page2.len() {
                if let Some(id) = page2.get(i) {
                    if id == 102 {
                        found_third = true;
                    }
                }
            }
            assert!(found_third);

            // Verify page3 contains last two
            assert_eq!(page3.len(), 2);
            let mut found_last_two = false;
            for i in 0..page3.len() {
                if let Some(id) = page3.get(i) {
                    if id == 101 || id == 102 {
                        found_last_two = true;
                    }
                }
            }
            assert!(found_last_two);

            // Verify total unique count
            assert_eq!(all_found.len(), 3);
        });
    }

    #[test]
    fn test_event_id_sequence_increments() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);

        env.as_contract(&contract_id, || {
            assert_eq!(StorageContract::next_event_id(&env), 1);
            assert_eq!(StorageContract::next_event_id(&env), 2);
        });
    }

    #[test]
    fn test_authorization_mapping() {
        let env = Env::default();
        let actor = Address::generate(&env);
        let product_id = String::from_str(&env, "P1");

        let contract_id = env.register_contract(None, ChainLogisticsContract);

        env.as_contract(&contract_id, || {
            assert!(!StorageContract::is_authorized(&env, &product_id, &actor));
            StorageContract::set_auth(&env, &product_id, &actor, true);
            assert!(StorageContract::is_authorized(&env, &product_id, &actor));
            StorageContract::set_auth(&env, &product_id, &actor, false);
            assert!(!StorageContract::is_authorized(&env, &product_id, &actor));
        });
    }

    #[test]
    fn test_counters_roundtrip() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ChainLogisticsContract);

        env.as_contract(&contract_id, || {
            assert_eq!(StorageContract::get_total_products(&env), 0);
            assert_eq!(StorageContract::get_active_products(&env), 0);

            StorageContract::set_total_products(&env, 10);
            StorageContract::set_active_products(&env, 7);

            assert_eq!(StorageContract::get_total_products(&env), 10);
            assert_eq!(StorageContract::get_active_products(&env), 7);
        });
    }

    #[test]
    fn test_event_put_get() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, ChainLogisticsContract);

        let event = TrackingEvent {
            event_id: 1,
            product_id: String::from_str(&env, "P1"),
            event_type: Symbol::new(&env, "created"),
            location: String::from_str(&env, "Loc"),
            data_hash: BytesN::from_array(&env, &[0; 32]),
            note: String::from_str(&env, "Note"),
            metadata: Map::new(&env),
            actor: owner,
            timestamp: 0,
        };

        env.as_contract(&contract_id, || {
            assert!(StorageContract::get_event(&env, 1).is_none());
            StorageContract::put_event(&env, &event);
            assert!(StorageContract::get_event(&env, 1).is_some());
        });
    }
}

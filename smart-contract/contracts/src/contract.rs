use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::error::Error;
use crate::types::{Product, TrackingEvent, TrackingEventFilter, TrackingEventPage};
use crate::validation_contract::ValidationContract;
use crate::{storage, AuthorizationContractClient};

// ─── Internal helpers ────────────────────────────────────────────────────────

fn require_not_paused(env: &Env) -> Result<(), Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    if &admin == caller {
        caller.require_auth();
        return Ok(());
    }

    if let Some(multisig) = storage::get_multisig_contract(env) {
        if &multisig == caller && env.current_contract_address() == multisig {
            return Ok(());
        }
    }

    Err(Error::Unauthorized)
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

fn require_can_add_event(
    env: &Env,
    product_id: &String,
    product: &Product,
    caller: &Address,
) -> Result<(), Error> {
    caller.require_auth();

    if !product.active {
        return Err(Error::ProductDeactivated);
    }

    let auth_contract = storage::get_auth_contract(env).ok_or(Error::NotInitialized)?;
    let auth_client = AuthorizationContractClient::new(env, &auth_contract);

    // Delegate check to AuthorizationContract
    if !auth_client.is_authorized(product_id, caller) {
        return Err(Error::Unauthorized);
    }

    Ok(())
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct ChainLogisticsContract;

#[contractimpl]
impl ChainLogisticsContract {
    const MAX_EVENT_ID_PAGE_LIMIT: u32 = 100;

    pub fn init(env: Env, admin: Address, auth_contract: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);
        storage::set_auth_contract(&env, &auth_contract);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    pub fn pause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }
        storage::set_paused(&env, true);
        Ok(())
    }

    pub fn unpause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        if !storage::is_paused(&env) {
            return Err(Error::ContractNotPaused);
        }
        storage::set_paused(&env, false);
        Ok(())
    }

    pub fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), Error> {
        require_admin(&env, &current_admin)?;
        new_admin.require_auth();
        storage::set_admin(&env, &new_admin);
        Ok(())
    }

    pub fn set_multisig_contract(
        env: Env,
        caller: Address,
        multisig_contract: Address,
    ) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        storage::set_multisig_contract(&env, &multisig_contract);
        Ok(())
    }

    // Note: register_product, deactivate_product, reactivate_product,
    // get_product, and get_stats have been extracted to ProductRegistryContract
    // in product_registry.rs

    // Note: transfer_product is now in ProductTransferContract
    // get_product_event_ids, get_event_count are now in ProductQueryContract

    pub fn add_tracking_event(
        env: Env,
        actor: Address,
        product_id: String,
        event_type: Symbol,
        location: String,
        data_hash: BytesN<32>,
        note: String,
        metadata: Map<Symbol, String>,
    ) -> Result<u64, Error> {
        require_not_paused(&env)?;
        let product = read_product(&env, &product_id)?;
        require_can_add_event(&env, &product_id, &product, &actor)?;

        ValidationContract::validate_event_location(&location)?;
        ValidationContract::validate_event_note(&note)?;
        ValidationContract::validate_metadata(&metadata)?;

        let event_id = storage::next_event_id(&env);
        let event = TrackingEvent {
            event_id,
            product_id: product_id.clone(),
            actor,
            timestamp: env.ledger().timestamp(),
            event_type: event_type.clone(),
            location,
            data_hash,
            note,
            metadata,
        };

        storage::put_event(&env, &event);

        let mut ids = storage::get_product_event_ids(&env, &product_id);
        ids.push_back(event_id);
        storage::put_product_event_ids(&env, &product_id, &ids);

        storage::index_event_by_type(&env, &product_id, &event_type, event_id);

        env.events().publish(
            (
                Symbol::new(&env, "tracking_event"),
                product_id.clone(),
                event_id,
            ),
            event.clone(),
        );

        Ok(event_id)
    }

    pub fn get_event(env: Env, event_id: u64) -> Result<TrackingEvent, Error> {
        storage::get_event(&env, event_id).ok_or(Error::EventNotFound)
    }

    pub fn get_product_events(
        env: Env,
        product_id: String,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let total_count = all_ids.len() as u64;

        let event_ids = storage::get_product_event_ids_paginated(&env, &product_id, offset, limit);

        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let eid = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (event_ids.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    pub fn get_events_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let total_count = storage::get_event_count_by_type(&env, &product_id, &event_type);
        let event_ids =
            storage::get_event_ids_by_type(&env, &product_id, &event_type, offset, limit);

        let mut events = Vec::new(&env);
        for i in 0..event_ids.len() {
            let eid = event_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (event_ids.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    pub fn get_events_by_time_range(
        env: Env,
        product_id: String,
        start_time: u64,
        end_time: u64,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);

        for i in 0..all_ids.len() {
            let eid = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                if event.timestamp >= start_time && event.timestamp <= end_time {
                    matching_ids.push_back(eid);
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        for i in start..end {
            let eid = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (events.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    pub fn get_filtered_events(
        env: Env,
        product_id: String,
        filter: TrackingEventFilter,
        offset: u64,
        limit: u64,
    ) -> Result<TrackingEventPage, Error> {
        let _ = read_product(&env, &product_id)?;

        let all_ids = storage::get_product_event_ids(&env, &product_id);
        let mut matching_ids = Vec::new(&env);

        let empty_sym = Symbol::new(&env, "");
        let empty_loc = String::from_str(&env, "");

        for i in 0..all_ids.len() {
            let eid = all_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                let mut matches = true;

                if filter.event_type != empty_sym && event.event_type != filter.event_type {
                    matches = false;
                }
                if filter.start_time > 0 && event.timestamp < filter.start_time {
                    matches = false;
                }
                if filter.end_time < u64::MAX && event.timestamp > filter.end_time {
                    matches = false;
                }
                if filter.location != empty_loc && event.location != filter.location {
                    matches = false;
                }

                if matches {
                    matching_ids.push_back(eid);
                }
            }
        }

        let total_count = matching_ids.len() as u64;

        let mut events = Vec::new(&env);
        let start = offset as u32;
        let end = ((offset + limit) as u32).min(matching_ids.len());

        for i in start..end {
            let eid = matching_ids.get_unchecked(i);
            if let Some(event) = storage::get_event(&env, eid) {
                events.push_back(event);
            }
        }

        let has_more = offset + (events.len() as u64) < total_count;

        Ok(TrackingEventPage {
            events,
            total_count,
            has_more,
        })
    }

    pub fn get_product_event_ids(env: Env, id: String) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &id)?;
        Ok(storage::get_product_event_ids(&env, &id))
    }

    pub fn get_product_event_ids_paginated(
        env: Env,
        product_id: String,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<u64>, Error> {
        let _ = read_product(&env, &product_id)?;
        if limit == 0 {
            return Ok(Vec::new(&env));
        }

        let bounded_limit = limit.min(Self::MAX_EVENT_ID_PAGE_LIMIT);
        Ok(storage::get_product_event_ids_paginated(
            &env,
            &product_id,
            offset as u64,
            bounded_limit as u64,
        ))
    }

    pub fn get_product_event_count(env: Env, product_id: String) -> Result<u32, Error> {
        let _ = read_product(&env, &product_id)?;
        Ok(storage::get_product_event_ids(&env, &product_id).len())
    }

    pub fn get_event_count(env: Env, product_id: String) -> Result<u64, Error> {
        let _ = read_product(&env, &product_id)?;
        let ids = storage::get_product_event_ids(&env, &product_id);
        Ok(ids.len() as u64)
    }

    pub fn get_event_count_by_type(
        env: Env,
        product_id: String,
        event_type: Symbol,
    ) -> Result<u64, Error> {
        let _ = read_product(&env, &product_id)?;
        Ok(storage::get_event_count_by_type(
            &env,
            &product_id,
            &event_type,
        ))
    }

    // Helper to simulate a multisig contract invoking pause via as_contract
    pub fn __simulate_multisig_pause(env: Env, caller: Address) -> Result<(), Error> {
        Self::pause(env, caller)
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_multisig_invoker_can_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        // Register the contract as the multisig address for this test
        let contract_id = env.register_contract(None, ChainLogisticsContract);
        env.as_contract(&contract_id.clone(), || {
            storage::set_admin(&env, &admin);
            // Set multisig contract to the contract itself and use contract_id as caller
            storage::set_multisig_contract(&env, &contract_id);
            assert!(!storage::is_paused(&env));
            ChainLogisticsContract::__simulate_multisig_pause(env.clone(), contract_id).unwrap();
            assert!(storage::is_paused(&env));
        });
    }
}

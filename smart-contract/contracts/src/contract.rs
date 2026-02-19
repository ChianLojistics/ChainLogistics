use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Vec};

use crate::error::Error;
use crate::storage::{
    get_counter, get_events, get_product, product_exists, save_counter, save_events, save_product,
};
use crate::types::{Product, SupplyChainEvent};

#[contract]
pub struct SupplyChainContract;

#[contractimpl]
impl SupplyChainContract {
    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn is_authorized(product: &Product, caller: &Address) -> bool {
        if &product.owner == caller {
            return true;
        }
        product.authorized_actors.contains(caller)
    }

    // -------------------------------------------------------------------------
    // Product management — Issue #3
    // -------------------------------------------------------------------------

    pub fn create_product(env: Env, owner: Address, name: String) -> Result<u64, Error> {
        owner.require_auth();

        if name.len() == 0 {
            return Err(Error::InvalidInput);
        }

        let counter = get_counter(&env);
        let product_id = counter + 1;
        save_counter(&env, product_id);

        let product = Product {
            id: product_id,
            name,
            owner,
            authorized_actors: Vec::new(&env),
            created_at: env.ledger().timestamp(),
        };

        save_product(&env, &product);

        env.events().publish(
            (symbol_short!("product"), symbol_short!("created")),
            product_id,
        );

        Ok(product_id)
    }

    pub fn get_product(env: Env, product_id: u64) -> Result<Product, Error> {
        get_product(&env, product_id).ok_or(Error::ProductNotFound)
    }

    pub fn add_event(
        env: Env,
        product_id: u64,
        caller: Address,
        description: String,
    ) -> Result<(), Error> {
        caller.require_auth();

        if description.len() == 0 {
            return Err(Error::InvalidInput);
        }

        let product = get_product(&env, product_id).ok_or(Error::ProductNotFound)?;

        if !Self::is_authorized(&product, &caller) {
            return Err(Error::NotAuthorized);
        }

        let event = SupplyChainEvent {
            product_id,
            description,
            timestamp: env.ledger().timestamp(),
            actor: caller.clone(),
        };

        let mut events = get_events(&env, product_id);
        events.push_back(event);
        save_events(&env, product_id, &events);

        env.events().publish(
            (symbol_short!("event"), symbol_short!("added")),
            (product_id, caller),
        );

        Ok(())
    }

    pub fn get_events(env: Env, product_id: u64) -> Result<Vec<SupplyChainEvent>, Error> {
        if !product_exists(&env, product_id) {
            return Err(Error::ProductNotFound);
        }
        Ok(get_events(&env, product_id))
    }

    // -------------------------------------------------------------------------
    // Authorization management — Issue #4
    // -------------------------------------------------------------------------

    pub fn add_authorized_actor(
        env: Env,
        product_id: u64,
        caller: Address,
        actor: Address,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut product = get_product(&env, product_id).ok_or(Error::ProductNotFound)?;

        if product.owner != caller {
            return Err(Error::NotAuthorized);
        }

        if product.authorized_actors.contains(&actor) {
            return Err(Error::AlreadyAuthorized);
        }

        product.authorized_actors.push_back(actor.clone());
        save_product(&env, &product);

        env.events().publish(
            (symbol_short!("auth"), symbol_short!("actradded")),
            (product_id, actor),
        );

        Ok(())
    }

    pub fn remove_authorized_actor(
        env: Env,
        product_id: u64,
        caller: Address,
        actor: Address,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut product = get_product(&env, product_id).ok_or(Error::ProductNotFound)?;

        if product.owner != caller {
            return Err(Error::NotAuthorized);
        }

        if actor == caller {
            return Err(Error::CannotRemoveSelf);
        }

        let pos = product
            .authorized_actors
            .iter()
            .position(|a| a == actor);

        let idx = pos.ok_or(Error::ActorNotFound)? as u32;
        product.authorized_actors.remove(idx);
        save_product(&env, &product);

        env.events().publish(
            (symbol_short!("auth"), symbol_short!("actrrmd")),
            (product_id, actor),
        );

        Ok(())
    }

    pub fn transfer_ownership(
        env: Env,
        product_id: u64,
        caller: Address,
        new_owner: Address,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut product = get_product(&env, product_id).ok_or(Error::ProductNotFound)?;

        if product.owner != caller {
            return Err(Error::NotAuthorized);
        }

        product.owner = new_owner.clone();
        save_product(&env, &product);

        env.events().publish(
            (symbol_short!("ownership"), symbol_short!("transfer")),
            (product_id, caller, new_owner),
        );

        Ok(())
    }
}

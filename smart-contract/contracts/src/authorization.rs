use soroban_sdk::{contract, contractimpl, Address, Env, String, contracttype};
use crate::error::Error;

#[contracttype]
#[derive(Clone)]
enum AuthDataKey {
    Owner(String),
    Authorized(String, Address),
}

#[contract]
pub struct AuthorizationContract;

#[contractimpl]
impl AuthorizationContract {
    /// Initialize product ownership in the authorization system.
    /// This should be called by the ChainLogisticsContract during product registration.
    pub fn init_product_owner(env: Env, product_id: String, owner: Address) -> Result<(), Error> {
        // In a real system, we'd want to restrict who can call this (e.g. only the main contract).
        // For this refactor, we'll keep it simple.
        if env.storage().persistent().has(&AuthDataKey::Owner(product_id.clone())) {
            return Err(Error::ProductAlreadyExists);
        }
        env.storage().persistent().set(&AuthDataKey::Owner(product_id), &owner);
        Ok(())
    }

    /// Update product ownership (transfer).
    pub fn update_product_owner(env: Env, old_owner: Address, product_id: String, new_owner: Address) -> Result<(), Error> {
        old_owner.require_auth();
        let owner: Address = env.storage().persistent().get(&AuthDataKey::Owner(product_id.clone())).ok_or(Error::ProductNotFound)?;
        
        if owner != old_owner {
            return Err(Error::Unauthorized);
        }

        env.storage().persistent().set(&AuthDataKey::Owner(product_id), &new_owner);
        Ok(())
    }

    /// Grant an actor the right to add tracking events to a product.
    pub fn add_authorized_actor(
        env: Env,
        owner: Address,
        product_id: String,
        actor: Address,
    ) -> Result<(), Error> {
        owner.require_auth();
        
        let current_owner: Address = env.storage().persistent().get(&AuthDataKey::Owner(product_id.clone())).ok_or(Error::ProductNotFound)?;
        if current_owner != owner {
            return Err(Error::Unauthorized);
        }

        env.storage().persistent().set(&AuthDataKey::Authorized(product_id, actor), &true);
        Ok(())
    }

    /// Revoke an actor's authorization.
    pub fn remove_authorized_actor(
        env: Env,
        owner: Address,
        product_id: String,
        actor: Address,
    ) -> Result<(), Error> {
        owner.require_auth();

        let current_owner: Address = env.storage().persistent().get(&AuthDataKey::Owner(product_id.clone())).ok_or(Error::ProductNotFound)?;
        if current_owner != owner {
            return Err(Error::Unauthorized);
        }

        env.storage().persistent().remove(&AuthDataKey::Authorized(product_id, actor));
        Ok(())
    }

    /// Check whether an actor is authorized.
    pub fn is_authorized(env: Env, product_id: String, actor: Address) -> Result<bool, Error> {
        let owner: Address = env.storage().persistent().get(&AuthDataKey::Owner(product_id.clone())).ok_or(Error::ProductNotFound)?;
        
        if owner == actor {
            return Ok(true);
        }

        Ok(env.storage().persistent().get(&AuthDataKey::Authorized(product_id, actor)).unwrap_or(false))
    }
}

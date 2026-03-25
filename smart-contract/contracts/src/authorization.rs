use crate::error::Error;
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone)]
enum AuthDataKey {
    Initializer,
    Owner(String),
    Authorized(String, Address),
}

#[contract]
pub struct AuthorizationContract;

#[contractimpl]
impl AuthorizationContract {
    pub fn configure_initializer(env: Env, initializer: Address) -> Result<(), Error> {
        match env
            .storage()
            .persistent()
            .get::<AuthDataKey, Address>(&AuthDataKey::Initializer)
        {
            None => {
                env.storage()
                    .persistent()
                    .set(&AuthDataKey::Initializer, &initializer);
                Ok(())
            }
            Some(existing) if existing == initializer => Ok(()),
            Some(_) => Err(Error::AlreadyInitialized),
        }
    }

    /// Initialize product ownership in the authorization system.
    /// This should be called by the ChainLogisticsContract during product registration.
    pub fn init_product_owner(
        env: Env,
        caller: Address,
        product_id: String,
        owner: Address,
    ) -> Result<(), Error> {
        let initializer: Address = env
            .storage()
            .persistent()
            .get(&AuthDataKey::Initializer)
            .ok_or(Error::NotInitialized)?;
        caller.require_auth();
        if caller != initializer {
            return Err(Error::Unauthorized);
        }

        if env
            .storage()
            .persistent()
            .has(&AuthDataKey::Owner(product_id.clone()))
        {
            return Err(Error::ProductAlreadyExists);
        }
        env.storage()
            .persistent()
            .set(&AuthDataKey::Owner(product_id), &owner);
        Ok(())
    }

    /// Update product ownership (transfer).
    pub fn update_product_owner(
        env: Env,
        old_owner: Address,
        product_id: String,
        new_owner: Address,
    ) -> Result<(), Error> {
        old_owner.require_auth();
        let owner: Address = env
            .storage()
            .persistent()
            .get(&AuthDataKey::Owner(product_id.clone()))
            .ok_or(Error::ProductNotFound)?;

        if owner != old_owner {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&AuthDataKey::Owner(product_id), &new_owner);
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

        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&AuthDataKey::Owner(product_id.clone()))
            .ok_or(Error::ProductNotFound)?;
        if current_owner != owner {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&AuthDataKey::Authorized(product_id, actor), &true);
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

        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&AuthDataKey::Owner(product_id.clone()))
            .ok_or(Error::ProductNotFound)?;
        if current_owner != owner {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .remove(&AuthDataKey::Authorized(product_id, actor));
        Ok(())
    }

    /// Check whether an actor is authorized.
    pub fn is_authorized(env: Env, product_id: String, actor: Address) -> Result<bool, Error> {
        let owner: Address = env
            .storage()
            .persistent()
            .get(&AuthDataKey::Owner(product_id.clone()))
            .ok_or(Error::ProductNotFound)?;

        if owner == actor {
            return Ok(true);
        }

        Ok(env
            .storage()
            .persistent()
            .get(&AuthDataKey::Authorized(product_id, actor))
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod test_authorization {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_init_product_owner_requires_trusted_initializer() {
        let env = Env::default();
        env.mock_all_auths();

        let auth_id = env.register_contract(None, AuthorizationContract);
        let auth_client = AuthorizationContractClient::new(&env, &auth_id);

        let trusted = Address::generate(&env);
        let attacker = Address::generate(&env);
        let owner = Address::generate(&env);
        let product_id = String::from_str(&env, "PROD1");

        auth_client.configure_initializer(&trusted);

        let res = auth_client.try_init_product_owner(&attacker, &product_id, &owner);
        assert_eq!(res, Err(Ok(Error::Unauthorized)));

        auth_client.init_product_owner(&trusted, &product_id, &owner);
        assert!(auth_client.is_authorized(&product_id, &owner));
    }
}

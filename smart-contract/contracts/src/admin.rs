use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

use crate::error::Error;
use crate::types::DataKey;

// ─── Storage helpers ─────────────────────────────────────────────────────────

fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

fn has_admin(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Admin)
}

fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&DataKey::Paused).unwrap_or(false)
}

fn set_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&DataKey::Paused, &paused);
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = get_admin(env).ok_or(Error::NotInitialized)?;
    caller.require_auth();
    if &admin != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// ─── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct AdminContract;

#[contractimpl]
impl AdminContract {
    /// Initialize the contract with an admin address.
    /// Can only be called once.
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        set_admin(&env, &admin);
        set_paused(&env, false);

        // Emit initialization event
        env.events().publish(
            (Symbol::new(&env, "admin_initialized"),),
            admin,
        );

        Ok(())
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        get_admin(&env).ok_or(Error::NotInitialized)
    }

    /// Check if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Pause contract operations.
    /// Only the admin can pause.
    pub fn pause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        if is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        set_paused(&env, true);

        // Emit pause event
        env.events().publish(
            (Symbol::new(&env, "contract_paused"),),
            caller,
        );

        Ok(())
    }

    /// Unpause contract operations.
    /// Only the admin can unpause.
    pub fn unpause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        if !is_paused(&env) {
            return Err(Error::ContractNotPaused);
        }

        set_paused(&env, false);

        // Emit unpause event
        env.events().publish(
            (Symbol::new(&env, "contract_unpaused"),),
            caller,
        );

        Ok(())
    }

    /// Transfer admin privileges to a new address.
    /// Requires authentication from both current and new admin.
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) -> Result<(), Error> {
        require_admin(&env, &current_admin)?;
        new_admin.require_auth();

        set_admin(&env, &new_admin);

        // Emit transfer event
        env.events().publish(
            (Symbol::new(&env, "admin_transferred"),),
            (current_admin, new_admin),
        );

        Ok(())
    }
}

#[cfg(test)]
mod test_admin {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup(env: &Env) -> (AdminContractClient, Address) {
        let contract_id = env.register_contract(None, AdminContract);
        let client = AdminContractClient::new(env, &contract_id);
        let admin = Address::generate(env);

        // Initialize the contract
        client.init(&admin);

        (client, admin)
    }

    #[test]
    fn test_init() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AdminContract);
        let client = AdminContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        // Initialize should succeed
        client.init(&admin);

        // Verify admin is set
        let retrieved_admin = client.get_admin();
        assert_eq!(retrieved_admin, admin);

        // Verify not paused
        assert!(!client.is_paused());
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _admin) = setup(&env);
        let new_admin = Address::generate(&env);

        // Second init should fail
        let res = client.try_init(&new_admin);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_get_admin_not_initialized() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AdminContract);
        let client = AdminContractClient::new(&env, &contract_id);

        // Get admin before init should fail
        let res = client.try_get_admin();
        assert_eq!(res, Err(Ok(Error::NotInitialized)));
    }

    #[test]
    fn test_pause_unpause() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);

        // Initially not paused
        assert!(!client.is_paused());

        // Pause
        client.pause(&admin);
        assert!(client.is_paused());

        // Unpause
        client.unpause(&admin);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_pause_already_paused_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);

        // Pause
        client.pause(&admin);
        assert!(client.is_paused());

        // Pausing again should fail
        let res = client.try_pause(&admin);
        assert_eq!(res, Err(Ok(Error::ContractPaused)));
    }

    #[test]
    fn test_unpause_not_paused_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);

        // Initially not paused
        assert!(!client.is_paused());

        // Unpausing should fail
        let res = client.try_unpause(&admin);
        assert_eq!(res, Err(Ok(Error::ContractNotPaused)));
    }

    #[test]
    fn test_pause_unpause_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _admin) = setup(&env);
        let attacker = Address::generate(&env);

        // Non-admin pause attempt should fail
        let res = client.try_pause(&attacker);
        assert!(res.is_err());

        // Non-admin unpause attempt should fail
        let res = client.try_unpause(&attacker);
        assert!(res.is_err());
    }

    #[test]
    fn test_transfer_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);
        let new_admin = Address::generate(&env);

        // Transfer admin
        client.transfer_admin(&admin, &new_admin);

        // Verify new admin
        let retrieved_admin = client.get_admin();
        assert_eq!(retrieved_admin, new_admin);
    }

    #[test]
    fn test_transfer_admin_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _admin) = setup(&env);
        let attacker = Address::generate(&env);
        let new_admin = Address::generate(&env);

        // Non-admin transfer attempt should fail
        let res = client.try_transfer_admin(&attacker, &new_admin);
        assert!(res.is_err());
    }

    #[test]
    fn test_old_admin_loses_privileges_after_transfer() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, old_admin) = setup(&env);
        let new_admin = Address::generate(&env);

        // Transfer admin
        client.transfer_admin(&old_admin, &new_admin);

        // Old admin should no longer be able to pause
        let res = client.try_pause(&old_admin);
        assert!(res.is_err());

        // New admin should be able to pause
        client.pause(&new_admin);
        assert!(client.is_paused());
    }
}

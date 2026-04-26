use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

use crate::error::Error;
use crate::types::{ContractVersion, DataKey, UpgradeInfo, UpgradeStatus};
use crate::ChainLogisticsContractClient;

// ─── Storage helpers ─────────────────────────────────────────────────────────

fn get_contract_version(env: &Env) -> ContractVersion {
    env.storage()
        .persistent()
        .get(&DataKey::ContractVersion)
        .unwrap_or(ContractVersion {
            major: 1,
            minor: 0,
            patch: 0,
        })
}

fn set_contract_version(env: &Env, version: &ContractVersion) {
    env.storage()
        .persistent()
        .set(&DataKey::ContractVersion, version);
}

fn get_upgrade_info(env: &Env) -> Option<UpgradeInfo> {
    env.storage().persistent().get(&DataKey::UpgradeInfo)
}

fn set_upgrade_info(env: &Env, info: &UpgradeInfo) {
    env.storage().persistent().set(&DataKey::UpgradeInfo, info);
}

fn get_upgrade_status(env: &Env) -> UpgradeStatus {
    env.storage()
        .persistent()
        .get(&DataKey::UpgradeStatus)
        .unwrap_or(UpgradeStatus::NotStarted)
}

fn set_upgrade_status(env: &Env, status: &UpgradeStatus) {
    env.storage()
        .persistent()
        .set(&DataKey::UpgradeStatus, status);
}

fn get_emergency_pause(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::EmergencyPause)
        .unwrap_or(false)
}

fn set_emergency_pause(env: &Env, paused: &bool) {
    env.storage()
        .persistent()
        .set(&DataKey::EmergencyPause, paused);
}

fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn get_timelock_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::TimelockContract)
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = get_admin(env).ok_or(Error::NotInitialized)?;
    if &admin == caller {
        caller.require_auth();
        return Ok(());
    }

    if let Some(multisig) = env
        .storage()
        .persistent()
        .get::<crate::types::DataKey, Address>(&crate::types::DataKey::MultiSigContract)
    {
        if &multisig == caller {
            if env.current_contract_address() != multisig {
                caller.require_auth();
            }
            return Ok(());
        }
    }

    if let Some(timelock) = get_timelock_contract(env) {
        if &timelock == caller {
            if env.current_contract_address() != timelock {
                caller.require_auth();
            }
            return Ok(());
        }
    }

    Err(Error::Unauthorized)
}

fn require_not_emergency_paused(env: &Env) -> Result<(), Error> {
    if get_emergency_pause(env) {
        return Err(Error::EmergencyPaused);
    }
    Ok(())
}

fn validate_version_upgrade(
    old_version: &ContractVersion,
    new_version: &ContractVersion,
) -> Result<(), Error> {
    // Ensure new version is actually newer
    if new_version.major < old_version.major {
        return Err(Error::InvalidUpgrade);
    }
    if new_version.major == old_version.major && new_version.minor < old_version.minor {
        return Err(Error::InvalidUpgrade);
    }
    if new_version.major == old_version.major
        && new_version.minor == old_version.minor
        && new_version.patch <= old_version.patch
    {
        return Err(Error::InvalidUpgrade);
    }

    // Ensure version jump is reasonable (no major jumps without proper process)
    if new_version.major > old_version.major + 1 {
        return Err(Error::InvalidUpgrade);
    }

    Ok(())
}

// ─── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct UpgradeContract;

#[contractimpl]
impl UpgradeContract {
    /// Get the current contract version
    pub fn get_version(env: Env) -> ContractVersion {
        get_contract_version(&env)
    }

    /// Get current upgrade status
    pub fn get_upgrade_status(env: Env) -> UpgradeStatus {
        get_upgrade_status(&env)
    }

    /// Set multisig contract
    pub fn set_multisig_contract(
        env: Env,
        caller: Address,
        multisig_contract: Address,
    ) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::MultiSigContract, &multisig_contract);
        Ok(())
    }

    pub fn set_timelock_contract(
        env: Env,
        caller: Address,
        timelock_contract: Address,
    ) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::TimelockContract, &timelock_contract);
        Ok(())
    }

    /// Get current upgrade information
    pub fn get_upgrade_info(env: Env) -> Option<UpgradeInfo> {
        get_upgrade_info(&env)
    }

    /// Check if emergency pause is active
    pub fn is_emergency_paused(env: Env) -> bool {
        get_emergency_pause(&env)
    }

    /// Initiate an upgrade process
    /// Only admin can call this
    pub fn initiate_upgrade(
        env: Env,
        caller: Address,
        new_version: ContractVersion,
        new_contract_address: Address,
        migration_required: bool,
    ) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        require_not_emergency_paused(&env)?;

        let current_version = get_contract_version(&env);
        validate_version_upgrade(&current_version, &new_version)?;

        // Check if there's already an upgrade in progress
        let current_status = get_upgrade_status(&env);
        if matches!(current_status, UpgradeStatus::InProgress) {
            return Err(Error::UpgradeInProgress);
        }

        // Create upgrade info
        let upgrade_info = UpgradeInfo {
            new_version: new_version.clone(),
            new_contract_address,
            upgrade_timestamp: env.ledger().timestamp(),
            upgraded_by: caller.clone(),
            migration_required,
        };

        // Set upgrade status to in progress
        set_upgrade_status(&env, &UpgradeStatus::InProgress);
        set_upgrade_info(&env, &upgrade_info);

        // Emit upgrade initiated event
        env.events().publish(
            (Symbol::new(&env, "upgrade_initiated"),),
            (current_version, new_version, caller.clone()),
        );

        Ok(())
    }

    /// Complete an upgrade process
    /// Only admin can call this
    pub fn complete_upgrade(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        let upgrade_status = get_upgrade_status(&env);
        if !matches!(upgrade_status, UpgradeStatus::InProgress) {
            return Err(Error::NoUpgradeInProgress);
        }

        let upgrade_info = get_upgrade_info(&env).ok_or(Error::NotInitialized)?;

        // Update contract version
        set_contract_version(&env, &upgrade_info.new_version);

        // Mark upgrade as completed
        set_upgrade_status(&env, &UpgradeStatus::Completed);

        // Emit upgrade completed event
        env.events().publish(
            (Symbol::new(&env, "upgrade_completed"),),
            (
                upgrade_info.new_version,
                upgrade_info.new_contract_address,
                caller,
            ),
        );

        Ok(())
    }

    /// Fail an upgrade process
    /// Only admin can call this
    pub fn fail_upgrade(env: Env, caller: Address, reason: Symbol) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        let upgrade_status = get_upgrade_status(&env);
        if !matches!(upgrade_status, UpgradeStatus::InProgress) {
            return Err(Error::NoUpgradeInProgress);
        }

        // Mark upgrade as failed
        set_upgrade_status(&env, &UpgradeStatus::Failed);

        // Emit upgrade failed event
        env.events()
            .publish((Symbol::new(&env, "upgrade_failed"),), (&caller, &reason));

        Ok(())
    }

    /// Emergency pause all operations
    /// Only admin can call this
    pub fn emergency_pause(env: Env, caller: Address, reason: Symbol) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        if get_emergency_pause(&env) {
            return Err(Error::EmergencyPaused);
        }

        // Set emergency pause
        let paused_true = true;
        set_emergency_pause(&env, &paused_true);

        // Also pause the main contract if available
        if let Some(main_contract) = get_main_contract(&env) {
            let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
            let _ = main_client.try_pause(&caller);
        }

        // Emit emergency pause event
        env.events()
            .publish((Symbol::new(&env, "emergency_pause"),), (&caller, &reason));

        Ok(())
    }

    /// Emergency unpause all operations
    /// Only admin can call this
    pub fn emergency_unpause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        if !get_emergency_pause(&env) {
            return Err(Error::NotEmergencyPaused);
        }

        // Clear emergency pause
        let paused_false = false;
        set_emergency_pause(&env, &paused_false);

        // Also unpause the main contract if available
        if let Some(main_contract) = get_main_contract(&env) {
            let main_client = ChainLogisticsContractClient::new(&env, &main_contract);
            let _ = main_client.try_unpause(&caller);
        }

        // Emit emergency unpause event
        env.events()
            .publish((Symbol::new(&env, "emergency_unpause"),), &caller);

        Ok(())
    }

    /// Reset upgrade status (for recovery from failed upgrades)
    /// Only admin can call this
    pub fn reset_upgrade_status(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        let upgrade_status = get_upgrade_status(&env);
        if matches!(upgrade_status, UpgradeStatus::InProgress) {
            return Err(Error::UpgradeInProgress);
        }

        // Clear upgrade info and reset status
        env.storage().persistent().remove(&DataKey::UpgradeInfo);
        set_upgrade_status(&env, &UpgradeStatus::NotStarted);

        // Emit upgrade reset event
        env.events()
            .publish((Symbol::new(&env, "upgrade_reset"),), &caller);

        Ok(())
    }
}

#[cfg(test)]
mod test_upgrade {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::{AuthorizationContract, ChainLogisticsContract, ChainLogisticsContractClient};

    fn setup(env: &Env) -> (UpgradeContractClient, Address) {
        let contract_id = env.register_contract(None, UpgradeContract);
        let client = UpgradeContractClient::new(env, &contract_id);
        let admin = Address::generate(env);

        let auth_id = env.register_contract(None, AuthorizationContract);
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        cl_client.init(&admin, &auth_id);

        // Set up admin and main contract references in the upgrade contract context
        let upgrade_contract_address = contract_id;
        env.as_contract(&upgrade_contract_address, || {
            env.storage().persistent().set(&DataKey::Admin, &admin);
            env.storage()
                .persistent()
                .set(&DataKey::MainContract, &cl_id);
        });

        (client, admin)
    }

    #[test]
    fn test_version_tracking() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _admin) = setup(&env);

        // Initial version should be 1.0.0
        let version = client.get_version();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_emergency_pause_unpause() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);

        // Initially not paused
        assert!(!client.is_emergency_paused());

        // Emergency pause
        client.emergency_pause(&admin, &Symbol::new(&env, "test"));
        assert!(client.is_emergency_paused());

        // Emergency unpause
        client.emergency_unpause(&admin);
        assert!(!client.is_emergency_paused());
    }

    #[test]
    fn test_upgrade_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);

        // Initial status
        assert_eq!(client.get_upgrade_status(), UpgradeStatus::NotStarted);
        assert!(client.get_upgrade_info().is_none());

        // Initiate upgrade
        let new_version = ContractVersion {
            major: 1,
            minor: 1,
            patch: 0,
        };
        let new_contract = Address::generate(&env);
        client.initiate_upgrade(&admin, &new_version, &new_contract, &false);

        // Check upgrade status
        assert_eq!(client.get_upgrade_status(), UpgradeStatus::InProgress);
        let upgrade_info = client.get_upgrade_info().unwrap();
        assert_eq!(upgrade_info.new_version, new_version);
        assert_eq!(upgrade_info.upgraded_by, admin);

        // Complete upgrade
        client.complete_upgrade(&admin);

        // Check final state
        assert_eq!(client.get_upgrade_status(), UpgradeStatus::Completed);
        let current_version = client.get_version();
        assert_eq!(current_version, new_version);
    }

    #[test]
    fn test_upgrade_validation() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup(&env);

        // Try to downgrade (should fail)
        let old_version = ContractVersion {
            major: 0,
            minor: 9,
            patch: 0,
        };
        let new_contract = Address::generate(&env);
        let res = client.try_initiate_upgrade(&admin, &old_version, &new_contract, &false);
        assert_eq!(res, Err(Ok(Error::InvalidUpgrade)));

        // Try major version jump (should fail)
        let jump_version = ContractVersion {
            major: 3,
            minor: 0,
            patch: 0,
        };
        let res = client.try_initiate_upgrade(&admin, &jump_version, &new_contract, &false);
        assert_eq!(res, Err(Ok(Error::InvalidUpgrade)));
    }

    #[test]
    fn test_unauthorized_access() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _admin) = setup(&env);
        let attacker = Address::generate(&env);

        // Unauthorized upgrade attempt should fail
        let new_version = ContractVersion {
            major: 1,
            minor: 1,
            patch: 0,
        };
        let new_contract = Address::generate(&env);
        let res = client.try_initiate_upgrade(&attacker, &new_version, &new_contract, &false);
        assert!(res.is_err());

        // Unauthorized emergency pause should fail
        let res = client.try_emergency_pause(&attacker, &Symbol::new(&env, "test"));
        assert!(res.is_err());
    }
}

use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, vec, Address, Env, IntoVal, Symbol, Val, Vec,
};

use crate::error::Error;
use crate::types::{DataKey, TimelockAction, TimelockConfig, TimelockOperation, TimelockStatus};
use crate::{ChainLogisticsContractClient, OracleSecurityContractClient, UpgradeContractClient};

const MIN_DELAY_FLOOR: u64 = 86_400;
const MAX_DELAY_CEILING: u64 = 7 * 86_400;

fn load_config(env: &Env) -> Option<TimelockConfig> {
    env.storage().persistent().get(&DataKey::TimelockConfig)
}

fn store_config(env: &Env, config: &TimelockConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::TimelockConfig, config);
}

fn get_next_operation_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextTimelockOperationId)
        .unwrap_or(1)
}

fn set_next_operation_id(env: &Env, next_id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::NextTimelockOperationId, &next_id);
}

fn load_operation(env: &Env, operation_id: u64) -> Option<TimelockOperation> {
    env.storage()
        .persistent()
        .get(&DataKey::TimelockOperation(operation_id))
}

fn store_operation(env: &Env, operation: &TimelockOperation) {
    env.storage()
        .persistent()
        .set(&DataKey::TimelockOperation(operation.id), operation);
}

fn require_signer(env: &Env, caller: &Address) -> Result<TimelockConfig, Error> {
    let config = load_config(env).ok_or(Error::MultiSigNotConfigured)?;
    if !config.signers.contains(caller) {
        return Err(Error::NotSigner);
    }
    caller.require_auth();
    Ok(config)
}

fn validate_config(env: &Env, config: &TimelockConfig) -> Result<(), Error> {
    if config.signers.is_empty() {
        return Err(Error::InvalidInput);
    }
    if config.threshold == 0 || config.threshold > config.signers.len() {
        return Err(Error::InvalidThreshold);
    }
    if config.min_delay_seconds < MIN_DELAY_FLOOR {
        return Err(Error::TimelockDelayTooShort);
    }
    if config.max_delay_seconds > MAX_DELAY_CEILING
        || config.max_delay_seconds < config.min_delay_seconds
    {
        return Err(Error::TimelockDelayTooLong);
    }
    if config.grace_period_seconds == 0 || config.grace_period_seconds > MAX_DELAY_CEILING {
        return Err(Error::TimelockDelayTooLong);
    }

    let mut seen = Vec::new(env);
    for signer in config.signers.iter() {
        if seen.contains(&signer) {
            return Err(Error::DuplicateSigner);
        }
        seen.push_back(signer);
    }

    Ok(())
}

fn maybe_queue(operation: &mut TimelockOperation, threshold: u32) {
    if operation.approvals.len() >= threshold
        && operation.status == TimelockStatus::PendingApprovals
    {
        operation.status = TimelockStatus::Queued;
    }
}

fn authorize_subcall(env: &Env, contract: &Address, fn_name: &str, args: Vec<Val>) {
    env.authorize_as_current_contract(vec![
        env,
        InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: contract.clone(),
                fn_name: Symbol::new(env, fn_name),
                args,
            },
            sub_invocations: vec![env],
        }),
    ]);
}

fn perform_action(env: &Env, config: &TimelockConfig, action: &TimelockAction) {
    let timelock = env.current_contract_address();

    match action {
        TimelockAction::PauseMain => {
            let args = vec![env, timelock.clone().into_val(env)];
            authorize_subcall(env, &config.main_contract, "pause", args);
            ChainLogisticsContractClient::new(env, &config.main_contract).pause(&timelock);
        }
        TimelockAction::UnpauseMain => {
            let args = vec![env, timelock.clone().into_val(env)];
            authorize_subcall(env, &config.main_contract, "unpause", args);
            ChainLogisticsContractClient::new(env, &config.main_contract).unpause(&timelock);
        }
        TimelockAction::SetMainMultisig(multisig_contract) => {
            let args = vec![
                env,
                timelock.clone().into_val(env),
                multisig_contract.clone().into_val(env),
            ];
            authorize_subcall(env, &config.main_contract, "set_multisig_contract", args);
            ChainLogisticsContractClient::new(env, &config.main_contract)
                .set_multisig_contract(&timelock, multisig_contract);
        }
        TimelockAction::InitiateUpgrade(version, new_contract, migration_required) => {
            let args = vec![
                env,
                timelock.clone().into_val(env),
                version.clone().into_val(env),
                new_contract.clone().into_val(env),
                (*migration_required).into_val(env),
            ];
            authorize_subcall(env, &config.upgrade_contract, "initiate_upgrade", args);
            UpgradeContractClient::new(env, &config.upgrade_contract).initiate_upgrade(
                &timelock,
                version,
                new_contract,
                migration_required,
            );
        }
        TimelockAction::CompleteUpgrade => {
            let args = vec![env, timelock.clone().into_val(env)];
            authorize_subcall(env, &config.upgrade_contract, "complete_upgrade", args);
            UpgradeContractClient::new(env, &config.upgrade_contract).complete_upgrade(&timelock);
        }
        TimelockAction::FailUpgrade(reason) => {
            let args = vec![
                env,
                timelock.clone().into_val(env),
                reason.clone().into_val(env),
            ];
            authorize_subcall(env, &config.upgrade_contract, "fail_upgrade", args);
            UpgradeContractClient::new(env, &config.upgrade_contract)
                .fail_upgrade(&timelock, reason);
        }
        TimelockAction::EmergencyPause(reason) => {
            let args = vec![
                env,
                timelock.clone().into_val(env),
                reason.clone().into_val(env),
            ];
            authorize_subcall(env, &config.upgrade_contract, "emergency_pause", args);
            UpgradeContractClient::new(env, &config.upgrade_contract)
                .emergency_pause(&timelock, reason);
        }
        TimelockAction::EmergencyUnpause => {
            let args = vec![env, timelock.clone().into_val(env)];
            authorize_subcall(env, &config.upgrade_contract, "emergency_unpause", args);
            UpgradeContractClient::new(env, &config.upgrade_contract).emergency_unpause(&timelock);
        }
        TimelockAction::ConfigureOracleFeed(oracle_contract, oracle_config) => {
            let args = vec![
                env,
                timelock.clone().into_val(env),
                oracle_config.clone().into_val(env),
            ];
            authorize_subcall(env, oracle_contract, "configure_feed", args);
            OracleSecurityContractClient::new(env, oracle_contract)
                .configure_feed(&timelock, oracle_config);
        }
        TimelockAction::SetOracleFallback(oracle_contract, feed_id, value, observed_at) => {
            let args = vec![
                env,
                timelock.clone().into_val(env),
                feed_id.clone().into_val(env),
                (*value).into_val(env),
                (*observed_at).into_val(env),
            ];
            authorize_subcall(env, oracle_contract, "set_fallback_value", args);
            OracleSecurityContractClient::new(env, oracle_contract).set_fallback_value(
                &timelock,
                feed_id,
                value,
                observed_at,
            );
        }
    }
}

#[contract]
pub struct TimelockContract;

#[contractimpl]
impl TimelockContract {
    pub fn timelock_init(env: Env, config: TimelockConfig) -> Result<(), Error> {
        if load_config(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        validate_config(&env, &config)?;
        for signer in config.signers.iter() {
            signer.require_auth();
        }
        store_config(&env, &config);
        set_next_operation_id(&env, 1);
        Ok(())
    }

    pub fn get_config(env: Env) -> Result<TimelockConfig, Error> {
        load_config(&env).ok_or(Error::MultiSigNotConfigured)
    }

    pub fn propose_action(
        env: Env,
        proposer: Address,
        action: TimelockAction,
        delay_seconds: u64,
    ) -> Result<u64, Error> {
        let config = require_signer(&env, &proposer)?;
        if delay_seconds < config.min_delay_seconds {
            return Err(Error::TimelockDelayTooShort);
        }
        if delay_seconds > config.max_delay_seconds {
            return Err(Error::TimelockDelayTooLong);
        }

        let operation_id = get_next_operation_id(&env);
        set_next_operation_id(&env, operation_id + 1);

        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer.clone());

        let mut operation = TimelockOperation {
            id: operation_id,
            proposer: proposer.clone(),
            action,
            created_at: env.ledger().timestamp(),
            ready_at: env.ledger().timestamp() + delay_seconds,
            execute_by: env.ledger().timestamp() + delay_seconds + config.grace_period_seconds,
            status: TimelockStatus::PendingApprovals,
            approvals,
        };
        maybe_queue(&mut operation, config.threshold);
        store_operation(&env, &operation);

        env.events().publish(
            (Symbol::new(&env, "timelock_proposed"), operation_id),
            operation.clone(),
        );
        Ok(operation_id)
    }

    pub fn approve_action(
        env: Env,
        approver: Address,
        operation_id: u64,
    ) -> Result<TimelockOperation, Error> {
        let config = require_signer(&env, &approver)?;
        let mut operation = load_operation(&env, operation_id).ok_or(Error::ProposalNotFound)?;

        if operation.status == TimelockStatus::Cancelled {
            return Err(Error::TimelockCancelled);
        }
        if operation.status == TimelockStatus::Executed {
            return Err(Error::ProposalAlreadyExecuted);
        }
        if operation.approvals.contains(&approver) {
            return Err(Error::AlreadyApproved);
        }

        operation.approvals.push_back(approver.clone());
        maybe_queue(&mut operation, config.threshold);
        store_operation(&env, &operation);
        Ok(operation)
    }

    pub fn cancel_action(
        env: Env,
        canceller: Address,
        operation_id: u64,
        reason: Symbol,
    ) -> Result<(), Error> {
        let _ = require_signer(&env, &canceller)?;
        let mut operation = load_operation(&env, operation_id).ok_or(Error::ProposalNotFound)?;
        if operation.status == TimelockStatus::Executed {
            return Err(Error::ProposalAlreadyExecuted);
        }
        operation.status = TimelockStatus::Cancelled;
        store_operation(&env, &operation);
        env.events().publish(
            (Symbol::new(&env, "timelock_cancelled"), operation_id),
            (canceller, reason),
        );
        Ok(())
    }

    pub fn execute_action(env: Env, executor: Address, operation_id: u64) -> Result<(), Error> {
        let config = require_signer(&env, &executor)?;
        let mut operation = load_operation(&env, operation_id).ok_or(Error::ProposalNotFound)?;

        if operation.status == TimelockStatus::Cancelled {
            return Err(Error::TimelockCancelled);
        }
        if operation.status == TimelockStatus::Executed {
            return Err(Error::ProposalAlreadyExecuted);
        }
        if operation.status != TimelockStatus::Queued {
            return Err(Error::ThresholdNotReached);
        }

        let now = env.ledger().timestamp();
        if now < operation.ready_at {
            return Err(Error::TimelockNotReady);
        }
        if now > operation.execute_by {
            return Err(Error::TimelockExpired);
        }

        perform_action(&env, &config, &operation.action);
        operation.status = TimelockStatus::Executed;
        store_operation(&env, &operation);
        Ok(())
    }

    pub fn get_operation(env: Env, operation_id: u64) -> Result<TimelockOperation, Error> {
        load_operation(&env, operation_id).ok_or(Error::ProposalNotFound)
    }
}

#[cfg(test)]
mod test_timelock {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, Env, Symbol};

    use crate::{
        AuthorizationContract, ChainLogisticsContract, OracleFeedConfig, OracleFeedType,
        OracleSecurityContract, OracleSecurityContractClient, TimelockAction, TimelockConfig,
        TimelockStatus, UpgradeContract, UpgradeContractClient,
    };

    fn setup(
        env: &Env,
    ) -> (
        TimelockContractClient,
        Address,
        Address,
        crate::ChainLogisticsContractClient,
        UpgradeContractClient,
        Address,
    ) {
        let timelock_id = env.register_contract(None, TimelockContract);
        let timelock_client = TimelockContractClient::new(env, &timelock_id);

        let auth_id = env.register_contract(None, AuthorizationContract);
        let main_id = env.register_contract(None, ChainLogisticsContract);
        let upgrade_id = env.register_contract(None, UpgradeContract);

        let admin = Address::generate(env);
        let reviewer = Address::generate(env);
        let main_client = crate::ChainLogisticsContractClient::new(env, &main_id);
        let upgrade_client = UpgradeContractClient::new(env, &upgrade_id);

        main_client.init(&admin, &auth_id);
        env.as_contract(&upgrade_id, || {
            env.storage().persistent().set(&DataKey::Admin, &admin);
            env.storage()
                .persistent()
                .set(&DataKey::MainContract, &main_id);
        });

        timelock_client.timelock_init(&TimelockConfig {
            signers: soroban_sdk::vec![env, admin.clone(), reviewer.clone()],
            threshold: 2,
            min_delay_seconds: MIN_DELAY_FLOOR,
            max_delay_seconds: 3 * 86_400,
            grace_period_seconds: 86_400,
            main_contract: main_id.clone(),
            upgrade_contract: upgrade_id.clone(),
        });

        main_client.set_timelock_contract(&admin, &timelock_id);
        upgrade_client.set_timelock_contract(&admin, &timelock_id);

        (
            timelock_client,
            admin,
            reviewer,
            main_client,
            upgrade_client,
            timelock_id,
        )
    }

    #[test]
    fn test_timelock_pauses_main_contract_after_delay() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let (timelock, admin, reviewer, main_client, _upgrade, _timelock_id) = setup(&env);
        let operation_id =
            timelock.propose_action(&admin, &TimelockAction::PauseMain, &MIN_DELAY_FLOOR);

        let pending = timelock.get_operation(&operation_id);
        assert_eq!(pending.status, TimelockStatus::PendingApprovals);

        let queued = timelock.approve_action(&reviewer, &operation_id);
        assert_eq!(queued.status, TimelockStatus::Queued);

        let too_early = timelock.try_execute_action(&admin, &operation_id);
        assert_eq!(too_early, Err(Ok(Error::TimelockNotReady)));

        env.ledger().set_timestamp(1_000_000 + MIN_DELAY_FLOOR);
        timelock.execute_action(&admin, &operation_id);

        assert!(main_client.is_paused());
        assert_eq!(
            timelock.get_operation(&operation_id).status,
            TimelockStatus::Executed
        );
    }

    #[test]
    fn test_timelock_can_cancel_operation() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(2_000_000);

        let (timelock, admin, reviewer, _main_client, _upgrade, _timelock_id) = setup(&env);
        let operation_id =
            timelock.propose_action(&admin, &TimelockAction::PauseMain, &MIN_DELAY_FLOOR);
        timelock.approve_action(&reviewer, &operation_id);
        timelock.cancel_action(&reviewer, &operation_id, &Symbol::new(&env, "veto"));

        let op = timelock.get_operation(&operation_id);
        assert_eq!(op.status, TimelockStatus::Cancelled);
        assert_eq!(
            timelock.try_execute_action(&admin, &operation_id),
            Err(Ok(Error::TimelockCancelled))
        );
    }

    #[test]
    fn test_timelock_executes_upgrade_after_review_window() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(3_000_000);

        let (timelock, admin, reviewer, _main_client, upgrade_client, _timelock_id) = setup(&env);
        let new_contract = Address::generate(&env);
        let version = crate::types::ContractVersion {
            major: 1,
            minor: 1,
            patch: 0,
        };

        let operation_id = timelock.propose_action(
            &admin,
            &TimelockAction::InitiateUpgrade(version.clone(), new_contract.clone(), false),
            &MIN_DELAY_FLOOR,
        );
        timelock.approve_action(&reviewer, &operation_id);

        env.ledger().set_timestamp(3_000_000 + MIN_DELAY_FLOOR);
        timelock.execute_action(&reviewer, &operation_id);

        assert_eq!(
            upgrade_client.get_upgrade_status(),
            crate::types::UpgradeStatus::InProgress
        );
        let info = upgrade_client.get_upgrade_info().unwrap();
        assert_eq!(info.new_version, version);
        assert_eq!(info.new_contract_address, new_contract);
    }

    #[test]
    fn test_timelock_can_update_oracle_configuration() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(4_000_000);

        let (timelock, admin, reviewer, _main_client, _upgrade, timelock_id) = setup(&env);
        let oracle_id = env.register_contract(None, OracleSecurityContract);
        let oracle_client = OracleSecurityContractClient::new(&env, &oracle_id);
        oracle_client.oracle_init(&admin);
        oracle_client.set_timelock_contract(&admin, &timelock_id);

        let feed_id = Symbol::new(&env, "fuel");
        let oracle_config = OracleFeedConfig {
            feed_id: feed_id.clone(),
            feed_type: OracleFeedType::FuelPrice,
            min_value: 0,
            max_value: 50_000,
            max_age_seconds: 300,
            min_sources: 2,
            max_deviation_bps: 500,
        };

        let operation_id = timelock.propose_action(
            &admin,
            &TimelockAction::ConfigureOracleFeed(oracle_id.clone(), oracle_config.clone()),
            &MIN_DELAY_FLOOR,
        );
        timelock.approve_action(&reviewer, &operation_id);

        env.ledger().set_timestamp(4_000_000 + MIN_DELAY_FLOOR);
        timelock.execute_action(&admin, &operation_id);

        assert_eq!(oracle_client.get_feed_config(&feed_id), oracle_config);
    }
}

/// Multi-signature contract for administrative actions.
/// This contract handles:
/// - Multi-sig configuration
/// - Proposal submission
/// - Proposal approval
/// - Proposal execution
use crate::error::Error;
use crate::types::{DataKey, MultiSigConfig, Proposal, ProposalStatus};
use crate::{storage, validation_contract::ValidationContract};
use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol, Val, Vec};

// ─── Storage helpers ─────────────────────────────────────────────────────────

/// Get the multi-signature configuration.
fn get_multisig_config(env: &Env) -> Option<MultiSigConfig> {
    env.storage().persistent().get(&DataKey::MultiSigConfig)
}

/// Set the multi-signature configuration.
fn set_multisig_config(env: &Env, config: &MultiSigConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::MultiSigConfig, config);
}

/// Get the next proposal ID.
fn get_next_proposal_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextProposalId)
        .unwrap_or(1)
}

/// Set the next proposal ID.
fn set_next_proposal_id(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::NextProposalId, &id);
}

/// Get a proposal by ID.
fn get_proposal(env: &Env, proposal_id: u64) -> Option<Proposal> {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
}

/// Store a proposal.
fn put_proposal(env: &Env, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.id), proposal);
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Ensure the caller is a signer.
/// Returns MultiSigNotConfigured if multi-sig is not configured.
/// Returns NotSigner if caller is not a signer.
fn require_signer(env: &Env, caller: &Address) -> Result<(), Error> {
    let config = get_multisig_config(env).ok_or(Error::MultiSigNotConfigured)?;
    if !config.signers.contains(caller) {
        return Err(Error::NotSigner);
    }
    Ok(())
}

/// Check if an address is a signer.
fn is_signer(env: &Env, address: &Address) -> bool {
    if let Some(config) = get_multisig_config(env) {
        config.signers.contains(address)
    } else {
        false
    }
}

/// Check if the threshold has been reached for a specific proposal kind.
fn threshold_reached(env: &Env, kind: &Symbol, approvals: &Vec<Address>) -> bool {
    if let Some(config) = get_multisig_config(env) {
        let threshold = config
            .thresholds
            .get(kind.clone())
            .unwrap_or(config.threshold);
        approvals.len() >= threshold
    } else {
        false
    }
}

/// Check if the rejection threshold has been reached.
/// For simplicity, we'll say if more than (Total - Threshold) have rejected, it's rejected.
/// Or just any signer can reject for now? The requirements say "Audit trail of approvals and rejections".
/// Let's say if 1/3 of signers reject, or just if any signer rejects?
/// Typically, "rejected" means it can no longer be approved.
fn rejection_threshold_reached(env: &Env, kind: &Symbol, rejections: &Vec<Address>) -> bool {
    if let Some(config) = get_multisig_config(env) {
        let threshold = config
            .thresholds
            .get(kind.clone())
            .unwrap_or(config.threshold);
        let max_rejections = config
            .signers
            .len()
            .saturating_sub(threshold)
            .saturating_add(1);
        rejections.len() >= max_rejections
    } else {
        false
    }
}

/// Get the time lock for a specific proposal kind.
fn get_time_lock(env: &Env, kind: &Symbol) -> u64 {
    if let Some(config) = get_multisig_config(env) {
        config.time_locks.get(kind.clone()).unwrap_or(0)
    } else {
        0
    }
}

// ─── Contract ──────────────────────────────────────────────────────────────────

/// The Multi-Signature contract manages administrative actions requiring multiple approvals.
#[contract]
pub struct MultiSigContract;

#[contractimpl]
impl MultiSigContract {
    /// Initialize multi-signature configuration.
    /// Can only be called once and requires authentication from all initial signers.
    ///
    /// # Arguments
    /// * `signers` - A list of signer addresses
    /// * `threshold` - The number of approvals required to execute proposals
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns error if initialization fails
    ///
    /// # Errors
    /// * `AlreadyInitialized` - If multi-sig is already configured
    /// * `InvalidInput` - If signers list is empty
    /// * `InvalidThreshold` - If threshold is invalid (0 or > signers count)
    /// * `TooManySigners` - If more than 10 signers
    /// * `DuplicateSigner` - If duplicate signers are provided
    pub fn init_multisig(
        env: Env,
        signers: Vec<Address>,
        threshold: u32,
        thresholds: Map<Symbol, u32>,
        time_locks: Map<Symbol, u64>,
    ) -> Result<(), Error> {
        if get_multisig_config(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }

        if signers.is_empty() {
            return Err(Error::InvalidInput);
        }

        if threshold == 0 || threshold > signers.len() {
            return Err(Error::InvalidThreshold);
        }

        if signers.len() > 10 {
            return Err(Error::TooManySigners);
        }

        // Check for duplicate signers
        let mut seen = Vec::new(&env);
        for signer in signers.iter() {
            if seen.contains(&signer) {
                return Err(Error::DuplicateSigner);
            }
            seen.push_back(signer.clone());
        }

        // Require authentication from all initial signers
        for signer in signers.iter() {
            signer.require_auth();
        }

        let config = MultiSigConfig {
            signers: signers.clone(),
            threshold,
            thresholds,
            time_locks,
        };
        set_multisig_config(&env, &config);
        set_next_proposal_id(&env, 1);

        // Emit initialization event
        env.events().publish(
            (Symbol::new(&env, "multisig_initialized"),),
            (signers, threshold),
        );

        Ok(())
    }

    /// Get current multi-signature configuration.
    ///
    /// # Returns
    /// * `Result<MultiSigConfig, Error>` - The multi-signature configuration
    ///
    /// # Errors
    /// * `MultiSigNotConfigured` - If multi-sig is not configured
    pub fn get_multisig_config(env: Env) -> Result<MultiSigConfig, Error> {
        get_multisig_config(&env).ok_or(Error::MultiSigNotConfigured)
    }

    /// Submit a new proposal.
    /// Only signers can submit proposals.
    ///
    /// # Arguments
    /// * `proposer` - The address submitting the proposal (must be a signer)
    /// * `kind` - The type of proposal (e.g., "transfer_admin", "pause")
    /// * `args` - Arguments for the proposal
    ///
    /// # Returns
    /// * `Result<u64, Error>` - The ID of the newly created proposal
    ///
    /// # Errors
    /// * `MultiSigNotConfigured` - If multi-sig is not configured
    /// * `NotSigner` - If proposer is not a signer
    pub fn submit_proposal(
        env: Env,
        proposer: Address,
        target: Address,
        kind: Symbol,
        args: Vec<Val>,
    ) -> Result<u64, Error> {
        require_signer(&env, &proposer)?;
        proposer.require_auth();
        ValidationContract::validate_contract_address(&env, &target)?;
        ValidationContract::validate_event_type(&env, &kind)?;

        let proposal_id = get_next_proposal_id(&env);
        let next_id = proposal_id
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        set_next_proposal_id(&env, next_id);

        let proposal = Proposal {
            id: proposal_id,
            kind: kind.clone(),
            target: target.clone(),
            args: args.clone(),
            proposer: proposer.clone(),
            created_at: env.ledger().timestamp(),
            approved_at: 0,
            status: ProposalStatus::Active,
            approvals: {
                let mut approvals = Vec::new(&env);
                approvals.push_back(proposer.clone());
                approvals
            },
            rejections: Vec::new(&env),
        };

        put_proposal(&env, &proposal);

        // Emit proposal submitted event
        env.events().publish(
            (
                Symbol::new(&env, "proposal_submitted"),
                &proposal_id,
                &proposer,
            ),
            (&kind, &args),
        );

        Ok(proposal_id)
    }

    /// Approve a proposal.
    /// Only signers can approve.
    ///
    /// # Arguments
    /// * `approver` - The address approving the proposal (must be a signer)
    /// * `proposal_id` - The ID of the proposal to approve
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns error if approval fails
    ///
    /// # Errors
    /// * `MultiSigNotConfigured` - If multi-sig is not configured
    /// * `NotSigner` - If approver is not a signer
    /// * `ProposalNotFound` - If the proposal does not exist
    /// * `ProposalAlreadyExecuted` - If the proposal has already been executed
    /// * `AlreadyApproved` - If the approver has already approved this proposal
    pub fn approve_proposal(env: Env, approver: Address, proposal_id: u64) -> Result<(), Error> {
        require_signer(&env, &approver)?;
        approver.require_auth();

        let mut proposal = get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalAlreadyExecuted); // Or use a more specific error if available
        }

        if proposal.approvals.contains(&approver) {
            return Err(Error::AlreadyApproved);
        }

        if proposal.rejections.contains(&approver) {
            return Err(Error::AlreadyRejected);
        }

        proposal.approvals.push_back(approver.clone());

        // Check if threshold is reached
        if threshold_reached(&env, &proposal.kind, &proposal.approvals) {
            proposal.status = ProposalStatus::Approved;
            proposal.approved_at = env.ledger().timestamp();
        }

        put_proposal(&env, &proposal);

        // Emit approval event
        env.events().publish(
            (
                Symbol::new(&env, "proposal_approved"),
                &proposal_id,
                &approver,
            ),
            (proposal.status.clone(),),
        );

        Ok(())
    }

    /// Reject a proposal.
    /// Only signers can reject.
    ///
    /// # Arguments
    /// * `rejecter` - The address rejecting the proposal
    /// * `proposal_id` - The ID of the proposal to reject
    pub fn reject_proposal(env: Env, rejecter: Address, proposal_id: u64) -> Result<(), Error> {
        require_signer(&env, &rejecter)?;
        rejecter.require_auth();

        let mut proposal = get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalAlreadyExecuted);
        }

        if proposal.rejections.contains(&rejecter) {
            return Err(Error::AlreadyRejected);
        }

        if proposal.approvals.contains(&rejecter) {
            return Err(Error::AlreadyApproved);
        }

        proposal.rejections.push_back(rejecter.clone());

        // Check if rejection threshold is reached
        if rejection_threshold_reached(&env, &proposal.kind, &proposal.rejections) {
            proposal.status = ProposalStatus::Rejected;
        }

        put_proposal(&env, &proposal);

        // Emit rejection event
        env.events().publish(
            (
                Symbol::new(&env, "proposal_rejected"),
                &proposal_id,
                &rejecter,
            ),
            (proposal.status.clone(),),
        );

        Ok(())
    }

    /// Execute a proposal if threshold is reached.
    /// Only signers can execute.
    ///
    /// # Arguments
    /// * `executor` - The address executing the proposal (must be a signer)
    /// * `proposal_id` - The ID of the proposal to execute
    ///
    /// # Returns
    /// * `Result<(), Error>` - Returns error if execution fails
    ///
    /// # Errors
    /// * `MultiSigNotConfigured` - If multi-sig is not configured
    /// * `NotSigner` - If executor is not a signer
    /// * `ProposalNotFound` - If the proposal does not exist
    /// * `ProposalAlreadyExecuted` - If the proposal has already been executed
    /// * `ThresholdNotReached` - If the threshold has not been reached
    /// * `InvalidInput` - If the proposal kind is invalid
    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u64) -> Result<(), Error> {
        require_signer(&env, &executor)?;
        executor.require_auth();

        let mut proposal = get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.status == ProposalStatus::Executed {
            return Err(Error::ProposalAlreadyExecuted);
        }

        if proposal.status == ProposalStatus::Rejected {
            return Err(Error::ProposalRejected);
        }

        if proposal.status != ProposalStatus::Approved {
            return Err(Error::ThresholdNotReached);
        }

        // Check time lock
        let time_lock = get_time_lock(&env, &proposal.kind);
        let unlock_at = proposal
            .approved_at
            .checked_add(time_lock)
            .ok_or(Error::ArithmeticOverflow)?;
        if env.ledger().timestamp() < unlock_at {
            return Err(Error::TimeLockNotExpired);
        }

        // Mark as executed BEFORE performing the action to prevent reentrancy
        proposal.status = ProposalStatus::Executed;
        put_proposal(&env, &proposal);

        // Execute the proposal via cross-contract call
        // We pass the current contract address as the first argument if the target expects a 'caller' argument
        // Many functions in our contracts take 'caller' as the first or second argument.
        // However, for generic support, we just pass the args as provided.
        let scope = Symbol::new(&env, "multisig_exec");
        storage::acquire_reentrancy_lock(&env, &scope)?;
        let _result: Val =
            env.invoke_contract(&proposal.target, &proposal.kind, proposal.args.clone());
        storage::release_reentrancy_lock(&env, &scope);

        // Emit execution event
        env.events().publish(
            (
                Symbol::new(&env, "proposal_executed"),
                &proposal_id,
                &executor,
            ),
            (&proposal.kind, &proposal.args),
        );

        Ok(())
    }

    /// Get a proposal by ID.
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal to retrieve
    ///
    /// # Returns
    /// * `Result<Proposal, Error>` - The proposal
    ///
    /// # Errors
    /// * `ProposalNotFound` - If the proposal does not exist
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, Error> {
        get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)
    }

    /// Get all proposal IDs (for enumeration).
    ///
    /// # Arguments
    /// * `from_id` - The starting proposal ID
    /// * `limit` - The maximum number of IDs to return
    ///
    /// # Returns
    /// * `Vec<u64>` - A vector of proposal IDs
    pub fn get_proposal_ids(env: Env, from_id: u64, limit: u32) -> Vec<u64> {
        let mut ids = Vec::new(&env);
        let next_id = get_next_proposal_id(&env);
        let mut current = from_id.max(1);
        let end = (current + limit as u64).min(next_id);

        while current < end {
            ids.push_back(current);
            current += 1;
        }

        ids
    }
}

#[cfg(test)]
mod test_multisig {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env, IntoVal};

    #[contract]
    pub struct MockContract;

    #[contractimpl]
    impl MockContract {
        pub fn transfer_admin(env: Env, _current: Address, _new: Address) {}
        pub fn pause(env: Env, _caller: Address) {}
    }

    fn setup(env: &Env) -> (MultiSigContractClient, Vec<Address>) {
        let contract_id = env.register_contract(None, MultiSigContract);
        let client = MultiSigContractClient::new(env, &contract_id);

        let mut signers = Vec::new(&env);
        signers.push_back(Address::generate(env));
        signers.push_back(Address::generate(env));
        signers.push_back(Address::generate(env));

        (client, signers)
    }

    #[test]
    fn test_init_multisig() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let thresholds = Map::new(&env);
        let time_locks = Map::new(&env);

        // Initialize with 3 signers, threshold 2
        client.init_multisig(&signers, &2, &thresholds, &time_locks);

        let config = client.get_multisig_config();
        assert_eq!(config.signers, signers);
        assert_eq!(config.threshold, 2);
    }

    #[test]
    fn test_init_multisig_invalid_threshold() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let thresholds = Map::new(&env);
        let time_locks = Map::new(&env);

        // Threshold too high
        let res = client.try_init_multisig(&signers, &4, &thresholds, &time_locks);
        assert_eq!(res, Err(Ok(Error::InvalidThreshold)));

        // Threshold zero
        let res = client.try_init_multisig(&signers, &0, &thresholds, &time_locks);
        assert_eq!(res, Err(Ok(Error::InvalidThreshold)));
    }

    #[test]
    fn test_submit_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let thresholds = Map::new(&env);
        let time_locks = Map::new(&env);
        client.init_multisig(&signers, &2, &thresholds, &time_locks);

        let proposer = signers.get(0).unwrap().clone();
        let new_admin = Address::generate(&env);

        let kind = Symbol::new(&env, "transfer_admin");
        let args = {
            let mut args = Vec::new(&env);
            args.push_back(proposer.clone().into_val(&env));
            args.push_back(new_admin.into_val(&env));
            args
        };

        let target = env.register_contract(None, MockContract);
        let proposal_id = client.submit_proposal(&proposer, &target, &kind, &args);

        assert_eq!(proposal_id, 1);

        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.approvals.len(), 1);
    }

    #[test]
    fn test_approve_and_execute_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let thresholds = Map::new(&env);
        let time_locks = Map::new(&env);
        client.init_multisig(&signers, &2, &thresholds, &time_locks);

        let proposer = signers.get(0).unwrap().clone();
        let approver = signers.get(1).unwrap().clone();
        let new_admin = Address::generate(&env);

        let kind = Symbol::new(&env, "transfer_admin");
        let args = {
            let mut args = Vec::new(&env);
            args.push_back(proposer.clone().into_val(&env));
            args.push_back(new_admin.into_val(&env));
            args
        };

        let target = env.register_contract(None, MockContract);
        let proposal_id = client.submit_proposal(&proposer, &target, &kind, &args);

        // Approve with second signer
        client.approve_proposal(&approver, &proposal_id);

        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Approved);

        // Execute
        client.execute_proposal(&approver, &proposal_id);

        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Executed);
        assert_eq!(proposal.approvals.len(), 2);
    }

    #[test]
    fn test_reject_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let thresholds = Map::new(&env);
        let time_locks = Map::new(&env);
        client.init_multisig(&signers, &2, &thresholds, &time_locks);

        let proposer = signers.get(0).unwrap().clone();
        let rejecter1 = signers.get(1).unwrap().clone();
        let rejecter2 = signers.get(2).unwrap().clone();

        let kind = Symbol::new(&env, "transfer_admin");
        let args = Vec::new(&env);

        let target = env.register_contract(None, MockContract);
        let proposal_id = client.submit_proposal(&proposer, &target, &kind, &args);

        // Reject with one signer
        client.reject_proposal(&rejecter1, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.rejections.len(), 1);

        // Reject with another signer (Total 2 rejections, threshold 2, so max_rejections is 3-2+1 = 2)
        client.reject_proposal(&rejecter2, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Rejected);

        // Execution should fail
        let res = client.try_execute_proposal(&proposer, &proposal_id);
        assert_eq!(res, Err(Ok(Error::ProposalRejected)));
    }

    #[test]
    fn test_time_lock() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let thresholds = Map::new(&env);
        let mut time_locks = Map::new(&env);
        let kind = Symbol::new(&env, "pause");
        time_locks.set(kind.clone(), 3600); // 1 hour time lock

        client.init_multisig(&signers, &2, &thresholds, &time_locks);

        let proposer = signers.get(0).unwrap().clone();
        let approver = signers.get(1).unwrap().clone();
        let args = {
            let mut args = Vec::new(&env);
            args.push_back(proposer.clone().into_val(&env));
            args
        };

        let target = env.register_contract(None, MockContract);
        let proposal_id = client.submit_proposal(&proposer, &target, &kind, &args);
        client.approve_proposal(&approver, &proposal_id);

        // Try to execute immediately
        let res = client.try_execute_proposal(&proposer, &proposal_id);
        assert_eq!(res, Err(Ok(Error::TimeLockNotExpired)));

        // Advance time
        env.ledger().set_timestamp(env.ledger().timestamp() + 3601);

        // Execute now should succeed
        client.execute_proposal(&proposer, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_per_type_threshold() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        let mut thresholds = Map::new(&env);
        let time_locks = Map::new(&env);
        let kind = Symbol::new(&env, "critical_op");
        thresholds.set(kind.clone(), 3); // Requires all 3 signers

        client.init_multisig(&signers, &2, &thresholds, &time_locks);

        let proposer = signers.get(0).unwrap().clone();
        let approver1 = signers.get(1).unwrap().clone();
        let approver2 = signers.get(2).unwrap().clone();
        let args = Vec::new(&env);

        let target = env.register_contract(None, MockContract);
        let proposal_id = client.submit_proposal(&proposer, &target, &kind, &args);

        // One approval (proposer) + one more = 2. Still Active because threshold is 3.
        client.approve_proposal(&approver1, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Active);

        // One more approval = 3. Now Approved.
        client.approve_proposal(&approver2, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Approved);
    }
}

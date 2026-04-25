/// Multi-signature contract for administrative actions.
/// This contract handles:
/// - Multi-sig configuration
/// - Proposal submission
/// - Proposal approval
/// - Proposal execution
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

use crate::error::Error;
use crate::types::{DataKey, MultiSigConfig, Proposal};

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

/// Check if the threshold has been reached.
fn threshold_reached(env: &Env, approvals: &Vec<Address>) -> bool {
    if let Some(config) = get_multisig_config(env) {
        approvals.len() >= config.threshold
    } else {
        false
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
    pub fn init_multisig(env: Env, signers: Vec<Address>, threshold: u32) -> Result<(), Error> {
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
        kind: Symbol,
        args: Vec<Val>,
    ) -> Result<u64, Error> {
        require_signer(&env, &proposer)?;
        proposer.require_auth();

        let proposal_id = get_next_proposal_id(&env);
        set_next_proposal_id(&env, proposal_id + 1);

        let proposal = Proposal {
            id: proposal_id,
            kind: kind.clone(),
            args: args.clone(),
            proposer: proposer.clone(),
            created_at: env.ledger().timestamp(),
            executed: false,
            approvals: {
                let mut approvals = Vec::new(&env);
                approvals.push_back(proposer.clone());
                approvals
            },
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

        if proposal.executed {
            return Err(Error::ProposalAlreadyExecuted);
        }

        if proposal.approvals.contains(&approver) {
            return Err(Error::AlreadyApproved);
        }

        proposal.approvals.push_back(approver.clone());
        put_proposal(&env, &proposal);

        // Emit approval event
        env.events().publish(
            (
                Symbol::new(&env, "proposal_approved"),
                &proposal_id,
                &approver,
            ),
            (),
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

        if proposal.executed {
            return Err(Error::ProposalAlreadyExecuted);
        }

        if !threshold_reached(&env, &proposal.approvals) {
            return Err(Error::ThresholdNotReached);
        }

        // Mark as executed BEFORE performing the action to prevent reentrancy
        proposal.executed = true;
        put_proposal(&env, &proposal);

        // Execute the proposal
        let transfer_admin = Symbol::new(&env, "transfer_admin");
        let initiate_upgrade = Symbol::new(&env, "initiate_upgrade");
        let complete_upgrade = Symbol::new(&env, "complete_upgrade");
        let fail_upgrade = Symbol::new(&env, "fail_upgrade");
        let pause = Symbol::new(&env, "pause");
        let unpause = Symbol::new(&env, "unpause");

        if proposal.kind == transfer_admin {
            // For now, just emit an event; integration will come later
            env.events().publish(
                (Symbol::new(&env, "admin_transfer_executed"),),
                (&proposal.args, &executor),
            );
        } else if proposal.kind == initiate_upgrade {
            env.events().publish(
                (Symbol::new(&env, "upgrade_initiate_executed"),),
                (&proposal.args, &executor),
            );
        } else if proposal.kind == complete_upgrade {
            env.events()
                .publish((Symbol::new(&env, "upgrade_complete_executed"),), ());
        } else if proposal.kind == fail_upgrade {
            env.events().publish(
                (Symbol::new(&env, "upgrade_fail_executed"),),
                &proposal.args,
            );
        } else if proposal.kind == pause {
            env.events()
                .publish((Symbol::new(&env, "pause_executed"),), ());
        } else if proposal.kind == unpause {
            env.events()
                .publish((Symbol::new(&env, "unpause_executed"),), ());
        } else {
            return Err(Error::InvalidInput);
        }

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
    use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};

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

        // Initialize with 3 signers, threshold 2
        client.init_multisig(&signers, &2);

        let config = client.get_multisig_config();
        assert_eq!(config.signers, signers);
        assert_eq!(config.threshold, 2);
    }

    #[test]
    fn test_init_multisig_invalid_threshold() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);

        // Threshold too high
        let res = client.try_init_multisig(&signers, &4);
        assert_eq!(res, Err(Ok(Error::InvalidThreshold)));

        // Threshold zero
        let res = client.try_init_multisig(&signers, &0);
        assert_eq!(res, Err(Ok(Error::InvalidThreshold)));
    }

    #[test]
    fn test_submit_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        client.init_multisig(&signers, &2);

        let proposer = signers.get(0).unwrap().clone();
        let new_admin = Address::generate(&env);

        let kind = Symbol::new(&env, "transfer_admin");
        let args = {
            let mut args = Vec::new(&env);
            args.push_back(proposer.clone().into_val(&env));
            args.push_back(new_admin.into_val(&env));
            args
        };

        let proposal_id = client.submit_proposal(&proposer, &kind, &args);

        assert_eq!(proposal_id, 1);

        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.proposer, proposer);
        assert!(!proposal.executed);
        assert_eq!(proposal.approvals.len(), 1);
    }

    #[test]
    fn test_approve_and_execute_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        client.init_multisig(&signers, &2);

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

        let proposal_id = client.submit_proposal(&proposer, &kind, &args);

        // Approve with second signer
        client.approve_proposal(&approver, &proposal_id);

        // Execute
        client.execute_proposal(&approver, &proposal_id);

        let proposal = client.get_proposal(&proposal_id);
        assert!(proposal.executed);
        assert_eq!(proposal.approvals.len(), 2);
    }

    #[test]
    fn test_execute_without_threshold_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signers) = setup(&env);
        client.init_multisig(&signers, &2);

        let proposer = signers.get(0).unwrap().clone();
        let new_admin = Address::generate(&env);

        let kind = Symbol::new(&env, "transfer_admin");
        let args = {
            let mut args = Vec::new(&env);
            args.push_back(proposer.clone().into_val(&env));
            args.push_back(new_admin.into_val(&env));
            args
        };

        let proposal_id = client.submit_proposal(&proposer, &kind, &args);
        // Try to execute without enough approvals
        let res = client.try_execute_proposal(&proposer, &proposal_id);
        assert_eq!(res, Err(Ok(Error::ThresholdNotReached)));
    }
}

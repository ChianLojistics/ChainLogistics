use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

use crate::error::Error;
use crate::events::{
    AdminTransferExecuted, MultisigInitialized, PauseExecuted, ProposalApproved, ProposalExecuted,
    ProposalSubmitted, UnpauseExecuted, UpgradeCompleteExecuted, UpgradeFailExecuted,
    UpgradeInitiateExecuted,
};
use crate::types::{DataKey, MultiSigConfig, Proposal};

// ─── Storage helpers ─────────────────────────────────────────────────────────

fn get_multisig_config(env: &Env) -> Option<MultiSigConfig> {
    env.storage().persistent().get(&DataKey::MultiSigConfig)
}

fn set_multisig_config(env: &Env, config: &MultiSigConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::MultiSigConfig, config);
}

fn get_next_proposal_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextProposalId)
        .unwrap_or(1)
}

fn set_next_proposal_id(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::NextProposalId, &id);
}

fn get_proposal(env: &Env, proposal_id: u64) -> Option<Proposal> {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
}

fn put_proposal(env: &Env, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.id), proposal);
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn require_signer(env: &Env, caller: &Address) -> Result<(), Error> {
    let config = get_multisig_config(env).ok_or(Error::MultiSigNotConfigured)?;
    if !config.signers.contains(caller) {
        return Err(Error::NotSigner);
    }
    Ok(())
}

fn threshold_reached(env: &Env, approvals: &Vec<Address>) -> bool {
    if let Some(config) = get_multisig_config(env) {
        approvals.len() >= config.threshold
    } else {
        false
    }
}

// ─── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct MultiSigContract;

#[contractimpl]
impl MultiSigContract {
    /// Initialize multi-signature configuration.
    /// Can only be called once and requires authentication from all initial signers.
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

        MultisigInitialized { signers, threshold }.publish(&env);

        Ok(())
    }

    /// Get current multi-signature configuration.
    pub fn get_multisig_config(env: Env) -> Result<MultiSigConfig, Error> {
        get_multisig_config(&env).ok_or(Error::MultiSigNotConfigured)
    }

    /// Submit a new proposal.
    /// Only signers can submit proposals.
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

        ProposalSubmitted {
            proposal_id,
            proposer: proposer.clone(),
            kind: kind.clone(),
            args: args.clone(),
        }
        .publish(&env);

        Ok(proposal_id)
    }

    /// Approve a proposal.
    /// Only signers can approve.
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

        ProposalApproved {
            proposal_id,
            approver: approver.clone(),
        }
        .publish(&env);

        Ok(())
    }

    /// Execute a proposal if threshold is reached.
    /// Only signers can execute.
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
            AdminTransferExecuted {
                args: proposal.args.clone(),
                executor: executor.clone(),
            }
            .publish(&env);
        } else if proposal.kind == initiate_upgrade {
            UpgradeInitiateExecuted {
                args: proposal.args.clone(),
                executor: executor.clone(),
            }
            .publish(&env);
        } else if proposal.kind == complete_upgrade {
            UpgradeCompleteExecuted { _unused: 0 }.publish(&env);
        } else if proposal.kind == fail_upgrade {
            UpgradeFailExecuted {
                args: proposal.args.clone(),
            }
            .publish(&env);
        } else if proposal.kind == pause {
            PauseExecuted { _unused: 0 }.publish(&env);
        } else if proposal.kind == unpause {
            UnpauseExecuted { _unused: 0 }.publish(&env);
        } else {
            return Err(Error::InvalidInput);
        }

        ProposalExecuted {
            proposal_id,
            executor,
            kind: proposal.kind,
            args: proposal.args,
        }
        .publish(&env);

        Ok(())
    }

    /// Get a proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, Error> {
        get_proposal(&env, proposal_id).ok_or(Error::ProposalNotFound)
    }

    /// Get all proposal IDs (for enumeration).
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

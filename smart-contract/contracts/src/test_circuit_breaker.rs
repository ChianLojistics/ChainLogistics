/// Comprehensive tests for the CircuitBreakerContract.
#[cfg(test)]
#[allow(clippy::module_inception)]
mod test_circuit_breaker {
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env, Vec};

    use crate::{
        circuit_breaker::{CircuitBreakerContract, CircuitBreakerContractClient},
        error::Error,
        types::{PauseLevel, PauseReason},
    };

    // ─── Helpers ─────────────────────────────────────────────────────────────

    fn deploy(env: &Env) -> (CircuitBreakerContractClient, Address, Vec<Address>) {
        let id = env.register_contract(None, CircuitBreakerContract);
        let client = CircuitBreakerContractClient::new(env, &id);
        let admin = Address::generate(env);
        let mut guardians = Vec::new(env);
        guardians.push_back(Address::generate(env));
        guardians.push_back(Address::generate(env));
        guardians.push_back(Address::generate(env));
        client.initialize(&admin, &guardians);
        (client, admin, guardians)
    }

    fn guardian(guardians: &Vec<Address>, i: u32) -> Address {
        guardians.get(i).unwrap()
    }

    // ─── Initialisation ───────────────────────────────────────────────────────

    #[test]
    fn test_initialize_sets_admin_and_guardians() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_guardians(), guardians);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_initialize_twice_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        let res = client.try_initialize(&admin, &guardians);
        assert_eq!(res, Err(Ok(Error::CircuitBreakerAlreadyInitialized)));
    }

    #[test]
    fn test_initialize_duplicate_guardian_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, CircuitBreakerContract);
        let client = CircuitBreakerContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let g = Address::generate(&env);
        let mut guardians = Vec::new(&env);
        guardians.push_back(g.clone());
        guardians.push_back(g.clone());

        let res = client.try_initialize(&admin, &guardians);
        assert_eq!(res, Err(Ok(Error::DuplicateGuardian)));
    }

    #[test]
    fn test_initialize_too_many_guardians_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, CircuitBreakerContract);
        let client = CircuitBreakerContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let mut guardians = Vec::new(&env);
        for _ in 0..21 {
            guardians.push_back(Address::generate(&env));
        }
        let res = client.try_initialize(&admin, &guardians);
        assert_eq!(res, Err(Ok(Error::TooManyGuardians)));
    }

    // ─── Guardian management ──────────────────────────────────────────────────

    #[test]
    fn test_add_and_remove_guardian() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _) = deploy(&env);

        let new_g = Address::generate(&env);
        client.add_guardian(&admin, &new_g);
        assert!(client.get_guardians().contains(&new_g));

        client.remove_guardian(&admin, &new_g);
        assert!(!client.get_guardians().contains(&new_g));
    }

    #[test]
    fn test_add_duplicate_guardian_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        let g = guardian(&guardians, 0);
        let res = client.try_add_guardian(&admin, &g);
        assert_eq!(res, Err(Ok(Error::DuplicateGuardian)));
    }

    #[test]
    fn test_remove_nonexistent_guardian_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _) = deploy(&env);

        let unknown = Address::generate(&env);
        let res = client.try_remove_guardian(&admin, &unknown);
        assert_eq!(res, Err(Ok(Error::NotGuardian)));
    }

    #[test]
    fn test_non_admin_cannot_add_guardian() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let attacker = Address::generate(&env);
        let new_g = Address::generate(&env);
        let res = client.try_add_guardian(&attacker, &new_g);
        assert!(res.is_err());

        // Existing guardian also cannot add
        let res = client.try_add_guardian(&guardian(&guardians, 0), &new_g);
        assert!(res.is_err());
    }

    // ─── Instant pause ────────────────────────────────────────────────────────

    #[test]
    fn test_instant_pause_by_guardian() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g = guardian(&guardians, 0);
        let record_id = client.instant_pause(
            &g,
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "Active exploit detected"),
            &3600,
        );

        assert_eq!(record_id, 1);
        assert!(client.is_paused());

        let state = client.get_state();
        assert!(state.is_paused);
        assert_eq!(state.level, PauseLevel::Full);
        assert_eq!(state.current_record_id, 1);
    }

    #[test]
    fn test_non_guardian_cannot_instant_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = deploy(&env);

        let attacker = Address::generate(&env);
        let res = client.try_instant_pause(
            &attacker,
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
        );
        assert_eq!(res, Err(Ok(Error::NotGuardian)));
    }

    #[test]
    fn test_instant_pause_description_too_long_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        // Build a 257-byte string
        let long_desc = soroban_sdk::String::from_str(
            &env,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        );
        let g = guardian(&guardians, 0);
        let res = client.try_instant_pause(
            &g,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &long_desc,
            &0,
        );
        assert_eq!(res, Err(Ok(Error::PauseDescriptionTooLong)));
    }

    #[test]
    fn test_instant_pause_invalid_duration_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g = guardian(&guardians, 0);
        // 31 days in seconds — exceeds MAX_PAUSE_DURATION_SECS
        let too_long: u64 = 31 * 24 * 60 * 60;
        let res = client.try_instant_pause(
            &g,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &too_long,
        );
        assert_eq!(res, Err(Ok(Error::InvalidPauseDuration)));
    }

    // ─── Pause levels ─────────────────────────────────────────────────────────

    #[test]
    fn test_partial_pause_blocks_writes_not_mutations() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Partial,
            &PauseReason::OracleFailure,
            &soroban_sdk::String::from_str(&env, "Oracle down"),
            &0,
        );

        assert!(!client.check_writes_allowed());
        assert!(client.check_mutations_allowed());
        assert!(client.check_reads_allowed());
    }

    #[test]
    fn test_full_pause_blocks_writes_and_mutations() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Full,
            &PauseReason::ContractBug,
            &soroban_sdk::String::from_str(&env, "Critical bug"),
            &0,
        );

        assert!(!client.check_writes_allowed());
        assert!(!client.check_mutations_allowed());
        assert!(client.check_reads_allowed());
    }

    #[test]
    fn test_emergency_pause_blocks_everything() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Emergency,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "Active exploit"),
            &0,
        );

        assert!(!client.check_writes_allowed());
        assert!(!client.check_mutations_allowed());
        assert!(!client.check_reads_allowed());
    }

    #[test]
    fn test_advisory_pause_blocks_nothing() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Advisory,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "Scheduled maintenance"),
            &0,
        );

        // Advisory is paused=true but all checks pass
        assert!(client.is_paused());
        assert!(client.check_writes_allowed());
        assert!(client.check_mutations_allowed());
        assert!(client.check_reads_allowed());
    }

    // ─── Lift pause ───────────────────────────────────────────────────────────

    #[test]
    fn test_admin_can_lift_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
        );
        assert!(client.is_paused());

        client.lift_pause(&admin);
        assert!(!client.is_paused());
        assert!(client.check_writes_allowed());
        assert!(client.check_mutations_allowed());
    }

    #[test]
    fn test_lift_when_not_paused_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _) = deploy(&env);

        let res = client.try_lift_pause(&admin);
        assert_eq!(res, Err(Ok(Error::ContractNotPaused)));
    }

    #[test]
    fn test_non_admin_cannot_lift_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g = guardian(&guardians, 0);
        client.instant_pause(
            &g,
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
        );

        let attacker = Address::generate(&env);
        let res = client.try_lift_pause(&attacker);
        assert!(res.is_err());
    }

    // ─── Pause record ─────────────────────────────────────────────────────────

    #[test]
    fn test_pause_record_stored_correctly() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g = guardian(&guardians, 0);
        let record_id = client.instant_pause(
            &g,
            &PauseLevel::Full,
            &PauseReason::RegulatoryAction,
            &soroban_sdk::String::from_str(&env, "Regulator request"),
            &7200,
        );

        let record = client.get_pause_record(&record_id);
        assert_eq!(record.record_id, record_id);
        assert_eq!(record.activated_by, g);
        assert_eq!(record.level, PauseLevel::Full);
        assert_eq!(record.reason, PauseReason::RegulatoryAction);
        assert_eq!(record.lifted_at, 0); // not yet lifted
    }

    #[test]
    fn test_pause_record_updated_on_lift() {
        let env = Env::default();
        env.mock_all_auths();
        // Set a non-zero starting timestamp so lifted_at is distinguishable from 0
        env.ledger().set_timestamp(1_000_000);
        let (client, admin, guardians) = deploy(&env);

        let record_id = client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
        );

        client.lift_pause(&admin);

        let record = client.get_pause_record(&record_id);
        assert!(record.lifted_at > 0);
        assert_eq!(record.lifted_by.len(), 1);
        assert_eq!(record.lifted_by.get(0).unwrap(), admin);
    }

    #[test]
    fn test_get_nonexistent_record_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = deploy(&env);

        let res = client.try_get_pause_record(&999);
        assert_eq!(res, Err(Ok(Error::PauseRecordNotFound)));
    }

    // ─── Time-limited pauses ──────────────────────────────────────────────────

    #[test]
    fn test_pause_auto_expires() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        // Pause for 100 seconds
        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Full,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "Short maintenance"),
            &100,
        );
        assert!(client.is_paused());

        // Advance ledger time past expiry
        env.ledger().set_timestamp(env.ledger().timestamp() + 101);

        // get_state should auto-expire
        let state = client.get_state();
        assert!(!state.is_paused);
        assert!(!client.is_paused());
        assert!(client.check_writes_allowed());
    }

    #[test]
    fn test_pause_with_zero_duration_does_not_expire() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "Indefinite"),
            &0, // no expiry
        );

        // Advance time significantly
        env.ledger()
            .set_timestamp(env.ledger().timestamp() + 999_999);

        assert!(client.is_paused());
    }

    // ─── Multi-authority pause ────────────────────────────────────────────────

    #[test]
    fn test_propose_approve_execute_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);
        let g1 = guardian(&guardians, 1);
        let g2 = guardian(&guardians, 2);

        // Propose requiring 2 approvals
        let approval_id = client.propose_pause(
            &g0,
            &PauseLevel::Partial,
            &PauseReason::MarketVolatility,
            &soroban_sdk::String::from_str(&env, "Market crash"),
            &3600,
            &2,
            &0,
        );

        // g0 already voted; g1 approves
        client.approve_pause_proposal(&g1, &approval_id);

        // Threshold reached — execute
        let record_id = client.execute_pause_proposal(&g2, &approval_id, &3600);

        assert!(client.is_paused());
        let state = client.get_state();
        assert_eq!(state.level, PauseLevel::Partial);
        assert_eq!(state.current_record_id, record_id);
    }

    #[test]
    fn test_execute_proposal_below_threshold_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);

        // Propose requiring 3 approvals — only proposer has voted
        let approval_id = client.propose_pause(
            &g0,
            &PauseLevel::Full,
            &PauseReason::ContractBug,
            &soroban_sdk::String::from_str(&env, "Bug found"),
            &0,
            &3,
            &0,
        );

        let res = client.try_execute_pause_proposal(&g0, &approval_id, &0);
        assert_eq!(res, Err(Ok(Error::ApprovalThresholdNotReached)));
    }

    #[test]
    fn test_double_vote_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);
        let g1 = guardian(&guardians, 1);

        let approval_id = client.propose_pause(
            &g0,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
            &2,
            &0,
        );

        client.approve_pause_proposal(&g1, &approval_id);

        // g1 tries to vote again
        let res = client.try_approve_pause_proposal(&g1, &approval_id);
        assert_eq!(res, Err(Ok(Error::ApprovalAlreadyVoted)));
    }

    #[test]
    fn test_execute_already_executed_proposal_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);
        let g1 = guardian(&guardians, 1);

        let approval_id = client.propose_pause(
            &g0,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
            &2,
            &0,
        );
        client.approve_pause_proposal(&g1, &approval_id);
        client.execute_pause_proposal(&g0, &approval_id, &0);

        // Try to execute again
        let res = client.try_execute_pause_proposal(&g0, &approval_id, &0);
        assert_eq!(res, Err(Ok(Error::ApprovalAlreadyExecuted)));
    }

    #[test]
    fn test_proposal_expires() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);
        let g1 = guardian(&guardians, 1);

        // Proposal window of 60 seconds
        let approval_id = client.propose_pause(
            &g0,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
            &2,
            &60,
        );

        // Advance past window
        env.ledger().set_timestamp(env.ledger().timestamp() + 61);

        let res = client.try_approve_pause_proposal(&g1, &approval_id);
        assert_eq!(res, Err(Ok(Error::ApprovalExpired)));
    }

    #[test]
    fn test_non_guardian_cannot_propose() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = deploy(&env);

        let attacker = Address::generate(&env);
        let res = client.try_propose_pause(
            &attacker,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
            &1,
            &0,
        );
        assert_eq!(res, Err(Ok(Error::NotGuardian)));
    }

    #[test]
    fn test_invalid_threshold_in_proposal_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);

        // Threshold 0
        let res = client.try_propose_pause(
            &g0,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
            &0,
            &0,
        );
        assert_eq!(res, Err(Ok(Error::InvalidThreshold)));

        // Threshold > number of guardians (3)
        let res = client.try_propose_pause(
            &g0,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "test"),
            &0,
            &4,
            &0,
        );
        assert_eq!(res, Err(Ok(Error::InvalidThreshold)));
    }

    // ─── Pause record retrieval ───────────────────────────────────────────────

    #[test]
    fn test_multiple_pause_records_sequential_ids() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        let g = guardian(&guardians, 0);

        let id1 = client.instant_pause(
            &g,
            &PauseLevel::Partial,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "First"),
            &0,
        );
        client.lift_pause(&admin);

        let id2 = client.instant_pause(
            &g,
            &PauseLevel::Full,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "Second"),
            &0,
        );
        client.lift_pause(&admin);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let r1 = client.get_pause_record(&id1);
        let r2 = client.get_pause_record(&id2);
        assert_eq!(r1.level, PauseLevel::Partial);
        assert_eq!(r2.level, PauseLevel::Full);
    }

    // ─── Supply chain scenario tests ─────────────────────────────────────────

    #[test]
    fn test_security_breach_scenario() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        // Security team detects exploit — instant Emergency pause
        let g = guardian(&guardians, 0);
        client.instant_pause(
            &g,
            &PauseLevel::Emergency,
            &PauseReason::SecurityBreach,
            &soroban_sdk::String::from_str(&env, "Active exploit on register_product"),
            &0,
        );

        // All operations blocked
        assert!(!client.check_writes_allowed());
        assert!(!client.check_mutations_allowed());
        assert!(!client.check_reads_allowed());

        // After investigation, admin lifts
        client.lift_pause(&admin);
        assert!(!client.is_paused());
        assert!(client.check_reads_allowed());
    }

    #[test]
    fn test_oracle_failure_scenario() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, guardians) = deploy(&env);

        // Oracle fails — Partial pause (reads still work)
        let g = guardian(&guardians, 1);
        client.instant_pause(
            &g,
            &PauseLevel::Partial,
            &PauseReason::OracleFailure,
            &soroban_sdk::String::from_str(&env, "Temperature oracle unreachable"),
            &1800, // 30 min auto-expiry
        );

        assert!(!client.check_writes_allowed());
        assert!(client.check_reads_allowed());

        // Oracle recovers — admin lifts early
        client.lift_pause(&admin);
        assert!(client.check_writes_allowed());
    }

    #[test]
    fn test_regulatory_action_multi_authority_scenario() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        let g0 = guardian(&guardians, 0);
        let g1 = guardian(&guardians, 1);
        let g2 = guardian(&guardians, 2);

        // Regulatory pause requires 2-of-3 guardian consensus
        let approval_id = client.propose_pause(
            &g0,
            &PauseLevel::Full,
            &PauseReason::RegulatoryAction,
            &soroban_sdk::String::from_str(&env, "Regulator freeze order"),
            &0,
            &2,
            &86400, // 24h approval window
        );

        // Second guardian approves
        client.approve_pause_proposal(&g1, &approval_id);

        // Third guardian executes
        client.execute_pause_proposal(&g2, &approval_id, &0);

        assert!(client.is_paused());
        let state = client.get_state();
        assert_eq!(state.level, PauseLevel::Full);
        assert_eq!(state.expires_at, 0); // indefinite
    }

    #[test]
    fn test_maintenance_window_auto_expires() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, guardians) = deploy(&env);

        // Scheduled maintenance — 2 hour window
        client.instant_pause(
            &guardian(&guardians, 0),
            &PauseLevel::Full,
            &PauseReason::Maintenance,
            &soroban_sdk::String::from_str(&env, "Scheduled DB migration"),
            &7200,
        );

        assert!(client.is_paused());

        // Maintenance completes — advance past window
        env.ledger().set_timestamp(env.ledger().timestamp() + 7201);

        assert!(!client.is_paused());
        assert!(client.check_writes_allowed());
        assert!(client.check_mutations_allowed());
    }
}

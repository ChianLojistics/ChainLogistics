/// Circuit Breaker — graduated emergency stop mechanism for ChainLogistics.
///
/// # Design
///
/// The circuit breaker is a **standalone Soroban contract** that other
/// contracts query via cross-contract calls.  This keeps pause logic
/// isolated and upgradeable independently of business logic.
///
/// ## Pause levels
///
/// | Level      | Blocked                                                  |
/// |------------|----------------------------------------------------------|
/// | Advisory   | Nothing — informational only                             |
/// | Partial    | `add_tracking_event`, `register_product`                 |
/// | Full       | All state-mutating operations                            |
/// | Emergency  | All operations including reads (except status queries)   |
///
/// ## Multi-authority activation
///
/// A guardian can either:
/// 1. **Instant pause** — immediately activates at `Emergency` level
///    (single guardian sufficient for speed during active exploits).
/// 2. **Proposal-based pause** — any guardian proposes; `required_approvals`
///    guardians must vote before the pause is activated.  Used for
///    lower-severity levels where deliberation is appropriate.
///
/// ## Time-limited pauses
///
/// Every pause carries an `expires_at` ledger timestamp.  When the
/// current ledger time exceeds `expires_at` the pause is automatically
/// considered lifted on the next `check_*` call.  Pass `0` for no expiry.
///
/// ## Events emitted
///
/// | Symbol                    | When                                      |
/// |---------------------------|-------------------------------------------|
/// | `cb_initialized`          | Contract initialised                      |
/// | `cb_guardian_added`       | Guardian added                            |
/// | `cb_guardian_removed`     | Guardian removed                          |
/// | `cb_paused`               | Pause activated (instant or proposal)     |
/// | `cb_lifted`               | Pause lifted                              |
/// | `cb_expired`              | Pause auto-expired on check               |
/// | `cb_proposal_created`     | Multi-auth proposal created               |
/// | `cb_proposal_approved`    | Guardian approved a proposal              |
/// | `cb_proposal_executed`    | Proposal threshold reached, pause applied |
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::types::{
    CircuitBreakerState, DataKey, PauseApproval, PauseLevel, PauseReason, PauseRecord,
};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Maximum number of guardians.
const MAX_GUARDIANS: u32 = 20;
/// Maximum description length (bytes).
const MAX_DESCRIPTION_LEN: u32 = 256;
/// Maximum pause duration in seconds (30 days).
const MAX_PAUSE_DURATION_SECS: u64 = 30 * 24 * 60 * 60;

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

fn get_guardians(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::CircuitBreakerGuardians)
        .unwrap_or_else(|| Vec::new(env))
}

fn set_guardians(env: &Env, guardians: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::CircuitBreakerGuardians, guardians);
}

fn get_state(env: &Env) -> CircuitBreakerState {
    env.storage()
        .persistent()
        .get(&DataKey::CircuitBreakerState)
        .unwrap_or(CircuitBreakerState {
            is_paused: false,
            level: PauseLevel::Advisory,
            current_record_id: 0,
            paused_at: 0,
            expires_at: 0,
        })
}

fn set_state(env: &Env, state: &CircuitBreakerState) {
    env.storage()
        .persistent()
        .set(&DataKey::CircuitBreakerState, state);
}

fn next_record_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::CircuitBreakerNextRecordId)
        .unwrap_or(0)
        + 1;
    env.storage()
        .persistent()
        .set(&DataKey::CircuitBreakerNextRecordId, &id);
    id
}

fn put_record(env: &Env, record: &PauseRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::CircuitBreakerPauseRecord(record.record_id), record);
}

fn get_record(env: &Env, record_id: u64) -> Option<PauseRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::CircuitBreakerPauseRecord(record_id))
}

fn next_approval_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::CircuitBreakerNextApprovalId)
        .unwrap_or(0)
        + 1;
    env.storage()
        .persistent()
        .set(&DataKey::CircuitBreakerNextApprovalId, &id);
    id
}

fn put_approval(env: &Env, approval: &PauseApproval) {
    env.storage()
        .persistent()
        .set(&DataKey::CircuitBreakerPendingApproval(approval.approval_id), approval);
}

fn get_approval(env: &Env, approval_id: u64) -> Option<PauseApproval> {
    env.storage()
        .persistent()
        .get(&DataKey::CircuitBreakerPendingApproval(approval_id))
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = get_admin(env).ok_or(Error::CircuitBreakerNotInitialized)?;
    caller.require_auth();
    if &admin != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn require_guardian(env: &Env, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    let guardians = get_guardians(env);
    if !guardians.contains(caller) {
        return Err(Error::NotGuardian);
    }
    Ok(())
}

/// Returns `true` when the current ledger time has passed `expires_at`
/// (and `expires_at != 0`).
fn is_expired(env: &Env, expires_at: u64) -> bool {
    expires_at != 0 && env.ledger().timestamp() >= expires_at
}

/// Validate description length.
fn validate_description(description: &String) -> Result<(), Error> {
    if description.len() > MAX_DESCRIPTION_LEN {
        return Err(Error::PauseDescriptionTooLong);
    }
    Ok(())
}

/// Validate pause duration (0 = no expiry, otherwise must be ≤ MAX).
fn validate_duration(duration_secs: u64) -> Result<(), Error> {
    if duration_secs != 0 && duration_secs > MAX_PAUSE_DURATION_SECS {
        return Err(Error::InvalidPauseDuration);
    }
    Ok(())
}

/// Core logic: write state + record and emit `cb_paused`.
fn activate_pause(
    env: &Env,
    activated_by: Address,
    level: PauseLevel,
    reason: PauseReason,
    description: String,
    duration_secs: u64,
) -> Result<u64, Error> {
    let now = env.ledger().timestamp();
    let expires_at = if duration_secs == 0 {
        0
    } else {
        now + duration_secs
    };

    let record_id = next_record_id(env);
    let record = PauseRecord {
        record_id,
        activated_by: activated_by.clone(),
        level: level.clone(),
        reason: reason.clone(),
        description: description.clone(),
        activated_at: now,
        expires_at,
        lifted_at: 0,
        lifted_by: Vec::new(env),
    };
    put_record(env, &record);

    let state = CircuitBreakerState {
        is_paused: true,
        level: level.clone(),
        current_record_id: record_id,
        paused_at: now,
        expires_at,
    };
    set_state(env, &state);

    env.events().publish(
        (Symbol::new(env, "cb_paused"), record_id),
        (activated_by, level, reason, description, expires_at),
    );

    Ok(record_id)
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct CircuitBreakerContract;

#[contractimpl]
impl CircuitBreakerContract {
    // ═══════════════════════════════════════════════════════════════════════
    // INITIALISATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Initialise the circuit breaker.
    ///
    /// * `admin`    — address that can add/remove guardians and lift pauses.
    /// * `guardians` — initial set of guardian addresses (may be empty).
    ///
    /// Can only be called once.
    pub fn initialize(
        env: Env,
        admin: Address,
        guardians: Vec<Address>,
    ) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::CircuitBreakerAlreadyInitialized);
        }
        admin.require_auth();

        if guardians.len() > MAX_GUARDIANS {
            return Err(Error::TooManyGuardians);
        }

        // Deduplicate check
        let mut seen = Vec::new(&env);
        for g in guardians.iter() {
            if seen.contains(&g) {
                return Err(Error::DuplicateGuardian);
            }
            seen.push_back(g.clone());
        }

        set_admin(&env, &admin);
        set_guardians(&env, &guardians);

        // Initialise state to unpaused
        set_state(
            &env,
            &CircuitBreakerState {
                is_paused: false,
                level: PauseLevel::Advisory,
                current_record_id: 0,
                paused_at: 0,
                expires_at: 0,
            },
        );

        env.events().publish(
            (Symbol::new(&env, "cb_initialized"),),
            (admin, guardians),
        );

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GUARDIAN MANAGEMENT  (admin only)
    // ═══════════════════════════════════════════════════════════════════════

    /// Add a new guardian.  Admin only.
    pub fn add_guardian(env: Env, caller: Address, guardian: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        let mut guardians = get_guardians(&env);
        if guardians.contains(&guardian) {
            return Err(Error::DuplicateGuardian);
        }
        if guardians.len() >= MAX_GUARDIANS {
            return Err(Error::TooManyGuardians);
        }
        guardians.push_back(guardian.clone());
        set_guardians(&env, &guardians);

        env.events()
            .publish((Symbol::new(&env, "cb_guardian_added"),), guardian);

        Ok(())
    }

    /// Remove an existing guardian.  Admin only.
    pub fn remove_guardian(env: Env, caller: Address, guardian: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;

        let mut guardians = get_guardians(&env);
        let mut idx: Option<u32> = None;
        for i in 0..guardians.len() {
            if guardians.get_unchecked(i) == guardian {
                idx = Some(i);
                break;
            }
        }
        let i = idx.ok_or(Error::NotGuardian)?;
        guardians.remove(i);
        set_guardians(&env, &guardians);

        env.events()
            .publish((Symbol::new(&env, "cb_guardian_removed"),), guardian);

        Ok(())
    }

    /// Return the current guardian list.
    pub fn get_guardians(env: Env) -> Vec<Address> {
        get_guardians(&env)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // INSTANT PAUSE  (single guardian — for active exploits)
    // ═══════════════════════════════════════════════════════════════════════

    /// Immediately activate a pause.
    ///
    /// Any single guardian can call this.  Intended for active security
    /// incidents where speed matters more than consensus.
    ///
    /// * `duration_secs` — seconds until auto-expiry; `0` = no expiry.
    pub fn instant_pause(
        env: Env,
        caller: Address,
        level: PauseLevel,
        reason: PauseReason,
        description: String,
        duration_secs: u64,
    ) -> Result<u64, Error> {
        require_guardian(&env, &caller)?;
        validate_description(&description)?;
        validate_duration(duration_secs)?;

        activate_pause(&env, caller, level, reason, description, duration_secs)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MULTI-AUTHORITY PAUSE  (proposal → approvals → execute)
    // ═══════════════════════════════════════════════════════════════════════

    /// Propose a pause that requires `required_approvals` guardian votes.
    ///
    /// The proposer's vote is counted automatically.
    /// `approval_window_secs` is how long the proposal stays open (0 = no
    /// expiry on the proposal itself).
    pub fn propose_pause(
        env: Env,
        proposer: Address,
        level: PauseLevel,
        reason: PauseReason,
        description: String,
        pause_duration_secs: u64,
        required_approvals: u32,
        approval_window_secs: u64,
    ) -> Result<u64, Error> {
        require_guardian(&env, &proposer)?;
        validate_description(&description)?;
        validate_duration(pause_duration_secs)?;

        let guardians = get_guardians(&env);
        if required_approvals == 0 || required_approvals > guardians.len() {
            return Err(Error::InvalidThreshold);
        }

        let now = env.ledger().timestamp();
        let expires_at = if approval_window_secs == 0 {
            0
        } else {
            now + approval_window_secs
        };

        let approval_id = next_approval_id(&env);
        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer.clone());

        let approval = PauseApproval {
            approval_id,
            proposed_level: level.clone(),
            proposed_reason: reason.clone(),
            description: description.clone(),
            expires_at,
            proposer: proposer.clone(),
            approvals,
            required_approvals,
            executed: false,
        };
        put_approval(&env, &approval);

        env.events().publish(
            (Symbol::new(&env, "cb_proposal_created"), approval_id),
            (proposer, level, reason, required_approvals),
        );

        Ok(approval_id)
    }

    /// Vote to approve a pending pause proposal.
    pub fn approve_pause_proposal(
        env: Env,
        approver: Address,
        approval_id: u64,
    ) -> Result<(), Error> {
        require_guardian(&env, &approver)?;

        let mut approval =
            get_approval(&env, approval_id).ok_or(Error::ApprovalNotFound)?;

        if approval.executed {
            return Err(Error::ApprovalAlreadyExecuted);
        }
        if is_expired(&env, approval.expires_at) {
            return Err(Error::ApprovalExpired);
        }
        if approval.approvals.contains(&approver) {
            return Err(Error::ApprovalAlreadyVoted);
        }

        approval.approvals.push_back(approver.clone());
        put_approval(&env, &approval);

        env.events().publish(
            (Symbol::new(&env, "cb_proposal_approved"), approval_id),
            approver,
        );

        Ok(())
    }

    /// Execute a proposal once the approval threshold is reached.
    ///
    /// Any guardian can trigger execution once enough votes are in.
    /// `pause_duration_secs` is the duration for the resulting pause.
    pub fn execute_pause_proposal(
        env: Env,
        executor: Address,
        approval_id: u64,
        pause_duration_secs: u64,
    ) -> Result<u64, Error> {
        require_guardian(&env, &executor)?;
        validate_duration(pause_duration_secs)?;

        let mut approval =
            get_approval(&env, approval_id).ok_or(Error::ApprovalNotFound)?;

        if approval.executed {
            return Err(Error::ApprovalAlreadyExecuted);
        }
        if is_expired(&env, approval.expires_at) {
            return Err(Error::ApprovalExpired);
        }
        if approval.approvals.len() < approval.required_approvals {
            return Err(Error::ApprovalThresholdNotReached);
        }

        // Mark executed before side-effects (reentrancy guard)
        approval.executed = true;
        put_approval(&env, &approval);

        let record_id = activate_pause(
            &env,
            executor.clone(),
            approval.proposed_level.clone(),
            approval.proposed_reason.clone(),
            approval.description.clone(),
            pause_duration_secs,
        )?;

        env.events().publish(
            (Symbol::new(&env, "cb_proposal_executed"), approval_id),
            (executor, record_id),
        );

        Ok(record_id)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LIFT PAUSE  (admin or multi-guardian)
    // ═══════════════════════════════════════════════════════════════════════

    /// Lift the active pause.  Admin only.
    ///
    /// Returns the record ID of the pause that was lifted.
    pub fn lift_pause(env: Env, caller: Address) -> Result<u64, Error> {
        require_admin(&env, &caller)?;

        let mut state = get_state(&env);
        if !state.is_paused {
            return Err(Error::ContractNotPaused);
        }

        let record_id = state.current_record_id;

        // Update the pause record with lift info
        if let Some(mut record) = get_record(&env, record_id) {
            record.lifted_at = env.ledger().timestamp();
            record.lifted_by.push_back(caller.clone());
            put_record(&env, &record);
        }

        state.is_paused = false;
        state.level = PauseLevel::Advisory;
        state.expires_at = 0;
        set_state(&env, &state);

        env.events().publish(
            (Symbol::new(&env, "cb_lifted"), record_id),
            caller,
        );

        Ok(record_id)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATUS QUERIES  (always available — never blocked)
    // ═══════════════════════════════════════════════════════════════════════

    /// Returns the live circuit breaker state.
    ///
    /// If the pause has expired this call auto-expires it and returns the
    /// updated (unpaused) state.
    pub fn get_state(env: Env) -> CircuitBreakerState {
        let mut state = get_state(&env);
        if state.is_paused && is_expired(&env, state.expires_at) {
            // Auto-expire
            if let Some(mut record) = get_record(&env, state.current_record_id) {
                record.lifted_at = env.ledger().timestamp();
                put_record(&env, &record);
            }
            env.events().publish(
                (Symbol::new(&env, "cb_expired"), state.current_record_id),
                env.ledger().timestamp(),
            );
            state.is_paused = false;
            state.level = PauseLevel::Advisory;
            state.expires_at = 0;
            set_state(&env, &state);
        }
        state
    }

    /// Returns `true` when the contract is paused at any level above Advisory.
    pub fn is_paused(env: Env) -> bool {
        let state = CircuitBreakerContract::get_state(env);
        state.is_paused
    }

    /// Returns `true` when write operations should be blocked
    /// (Partial, Full, or Emergency pause).
    pub fn check_writes_allowed(env: Env) -> bool {
        let state = CircuitBreakerContract::get_state(env);
        if !state.is_paused {
            return true;
        }
        matches!(state.level, PauseLevel::Advisory)
    }

    /// Returns `true` when all mutations are blocked (Full or Emergency).
    pub fn check_mutations_allowed(env: Env) -> bool {
        let state = CircuitBreakerContract::get_state(env);
        if !state.is_paused {
            return true;
        }
        matches!(state.level, PauseLevel::Advisory | PauseLevel::Partial)
    }

    /// Returns `true` when reads are allowed (everything except Emergency).
    pub fn check_reads_allowed(env: Env) -> bool {
        let state = CircuitBreakerContract::get_state(env);
        if !state.is_paused {
            return true;
        }
        !matches!(state.level, PauseLevel::Emergency)
    }

    /// Retrieve a historical pause record by ID.
    pub fn get_pause_record(env: Env, record_id: u64) -> Result<PauseRecord, Error> {
        get_record(&env, record_id).ok_or(Error::PauseRecordNotFound)
    }

    /// Retrieve a pending approval proposal by ID.
    pub fn get_pause_approval(env: Env, approval_id: u64) -> Result<PauseApproval, Error> {
        get_approval(&env, approval_id).ok_or(Error::ApprovalNotFound)
    }

    /// Return the admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        get_admin(&env).ok_or(Error::CircuitBreakerNotInitialized)
    }
}

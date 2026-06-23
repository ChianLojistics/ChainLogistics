//! High-frequency state channel for IoT tracking.
//!
//! Individual sensor updates are far too expensive to anchor 1:1 on-chain. This
//! contract lets two parties (typically an IoT data producer/backend and an
//! auditor/receiver) open a channel, exchange an arbitrary number of
//! bidirectionally-signed state updates **off-chain**, and anchor only the final
//! aggregate on-chain. A single `channel_close` / `channel_checkpoint` call can
//! therefore anchor tens of thousands of updates in one transaction.
//!
//! ## Lifecycle (Open / Update / Close)
//! 1. [`StateChannelContract::channel_open`] – register the channel and the two
//!    Ed25519 keys the parties use to sign off-chain states.
//! 2. [`StateChannelContract::channel_checkpoint`] – *(Update)* optionally anchor
//!    a cooperatively-signed higher-nonce state while keeping the channel open.
//! 3. [`StateChannelContract::channel_close`] – submit the latest signed state
//!    and start the dispute window.
//! 4. [`StateChannelContract::channel_dispute`] – during the window, anyone
//!    holding a strictly-higher-nonce signed state can override a stale/fraudulent
//!    close. This is the fraud-proof mechanism.
//! 5. [`StateChannelContract::channel_finalize`] – after the window elapses, lock
//!    in the last state as the canonical anchored result.
//!
//! ## Off-chain signatures
//! Each state binds `(channel_id, nonce, batch_count, state_root)`. Both parties
//! sign that tuple with Ed25519. Binding `channel_id` prevents cross-channel
//! replay and the strictly-monotonic `nonce` prevents replay of stale states.
//! Invalid signatures cause the host to trap (the transaction reverts), so a
//! forged state can never advance a channel.
//!
//! ## Weighted transitions (ElGamal commitments)
//! `state_root` is treated as an opaque 32-byte commitment. Off-chain it is an
//! ElGamal/Pedersen aggregate of the weighted sensor transitions, which lets the
//! backend fold many readings (with per-reading weights) into one root while
//! preserving additive homomorphism. The contract only needs the root + the
//! signatures + the monotonic nonce to settle, so the commitment scheme can
//! evolve without changing on-chain logic.

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};

use crate::error::Error;
use crate::events::{
    ChannelClosing, ChannelDisputed, ChannelFinalized, ChannelOpened, ChannelStateAnchored,
};
use crate::types::{Channel, ChannelStatus, DataKey, SignedState};
use crate::validation_contract::ValidationContract;
use crate::ChainLogisticsContractClient;

/// Minimum dispute window (1 minute). Backends must stay live enough to post a
/// fraud proof within the window, so it cannot be set arbitrarily small.
pub const MIN_DISPUTE_WINDOW: u64 = 60;
/// Maximum dispute window (7 days) to bound how long settlement can be delayed.
pub const MAX_DISPUTE_WINDOW: u64 = 7 * 24 * 60 * 60;

// ─── Storage helpers ──────────────────────────────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::MainContract, address);
}

fn require_init(env: &Env) -> Result<(), Error> {
    if get_main_contract(env).is_none() {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

/// Ensure the main contract is not paused. State-advancing cooperative calls
/// (open/checkpoint/close) respect the pause switch; `dispute`/`finalize` do not,
/// so a pause can never trap an in-flight dispute.
fn require_not_paused(env: &Env) -> Result<(), Error> {
    let main_contract = get_main_contract(env).ok_or(Error::NotInitialized)?;
    let main_client = ChainLogisticsContractClient::new(env, &main_contract);
    if main_client.is_paused() {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn get_channel(env: &Env, channel_id: u64) -> Option<Channel> {
    env.storage()
        .persistent()
        .get(&DataKey::Channel(channel_id))
}

fn put_channel(env: &Env, channel: &Channel) {
    env.storage()
        .persistent()
        .set(&DataKey::Channel(channel.channel_id), channel);
}

fn next_channel_id(env: &Env) -> Result<u64, Error> {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::ChannelSeq)
        .unwrap_or(0);
    let next = current.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    env.storage().persistent().set(&DataKey::ChannelSeq, &next);
    Ok(next)
}

fn index_channel_for_product(env: &Env, product_id: &String, channel_id: u64) {
    let key = DataKey::ProductChannels(product_id.clone());
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(channel_id);
    env.storage().persistent().set(&key, &ids);
}

// ─── Signature verification ────────────────────────────────────────────────────

/// Deterministic message that both parties sign: big-endian
/// `channel_id || nonce || batch_count || state_root` (56 bytes).
fn state_message(env: &Env, state: &SignedState) -> Bytes {
    let mut msg = Bytes::new(env);
    msg.extend_from_array(&state.channel_id.to_be_bytes());
    msg.extend_from_array(&state.nonce.to_be_bytes());
    msg.extend_from_array(&state.batch_count.to_be_bytes());
    msg.extend_from_array(&state.state_root.to_array());
    msg
}

/// Verify that `state` belongs to `channel` and carries both parties' valid
/// signatures. A bad signature traps the host, reverting the transaction.
fn verify_signed_state(env: &Env, channel: &Channel, state: &SignedState) -> Result<(), Error> {
    if state.channel_id != channel.channel_id {
        return Err(Error::ChannelMismatch);
    }
    let msg = state_message(env, state);
    env.crypto()
        .ed25519_verify(&channel.pubkey_a, &msg, &state.sig_a);
    env.crypto()
        .ed25519_verify(&channel.pubkey_b, &msg, &state.sig_b);
    Ok(())
}

/// Require that `actor` is one of the two channel participants.
fn require_participant(channel: &Channel, actor: &Address) -> Result<(), Error> {
    if actor != &channel.party_a && actor != &channel.party_b {
        return Err(Error::NotChannelParticipant);
    }
    Ok(())
}

// ─── Contract ───────────────────────────────────────────────────────────────

/// State-channel contract for high-frequency IoT tracking updates.
#[contract]
pub struct StateChannelContract;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl StateChannelContract {
    /// Initialize the contract with the main ChainLogistics contract address
    /// (used for the global pause switch).
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        ValidationContract::validate_contract_address(&env, &main_contract)?;
        set_main_contract(&env, &main_contract);
        Ok(())
    }

    /// Open a new state channel between two parties for a product.
    ///
    /// The opener must be one of the two parties and must authenticate. The two
    /// Ed25519 public keys are the keys the parties will use to sign off-chain
    /// states. `dispute_window` is the settlement challenge period, in seconds.
    // 8 parameters (incl. env): this is a public contract entrypoint whose
    // arguments are all required for one atomic open operation.
    #[allow(clippy::too_many_arguments)]
    pub fn channel_open(
        env: Env,
        opener: Address,
        product_id: String,
        party_a: Address,
        party_b: Address,
        pubkey_a: BytesN<32>,
        pubkey_b: BytesN<32>,
        dispute_window: u64,
    ) -> Result<u64, Error> {
        require_init(&env)?;
        require_not_paused(&env)?;
        opener.require_auth();

        if opener != party_a && opener != party_b {
            return Err(Error::NotChannelParticipant);
        }
        ValidationContract::validate_distinct_addresses(&party_a, &party_b)?;
        ValidationContract::non_empty(&product_id)?;
        ValidationContract::max_len(&product_id, ValidationContract::MAX_PRODUCT_ID_LEN)?;

        if dispute_window < MIN_DISPUTE_WINDOW {
            return Err(Error::DisputeWindowTooShort);
        }
        if dispute_window > MAX_DISPUTE_WINDOW {
            return Err(Error::DisputeWindowTooLong);
        }

        let channel_id = next_channel_id(&env)?;
        let channel = Channel {
            channel_id,
            product_id: product_id.clone(),
            party_a: party_a.clone(),
            party_b: party_b.clone(),
            pubkey_a,
            pubkey_b,
            status: ChannelStatus::Open,
            nonce: 0,
            batch_count: 0,
            state_root: BytesN::from_array(&env, &[0u8; 32]),
            dispute_window,
            dispute_deadline: 0,
            opened_at: env.ledger().timestamp(),
            closed_at: 0,
        };
        put_channel(&env, &channel);
        index_channel_for_product(&env, &product_id, channel_id);

        ChannelOpened {
            channel_id,
            product_id,
            party_a,
            party_b,
            dispute_window,
        }
        .publish(&env);

        Ok(channel_id)
    }

    /// *(Update)* Anchor a cooperatively-signed higher-nonce state while keeping
    /// the channel open. Lets long-lived channels checkpoint progress (and the
    /// running `batch_count`) without closing. Requires a strictly higher nonce.
    pub fn channel_checkpoint(
        env: Env,
        submitter: Address,
        state: SignedState,
    ) -> Result<(), Error> {
        require_init(&env)?;
        require_not_paused(&env)?;
        submitter.require_auth();

        let mut channel = get_channel(&env, state.channel_id).ok_or(Error::ChannelNotFound)?;
        require_participant(&channel, &submitter)?;
        if channel.status != ChannelStatus::Open {
            return Err(Error::ChannelNotOpen);
        }
        if state.nonce <= channel.nonce {
            return Err(Error::StaleStateNonce);
        }
        verify_signed_state(&env, &channel, &state)?;

        channel.nonce = state.nonce;
        channel.batch_count = state.batch_count;
        channel.state_root = state.state_root.clone();
        put_channel(&env, &channel);

        ChannelStateAnchored {
            channel_id: channel.channel_id,
            nonce: state.nonce,
            batch_count: state.batch_count,
            state_root: state.state_root,
        }
        .publish(&env);

        Ok(())
    }

    /// Submit the latest signed state to close the channel and start the dispute
    /// window. Accepts a nonce greater than or equal to the anchored nonce so a
    /// channel can be closed at its current checkpoint.
    pub fn channel_close(env: Env, submitter: Address, state: SignedState) -> Result<(), Error> {
        require_init(&env)?;
        require_not_paused(&env)?;
        submitter.require_auth();

        let mut channel = get_channel(&env, state.channel_id).ok_or(Error::ChannelNotFound)?;
        require_participant(&channel, &submitter)?;
        match channel.status {
            ChannelStatus::Open => {}
            ChannelStatus::Closing => return Err(Error::ChannelNotOpen),
            ChannelStatus::Closed => return Err(Error::ChannelAlreadyClosed),
        }
        if state.nonce < channel.nonce {
            return Err(Error::StaleStateNonce);
        }
        verify_signed_state(&env, &channel, &state)?;

        let deadline = env
            .ledger()
            .timestamp()
            .checked_add(channel.dispute_window)
            .ok_or(Error::ArithmeticOverflow)?;

        channel.nonce = state.nonce;
        channel.batch_count = state.batch_count;
        channel.state_root = state.state_root;
        channel.status = ChannelStatus::Closing;
        channel.dispute_deadline = deadline;
        put_channel(&env, &channel);

        ChannelClosing {
            channel_id: channel.channel_id,
            nonce: channel.nonce,
            batch_count: channel.batch_count,
            dispute_deadline: deadline,
        }
        .publish(&env);

        Ok(())
    }

    /// Override a closing channel with a strictly-higher-nonce signed state
    /// (fraud proof). Callable by any participant while the dispute window is
    /// open; it refreshes the deadline so the counterparty cannot rush a stale
    /// close past the challenger. Works even while the contract is paused.
    pub fn channel_dispute(env: Env, challenger: Address, state: SignedState) -> Result<(), Error> {
        require_init(&env)?;
        challenger.require_auth();

        let mut channel = get_channel(&env, state.channel_id).ok_or(Error::ChannelNotFound)?;
        require_participant(&channel, &challenger)?;
        if channel.status != ChannelStatus::Closing {
            return Err(Error::ChannelNotClosing);
        }
        if env.ledger().timestamp() > channel.dispute_deadline {
            return Err(Error::DisputePeriodExpired);
        }
        if state.nonce <= channel.nonce {
            return Err(Error::StaleStateNonce);
        }
        verify_signed_state(&env, &channel, &state)?;

        let old_nonce = channel.nonce;
        let deadline = env
            .ledger()
            .timestamp()
            .checked_add(channel.dispute_window)
            .ok_or(Error::ArithmeticOverflow)?;

        channel.nonce = state.nonce;
        channel.batch_count = state.batch_count;
        channel.state_root = state.state_root;
        channel.dispute_deadline = deadline;
        put_channel(&env, &channel);

        ChannelDisputed {
            channel_id: channel.channel_id,
            challenger,
            old_nonce,
            new_nonce: state.nonce,
            dispute_deadline: deadline,
        }
        .publish(&env);

        Ok(())
    }

    /// Finalize a closing channel once its dispute window has elapsed, anchoring
    /// the last state as the canonical result. Works even while paused so funds
    /// and settlement can never be trapped by an admin pause.
    pub fn channel_finalize(env: Env, submitter: Address, channel_id: u64) -> Result<(), Error> {
        require_init(&env)?;
        submitter.require_auth();

        let mut channel = get_channel(&env, channel_id).ok_or(Error::ChannelNotFound)?;
        require_participant(&channel, &submitter)?;
        if channel.status != ChannelStatus::Closing {
            return Err(Error::ChannelNotClosing);
        }
        if env.ledger().timestamp() <= channel.dispute_deadline {
            return Err(Error::DisputePeriodActive);
        }

        channel.status = ChannelStatus::Closed;
        channel.closed_at = env.ledger().timestamp();
        put_channel(&env, &channel);

        ChannelFinalized {
            channel_id: channel.channel_id,
            nonce: channel.nonce,
            batch_count: channel.batch_count,
            state_root: channel.state_root.clone(),
        }
        .publish(&env);

        Ok(())
    }

    // ─── Views ────────────────────────────────────────────────────────────

    /// Get the full channel record.
    pub fn channel_get(env: Env, channel_id: u64) -> Result<Channel, Error> {
        get_channel(&env, channel_id).ok_or(Error::ChannelNotFound)
    }

    /// Get the current status of a channel.
    pub fn channel_status(env: Env, channel_id: u64) -> Result<ChannelStatus, Error> {
        Ok(get_channel(&env, channel_id)
            .ok_or(Error::ChannelNotFound)?
            .status)
    }

    /// Get the highest anchored nonce for a channel.
    pub fn channel_nonce(env: Env, channel_id: u64) -> Result<u64, Error> {
        Ok(get_channel(&env, channel_id)
            .ok_or(Error::ChannelNotFound)?
            .nonce)
    }

    /// List the channel ids opened against a product.
    pub fn channel_list_by_product(env: Env, product_id: String) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ProductChannels(product_id))
            .unwrap_or_else(|| Vec::new(&env))
    }
}

#[cfg(test)]
mod test_state_channel {
    use super::*;
    use crate::{ChainLogisticsContract, ChainLogisticsContractClient};
    use ed25519_dalek::{Signer, SigningKey};
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, BytesN, Env, String,
    };

    struct Party {
        signing: SigningKey,
    }

    impl Party {
        fn new(seed: u8) -> Self {
            Party {
                signing: SigningKey::from_bytes(&[seed; 32]),
            }
        }

        fn pubkey(&self, env: &Env) -> BytesN<32> {
            BytesN::from_array(env, &self.signing.verifying_key().to_bytes())
        }
    }

    /// Build the same `channel_id || nonce || batch_count || state_root` message
    /// the contract verifies, then return a signature over it.
    fn sign(
        party: &Party,
        channel_id: u64,
        nonce: u64,
        batch_count: u64,
        root: &[u8; 32],
    ) -> [u8; 64] {
        let mut msg = [0u8; 56];
        msg[0..8].copy_from_slice(&channel_id.to_be_bytes());
        msg[8..16].copy_from_slice(&nonce.to_be_bytes());
        msg[16..24].copy_from_slice(&batch_count.to_be_bytes());
        msg[24..56].copy_from_slice(root);
        party.signing.sign(&msg).to_bytes()
    }

    fn signed_state(
        env: &Env,
        a: &Party,
        b: &Party,
        channel_id: u64,
        nonce: u64,
        batch_count: u64,
        root: &[u8; 32],
    ) -> SignedState {
        SignedState {
            channel_id,
            nonce,
            batch_count,
            state_root: BytesN::from_array(env, root),
            sig_a: BytesN::from_array(env, &sign(a, channel_id, nonce, batch_count, root)),
            sig_b: BytesN::from_array(env, &sign(b, channel_id, nonce, batch_count, root)),
        }
    }

    fn setup(
        env: &Env,
    ) -> (
        StateChannelContractClient<'static>,
        ChainLogisticsContractClient<'static>,
        Address,
    ) {
        let cl_id = env.register_contract(None, ChainLogisticsContract);
        let sc_id = env.register_contract(None, StateChannelContract);
        let cl_client = ChainLogisticsContractClient::new(env, &cl_id);
        let sc_client = StateChannelContractClient::new(env, &sc_id);

        // ChainLogisticsContract::init needs an auth contract address distinct
        // from its own; any other address satisfies the validation here.
        let admin = Address::generate(env);
        let auth_contract = Address::generate(env);
        cl_client.init(&admin, &auth_contract);
        sc_client.init(&cl_id);
        (sc_client, cl_client, admin)
    }

    fn open_channel(
        env: &Env,
        sc: &StateChannelContractClient,
        a: &Party,
        b: &Party,
        window: u64,
    ) -> (Address, Address, u64) {
        let addr_a = Address::generate(env);
        let addr_b = Address::generate(env);
        let product_id = String::from_str(env, "PROD-IOT-1");
        let channel_id = sc.channel_open(
            &addr_a,
            &product_id,
            &addr_a,
            &addr_b,
            &a.pubkey(env),
            &b.pubkey(env),
            &window,
        );
        (addr_a, addr_b, channel_id)
    }

    #[test]
    fn test_open_channel() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        assert_eq!(channel_id, 1);
        let channel = sc.channel_get(&channel_id);
        assert_eq!(channel.party_a, addr_a);
        assert_eq!(channel.party_b, addr_b);
        assert_eq!(channel.status, ChannelStatus::Open);
        assert_eq!(channel.nonce, 0);
        let ids = sc.channel_list_by_product(&String::from_str(&env, "PROD-IOT-1"));
        assert_eq!(ids.len(), 1);
        assert_eq!(ids.get(0), Some(1));
    }

    #[test]
    fn test_checkpoint_anchors_high_frequency_batch() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, _addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        // A single on-chain submission anchors 12_000 off-chain updates.
        let root = [7u8; 32];
        let state = signed_state(&env, &a, &b, channel_id, 1, 12_000, &root);
        sc.channel_checkpoint(&addr_a, &state);

        let channel = sc.channel_get(&channel_id);
        assert_eq!(channel.nonce, 1);
        assert_eq!(channel.batch_count, 12_000);
        assert_eq!(channel.state_root, BytesN::from_array(&env, &root));
    }

    #[test]
    fn test_checkpoint_rejects_stale_nonce() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, _addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        let root = [1u8; 32];
        sc.channel_checkpoint(
            &addr_a,
            &signed_state(&env, &a, &b, channel_id, 5, 100, &root),
        );
        // Re-submitting an equal or lower nonce is rejected.
        let res = sc.try_channel_checkpoint(
            &addr_a,
            &signed_state(&env, &a, &b, channel_id, 5, 100, &root),
        );
        assert_eq!(res, Err(Ok(Error::StaleStateNonce)));
    }

    #[test]
    fn test_full_close_and_finalize() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, _addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        let root = [9u8; 32];
        sc.channel_close(
            &addr_a,
            &signed_state(&env, &a, &b, channel_id, 10, 50_000, &root),
        );
        assert_eq!(sc.channel_status(&channel_id), ChannelStatus::Closing);

        // Cannot finalize while the dispute window is active.
        let early = sc.try_channel_finalize(&addr_a, &channel_id);
        assert_eq!(early, Err(Ok(Error::DisputePeriodActive)));

        // Advance past the dispute window.
        env.ledger().with_mut(|l| l.timestamp += 3601);
        sc.channel_finalize(&addr_a, &channel_id);

        let channel = sc.channel_get(&channel_id);
        assert_eq!(channel.status, ChannelStatus::Closed);
        assert_eq!(channel.nonce, 10);
        assert_eq!(channel.batch_count, 50_000);
    }

    #[test]
    fn test_successful_fraud_proof_challenge() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        // Party A tries to settle on a stale state (nonce 3).
        sc.channel_close(
            &addr_a,
            &signed_state(&env, &a, &b, channel_id, 3, 300, &[3u8; 32]),
        );

        // Party B challenges with a newer co-signed state (nonce 7).
        sc.channel_dispute(
            &addr_b,
            &signed_state(&env, &a, &b, channel_id, 7, 700, &[7u8; 32]),
        );
        let channel = sc.channel_get(&channel_id);
        assert_eq!(channel.nonce, 7);
        assert_eq!(channel.batch_count, 700);

        // After the refreshed window passes, the honest state finalizes.
        env.ledger().with_mut(|l| l.timestamp += 3601);
        sc.channel_finalize(&addr_b, &channel_id);
        assert_eq!(sc.channel_get(&channel_id).nonce, 7);
    }

    #[test]
    fn test_dispute_rejects_stale_nonce() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        sc.channel_close(
            &addr_a,
            &signed_state(&env, &a, &b, channel_id, 5, 500, &[5u8; 32]),
        );
        // A challenge that is not strictly newer is rejected.
        let res = sc.try_channel_dispute(
            &addr_b,
            &signed_state(&env, &a, &b, channel_id, 5, 500, &[5u8; 32]),
        );
        assert_eq!(res, Err(Ok(Error::StaleStateNonce)));
    }

    #[test]
    fn test_dispute_after_window_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        sc.channel_close(
            &addr_a,
            &signed_state(&env, &a, &b, channel_id, 2, 20, &[2u8; 32]),
        );
        env.ledger().with_mut(|l| l.timestamp += 3601);
        let res = sc.try_channel_dispute(
            &addr_b,
            &signed_state(&env, &a, &b, channel_id, 9, 90, &[9u8; 32]),
        );
        assert_eq!(res, Err(Ok(Error::DisputePeriodExpired)));
    }

    #[test]
    fn test_forged_signature_traps() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (addr_a, _addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        // Sign with the wrong key for party B (impostor) — host must trap.
        let impostor = Party::new(99);
        let root = [4u8; 32];
        let forged = SignedState {
            channel_id,
            nonce: 1,
            batch_count: 10,
            state_root: BytesN::from_array(&env, &root),
            sig_a: BytesN::from_array(&env, &sign(&a, channel_id, 1, 10, &root)),
            sig_b: BytesN::from_array(&env, &sign(&impostor, channel_id, 1, 10, &root)),
        };
        let res = sc.try_channel_checkpoint(&addr_a, &forged);
        assert!(res.is_err());
    }

    #[test]
    fn test_non_participant_cannot_act() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let (_addr_a, _addr_b, channel_id) = open_channel(&env, &sc, &a, &b, 3600);

        let outsider = Address::generate(&env);
        let res = sc.try_channel_checkpoint(
            &outsider,
            &signed_state(&env, &a, &b, channel_id, 1, 1, &[1u8; 32]),
        );
        assert_eq!(res, Err(Ok(Error::NotChannelParticipant)));
    }

    #[test]
    fn test_dispute_window_bounds() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, _cl, _admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);
        let product_id = String::from_str(&env, "PROD-IOT-1");

        let too_short = sc.try_channel_open(
            &addr_a,
            &product_id,
            &addr_a,
            &addr_b,
            &a.pubkey(&env),
            &b.pubkey(&env),
            &(MIN_DISPUTE_WINDOW - 1),
        );
        assert_eq!(too_short, Err(Ok(Error::DisputeWindowTooShort)));

        let too_long = sc.try_channel_open(
            &addr_a,
            &product_id,
            &addr_a,
            &addr_b,
            &a.pubkey(&env),
            &b.pubkey(&env),
            &(MAX_DISPUTE_WINDOW + 1),
        );
        assert_eq!(too_long, Err(Ok(Error::DisputeWindowTooLong)));
    }

    #[test]
    fn test_init_already_initialized_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, cl, _admin) = setup(&env);
        let res = sc.try_init(&cl.address);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_open_when_paused_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (sc, cl, admin) = setup(&env);
        let (a, b) = (Party::new(1), Party::new(2));
        cl.pause(&admin);

        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);
        let res = sc.try_channel_open(
            &addr_a,
            &String::from_str(&env, "PROD-IOT-1"),
            &addr_a,
            &addr_b,
            &a.pubkey(&env),
            &b.pubkey(&env),
            &3600,
        );
        assert_eq!(res, Err(Ok(Error::ContractPaused)));
    }
}

# High-Frequency State-Channel for IoT Tracking

## Description

Individual IoT sensor updates are too expensive to anchor 1:1 on-chain. This PR
adds a **state-channel contract** that lets two parties (e.g. an IoT data
producer/backend and an auditor/receiver) exchange an arbitrary number of
bidirectionally-signed state updates **off-chain** and anchor only the aggregate
commitment on-chain. A single on-chain submission can therefore settle
**10,000+ off-chain updates** in one transaction.

The contract implements the full `Open → Update → Close` lifecycle, verifies
Ed25519-signed off-chain states, and provides a **dispute-period** mechanism with
working fraud-proof challenges.

## Type of Change
- [x] ✨ New feature (non-breaking change that adds functionality)
- [x] 🔒 Security (signature verification + fraud-proof dispute mechanism)

## Changes Made

### Code Changes
- **`contracts/src/state_channel.rs`** (new): `StateChannelContract` with:
  - `channel_open` — register a channel for a product with both parties'
    Ed25519 public keys and a bounded dispute window.
  - `channel_checkpoint` — *(Update)* anchor a cooperatively-signed, strictly
    higher-nonce state while the channel stays open (rolling `batch_count`).
  - `channel_close` — submit the latest signed state and start the dispute window.
  - `channel_dispute` — override a closing channel with a strictly-higher-nonce
    co-signed state (fraud proof); refreshes the deadline. Works while paused so
    a pause can never block a challenge.
  - `channel_finalize` — anchor the last state as canonical once the window
    elapses. Also works while paused so settlement can't be trapped.
  - Views: `channel_get`, `channel_status`, `channel_nonce`,
    `channel_list_by_product`.
- **Signed off-chain states**: each state binds `(channel_id, nonce,
  batch_count, state_root)`; both parties sign with Ed25519. `channel_id`
  binding prevents cross-channel replay and the strictly-monotonic `nonce`
  prevents stale-state replay. Invalid signatures trap the host (tx reverts).
- **`contracts/src/types.rs`**: added `Channel`, `SignedState`, `ChannelStatus`
  and `DataKey::{Channel, ChannelSeq, ProductChannels}`.
- **`contracts/src/error.rs`**: added the `130–141` state-channel error range.
- **`contracts/src/events.rs`**: added `ChannelOpened`, `ChannelStateAnchored`,
  `ChannelClosing`, `ChannelDisputed`, `ChannelFinalized` typed events for
  dashboard integration.
- **`contracts/src/lib.rs`**: registered/re-exported the module (host-side, like
  the other satellite contracts, to avoid WASM `#[contract]` symbol collisions).

### Documentation Updates
- `smart-contract/README.md`: documented the new module under Key Modules.
- Module-level rustdoc in `state_channel.rs` covers the lifecycle, the off-chain
  signature scheme, and the ElGamal/Pedersen commitment design for `state_root`.

### Database/Schema Changes
- None.

## Testing

### Automated Tests
12 new unit tests in `state_channel.rs` use **real Ed25519 keypairs** (via the
`ed25519-dalek` dev-dependency, already present transitively in `Cargo.lock`) to
produce signatures matching the on-chain message layout:

- `test_open_channel`
- `test_checkpoint_anchors_high_frequency_batch` — anchors **12,000** updates in
  one call
- `test_checkpoint_rejects_stale_nonce`
- `test_full_close_and_finalize` — anchors **50,000** updates; rejects early
  finalize
- `test_successful_fraud_proof_challenge` — stale close overridden by a newer
  co-signed state
- `test_dispute_rejects_stale_nonce`
- `test_dispute_after_window_rejected`
- `test_forged_signature_traps`
- `test_non_participant_cannot_act`
- `test_dispute_window_bounds`
- `test_init_already_initialized_fails`
- `test_open_when_paused_fails`

- [x] Unit tests pass (`192 passed; 0 failed` for the full suite)
- [x] `cargo clippy --all-targets --all-features -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean
- [x] WASM build succeeds (`wasm32v1-none`)

```bash
cd smart-contract
cargo test -p chainlogistics --lib state_channel
```

## Breaking Changes
None. The contract is additive and gated host-side like the other satellite
contracts; existing import paths are unchanged.

## Checklist
- [x] Code follows project style guidelines
- [x] Code is self-documenting / properly commented
- [x] No unused imports or variables
- [x] Code passes linting checks
- [x] Unit tests added for new functionality
- [x] Edge cases considered and tested (stale nonces, expired/active windows,
      forged signatures, non-participants, pause behavior)
- [x] Input validation added (dispute-window bounds, participant checks,
      distinct addresses, product id length)

## Acceptance Criteria
- [x] **10,000+ updates anchored per TX** — `batch_count` folds an unbounded
      number of off-chain updates into one anchored root (tests anchor 12k & 50k).
- [x] **Successful fraud-proof challenges** — `channel_dispute` overrides a stale
      close with a higher-nonce co-signed state (`test_successful_fraud_proof_challenge`).
- [x] **Dashboard integration** — typed `Channel*` events + read-only views
      (`channel_get`, `channel_status`, `channel_nonce`, `channel_list_by_product`).

### Design answers
- **Ideal dispute window?** Configurable per channel, bounded to
  `[60s, 7 days]` (`MIN/MAX_DISPUTE_WINDOW`) — short enough for low-latency
  settlement, long enough for a backend to post a fraud proof.
- **Liveness requirements for backend?** The backend (or either party) only
  needs to be online once within the dispute window to submit a higher-nonce
  state. `dispute`/`finalize` intentionally work even while the contract is
  paused, so liveness is never blocked by admin pause.

## Related Issues
Closes #468

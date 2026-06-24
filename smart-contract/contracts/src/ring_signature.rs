//! AOS/SAG ring signatures over BLS12-381 G1 for anonymous auditor
//! attestations. A signature proves the signer is one of the ring's members
//! without revealing which. Scheme, wire format and design rationale live in
//! `RING_SIGNATURE.md`; the SDKs and the test signer below must stay byte-for-
//! byte compatible with the verification here.

#![allow(clippy::too_many_arguments)]

use soroban_sdk::crypto::bls12_381::{Bls12381G1Affine as G1, Fr};
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Vec,
};

use crate::error::Error;
use crate::ChainLogisticsContractClient;

const DST_CHALLENGE: &[u8] = b"CHAINLOGISTICS-RINGSIG-V1-CHALLENGE";
const DST_RING: &[u8] = b"CHAINLOGISTICS-RINGSIG-V1-RING";

/// A ring of one is a deanonymized Schnorr signature, so reject it.
pub const MIN_RING_SIZE: u32 = 2;
/// Verification cost is linear in ring size; cap it to bound the per-call gas.
pub const MAX_RING_SIZE: u32 = 32;

/// Standard BLS12-381 G1 generator, uncompressed (`x ‖ y`, 48 bytes each,
/// big-endian). Shared by every BLS library, which is what lets off-chain
/// signers and this verifier agree. Validated by `tests::generator_is_valid`.
const G1_GENERATOR: [u8; 96] = [
    // x
    0x17, 0xf1, 0xd3, 0xa7, 0x31, 0x97, 0xd7, 0x94, 0x26, 0x95, 0x63, 0x8c, 0x4f, 0xa9, 0xac, 0x0f,
    0xc3, 0x68, 0x8c, 0x4f, 0x97, 0x74, 0xb9, 0x05, 0xa1, 0x4e, 0x3a, 0x3f, 0x17, 0x1b, 0xac, 0x58,
    0x6c, 0x55, 0xe8, 0x3f, 0xf9, 0x7a, 0x1a, 0xef, 0xfb, 0x3a, 0xf0, 0x0a, 0xdb, 0x22, 0xc6, 0xbb,
    // y
    0x08, 0xb3, 0xf4, 0x81, 0xe3, 0xaa, 0xa0, 0xf1, 0xa0, 0x9e, 0x30, 0xed, 0x74, 0x1d, 0x8a, 0xe4,
    0xfc, 0xf5, 0xe0, 0x95, 0xd5, 0xd0, 0x0a, 0xf6, 0x00, 0xdb, 0x18, 0xcb, 0x2c, 0x04, 0xb3, 0xed,
    0xd0, 0x3c, 0xc7, 0x44, 0xa2, 0x88, 0x8a, 0xe4, 0x0c, 0xaa, 0x23, 0x29, 0x46, 0xc5, 0xe7, 0xe1,
];

/// `c0` and each `s` are 32-byte big-endian canonical `F_r` scalars; `s` has
/// one entry per ring member, in ring order.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RingSignature {
    pub c0: BytesN<32>,
    pub s: Vec<BytesN<32>>,
}

/// An anonymous handover attestation. Carries no signer identity — only the
/// anonymity set commitment, the product and the statement hash.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoverRecord {
    pub record_id: u64,
    pub product_id: String,
    pub ring_commitment: BytesN<32>,
    pub ring_size: u32,
    pub statement_hash: BytesN<32>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
enum AuditKey {
    MainContract,
    RecordSeq,
    Record(u64),
    ProductRecords(String),
}

// ─── Events ───────────────────────────────────────────────────────────────

#[soroban_sdk::contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivacyHandoverRecorded {
    pub record_id: u64,
    pub product_id: String,
    pub ring_commitment: BytesN<32>,
    pub ring_size: u32,
    pub statement_hash: BytesN<32>,
}

// ─── Core cryptography (shared by both contracts and the tests) ─────────────

fn generator(env: &Env) -> G1 {
    G1::from_array(env, &G1_GENERATOR)
}

/// `SHA256(DST_RING ‖ n ‖ P_0 ‖ … ‖ P_{n-1})`. `n` is included so rings whose
/// member concatenations would otherwise collide stay distinct.
fn ring_commitment(env: &Env, ring: &Vec<BytesN<96>>) -> BytesN<32> {
    let mut buf = Bytes::new(env);
    buf.extend_from_slice(DST_RING);
    buf.extend_from_array(&ring.len().to_be_bytes());
    for member in ring.iter() {
        buf.extend_from_array(&member.to_array());
    }
    env.crypto().sha256(&buf).to_bytes()
}

/// Fiat–Shamir challenge in `F_r`. `Fr::from_bytes` reduces the big-endian
/// digest mod `r`; the SDKs mirror this as `int.from_bytes(digest,'big') % r`.
fn challenge(env: &Env, commit: &BytesN<32>, msg_digest: &BytesN<32>, l: &G1) -> Fr {
    let mut buf = Bytes::new(env);
    buf.extend_from_slice(DST_CHALLENGE);
    buf.extend_from_array(&commit.to_array());
    buf.extend_from_array(&msg_digest.to_array());
    buf.extend_from_array(&l.to_array());
    Fr::from_bytes(env.crypto().sha256(&buf).to_bytes())
}

fn check_ring(env: &Env, ring: &Vec<BytesN<96>>, sig: &RingSignature) -> Result<(), Error> {
    let n = ring.len();
    if n < MIN_RING_SIZE {
        return Err(Error::RingTooSmall);
    }
    if n > MAX_RING_SIZE {
        return Err(Error::RingTooLarge);
    }
    if sig.s.len() != n {
        return Err(Error::RingSizeMismatch);
    }
    let bls = env.crypto().bls12_381();
    for i in 0..n {
        let member = ring.get(i).unwrap();
        // Duplicates shrink the effective anonymity set.
        for j in (i + 1)..n {
            if member == ring.get(j).unwrap() {
                return Err(Error::DuplicateRingMember);
            }
        }
        let pk = G1::from_bytes(member);
        // Soundness-critical: rejects small-subgroup / invalid-curve points.
        if !bls.g1_is_in_subgroup(&pk) {
            return Err(Error::InvalidRingMember);
        }
    }
    Ok(())
}

/// Returns `Ok(())` if `sig` is a valid ring signature over `message` for
/// `ring`, else the specific rejection reason. Read-only.
pub fn verify_ring_signature(
    env: &Env,
    ring: &Vec<BytesN<96>>,
    message: &Bytes,
    sig: &RingSignature,
) -> Result<(), Error> {
    check_ring(env, ring, sig)?;

    let bls = env.crypto().bls12_381();
    let g = generator(env);
    let commit = ring_commitment(env, ring);
    let msg_digest = env.crypto().sha256(message).to_bytes();

    let c0 = Fr::from_bytes(sig.c0.clone());
    let mut c = c0.clone();
    for i in 0..ring.len() {
        let pk = G1::from_bytes(ring.get(i).unwrap());
        let si = Fr::from_bytes(sig.s.get(i).unwrap());
        // L_i = s_i · G + c · P_i
        let l = bls.g1_add(&bls.g1_mul(&g, &si), &bls.g1_mul(&pk, &c));
        c = challenge(env, &commit, &msg_digest, &l);
    }

    if c == c0 {
        Ok(())
    } else {
        Err(Error::RingSignatureInvalid)
    }
}

// ─── Stateless verifier contract ────────────────────────────────────────────

/// Stateless ring-signature verification gateway.
#[contract]
pub struct RingSignatureVerifier;

#[contractimpl]
impl RingSignatureVerifier {
    /// `true` iff `sig` was produced by a holder of one of the `ring` keys over
    /// `message`. Returns `false` for invalid signatures; traps only on
    /// malformed point encodings.
    pub fn verify(env: Env, ring: Vec<BytesN<96>>, message: Bytes, sig: RingSignature) -> bool {
        verify_ring_signature(&env, &ring, &message, &sig).is_ok()
    }

    /// Like [`Self::verify`] but returns the rejection reason. Named
    /// `verify_strict` because the client reserves `try_verify`.
    pub fn verify_strict(
        env: Env,
        ring: Vec<BytesN<96>>,
        message: Bytes,
        sig: RingSignature,
    ) -> Result<(), Error> {
        verify_ring_signature(&env, &ring, &message, &sig)
    }

    /// Canonical ring commitment binding a signature to its anonymity set.
    pub fn aggregate_ring(env: Env, ring: Vec<BytesN<96>>) -> BytesN<32> {
        ring_commitment(&env, &ring)
    }

    /// The generator signers must use, so clients can assert parameter parity.
    pub fn generator(env: Env) -> BytesN<96> {
        BytesN::from_array(&env, &G1_GENERATOR)
    }
}

// ─── Privacy audit-trail contract ───────────────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&AuditKey::MainContract)
}

fn require_init(env: &Env) -> Result<(), Error> {
    if get_main_contract(env).is_none() {
        return Err(Error::AuditTrailNotInitialized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), Error> {
    let main = get_main_contract(env).ok_or(Error::AuditTrailNotInitialized)?;
    if ChainLogisticsContractClient::new(env, &main).is_paused() {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn next_record_id(env: &Env) -> Result<u64, Error> {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&AuditKey::RecordSeq)
        .unwrap_or(0);
    let next = current.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    env.storage().persistent().set(&AuditKey::RecordSeq, &next);
    Ok(next)
}

/// Records anonymous, ring-signed handover attestations. No signer identity is
/// ever stored or emitted — only the ring commitment, ring size and statement
/// hash.
#[contract]
pub struct AuditTrailContract;

#[contractimpl]
impl AuditTrailContract {
    /// Wire in the main contract address (for the global pause). Callable once.
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&AuditKey::MainContract, &main_contract);
        Ok(())
    }

    /// Verify a ring signature over `statement` and, on success, store an
    /// anonymous [`HandoverRecord`] and emit [`PrivacyHandoverRecorded`].
    /// `statement` is an opaque caller-defined message (bind the action context
    /// into it, e.g. `product_id ‖ from ‖ to ‖ nonce`). Returns the record id.
    pub fn record_handover(
        env: Env,
        product_id: String,
        ring: Vec<BytesN<96>>,
        statement: Bytes,
        sig: RingSignature,
    ) -> Result<u64, Error> {
        require_init(&env)?;
        require_not_paused(&env)?;

        verify_ring_signature(&env, &ring, &statement, &sig)?;

        let record_id = next_record_id(&env)?;
        let record = HandoverRecord {
            record_id,
            product_id: product_id.clone(),
            ring_commitment: ring_commitment(&env, &ring),
            ring_size: ring.len(),
            statement_hash: env.crypto().sha256(&statement).to_bytes(),
            timestamp: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&AuditKey::Record(record_id), &record);

        let key = AuditKey::ProductRecords(product_id.clone());
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));
        ids.push_back(record_id);
        env.storage().persistent().set(&key, &ids);

        PrivacyHandoverRecorded {
            record_id,
            product_id,
            ring_commitment: record.ring_commitment.clone(),
            ring_size: record.ring_size,
            statement_hash: record.statement_hash.clone(),
        }
        .publish(&env);

        Ok(record_id)
    }

    /// Fetch a recorded handover by id.
    pub fn get_handover(env: Env, record_id: u64) -> Option<HandoverRecord> {
        env.storage().persistent().get(&AuditKey::Record(record_id))
    }

    /// List the handover record ids attached to a product.
    pub fn product_handovers(env: Env, product_id: String) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&AuditKey::ProductRecords(product_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Total number of handovers recorded so far.
    pub fn total_handovers(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&AuditKey::RecordSeq)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;

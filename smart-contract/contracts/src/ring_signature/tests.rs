//! Tests for the ring-signature verifier and the privacy audit trail. [`sign`]
//! is the reference signer the SDKs mirror; it uses the same host functions as
//! the verifier, so both agree bit-for-bit and the on-chain `verify` is an
//! oracle for the SDK signers (cross-checked via `emit_sdk_test_vector`).

extern crate std;

use super::*;
use crate::{ChainLogisticsContract, ChainLogisticsContractClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, Bytes, BytesN, Env, String, Vec,
};

// ─── Reference signer (mirrors the SDKs) ────────────────────────────────────

// Deterministic seeds keep test failures reproducible; the verifier only needs
// each value to be a valid F_r scalar.
fn fr_from_seed(env: &Env, tag: &[u8], i: u32) -> Fr {
    let mut b = Bytes::new(env);
    b.extend_from_slice(tag);
    b.extend_from_array(&i.to_be_bytes());
    Fr::from_bytes(env.crypto().sha256(&b).to_bytes())
}

fn keypair(env: &Env, i: u32) -> (Fr, BytesN<96>) {
    let x = fr_from_seed(env, b"chainlogistics-sk", i);
    let p = env.crypto().bls12_381().g1_mul(&generator(env), &x);
    (x, p.to_bytes())
}

fn build_ring(env: &Env, n: u32, signer_index: u32) -> (Vec<BytesN<96>>, Fr) {
    let mut ring = Vec::new(env);
    let mut secret = fr_from_seed(env, b"placeholder", 0);
    for i in 0..n {
        let (x, p) = keypair(env, i);
        if i == signer_index {
            secret = x;
        }
        ring.push_back(p);
    }
    (ring, secret)
}

/// Produce a ring signature over `message` as ring member `signer_index`.
// Produce a ring signature over `message` as ring member `signer_index`.
fn sign(
    env: &Env,
    ring: &Vec<BytesN<96>>,
    signer_index: u32,
    secret: &Fr,
    message: &Bytes,
) -> RingSignature {
    let n = ring.len();
    let bls = env.crypto().bls12_381();
    let g = generator(env);
    let commit = ring_commitment(env, ring);
    let md = env.crypto().sha256(message).to_bytes();

    let zero = BytesN::from_array(env, &[0u8; 32]);
    let mut c_arr: Vec<BytesN<32>> = Vec::new(env);
    let mut s_arr: Vec<BytesN<32>> = Vec::new(env);
    for _ in 0..n {
        c_arr.push_back(zero.clone());
        s_arr.push_back(zero.clone());
    }

    // Seed the chain at the signer's index with a random nonce α.
    let alpha = fr_from_seed(env, b"chainlogistics-alpha", signer_index);
    let l_pi = bls.g1_mul(&g, &alpha);
    c_arr.set(
        (signer_index + 1) % n,
        challenge(env, &commit, &md, &l_pi).to_bytes(),
    );

    // Walk the remaining ring members with random decoy responses.
    for j in 1..n {
        let idx = (signer_index + j) % n;
        let s_idx = fr_from_seed(env, b"chainlogistics-decoy", idx);
        s_arr.set(idx, s_idx.to_bytes());

        let c_idx = Fr::from_bytes(c_arr.get(idx).unwrap());
        let pk = G1::from_bytes(ring.get(idx).unwrap());
        let l = bls.g1_add(&bls.g1_mul(&g, &s_idx), &bls.g1_mul(&pk, &c_idx));
        c_arr.set((idx + 1) % n, challenge(env, &commit, &md, &l).to_bytes());
    }

    // Close the ring: s_π = α − c_π · x  (mod r).
    let c_pi = Fr::from_bytes(c_arr.get(signer_index).unwrap());
    let s_pi = alpha - (c_pi * secret.clone());
    s_arr.set(signer_index, s_pi.to_bytes());

    RingSignature {
        c0: c_arr.get(0).unwrap(),
        s: s_arr,
    }
}

fn msg(env: &Env, bytes: &[u8]) -> Bytes {
    Bytes::from_slice(env, bytes)
}

fn hex32(b: &BytesN<32>) -> std::string::String {
    let mut s = std::string::String::new();
    for byte in b.to_array().iter() {
        s.push_str(&std::format!("{byte:02x}"));
    }
    s
}

fn hex96(b: &BytesN<96>) -> std::string::String {
    let mut s = std::string::String::new();
    for byte in b.to_array().iter() {
        s.push_str(&std::format!("{byte:02x}"));
    }
    s
}

// Emits the known-answer vector the SDK test suites embed. An SDK `verify`
// accepting this chain-produced signature proves bit-for-bit agreement.
// Run: `cargo test emit_sdk_test_vector -- --nocapture`.
#[test]
fn emit_sdk_test_vector() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let n = 3u32;
    let signer = 1u32;
    let message = msg(&env, b"handover: PROD-7 alice->bob nonce=42");
    let (ring, secret) = build_ring(&env, n, signer);
    let sig = sign(&env, &ring, signer, &secret, &message);
    assert_eq!(verify_ring_signature(&env, &ring, &message, &sig), Ok(()));

    std::eprintln!("--- BEGIN RING SIGNATURE TEST VECTOR ---");
    std::eprintln!("message_utf8=handover: PROD-7 alice->bob nonce=42");
    for i in 0..n {
        std::eprintln!("ring[{i}]={}", hex96(&ring.get(i).unwrap()));
    }
    std::eprintln!("c0={}", hex32(&sig.c0));
    for i in 0..n {
        std::eprintln!("s[{i}]={}", hex32(&sig.s.get(i).unwrap()));
    }
    std::eprintln!("ring_commitment={}", hex32(&ring_commitment(&env, &ring)));
    std::eprintln!("--- END RING SIGNATURE TEST VECTOR ---");
}

// ─── Pure-crypto tests ──────────────────────────────────────────────────────

#[test]
fn generator_is_valid() {
    let env = Env::default();
    let bls = env.crypto().bls12_381();
    let g = generator(&env);
    // Subgroup membership implies on-curve, validating the generator constant.
    assert!(
        bls.g1_is_in_subgroup(&g),
        "hardcoded G1 generator must be in the prime-order subgroup"
    );
}

#[test]
fn sign_and_verify_round_trip_all_indices() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let n = 5u32;
    let message = msg(&env, b"handover: product-42 from alice to bob");
    for signer in 0..n {
        let (ring, secret) = build_ring(&env, n, signer);
        let sig = sign(&env, &ring, signer, &secret, &message);
        assert_eq!(
            verify_ring_signature(&env, &ring, &message, &sig),
            Ok(()),
            "valid signature from index {signer} must verify"
        );
    }
}

#[test]
fn verify_various_ring_sizes() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let message = msg(&env, b"audit attestation");
    for &n in &[2u32, 3, 4, 8, 16] {
        let signer = n / 2;
        let (ring, secret) = build_ring(&env, n, signer);
        let sig = sign(&env, &ring, signer, &secret, &message);
        assert_eq!(verify_ring_signature(&env, &ring, &message, &sig), Ok(()));
    }
}

#[test]
fn wrong_message_is_rejected() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let (ring, secret) = build_ring(&env, 4, 1);
    let sig = sign(&env, &ring, 1, &secret, &msg(&env, b"original statement"));
    assert_eq!(
        verify_ring_signature(&env, &ring, &msg(&env, b"tampered statement"), &sig),
        Err(Error::RingSignatureInvalid)
    );
}

#[test]
fn tampered_response_is_rejected() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let message = msg(&env, b"statement");
    let (ring, secret) = build_ring(&env, 4, 2);
    let mut sig = sign(&env, &ring, 2, &secret, &message);
    // Flip one response scalar.
    sig.s.set(0, BytesN::from_array(&env, &[7u8; 32]));
    assert_eq!(
        verify_ring_signature(&env, &ring, &message, &sig),
        Err(Error::RingSignatureInvalid)
    );
}

#[test]
fn signature_does_not_verify_against_a_different_ring() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let message = msg(&env, b"statement");
    let (ring, secret) = build_ring(&env, 4, 0);
    let sig = sign(&env, &ring, 0, &secret, &message);

    // Replace one decoy member with an unrelated key: the ring commitment
    // changes, so the bound signature must fail.
    let mut other = ring.clone();
    let (_x, p) = keypair(&env, 999);
    other.set(3, p);
    assert_eq!(
        verify_ring_signature(&env, &other, &message, &sig),
        Err(Error::RingSignatureInvalid)
    );
}

#[test]
fn non_member_cannot_forge() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let message = msg(&env, b"statement");
    // Ring of members 0..4; the attacker holds key 50, which is NOT in the ring.
    let (ring, _secret) = build_ring(&env, 4, 0);
    let (outsider_sk, outsider_pk) = keypair(&env, 50);
    assert!(!ring.contains(&outsider_pk));
    // Attacker signs as if they were index 0 with their own (wrong) secret.
    let forged = sign(&env, &ring, 0, &outsider_sk, &message);
    assert_eq!(
        verify_ring_signature(&env, &ring, &message, &forged),
        Err(Error::RingSignatureInvalid)
    );
}

#[test]
fn ring_too_small_is_rejected() {
    let env = Env::default();
    let (ring, secret) = build_ring(&env, 1, 0);
    let sig = sign(&env, &ring, 0, &secret, &msg(&env, b"x"));
    assert_eq!(
        verify_ring_signature(&env, &ring, &msg(&env, b"x"), &sig),
        Err(Error::RingTooSmall)
    );
}

#[test]
fn ring_size_mismatch_is_rejected() {
    let env = Env::default();
    let (ring, secret) = build_ring(&env, 3, 0);
    let mut sig = sign(&env, &ring, 0, &secret, &msg(&env, b"x"));
    sig.s.pop_back(); // now len 2 != ring len 3
    assert_eq!(
        verify_ring_signature(&env, &ring, &msg(&env, b"x"), &sig),
        Err(Error::RingSizeMismatch)
    );
}

#[test]
fn duplicate_member_is_rejected() {
    let env = Env::default();
    let (ring, secret) = build_ring(&env, 3, 0);
    let mut dup = ring.clone();
    dup.set(2, ring.get(0).unwrap()); // member 0 appears twice
    let sig = sign(&env, &ring, 0, &secret, &msg(&env, b"x"));
    assert_eq!(
        verify_ring_signature(&env, &dup, &msg(&env, b"x"), &sig),
        Err(Error::DuplicateRingMember)
    );
}

#[test]
fn aggregate_ring_is_deterministic_and_order_sensitive() {
    let env = Env::default();
    let (ring, _) = build_ring(&env, 4, 0);
    let c1 = ring_commitment(&env, &ring);
    let c2 = ring_commitment(&env, &ring);
    assert_eq!(c1, c2, "commitment must be deterministic");

    let mut reordered = Vec::new(&env);
    reordered.push_back(ring.get(1).unwrap());
    reordered.push_back(ring.get(0).unwrap());
    reordered.push_back(ring.get(2).unwrap());
    reordered.push_back(ring.get(3).unwrap());
    assert_ne!(
        c1,
        ring_commitment(&env, &reordered),
        "commitment must depend on member order"
    );
}

// ─── Verifier contract tests ────────────────────────────────────────────────

#[test]
fn verifier_contract_accepts_valid_and_rejects_invalid() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let id = env.register_contract(None, RingSignatureVerifier);
    let client = RingSignatureVerifierClient::new(&env, &id);

    let message = msg(&env, b"handover statement");
    let (ring, secret) = build_ring(&env, 6, 3);
    let sig = sign(&env, &ring, 3, &secret, &message);

    assert!(client.verify(&ring, &message, &sig));
    assert!(!client.verify(&ring, &msg(&env, b"other"), &sig));
    assert_eq!(client.aggregate_ring(&ring), ring_commitment(&env, &ring));
    assert_eq!(client.generator(), BytesN::from_array(&env, &G1_GENERATOR));
}

// ─── Audit-trail contract tests ─────────────────────────────────────────────

fn setup_audit(
    env: &Env,
) -> (
    AuditTrailContractClient<'static>,
    ChainLogisticsContractClient<'static>,
    Address,
) {
    env.mock_all_auths();
    let cl_id = env.register_contract(None, ChainLogisticsContract);
    let at_id = env.register_contract(None, AuditTrailContract);
    let cl = ChainLogisticsContractClient::new(env, &cl_id);
    let at = AuditTrailContractClient::new(env, &at_id);

    let admin = Address::generate(env);
    let auth_contract = Address::generate(env);
    cl.init(&admin, &auth_contract);
    at.init(&cl_id);
    (at, cl, admin)
}

#[test]
fn record_handover_stores_anonymous_record() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let (at, _cl, _admin) = setup_audit(&env);

    let product_id = String::from_str(&env, "PROD-001");
    let statement = msg(&env, b"custody: warehouse-A -> truck-7");
    let (ring, secret) = build_ring(&env, 5, 2);
    let sig = sign(&env, &ring, 2, &secret, &statement);

    let record_id = at.record_handover(&product_id, &ring, &statement, &sig);
    assert_eq!(record_id, 1);
    assert_eq!(at.total_handovers(), 1);

    let record = at.get_handover(&record_id).unwrap();
    assert_eq!(record.product_id, product_id);
    assert_eq!(record.ring_size, 5);
    assert_eq!(record.ring_commitment, ring_commitment(&env, &ring));
    assert_eq!(record.statement_hash, env.crypto().sha256(&statement).to_bytes());

    let ids = at.product_handovers(&product_id);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), 1);
}

#[test]
fn record_handover_rejects_invalid_signature() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let (at, _cl, _admin) = setup_audit(&env);

    let product_id = String::from_str(&env, "PROD-002");
    let statement = msg(&env, b"custody change");
    let (ring, secret) = build_ring(&env, 4, 1);
    let mut sig = sign(&env, &ring, 1, &secret, &statement);
    sig.c0 = BytesN::from_array(&env, &[1u8; 32]); // corrupt seed challenge

    let res = at.try_record_handover(&product_id, &ring, &statement, &sig);
    assert_eq!(res, Err(Ok(Error::RingSignatureInvalid)));
    assert_eq!(at.total_handovers(), 0);
}

#[test]
fn record_handover_respects_global_pause() {
    let env = Env::default();
    env.budget().reset_unlimited();
    let (at, cl, admin) = setup_audit(&env);

    // Pause the main contract; admin auth is mocked.
    cl.pause(&admin);

    let product_id = String::from_str(&env, "PROD-003");
    let statement = msg(&env, b"statement");
    let (ring, secret) = build_ring(&env, 3, 0);
    let sig = sign(&env, &ring, 0, &secret, &statement);

    let res = at.try_record_handover(&product_id, &ring, &statement, &sig);
    assert_eq!(res, Err(Ok(Error::ContractPaused)));
}

// ─── Gas / cost documentation ───────────────────────────────────────────────

// Prints the per-ring-size cost table in RING_SIGNATURE.md. Measured under an
// unlimited budget because native estimates underestimate the WASM equivalent,
// so the real per-tx limit is validated on testnet, not here.
// Run: `cargo test ring_signature -- --nocapture`.
#[test]
fn gas_costs_across_ring_sizes() {
    let env = Env::default();
    let id = env.register_contract(None, RingSignatureVerifier);
    let client = RingSignatureVerifierClient::new(&env, &id);
    let message = msg(&env, b"handover statement");

    for &n in &[2u32, 4, 8, 16, 32] {
        let signer = n / 2;
        let (ring, secret) = build_ring(&env, n, signer);
        let sig = sign(&env, &ring, signer, &secret, &message);

        env.budget().reset_unlimited();
        let ok = client.verify(&ring, &message, &sig);
        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();
        assert!(ok);
        // Loose ceiling to catch a super-linear regression.
        assert!(cpu < 12_000_000 * (n as u64), "ring size {n}: cpu {cpu}");
        std::eprintln!("ring_size={n} cpu_insns={cpu} mem_bytes={mem}");
    }
}

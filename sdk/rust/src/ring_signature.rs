//! AOS/SAG ring signatures over BLS12-381 G1 — the off-chain counterpart to
//! the `RingSignatureVerifier` / `AuditTrailContract` Soroban contracts. An
//! auditor signs a statement as an anonymous member of a ring; the signature
//! verifies on-chain unchanged (cross-checked against a contract-produced
//! vector in [`tests`]).
//!
//! Wire format (must match the contract): a public key is the uncompressed G1
//! encoding (`x ‖ y`, 48 bytes each, big-endian) = `[u8; 96]`; each scalar
//! (`c0`, `s_i`) is a 32-byte big-endian canonical `F_r` element; the challenge
//! reduces `SHA-256(...)` as a big-endian integer mod `r`.
//!
//! ```
//! use chainlogistics_sdk::ring_signature::{KeyPair, sign, verify};
//! use rand_core::OsRng;
//!
//! let auditors: Vec<KeyPair> = (0..3).map(|_| KeyPair::generate(OsRng)).collect();
//! let ring: Vec<[u8; 96]> = auditors.iter().map(|k| k.public_key()).collect();
//!
//! let msg = b"custody: warehouse-A -> truck-7";
//! let sig = sign(&ring, 1, &auditors[1], msg, OsRng).unwrap();
//! assert!(verify(&ring, msg, &sig));
//! assert!(!verify(&ring, b"different message", &sig));
//! ```

use bls12_381::{G1Affine, G1Projective, Scalar};
use rand_core::RngCore;
use sha2::{Digest, Sha256};

// Must match the Soroban contract byte-for-byte.
const DST_CHALLENGE: &[u8] = b"CHAINLOGISTICS-RINGSIG-V1-CHALLENGE";
const DST_RING: &[u8] = b"CHAINLOGISTICS-RINGSIG-V1-RING";

pub const MIN_RING_SIZE: usize = 2;
pub const MAX_RING_SIZE: usize = 32;

/// Errors returned by signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingError {
    RingTooSmall,
    RingTooLarge,
    SignerIndexOutOfRange,
    InvalidRingMember,
    SecretKeyMismatch,
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RingError::RingTooSmall => "ring smaller than the minimum anonymity set",
            RingError::RingTooLarge => "ring larger than the maximum supported size",
            RingError::SignerIndexOutOfRange => "signer index is outside the ring",
            RingError::InvalidRingMember => "ring contains an invalid G1 point",
            RingError::SecretKeyMismatch => "secret key does not match the ring member",
        };
        f.write_str(s)
    }
}

impl std::error::Error for RingError {}

/// A ring signature in the contract's wire format (`c0` + one `s` per member).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RingSignature {
    pub c0: [u8; 32],
    pub s: Vec<[u8; 32]>,
}

/// An auditor keypair. `secret` is the BLS12-381 scalar private key.
#[derive(Debug, Clone)]
pub struct KeyPair {
    secret: Scalar,
}

impl KeyPair {
    pub fn generate<R: RngCore>(mut rng: R) -> Self {
        let mut wide = [0u8; 64];
        rng.fill_bytes(&mut wide);
        KeyPair {
            secret: Scalar::from_bytes_wide(&wide),
        }
    }

    /// `None` if `bytes` is not a canonical scalar (`>= r`).
    pub fn from_secret_be_bytes(bytes: &[u8; 32]) -> Option<Self> {
        scalar_from_be_canonical(bytes).map(|secret| KeyPair { secret })
    }

    pub fn secret_be_bytes(&self) -> [u8; 32] {
        scalar_to_be(&self.secret)
    }

    pub fn public_key(&self) -> [u8; 96] {
        G1Affine::from(G1Projective::generator() * self.secret).to_uncompressed()
    }
}

// ─── Encoding helpers (big-endian wire <-> zkcrypto little-endian) ──────────

// Big-endian digest reduced mod r, mirroring the contract's `Fr::from_bytes`.
fn scalar_from_be_reduce(be: &[u8; 32]) -> Scalar {
    let mut le_wide = [0u8; 64];
    for i in 0..32 {
        le_wide[i] = be[31 - i];
    }
    Scalar::from_bytes_wide(&le_wide)
}

fn scalar_from_be_canonical(be: &[u8; 32]) -> Option<Scalar> {
    let mut le = [0u8; 32];
    for i in 0..32 {
        le[i] = be[31 - i];
    }
    Option::from(Scalar::from_bytes(&le))
}

fn scalar_to_be(s: &Scalar) -> [u8; 32] {
    let le = s.to_bytes();
    let mut be = [0u8; 32];
    for i in 0..32 {
        be[i] = le[31 - i];
    }
    be
}

fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// Ring commitment (public-key aggregation); mirrors the contract.
pub fn aggregate_ring(ring: &[[u8; 96]]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(DST_RING);
    h.update((ring.len() as u32).to_be_bytes());
    for member in ring {
        h.update(member);
    }
    h.finalize().into()
}

fn challenge(commit: &[u8; 32], msg_digest: &[u8; 32], l_uncompressed: &[u8; 96]) -> Scalar {
    let digest = sha256(&[DST_CHALLENGE, commit, msg_digest, l_uncompressed]);
    scalar_from_be_reduce(&digest)
}

fn parse_member(bytes: &[u8; 96]) -> Option<G1Affine> {
    // from_uncompressed enforces on-curve + prime-subgroup, matching the
    // contract's g1_is_in_subgroup.
    Option::from(G1Affine::from_uncompressed(bytes))
}

// ─── Verify ─────────────────────────────────────────────────────────────────

/// `true` iff `sig` was produced by a holder of one of the `ring` keys over
/// `message`. Mirrors the on-chain `verify`.
pub fn verify(ring: &[[u8; 96]], message: &[u8], sig: &RingSignature) -> bool {
    let n = ring.len();
    if n < MIN_RING_SIZE || n > MAX_RING_SIZE || sig.s.len() != n {
        return false;
    }

    let mut members = Vec::with_capacity(n);
    for member in ring {
        match parse_member(member) {
            Some(p) => members.push(G1Projective::from(p)),
            None => return false,
        }
        // Reject duplicates (shrinks the effective anonymity set).
        for j in 0..members.len() - 1 {
            if ring[j] == *member {
                return false;
            }
        }
    }

    let g = G1Projective::generator();
    let commit = aggregate_ring(ring);
    let msg_digest = sha256(&[message]);

    let c0 = scalar_from_be_reduce(&sig.c0);
    let mut c = c0;
    for i in 0..n {
        let si = scalar_from_be_reduce(&sig.s[i]);
        // L_i = s_i · G + c · P_i
        let l = g * si + members[i] * c;
        let l_bytes = G1Affine::from(l).to_uncompressed();
        c = challenge(&commit, &msg_digest, &l_bytes);
    }

    c == c0
}

// ─── Sign ─────────────────────────────────────────────────────────────────

/// Sign `message` as ring member `signer_index`. `rng` must be a CSPRNG.
pub fn sign<R: RngCore>(
    ring: &[[u8; 96]],
    signer_index: usize,
    signer: &KeyPair,
    message: &[u8],
    mut rng: R,
) -> Result<RingSignature, RingError> {
    let n = ring.len();
    if n < MIN_RING_SIZE {
        return Err(RingError::RingTooSmall);
    }
    if n > MAX_RING_SIZE {
        return Err(RingError::RingTooLarge);
    }
    if signer_index >= n {
        return Err(RingError::SignerIndexOutOfRange);
    }
    if signer.public_key() != ring[signer_index] {
        return Err(RingError::SecretKeyMismatch);
    }

    let mut members = Vec::with_capacity(n);
    for member in ring {
        members.push(G1Projective::from(
            parse_member(member).ok_or(RingError::InvalidRingMember)?,
        ));
    }

    let g = G1Projective::generator();
    let commit = aggregate_ring(ring);
    let msg_digest = sha256(&[message]);

    let mut c = vec![Scalar::from(0u64); n];
    let mut s = vec![Scalar::from(0u64); n];

    let random_scalar = |rng: &mut R| {
        let mut wide = [0u8; 64];
        rng.fill_bytes(&mut wide);
        Scalar::from_bytes_wide(&wide)
    };

    // Seed the chain at the signer's index with a random nonce α.
    let alpha = random_scalar(&mut rng);
    let l_pi = G1Affine::from(g * alpha).to_uncompressed();
    c[(signer_index + 1) % n] = challenge(&commit, &msg_digest, &l_pi);

    // Walk the decoys with random responses.
    for j in 1..n {
        let idx = (signer_index + j) % n;
        let s_idx = random_scalar(&mut rng);
        s[idx] = s_idx;
        let l = g * s_idx + members[idx] * c[idx];
        let l_bytes = G1Affine::from(l).to_uncompressed();
        c[(idx + 1) % n] = challenge(&commit, &msg_digest, &l_bytes);
    }

    // Close the ring: s_π = α − c_π · x.
    s[signer_index] = alpha - c[signer_index] * signer.secret;

    Ok(RingSignature {
        c0: scalar_to_be(&c[0]),
        s: s.iter().map(scalar_to_be).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    // The contract's hardcoded `G1_GENERATOR`.
    const G1_GENERATOR: [u8; 96] = [
        0x17, 0xf1, 0xd3, 0xa7, 0x31, 0x97, 0xd7, 0x94, 0x26, 0x95, 0x63, 0x8c, 0x4f, 0xa9, 0xac,
        0x0f, 0xc3, 0x68, 0x8c, 0x4f, 0x97, 0x74, 0xb9, 0x05, 0xa1, 0x4e, 0x3a, 0x3f, 0x17, 0x1b,
        0xac, 0x58, 0x6c, 0x55, 0xe8, 0x3f, 0xf9, 0x7a, 0x1a, 0xef, 0xfb, 0x3a, 0xf0, 0x0a, 0xdb,
        0x22, 0xc6, 0xbb, 0x08, 0xb3, 0xf4, 0x81, 0xe3, 0xaa, 0xa0, 0xf1, 0xa0, 0x9e, 0x30, 0xed,
        0x74, 0x1d, 0x8a, 0xe4, 0xfc, 0xf5, 0xe0, 0x95, 0xd5, 0xd0, 0x0a, 0xf6, 0x00, 0xdb, 0x18,
        0xcb, 0x2c, 0x04, 0xb3, 0xed, 0xd0, 0x3c, 0xc7, 0x44, 0xa2, 0x88, 0x8a, 0xe4, 0x0c, 0xaa,
        0x23, 0x29, 0x46, 0xc5, 0xe7, 0xe1,
    ];

    fn hex_to_array<const N: usize>(s: &str) -> [u8; N] {
        let mut out = [0u8; N];
        for i in 0..N {
            out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
        }
        out
    }

    fn rng(seed: u64) -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(seed)
    }

    #[test]
    fn generator_matches_contract_constant() {
        // Interop requires both sides to use the same fixed generator.
        assert_eq!(G1Affine::generator().to_uncompressed(), G1_GENERATOR);
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let mut r = rng(1);
        let keys: Vec<KeyPair> = (0..5).map(|_| KeyPair::generate(&mut r)).collect();
        let ring: Vec<[u8; 96]> = keys.iter().map(|k| k.public_key()).collect();
        let msg = b"audit attestation";
        for signer in 0..ring.len() {
            let sig = sign(&ring, signer, &keys[signer], msg, rng(100 + signer as u64)).unwrap();
            assert!(verify(&ring, msg, &sig), "signer {signer} must verify");
        }
    }

    #[test]
    fn wrong_message_fails() {
        let mut r = rng(2);
        let keys: Vec<KeyPair> = (0..3).map(|_| KeyPair::generate(&mut r)).collect();
        let ring: Vec<[u8; 96]> = keys.iter().map(|k| k.public_key()).collect();
        let sig = sign(&ring, 0, &keys[0], b"original", rng(7)).unwrap();
        assert!(!verify(&ring, b"tampered", &sig));
    }

    #[test]
    fn anonymity_set_must_match() {
        let mut r = rng(3);
        let keys: Vec<KeyPair> = (0..3).map(|_| KeyPair::generate(&mut r)).collect();
        let mut ring: Vec<[u8; 96]> = keys.iter().map(|k| k.public_key()).collect();
        let sig = sign(&ring, 0, &keys[0], b"x", rng(8)).unwrap();
        // Swap a member -> ring commitment changes -> verification fails.
        ring[2] = KeyPair::generate(&mut r).public_key();
        assert!(!verify(&ring, b"x", &sig));
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let mut r = rng(4);
        let keys: Vec<KeyPair> = (0..3).map(|_| KeyPair::generate(&mut r)).collect();
        let ring: Vec<[u8; 96]> = keys.iter().map(|k| k.public_key()).collect();
        // keys[1] tries to sign claiming to be index 0.
        let err = sign(&ring, 0, &keys[1], b"x", rng(9)).unwrap_err();
        assert_eq!(err, RingError::SecretKeyMismatch);
    }

    #[test]
    fn ring_too_small_is_rejected() {
        let k = KeyPair::generate(rng(5));
        let ring = vec![k.public_key()];
        assert_eq!(
            sign(&ring, 0, &k, b"x", rng(5)).unwrap_err(),
            RingError::RingTooSmall
        );
    }

    // Known-answer vector from the contract's `emit_sdk_test_vector`. Accepting
    // it proves byte-for-byte agreement with the on-chain verifier.
    #[test]
    fn verifies_contract_test_vector() {
        let ring = vec![
            hex_to_array::<96>("102b6a1c88da96b327e995c2159fb4f88070cd144de9e1f0a7aaa2dd37b3bb2b643a7dcfcdab05352d0156ffec6070d8054b2cef273b023043e72ed27862dd2473202e84cf1365128dafd26ba683b24fa7b527d2242d285cae0a77cbb0d9f396"),
            hex_to_array::<96>("15f5d598f843ec0b0a4d2368f516ead2e877ba2300148c10a56296de419cee64de75225a023341475bb67eb260f1edf20b8f39782375c2c7f0f2b1b975e9611f84497ff5920dd56aa3907e8a6ef1653af2f2bfdec459770fef0d799bd2d8cb31"),
            hex_to_array::<96>("04ff5071f60786edbd7f589e91c5c9ab0d7d0066b00dfbdf35520f1d50c0b1f94e30a5c4abd093af78c762b7ab9709171297418166538f09f4cb6b89d46f6cc5c5e516234b99966cd092a8e34456db97b3fda3e1031c53ad159703f4f85ab514"),
        ];
        let message = b"handover: PROD-7 alice->bob nonce=42";
        let sig = RingSignature {
            c0: hex_to_array::<32>("3a56f0800412258de421b9146df64ee8db80385159874874b6e547b6351aef5f"),
            s: vec![
                hex_to_array::<32>("3500922125d31fe353ae32a2feed0e71dbf5b17df41348cae610abdd040dc442"),
                hex_to_array::<32>("1a6b9d69a28c59dbbf06001b3a7c0f96db375e65aab99db4d8c5ac885b908adb"),
                hex_to_array::<32>("3fb8309bbe0e15d5dc1c5ad87ab8e74647e660e19d4d8e99d3a1434ee9f3b0b1"),
            ],
        };

        assert!(verify(&ring, message, &sig), "must accept contract vector");
        assert!(!verify(&ring, b"wrong", &sig));

        // Aggregation must also match the contract's commitment.
        assert_eq!(
            aggregate_ring(&ring),
            hex_to_array::<32>("0bbfd715e9206cdf3e965c10b5a1b7f57a099a7b72e105151852e51cfbe7fc80")
        );
    }
}

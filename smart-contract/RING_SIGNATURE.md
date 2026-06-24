# Privacy-Preserving Audit Trail with Ring Signatures

> Resolves issue #413. Depends on the BLS12-381 host functions (CAP-0059)
> already used by the Groth16 `ComplianceVerifier`.

Auditors of a supply chain must be able to attest to product handovers (custody
changes, inspections, sign-offs) **without revealing which specific auditor
signed**. A named signature exposes the auditor to bribery, coercion or
retaliation. A *ring signature* solves this: a member of an *anonymity set* (the
*ring*) produces a signature that anyone can verify came from **one of** the
ring members, but no one — not even the verifier or the relayer that submits the
transaction — can tell **which** one.

This document specifies the scheme, the on-chain/off-chain APIs, the measured
gas costs, and answers the design questions raised in the issue (gas
feasibility, linkability, and short ring signatures).

---

## 1. Scheme

We use an **AOS / SAG ring signature** (Abe–Ohkubo–Suzuki / Spontaneous
Anonymous Group) over the **BLS12-381 G1** prime-order group. It needs only host
functions Soroban exposes today — `g1_mul`, `g1_add`, `g1_is_in_subgroup`,
`sha256` — and **no pairings**, so verification is far cheaper than a SNARK.

Let `G` be the standard BLS12-381 G1 generator and `r` the scalar field order.
Each auditor has a secret `x_i ∈ F_r` and public key `P_i = x_i · G`. For a ring
`P_0 … P_{n-1}` and message `m`, a signature is `(c_0, [s_0 … s_{n-1}])` where
every `s_i` and `c_0` is a scalar in `F_r`.

### Verification

```
commit = SHA256( DST_RING ‖ n(be32) ‖ P_0 ‖ … ‖ P_{n-1} )      // ring commitment
md     = SHA256(m)
c      = c_0
for i in 0..n:
    L_i = s_i · G + c · P_i
    c   = SHA256( DST_CHALLENGE ‖ commit ‖ md ‖ L_i ) mod r
accept iff c == c_0
```

### Signing (performed by the SDKs)

The signer knows one secret `x_π`. It picks a uniformly random nonce `α`, seeds
the challenge chain at its own index with `L_π = α · G`, fills the other
positions with uniformly random responses `s_i`, then **closes the ring** with

```
s_π = α − c_π · x_π   (mod r)
```

Anonymity follows from `α` and the decoy `s_i` being uniformly random: given a
valid signature, every index is equally likely to be the true signer.

### Domain separation & parameters

| Parameter        | Value                                                      |
|------------------|------------------------------------------------------------|
| Curve / group    | BLS12-381, G1 (prime order `r`)                            |
| Generator `G`    | Standard G1 generator (uncompressed, hardcoded + validated)|
| `DST_CHALLENGE`  | `CHAINLOGISTICS-RINGSIG-V1-CHALLENGE`                      |
| `DST_RING`       | `CHAINLOGISTICS-RINGSIG-V1-RING`                           |
| Hash-to-scalar   | `int.from_bytes(SHA256(...), "big") mod r`                |
| Min / max ring   | 2 / 32                                                      |

### Wire format

* **Public key** — uncompressed G1 (`x ‖ y`, 48 bytes each, big-endian) = 96 bytes.
* **Scalar** (`c_0`, each `s_i`) — 32-byte **big-endian** canonical `F_r` element.

These match `soroban_sdk` `Bls12381G1Affine` and `Fr` serialization exactly,
which is what lets a signature cross the off-chain/on-chain boundary unchanged.

---

## 2. Public-key aggregation

`aggregate_ring(ring)` derives the canonical, **order- and size-binding** ring
commitment `commit = SHA256(DST_RING ‖ n ‖ P_0 ‖ … ‖ P_{n-1})`. It serves two
purposes:

1. A stable identifier for an anonymity set (stored with each audit record).
2. It is folded into every challenge, cryptographically binding a signature to
   *exactly* the ring it was produced for. Members cannot be added, removed or
   reordered after the fact without invalidating the signature.

---

## 3. APIs

### On-chain (Soroban — `contracts/src/ring_signature.rs`)

`RingSignatureVerifier` (stateless verification gateway):

| Method | Description |
|--------|-------------|
| `verify(ring, message, sig) -> bool` | `true` iff `sig` is valid for the ring. |
| `verify_strict(ring, message, sig) -> Result<(), Error>` | Same, but returns the precise rejection reason. |
| `aggregate_ring(ring) -> BytesN<32>` | Ring commitment (public-key aggregation). |
| `generator() -> BytesN<96>` | The fixed generator, so clients can assert parameter agreement. |

`AuditTrailContract` (stateful, records anonymous handovers):

| Method | Description |
|--------|-------------|
| `init(main_contract)` | Wire in the main contract (for the global pause). |
| `record_handover(product_id, ring, statement, sig) -> u64` | Verify a ring signature and store an **anonymous** `HandoverRecord` + emit `PrivacyHandoverRecorded`. Returns the record id. |
| `get_handover(record_id)` | Fetch a record. |
| `product_handovers(product_id)` | Record ids for a product. |
| `total_handovers()` | Global count. |

A `HandoverRecord` stores only `{ record_id, product_id, ring_commitment,
ring_size, statement_hash, timestamp }` — **never a signer identity**. The
submitting account is not recorded either, so a relayer learns nothing about the
signer beyond ring membership.

### Off-chain (Rust SDK — `sdk/rust`, feature `ring-signatures`)

```rust
use chainlogistics_sdk::ring_signature::{KeyPair, sign, verify};
use rand_core::OsRng;

let auditors: Vec<KeyPair> = (0..8).map(|_| KeyPair::generate(OsRng)).collect();
let ring: Vec<[u8; 96]> = auditors.iter().map(|k| k.public_key()).collect();

let msg = b"custody: warehouse-A -> truck-7";
let sig = sign(&ring, 3, &auditors[3], msg, OsRng)?;   // auditor #3 signs anonymously
assert!(verify(&ring, msg, &sig));
```

### Off-chain (Python SDK — `sdk/python`, no extra dependencies)

```python
from chainlogistics_sdk.ring_signature import KeyPair, sign, verify

auditors = [KeyPair.generate() for _ in range(8)]
ring = [k.public_key() for k in auditors]

msg = b"custody: warehouse-A -> truck-7"
sig = sign(ring, 3, auditors[3], msg)
assert verify(ring, msg, sig)
```

`sig.c0` / `sig.s` (and `KeyPair.public_key()`) are exactly the bytes the
contract's `record_handover` expects.

---

## 4. Design question — on-chain gas feasibility

Verification is `~2` G1 scalar-multiplications + `1` point-addition + `1`
SHA-256 **per ring member**. Measured with the soroban test host
(`gas_costs_across_ring_sizes`, run `cargo test ring_signature -- --nocapture`):

| Ring size | CPU instructions | Memory (bytes) |
|----------:|-----------------:|---------------:|
| 2         | 14,978,928       | 12,245         |
| 4         | 29,910,532       | 17,617         |
| 8         | 59,816,133       | 30,408         |
| 16        | 119,599,717      | 58,808         |
| 32        | 239,391,636      | 136,009        |

Cost is **linear** at ≈ **7.5M CPU instructions / member**.

**Feasibility.** Soroban's per-transaction CPU budget is 100M instructions. The
host documents that *native* estimates **underestimate** the WASM equivalent, so
treat the table as a lower bound. Practical guidance:

* **Recommended on-chain ring size: ≤ 8** (comfortably inside budget with WASM
  headroom). A ring of 8 gives 1-in-8 anonymity per attestation.
* **Hard cap: 32** (`MAX_RING_SIZE`) — enforced so a single call can never be
  pushed past the budget by an oversized ring. Sizes above ~13 will likely
  exceed the budget once compiled to WASM and should be validated on testnet
  first, or use the short-ring-signature path (§6).
* For larger *effective* anonymity sets without per-call cost, rotate rings over
  time or aggregate anonymity across many attestations.

Memory is negligible (≤ 136 KB at the cap).

---

## 5. Design question — linkability requirements

**The V1 wire scheme is intentionally _unlinkable_.** Two attestations from the
same auditor over the same ring cannot be correlated. This maximizes anonymity
and is the correct default for "anonymous attestation": an auditor signing many
handovers leaks no graph linking those handovers to one identity.

When a deployment instead needs **one-attestation-per-auditor** (e.g. anonymous
*voting* on a dispute, where double-voting must be detectable), upgrade to the
**LSAG** (Linkable Spontaneous Anonymous Group) variant:

* Add a **key image** `I = x · H_p(P)`, where `H_p` maps a public key to a G1
  point. Soroban exposes `hash_to_g1(msg, dst)` (CAP-0059), so this is available
  on-chain.
* Add a second challenge term `R_i = s_i · H_p(P_i) + c · I` and fold `R_i` into
  the challenge hash alongside `L_i`.
* `I` is deterministic in the signer's key, so the contract can reject a second
  attestation carrying a previously-seen `I` (the `KeyImageAlreadyUsed` error and
  the key-image storage hooks are reserved for this).

LSAG keeps signer anonymity *within* the ring while making repeat-signing
detectable. The trade-off: it requires both SDKs to reproduce the RFC 9380
`BLS12381G1_XMD:SHA-256_SSWU_RO_` hash-to-curve **bit-for-bit** with the
contract's DST; that interop must be pinned with cross-implementation test
vectors (as done for V1) before enabling it. It is therefore staged as a V2
rather than shipped on by default.

---

## 6. Optimization idea — short ring signatures

The V1 signature size is `O(n)` (one 32-byte scalar per member, plus `c_0`).
Options to shrink the ledger footprint / raise the practical ring size:

1. **bLSAG / compact AOS** — already minimal in field elements; the main lever
   is encoding (e.g. drop to compressed 48-byte G1 keys on the wire and
   decompress on-chain, halving the public-key payload).
2. **Accumulator membership** — commit the ring to a Merkle root or a
   pairing-based accumulator and prove membership with a single Groth16 proof.
   Verification becomes `O(1)` in ring size, reusing the existing
   `ComplianceVerifier` pairing path. This is the most promising route to large
   anonymity sets (hundreds of auditors) at constant gas.
3. **Back-reference rings** — store the ring once (keyed by its commitment) and
   have later attestations reference the commitment instead of re-sending all
   public keys, eliminating the dominant on-chain data cost for repeated rings.

(2) is the recommended long-term direction; (1) and (3) are incremental wins
that need no new cryptography.

---

## 7. Security notes

* **Subgroup checks.** Every ring member is checked with `g1_is_in_subgroup`
  (which also validates on-curve), rejecting small-subgroup / invalid-curve
  forgeries (`InvalidRingMember`).
* **Duplicate members** are rejected (`DuplicateRingMember`) — they would shrink
  the effective anonymity set.
* **Anti-replay.** The statement `m` is opaque and should bind the action
  context (e.g. `product_id ‖ from ‖ to ‖ nonce`) so a valid attestation cannot
  be replayed for a different handover. The verifier is otherwise stateless.
* **Generator integrity.** The hardcoded generator is validated against the host
  (`generator_is_valid` test) and against both SDKs' independent generators
  (`generator_matches_contract_constant`, `test_generator_*`).
* **RNG.** Signing requires a CSPRNG for `α` and the decoy responses. The Rust
  SDK takes any `RngCore` (use `OsRng`); the Python SDK uses `secrets`. A
  predictable nonce leaks the secret key, exactly as in Schnorr/EdDSA.
* The pure-Python G1 arithmetic is **not constant-time** — acceptable for
  attestation signing, but do not repurpose it for high-frequency secret-key
  operations under a timing adversary.

---

## 8. Cross-implementation verification

All three implementations are proven to agree on a single **known-answer test
vector** emitted by the trusted on-chain primitive
(`emit_sdk_test_vector` in the contract test suite):

| Implementation | Test | Result |
|----------------|------|--------|
| Soroban contract | `cargo test -p chainlogistics ring_signature` | 16 passing |
| Rust SDK | `cargo test --features ring-signatures ring_signature` (incl. `verifies_contract_test_vector`, `generator_matches_contract_constant`) | 7 passing |
| Python SDK | `pytest sdk/python/tests/test_ring_signature.py` (incl. `test_verifies_contract_test_vector`) | passing |

Because the vector is produced on-chain and accepted by both SDKs' independent
verifiers (zkcrypto `bls12_381` in Rust; pure-Python G1 in Python), the
serialization and scalar reduction are guaranteed identical across all three —
i.e. **a signature produced by either SDK verifies on-chain unchanged.**

---

## 9. Deployment

`AuditTrailContract` and `RingSignatureVerifier` follow the same modular pattern
as `state_channel` / `oracle`: compiled host-side for the full test suite and
gated out of the main `ChainLogisticsContract` WASM artifact to avoid export
symbol collisions. To deploy either as its own contract, build it as a dedicated
WASM target and `init` the audit trail with the main contract address.

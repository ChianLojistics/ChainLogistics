"""AOS/SAG ring signatures over BLS12-381 G1 — the off-chain counterpart to the
``RingSignatureVerifier`` / ``AuditTrailContract`` Soroban contracts. An auditor
signs a statement as an anonymous member of a ring; the signature verifies
on-chain unchanged (cross-checked against a contract-produced vector in the
tests).

The G1 arithmetic (no pairings) is implemented here over the standard library
so the SDK stays dependency-free. It is **not** constant-time — fine for signing
attestations, but do not reuse it for high-frequency secret-key operations under
a timing adversary.

Wire format (must match the contract): public key = uncompressed G1 ``x ‖ y``
(48 bytes each, big-endian) = 96 bytes; each scalar (``c0``, ``s_i``) = 32-byte
big-endian canonical ``F_r``; challenge = ``SHA-256(...)`` as a big-endian
integer mod ``r``.

Example::

    from chainlogistics_sdk.ring_signature import KeyPair, sign, verify

    auditors = [KeyPair.generate() for _ in range(3)]
    ring = [k.public_key() for k in auditors]
    sig = sign(ring, 1, auditors[1], b"custody: warehouse-A -> truck-7")
    assert verify(ring, b"custody: warehouse-A -> truck-7", sig)
"""

from __future__ import annotations

import hashlib
import secrets
from dataclasses import dataclass
from typing import List, Optional, Tuple

# ─── BLS12-381 parameters ───────────────────────────────────────────────────

# Base field modulus p.
P = 0x1A0111EA397FE69A4B1BA7B6434BACD764774B84F38512BF6730D2A0F6B0F6241EABFFFEB153FFFFB9FEFFFFFFFFAAAB
# Scalar field order r.
R = 0x73EDA753299D7D483339D80809A1D80553BDA402FFFE5BFEFFFFFFFF00000001
# Curve: y^2 = x^3 + B over F_p.
B = 4
# Standard G1 generator (x, y).
GX = 0x17F1D3A73197D7942695638C4FA9AC0FC3688C4F9774B905A14E3A3F171BAC586C55E83FF97A1AEFFB3AF00ADB22C6BB
GY = 0x08B3F481E3AAA0F1A09E30ED741D8AE4FCF5E095D5D00AF600DB18CB2C04B3EDD03CC744A2888AE40CAA232946C5E7E1

# Must match the Soroban contract byte-for-byte.
_DST_CHALLENGE = b"CHAINLOGISTICS-RINGSIG-V1-CHALLENGE"
_DST_RING = b"CHAINLOGISTICS-RINGSIG-V1-RING"

MIN_RING_SIZE = 2
MAX_RING_SIZE = 32

_FP_BYTES = 48
_PK_BYTES = 96
_SCALAR_BYTES = 32

# None is the point at infinity; otherwise an affine (x, y) tuple.
Point = Optional[Tuple[int, int]]


class RingError(Exception):
    """Raised when signing inputs are invalid."""


# ─── Minimal G1 elliptic-curve arithmetic (affine, over F_p) ────────────────


def _is_on_curve(pt: Point) -> bool:
    if pt is None:
        return True
    x, y = pt
    if not (0 <= x < P and 0 <= y < P):
        return False
    return (y * y - (x * x * x + B)) % P == 0


def _add(p1: Point, p2: Point) -> Point:
    if p1 is None:
        return p2
    if p2 is None:
        return p1
    x1, y1 = p1
    x2, y2 = p2
    if x1 == x2 and (y1 + y2) % P == 0:
        return None  # P + (-P) = O
    if x1 == x2 and y1 == y2:
        # Doubling: lambda = 3x^2 / 2y
        lam = (3 * x1 * x1 % P) * pow(2 * y1 % P, -1, P) % P
    else:
        lam = (y2 - y1) % P * pow((x2 - x1) % P, -1, P) % P
    x3 = (lam * lam - x1 - x2) % P
    y3 = (lam * (x1 - x3) - y1) % P
    return (x3, y3)


def _mul(k: int, pt: Point) -> Point:
    k %= R
    result: Point = None
    addend = pt
    while k:
        if k & 1:
            result = _add(result, addend)
        addend = _add(addend, addend)
        k >>= 1
    return result


_G: Point = (GX, GY)


# ─── Encoding helpers ───────────────────────────────────────────────────────


def _serialize(pt: Point) -> bytes:
    """Uncompressed G1 encoding (``x ‖ y``, 48 bytes each, big-endian)."""
    if pt is None:
        # Never occurs for a valid ring signature.
        raise RingError("cannot serialize the point at infinity")
    x, y = pt
    return x.to_bytes(_FP_BYTES, "big") + y.to_bytes(_FP_BYTES, "big")


def _deserialize(data: bytes) -> Point:
    if len(data) != _PK_BYTES:
        raise RingError("ring member must be 96 bytes")
    x = int.from_bytes(data[:_FP_BYTES], "big")
    y = int.from_bytes(data[_FP_BYTES:], "big")
    pt = (x, y)
    if not _is_on_curve(pt):
        raise RingError("ring member is not on the BLS12-381 curve")
    return pt


# Big-endian integer reduced mod r, mirroring the contract's ``Fr::from_bytes``.
def _scalar_from_be_reduce(digest: bytes) -> int:
    return int.from_bytes(digest, "big") % R


def _scalar_to_be(s: int) -> bytes:
    return (s % R).to_bytes(_SCALAR_BYTES, "big")


def _sha256(*parts: bytes) -> bytes:
    h = hashlib.sha256()
    for p in parts:
        h.update(p)
    return h.digest()


def aggregate_ring(ring: List[bytes]) -> bytes:
    """Ring commitment (public-key aggregation); mirrors the contract."""
    h = hashlib.sha256()
    h.update(_DST_RING)
    h.update(len(ring).to_bytes(4, "big"))
    for member in ring:
        h.update(member)
    return h.digest()


def _challenge(commit: bytes, msg_digest: bytes, l_bytes: bytes) -> int:
    return _scalar_from_be_reduce(_sha256(_DST_CHALLENGE, commit, msg_digest, l_bytes))


# ─── Public types ───────────────────────────────────────────────────────────


@dataclass(frozen=True)
class RingSignature:
    """Wire-format ring signature: ``c0`` plus one ``s`` per member, in order."""

    c0: bytes
    s: List[bytes]


@dataclass(frozen=True)
class KeyPair:
    """An auditor keypair. ``secret`` is the scalar private key (``0 < x < r``)."""

    secret: int

    @classmethod
    def generate(cls) -> "KeyPair":
        return cls(secret=1 + secrets.randbelow(R - 1))

    @classmethod
    def from_secret_be_bytes(cls, data: bytes) -> "KeyPair":
        if len(data) != _SCALAR_BYTES:
            raise RingError("secret key must be 32 bytes")
        x = int.from_bytes(data, "big")
        if not (0 < x < R):
            raise RingError("secret key must be a canonical scalar in [1, r)")
        return cls(secret=x)

    def secret_be_bytes(self) -> bytes:
        return _scalar_to_be(self.secret)

    def public_key(self) -> bytes:
        """Public key ``secret · G``, uncompressed G1 (96 bytes)."""
        return _serialize(_mul(self.secret, _G))


# ─── Verify ─────────────────────────────────────────────────────────────────


def verify(ring: List[bytes], message: bytes, sig: RingSignature) -> bool:
    """``True`` iff ``sig`` is valid for ``ring`` over ``message``. Mirrors the
    on-chain ``verify``; the authoritative subgroup check is done on-chain."""
    n = len(ring)
    if n < MIN_RING_SIZE or n > MAX_RING_SIZE or len(sig.s) != n:
        return False
    if len(set(ring)) != n:  # reject duplicate members
        return False

    try:
        members = [_deserialize(m) for m in ring]
    except RingError:
        return False

    commit = aggregate_ring(ring)
    msg_digest = _sha256(message)

    c0 = _scalar_from_be_reduce(sig.c0)
    c = c0
    for i in range(n):
        si = _scalar_from_be_reduce(sig.s[i])
        # L_i = s_i · G + c · P_i
        l = _add(_mul(si, _G), _mul(c, members[i]))
        try:
            l_bytes = _serialize(l)
        except RingError:
            return False
        c = _challenge(commit, msg_digest, l_bytes)

    return c == c0


# ─── Sign ─────────────────────────────────────────────────────────────────


def sign(
    ring: List[bytes],
    signer_index: int,
    signer: KeyPair,
    message: bytes,
) -> RingSignature:
    """Sign ``message`` as ring member ``signer_index`` (uses a secure RNG)."""
    n = len(ring)
    if n < MIN_RING_SIZE:
        raise RingError("ring smaller than the minimum anonymity set")
    if n > MAX_RING_SIZE:
        raise RingError("ring larger than the maximum supported size")
    if not (0 <= signer_index < n):
        raise RingError("signer index is outside the ring")
    if signer.public_key() != ring[signer_index]:
        raise RingError("secret key does not match the ring member")

    members = [_deserialize(m) for m in ring]
    commit = aggregate_ring(ring)
    msg_digest = _sha256(message)

    c = [0] * n
    s = [0] * n

    def rand_scalar() -> int:
        return secrets.randbelow(R)

    # Seed the chain at the signer's index with a random nonce alpha.
    alpha = rand_scalar()
    l_pi = _serialize(_mul(alpha, _G))
    c[(signer_index + 1) % n] = _challenge(commit, msg_digest, l_pi)

    # Walk the decoys with random responses.
    for j in range(1, n):
        idx = (signer_index + j) % n
        s[idx] = rand_scalar()
        l = _add(_mul(s[idx], _G), _mul(c[idx], members[idx]))
        c[(idx + 1) % n] = _challenge(commit, msg_digest, _serialize(l))

    # Close the ring: s_pi = alpha - c_pi · x  (mod r).
    s[signer_index] = (alpha - c[signer_index] * signer.secret) % R

    return RingSignature(
        c0=_scalar_to_be(c[0]),
        s=[_scalar_to_be(v) for v in s],
    )

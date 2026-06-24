"""Tests for the ring-signature module, including a known-answer vector from the
Soroban contract (``test_verifies_contract_test_vector``) that proves on-chain
agreement."""

from chainlogistics_sdk.ring_signature import (
    GX,
    GY,
    KeyPair,
    RingSignature,
    aggregate_ring,
    sign,
    verify,
    _G,
    _is_on_curve,
    _mul,
    _serialize,
    R,
)


# Generator uncompressed bytes, matching the contract's `G1_GENERATOR`.
G1_GENERATOR = bytes.fromhex(
    "17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac58"
    "6c55e83ff97a1aeffb3af00adb22c6bb08b3f481e3aaa0f1a09e30ed741d8ae4"
    "fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1"
)


def test_generator_is_on_curve_and_matches_constant():
    assert _is_on_curve(_G)
    assert _serialize(_G) == G1_GENERATOR
    assert (GX, GY) == _G


def test_generator_has_prime_order():
    # r · G == O (point at infinity): confirms G is in the prime-order subgroup.
    assert _mul(R, _G) is None


def test_sign_and_verify_round_trip_all_indices():
    keys = [KeyPair.generate() for _ in range(5)]
    ring = [k.public_key() for k in keys]
    msg = b"audit attestation"
    for signer in range(len(ring)):
        sig = sign(ring, signer, keys[signer], msg)
        assert verify(ring, msg, sig), f"signer {signer} must verify"


def test_wrong_message_fails():
    keys = [KeyPair.generate() for _ in range(3)]
    ring = [k.public_key() for k in keys]
    sig = sign(ring, 0, keys[0], b"original")
    assert not verify(ring, b"tampered", sig)


def test_anonymity_set_must_match():
    keys = [KeyPair.generate() for _ in range(3)]
    ring = [k.public_key() for k in keys]
    sig = sign(ring, 0, keys[0], b"x")
    ring[2] = KeyPair.generate().public_key()  # change the set
    assert not verify(ring, b"x", sig)


def test_tampered_response_fails():
    keys = [KeyPair.generate() for _ in range(4)]
    ring = [k.public_key() for k in keys]
    sig = sign(ring, 2, keys[2], b"statement")
    bad = RingSignature(c0=sig.c0, s=list(sig.s))
    bad.s[0] = bytes([7]) * 32
    assert not verify(ring, b"statement", bad)


def test_wrong_secret_is_rejected():
    keys = [KeyPair.generate() for _ in range(3)]
    ring = [k.public_key() for k in keys]
    try:
        sign(ring, 0, keys[1], b"x")  # keys[1] claims to be index 0
        assert False, "expected RingError"
    except Exception as exc:  # noqa: BLE001
        assert "secret key" in str(exc)


def test_aggregate_ring_order_sensitive():
    keys = [KeyPair.generate() for _ in range(4)]
    ring = [k.public_key() for k in keys]
    assert aggregate_ring(ring) == aggregate_ring(ring)
    reordered = [ring[1], ring[0], ring[2], ring[3]]
    assert aggregate_ring(ring) != aggregate_ring(reordered)


def test_verifies_contract_test_vector():
    """Known-answer cross-implementation test against the Soroban contract."""
    ring = [
        bytes.fromhex(
            "102b6a1c88da96b327e995c2159fb4f88070cd144de9e1f0a7aaa2dd37b3bb2b"
            "643a7dcfcdab05352d0156ffec6070d8054b2cef273b023043e72ed27862dd24"
            "73202e84cf1365128dafd26ba683b24fa7b527d2242d285cae0a77cbb0d9f396"
        ),
        bytes.fromhex(
            "15f5d598f843ec0b0a4d2368f516ead2e877ba2300148c10a56296de419cee64"
            "de75225a023341475bb67eb260f1edf20b8f39782375c2c7f0f2b1b975e9611f"
            "84497ff5920dd56aa3907e8a6ef1653af2f2bfdec459770fef0d799bd2d8cb31"
        ),
        bytes.fromhex(
            "04ff5071f60786edbd7f589e91c5c9ab0d7d0066b00dfbdf35520f1d50c0b1f9"
            "4e30a5c4abd093af78c762b7ab9709171297418166538f09f4cb6b89d46f6cc5"
            "c5e516234b99966cd092a8e34456db97b3fda3e1031c53ad159703f4f85ab514"
        ),
    ]
    message = b"handover: PROD-7 alice->bob nonce=42"
    sig = RingSignature(
        c0=bytes.fromhex(
            "3a56f0800412258de421b9146df64ee8db80385159874874b6e547b6351aef5f"
        ),
        s=[
            bytes.fromhex(
                "3500922125d31fe353ae32a2feed0e71dbf5b17df41348cae610abdd040dc442"
            ),
            bytes.fromhex(
                "1a6b9d69a28c59dbbf06001b3a7c0f96db375e65aab99db4d8c5ac885b908adb"
            ),
            bytes.fromhex(
                "3fb8309bbe0e15d5dc1c5ad87ab8e74647e660e19d4d8e99d3a1434ee9f3b0b1"
            ),
        ],
    )

    assert verify(ring, message, sig), "must accept contract-produced signature"
    assert not verify(ring, b"wrong", sig)
    assert aggregate_ring(ring) == bytes.fromhex(
        "0bbfd715e9206cdf3e965c10b5a1b7f57a099a7b72e105151852e51cfbe7fc80"
    )

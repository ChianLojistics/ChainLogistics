"""Tests for decentralized storage bridge."""

import pytest

from chainlogistics_sdk.storage import (
    MAX_FILE_BYTES,
    StorageBackend,
    StorageBridge,
)


def test_content_hash_sha256() -> None:
    assert len(StorageBridge.content_hash(b"manual")) == 64


def test_cid_v0_from_hash() -> None:
    h = StorageBridge.content_hash(b"hello")
    cid = StorageBridge.cid_v0_from_hash(h)
    assert len(cid) >= 40


def test_rejects_oversized_file() -> None:
    bridge = StorageBridge()
    with pytest.raises(Exception):
        bridge.upload(b"\x00" * (MAX_FILE_BYTES + 1), StorageBackend.IPFS)

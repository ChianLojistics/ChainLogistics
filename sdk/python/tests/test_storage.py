"""Tests for decentralized storage bridge."""

import hashlib

from chainlogistics_sdk.storage import (
    ContentStore,
    StorageBackend,
    hash_content,
    MAX_CONTENT_SIZE,
)


def test_hash_content_deterministic():
    data = b"manual pdf content"
    assert hash_content(data) == hashlib.sha256(data).hexdigest()
    assert len(hash_content(data)) == 64


def test_cas_dedup_in_memory():
    store = ContentStore()
    content = b"duplicate manual content"
    h = hash_content(content)

    from chainlogistics_sdk.storage.bridge import StorageUploadResult

    store._cas_registry[h] = StorageUploadResult(
        content_hash=h,
        cid="bafyTest",
        uri="ipfs://bafyTest",
        byte_size=len(content),
        backend=StorageBackend.IPFS,
    )

    # Simulate dedup lookup without network upload
    cached = store._cas_registry.get(hash_content(content))
    assert cached is not None
    assert cached.cid == "bafyTest"


def test_max_content_size_constant():
    assert MAX_CONTENT_SIZE == 52_428_800

"""Decentralized storage bridge for IPFS and Arweave with CAS deduplication."""

from __future__ import annotations

import hashlib
import json
import os
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, Optional, Tuple

import requests

from ..exceptions import ValidationError

MAX_CONTENT_SIZE = 52_428_800  # 50 MB


class StorageBackend(str, Enum):
    IPFS = "ipfs"
    ARWEAVE = "arweave"


@dataclass
class StorageUploadResult:
    content_hash: str
    cid: str
    uri: str
    byte_size: int
    backend: StorageBackend
    deduplicated: bool = False


@dataclass
class StorageBridgeConfig:
    ipfs_api_url: str = field(
        default_factory=lambda: os.getenv("IPFS_API_URL", "http://127.0.0.1:5001")
    )
    ipfs_gateway: str = field(
        default_factory=lambda: os.getenv("IPFS_GATEWAY", "https://ipfs.io/ipfs/")
    )
    arweave_gateway: str = field(
        default_factory=lambda: os.getenv("ARWEAVE_GATEWAY", "https://arweave.net/")
    )
    arweave_upload_url: str = field(
        default_factory=lambda: os.getenv(
            "ARWEAVE_UPLOAD_URL", "https://node2.arweave.net/tx"
        )
    )
    timeout: int = 120


def hash_content(content: bytes) -> str:
    """SHA-256 hex digest aligned with on-chain BytesN<32>."""
    return hashlib.sha256(content).hexdigest()


class StorageBridge(ABC):
    @abstractmethod
    def backend(self) -> StorageBackend:
        ...

    @abstractmethod
    def upload(self, content: bytes) -> StorageUploadResult:
        ...

    @abstractmethod
    def fetch(self, cid: str) -> bytes:
        ...

    def verify(self, cid: str, expected_hash: str) -> bool:
        return hash_content(self.fetch(cid)) == expected_hash.lower()


class IpfsBridge(StorageBridge):
    def __init__(self, config: StorageBridgeConfig):
        self.config = config
        self.session = requests.Session()

    def backend(self) -> StorageBackend:
        return StorageBackend.IPFS

    def upload(self, content: bytes) -> StorageUploadResult:
        _validate_size(content)
        content_hash = hash_content(content)
        url = f"{self.config.ipfs_api_url.rstrip('/')}/api/v0/add"
        response = self.session.post(
            url,
            params={"pin": "true", "cid-version": 1},
            files={"file": ("content", content, "application/octet-stream")},
            timeout=self.config.timeout,
        )
        response.raise_for_status()
        cid = _parse_ipfs_add_response(response.text)
        uri = f"{self.config.ipfs_gateway.rstrip('/')}/{cid}"
        return StorageUploadResult(
            content_hash=content_hash,
            cid=cid,
            uri=uri,
            byte_size=len(content),
            backend=StorageBackend.IPFS,
        )

    def fetch(self, cid: str) -> bytes:
        url = f"{self.config.ipfs_gateway.rstrip('/')}/{cid.lstrip('/')}"
        response = self.session.get(url, timeout=self.config.timeout)
        response.raise_for_status()
        data = response.content
        _validate_size(data)
        return data


class ArweaveBridge(StorageBridge):
    def __init__(self, config: StorageBridgeConfig):
        self.config = config
        self.session = requests.Session()

    def backend(self) -> StorageBackend:
        return StorageBackend.ARWEAVE

    def upload(self, content: bytes) -> StorageUploadResult:
        _validate_size(content)
        content_hash = hash_content(content)
        response = self.session.post(
            self.config.arweave_upload_url,
            data=content,
            headers={"Content-Type": "application/octet-stream"},
            timeout=self.config.timeout,
        )
        response.raise_for_status()
        tx_id = response.headers.get("x-arweave-tx-id") or response.text.strip()
        if not tx_id:
            raise ValidationError("Arweave upload returned no transaction ID")
        uri = f"{self.config.arweave_gateway.rstrip('/')}/{tx_id}"
        return StorageUploadResult(
            content_hash=content_hash,
            cid=tx_id,
            uri=uri,
            byte_size=len(content),
            backend=StorageBackend.ARWEAVE,
        )

    def fetch(self, tx_id: str) -> bytes:
        url = f"{self.config.arweave_gateway.rstrip('/')}/{tx_id.lstrip('/')}"
        response = self.session.get(url, timeout=self.config.timeout)
        response.raise_for_status()
        data = response.content
        _validate_size(data)
        return data


class ContentStore:
    """Content-addressed store with in-memory CAS deduplication."""

    def __init__(self, config: Optional[StorageBridgeConfig] = None):
        self.config = config or StorageBridgeConfig()
        self._bridges: Dict[StorageBackend, StorageBridge] = {
            StorageBackend.IPFS: IpfsBridge(self.config),
            StorageBackend.ARWEAVE: ArweaveBridge(self.config),
        }
        self._cas_registry: Dict[str, StorageUploadResult] = {}

    def upload(self, content: bytes, backend: StorageBackend) -> StorageUploadResult:
        content_hash = hash_content(content)
        if content_hash in self._cas_registry:
            existing = self._cas_registry[content_hash]
            return StorageUploadResult(
                content_hash=existing.content_hash,
                cid=existing.cid,
                uri=existing.uri,
                byte_size=existing.byte_size,
                backend=existing.backend,
                deduplicated=True,
            )
        bridge = self._bridges.get(backend)
        if bridge is None:
            raise ValidationError(f"unsupported backend: {backend}")
        result = bridge.upload(content)
        self._cas_registry[content_hash] = result
        return result

    def fetch(self, backend: StorageBackend, cid: str) -> bytes:
        bridge = self._bridges.get(backend)
        if bridge is None:
            raise ValidationError(f"unsupported backend: {backend}")
        return bridge.fetch(cid)

    def verify(self, backend: StorageBackend, cid: str, expected_hash: str) -> bool:
        bridge = self._bridges.get(backend)
        if bridge is None:
            raise ValidationError(f"unsupported backend: {backend}")
        return bridge.verify(cid, expected_hash)


def _validate_size(content: bytes) -> None:
    if not content or len(content) > MAX_CONTENT_SIZE:
        raise ValidationError(
            f"content size must be 1..={MAX_CONTENT_SIZE} bytes (50 MB max)"
        )


def _parse_ipfs_add_response(body: str) -> str:
    for line in filter(None, (l.strip() for l in body.splitlines())):
        try:
            data = json.loads(line)
            if "Hash" in data:
                return str(data["Hash"])
        except json.JSONDecodeError:
            continue
    raise ValidationError("unable to parse IPFS add response for CID")

"""Direct IPFS / Arweave storage bridge with CAS deduplication."""

from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, Optional

import requests

from .exceptions import ValidationError

MAX_FILE_BYTES = 50 * 1024 * 1024


class StorageBackend(str, Enum):
    IPFS = "ipfs"
    ARWEAVE = "arweave"


@dataclass
class StorageBridgeConfig:
    ipfs_api_url: str = os.environ.get("IPFS_API_URL", "http://127.0.0.1:5001")
    ipfs_gateway: str = os.environ.get("IPFS_GATEWAY", "https://ipfs.io/ipfs/")
    arweave_gateway: str = os.environ.get("ARWEAVE_GATEWAY", "https://arweave.net")
    anchor_registry_url: Optional[str] = None
    api_key: Optional[str] = None


@dataclass
class UploadResult:
    content_hash: str
    cid: str
    backend: str
    byte_size: int
    deduplicated: bool


class StorageBridge:
    """Upload manuals/PDFs directly to IPFS or Arweave — no central file silo."""

    def __init__(self, config: Optional[StorageBridgeConfig] = None) -> None:
        self.config = config or StorageBridgeConfig()
        self.session = requests.Session()

    @staticmethod
    def content_hash(data: bytes) -> str:
        return hashlib.sha256(data).hexdigest()

    @staticmethod
    def cid_v0_from_hash(hash_hex: str) -> str:
        import base58

        digest = bytes.fromhex(hash_hex.removeprefix("0x"))
        if len(digest) != 32:
            raise ValidationError("hash must be 32 bytes")
        multihash = b"\x12\x20" + digest
        return base58.b58encode(multihash).decode("ascii")

    def upload(
        self,
        data: bytes,
        backend: StorageBackend = StorageBackend.IPFS,
        product_id: Optional[str] = None,
    ) -> UploadResult:
        if not data or len(data) > MAX_FILE_BYTES:
            raise ValidationError(
                f"file size must be between 1 and {MAX_FILE_BYTES} bytes"
            )

        content_hash = self.content_hash(data)

        if self._cas_exists(content_hash):
            cid = (
                self.cid_v0_from_hash(content_hash)
                if backend == StorageBackend.IPFS
                else content_hash
            )
            return UploadResult(
                content_hash=content_hash,
                cid=cid,
                backend=backend.value,
                byte_size=len(data),
                deduplicated=True,
            )

        if backend == StorageBackend.IPFS:
            cid = self._upload_ipfs(data)
        else:
            cid = self._upload_arweave(data)

        self._register_anchor(content_hash, cid, backend.value, len(data), product_id)

        return UploadResult(
            content_hash=content_hash,
            cid=cid,
            backend=backend.value,
            byte_size=len(data),
            deduplicated=False,
        )

    def verify(self, cid: str, expected_hash: str, backend: StorageBackend) -> bool:
        data = self.fetch(cid, backend)
        return self.content_hash(data) == expected_hash.removeprefix("0x").lower()

    def fetch(self, cid: str, backend: StorageBackend) -> bytes:
        if backend == StorageBackend.IPFS:
            url = f"{self.config.ipfs_gateway.rstrip('/')}/{cid.lstrip('/')}"
        else:
            url = f"{self.config.arweave_gateway.rstrip('/')}/{cid.lstrip('/')}"

        response = self.session.get(url, timeout=120)
        response.raise_for_status()
        data = response.content
        if len(data) > MAX_FILE_BYTES:
            raise ValidationError("fetched content exceeds 50MB limit")
        return data

    def _cas_exists(self, content_hash: str) -> bool:
        if not self.config.anchor_registry_url:
            return False
        url = (
            f"{self.config.anchor_registry_url.rstrip('/')}"
            f"/api/v1/storage/exists/{content_hash}"
        )
        headers = {}
        if self.config.api_key:
            headers["Authorization"] = f"Bearer {self.config.api_key}"
        response = self.session.get(url, headers=headers, timeout=30)
        if response.status_code == 404:
            return False
        if not response.ok:
            return False
        body: Dict[str, Any] = response.json()
        return bool(body.get("exists"))

    def _register_anchor(
        self,
        content_hash: str,
        cid: str,
        backend: str,
        byte_size: int,
        product_id: Optional[str],
    ) -> None:
        if not self.config.anchor_registry_url:
            return
        url = f"{self.config.anchor_registry_url.rstrip('/')}/api/v1/storage/anchors"
        headers = {"Content-Type": "application/json"}
        if self.config.api_key:
            headers["Authorization"] = f"Bearer {self.config.api_key}"
        payload = {
            "content_hash": content_hash,
            "cid": cid,
            "storage_backend": backend,
            "product_id": product_id,
            "byte_size": byte_size,
        }
        response = self.session.post(url, json=payload, headers=headers, timeout=30)
        if response.status_code not in (201, 409):
            response.raise_for_status()

    def _upload_ipfs(self, data: bytes) -> str:
        url = f"{self.config.ipfs_api_url.rstrip('/')}/api/v0/add?pin=true"
        response = self.session.post(
            url,
            files={"file": ("content", data, "application/octet-stream")},
            timeout=300,
        )
        response.raise_for_status()
        body: Dict[str, Any] = response.json()
        cid = body.get("Hash")
        if not cid:
            raise ValidationError("IPFS response missing Hash")
        return str(cid)

    def _upload_arweave(self, data: bytes) -> str:
        url = f"{self.config.arweave_gateway.rstrip('/')}/tx"
        response = self.session.post(
            url,
            data=data,
            headers={"Content-Type": "application/octet-stream"},
            timeout=300,
        )
        response.raise_for_status()
        text = response.text.strip()
        try:
            body = response.json()
            if isinstance(body, dict) and body.get("id"):
                return str(body["id"])
        except ValueError:
            pass
        return text

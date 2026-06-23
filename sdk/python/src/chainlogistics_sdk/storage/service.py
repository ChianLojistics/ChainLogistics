"""High-level storage service combining decentralized upload and API registration."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Optional, Tuple

from ..client import ChainLogisticsClient
from .bridge import ContentStore, StorageBackend, StorageBridgeConfig, StorageUploadResult


@dataclass
class AnchorRegistration:
    product_id: str
    content_hash: str
    cid: str
    storage_scheme: str
    byte_size: int
    storage_uri: str
    on_chain_anchor_id: Optional[int] = None
    anchored_by: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "product_id": self.product_id,
            "content_hash": self.content_hash,
            "cid": self.cid,
            "storage_scheme": self.storage_scheme,
            "byte_size": self.byte_size,
            "storage_uri": self.storage_uri,
            "on_chain_anchor_id": self.on_chain_anchor_id,
            "anchored_by": self.anchored_by,
        }


class StorageService:
    """Decentralized storage bridge with CAS dedup and anchor registration."""

    def __init__(
        self,
        client: ChainLogisticsClient,
        bridge_config: Optional[StorageBridgeConfig] = None,
    ):
        self._client = client
        self._store = ContentStore(bridge_config or StorageBridgeConfig())

    @property
    def content_store(self) -> ContentStore:
        return self._store

    def upload_manual(
        self, content: bytes, backend: StorageBackend = StorageBackend.IPFS
    ) -> StorageUploadResult:
        return self._store.upload(content, backend)

    def anchor_manual(
        self,
        product_id: str,
        content: bytes,
        backend: StorageBackend = StorageBackend.IPFS,
    ) -> Tuple[StorageUploadResult, Dict[str, Any]]:
        upload = self.upload_manual(content, backend)
        registration = AnchorRegistration(
            product_id=product_id,
            content_hash=upload.content_hash,
            cid=upload.cid,
            storage_scheme=upload.backend.value,
            byte_size=upload.byte_size,
            storage_uri=upload.uri,
        )
        response = self._client.post(
            "api/v1/admin/storage/anchors", data=registration.to_dict()
        )
        return upload, response

    def verify_content(
        self, backend: StorageBackend, cid: str, expected_hash: str
    ) -> bool:
        return self._store.verify(backend, cid, expected_hash)

    def list_anchors(self, product_id: str) -> Any:
        return self._client.get(f"api/v1/storage/anchors/{product_id}")

    def trigger_verification(self) -> Any:
        return self._client.post("api/v1/admin/storage/verify")

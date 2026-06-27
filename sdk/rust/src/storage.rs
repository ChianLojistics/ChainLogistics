//! Decentralized storage bridge — direct IPFS / Arweave uploads with CAS dedup.
//!
//! Files go straight to configured decentralized endpoints (no central file silo).
//! Content is keyed by SHA-256; identical manuals deduplicate before upload.

use reqwest::multipart;
use sha2::{Digest, Sha256};

use crate::{Error, Result};

/// Maximum supported file size (50 MiB).
pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;

/// Target decentralized storage network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    Ipfs,
    Arweave,
}

impl StorageBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            StorageBackend::Ipfs => "ipfs",
            StorageBackend::Arweave => "arweave",
        }
    }
}

/// Configuration for direct decentralized storage access.
#[derive(Debug, Clone)]
pub struct StorageBridgeConfig {
    /// Kubo / IPFS HTTP API base (e.g. `http://127.0.0.1:5001`).
    pub ipfs_api_url: String,
    /// Read gateway for IPFS fetches (e.g. `https://ipfs.io/ipfs/`).
    pub ipfs_gateway: String,
    /// Arweave gateway for uploads and reads (e.g. `https://arweave.net`).
    pub arweave_gateway: String,
    /// Optional ChainLogistics API for CAS anchor registry (metadata only).
    pub anchor_registry_url: Option<String>,
    pub api_key: Option<String>,
}

impl Default for StorageBridgeConfig {
    fn default() -> Self {
        Self {
            ipfs_api_url: std::env::var("IPFS_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5001".to_string()),
            ipfs_gateway: std::env::var("IPFS_GATEWAY")
                .unwrap_or_else(|_| "https://ipfs.io/ipfs/".to_string()),
            arweave_gateway: std::env::var("ARWEAVE_GATEWAY")
                .unwrap_or_else(|_| "https://arweave.net".to_string()),
            anchor_registry_url: None,
            api_key: None,
        }
    }
}

/// Result of a successful decentralized upload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadResult {
    pub content_hash: String,
    pub cid: String,
    pub backend: String,
    pub byte_size: u64,
    pub deduplicated: bool,
}

/// Direct IPFS / Arweave bridge with content-addressed deduplication.
#[derive(Debug, Clone)]
pub struct StorageBridge {
    config: StorageBridgeConfig,
    http: reqwest::Client,
}

impl StorageBridge {
    pub fn new(config: StorageBridgeConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;
        Ok(Self { config, http })
    }

    /// SHA-256 hex digest of content.
    pub fn content_hash(data: &[u8]) -> String {
        hex::encode(Sha256::digest(data))
    }

    /// IPFS CIDv0 (base58btc) derived from SHA-256 multihash.
    pub fn cid_v0_from_hash(hash_hex: &str) -> Result<String> {
        let hash_bytes = hex::decode(hash_hex.trim_start_matches("0x"))
            .map_err(|e| Error::Validation(format!("invalid hash hex: {e}")))?;
        if hash_bytes.len() != 32 {
            return Err(Error::Validation("hash must be 32 bytes".into()));
        }
        let mut multihash = Vec::with_capacity(34);
        multihash.push(0x12); // sha2-256
        multihash.push(0x20);
        multihash.extend_from_slice(&hash_bytes);
        Ok(bs58::encode(multihash).into_string())
    }

    /// Upload content with CAS dedup — skips re-upload when hash already exists.
    pub async fn upload(
        &self,
        data: &[u8],
        backend: StorageBackend,
        product_id: Option<&str>,
    ) -> Result<UploadResult> {
        if data.is_empty() || data.len() as u64 > MAX_FILE_BYTES {
            return Err(Error::Validation(format!(
                "file size must be between 1 and {MAX_FILE_BYTES} bytes"
            )));
        }

        let content_hash = Self::content_hash(data);

        if self.cas_exists(&content_hash).await? {
            let cid = match backend {
                StorageBackend::Ipfs => Self::cid_v0_from_hash(&content_hash)?,
                StorageBackend::Arweave => content_hash.clone(),
            };
            return Ok(UploadResult {
                content_hash,
                cid,
                backend: backend.as_str().to_string(),
                byte_size: data.len() as u64,
                deduplicated: true,
            });
        }

        let (cid, backend_str) = match backend {
            StorageBackend::Ipfs => (self.upload_ipfs(data).await?, "ipfs".to_string()),
            StorageBackend::Arweave => (self.upload_arweave(data).await?, "arweave".to_string()),
        };

        self.register_anchor(&content_hash, &cid, &backend_str, data.len() as u64, product_id)
            .await?;

        Ok(UploadResult {
            content_hash,
            cid,
            backend: backend_str,
            byte_size: data.len() as u64,
            deduplicated: false,
        })
    }

    /// Fetch content from decentralized storage and verify against expected hash.
    pub async fn verify(
        &self,
        cid: &str,
        expected_hash: &str,
        backend: StorageBackend,
    ) -> Result<bool> {
        let bytes = self.fetch(cid, backend).await?;
        Ok(Self::content_hash(&bytes) == expected_hash.trim_start_matches("0x").to_lowercase())
    }

    /// Fetch raw bytes from IPFS or Arweave.
    pub async fn fetch(&self, cid: &str, backend: StorageBackend) -> Result<Vec<u8>> {
        let url = match backend {
            StorageBackend::Ipfs => format!(
                "{}/{}",
                self.config.ipfs_gateway.trim_end_matches('/'),
                cid.trim_start_matches('/')
            ),
            StorageBackend::Arweave => format!(
                "{}/{}",
                self.config.arweave_gateway.trim_end_matches('/'),
                cid.trim_start_matches('/')
            ),
        };

        let response = self.http.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(Error::api(
                response.status().as_u16(),
                format!("failed to fetch {cid}"),
            ));
        }
        let bytes = response.bytes().await?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(Error::Validation("fetched content exceeds 50MB limit".into()));
        }
        Ok(bytes.to_vec())
    }

    async fn cas_exists(&self, content_hash: &str) -> Result<bool> {
        let Some(base) = &self.config.anchor_registry_url else {
            return Ok(false);
        };

        let mut req = self
            .http
            .get(format!(
                "{}/api/v1/storage/exists/{}",
                base.trim_end_matches('/'),
                content_hash
            ));

        if let Some(key) = &self.config.api_key {
            req = req.bearer_auth(key);
        }

        let response = req.send().await?;
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        if !response.status().is_success() {
            return Ok(false);
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body.get("exists").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    async fn register_anchor(
        &self,
        content_hash: &str,
        cid: &str,
        backend: &str,
        byte_size: u64,
        product_id: Option<&str>,
    ) -> Result<()> {
        let Some(base) = &self.config.anchor_registry_url else {
            return Ok(());
        };

        let body = serde_json::json!({
            "content_hash": content_hash,
            "cid": cid,
            "storage_backend": backend,
            "product_id": product_id,
            "byte_size": byte_size,
        });

        let mut req = self
            .http
            .post(format!("{}/api/v1/storage/anchors", base.trim_end_matches('/')))
            .json(&body);

        if let Some(key) = &self.config.api_key {
            req = req.bearer_auth(key);
        }

        let response = req.send().await?;
        if response.status().is_success() || response.status().as_u16() == 409 {
            return Ok(());
        }
        Err(Error::api(
            response.status().as_u16(),
            "failed to register anchor metadata",
        ))
    }

    async fn upload_ipfs(&self, data: &[u8]) -> Result<String> {
        let part = multipart::Part::bytes(data.to_vec())
            .file_name("content".to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| Error::Validation(e.to_string()))?;

        let form = multipart::Form::new().part("file", part);

        let url = format!(
            "{}/api/v0/add?pin=true",
            self.config.ipfs_api_url.trim_end_matches('/')
        );

        let response = self.http.post(&url).multipart(form).send().await?;
        if !response.status().is_success() {
            return Err(Error::api(
                response.status().as_u16(),
                "IPFS add failed",
            ));
        }

        let body: serde_json::Value = response.json().await?;
        body.get("Hash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Validation("IPFS response missing Hash".into()))
    }

    async fn upload_arweave(&self, data: &[u8]) -> Result<String> {
        let url = format!(
            "{}/tx",
            self.config.arweave_gateway.trim_end_matches('/')
        );

        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::api(
                response.status().as_u16(),
                "Arweave upload failed",
            ));
        }

        // Arweave gateways return the transaction id as plain text or JSON.
        let text = response.text().await?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                return Ok(id.to_string());
            }
        }
        Ok(text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_sha256_hex() {
        let hash = StorageBridge::content_hash(b"manual");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn cid_v0_from_known_hash() {
        let hash = StorageBridge::content_hash(b"hello");
        let cid = StorageBridge::cid_v0_from_hash(&hash).unwrap();
        assert!(cid.starts_with('Q') || cid.starts_with('b')); // base58btc
    }

    #[test]
    fn rejects_oversized_payload() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let bridge = StorageBridge::new(StorageBridgeConfig::default()).unwrap();
        let oversized = vec![0u8; (MAX_FILE_BYTES + 1) as usize];
        let err = rt
            .block_on(bridge.upload(&oversized, StorageBackend::Ipfs, None))
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }
}

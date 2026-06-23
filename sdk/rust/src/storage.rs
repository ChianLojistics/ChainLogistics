//! Decentralized storage bridge for IPFS and Arweave.
//!
//! Uploads go directly to configured gateways/nodes — no central middleman.
//! Identical manuals are deduplicated via content-addressed storage (CAS).

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::{Error, Result};

/// Maximum supported file size: 50 MB
pub const MAX_CONTENT_SIZE: u64 = 52_428_800;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackend {
    Ipfs,
    Arweave,
}

impl StorageBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ipfs => "ipfs",
            Self::Arweave => "arweave",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUploadResult {
    pub content_hash: String,
    pub cid: String,
    pub uri: String,
    pub byte_size: u64,
    pub backend: StorageBackend,
    pub deduplicated: bool,
}

#[derive(Debug, Clone)]
pub struct StorageBridgeConfig {
    pub ipfs_api_url: String,
    pub ipfs_gateway: String,
    pub arweave_gateway: String,
    pub arweave_upload_url: String,
    pub timeout: Duration,
}

impl Default for StorageBridgeConfig {
    fn default() -> Self {
        Self {
            ipfs_api_url: std::env::var("IPFS_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5001".to_string()),
            ipfs_gateway: std::env::var("IPFS_GATEWAY")
                .unwrap_or_else(|_| "https://ipfs.io/ipfs/".to_string()),
            arweave_gateway: std::env::var("ARWEAVE_GATEWAY")
                .unwrap_or_else(|_| "https://arweave.net/".to_string()),
            arweave_upload_url: std::env::var("ARWEAVE_UPLOAD_URL")
                .unwrap_or_else(|_| "https://node2.arweave.net/tx".to_string()),
            timeout: Duration::from_secs(120),
        }
    }
}

/// Bridge trait for pluggable decentralized storage backends.
#[async_trait]
pub trait StorageBridge: Send + Sync {
    fn backend(&self) -> StorageBackend;

    async fn upload(&self, content: &[u8]) -> Result<StorageUploadResult>;

    async fn fetch(&self, cid: &str) -> Result<Vec<u8>>;

    async fn verify(&self, cid: &str, expected_hash: &str) -> Result<bool>;
}

/// Compute SHA-256 hex digest aligned with on-chain `BytesN<32>`.
pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// IPFS bridge using Kubo-compatible HTTP API (`/api/v0/add`).
#[derive(Debug, Clone)]
pub struct IpfsBridge {
    client: Client,
    config: StorageBridgeConfig,
}

impl IpfsBridge {
    pub fn new(config: StorageBridgeConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(Error::Http)?;
        Ok(Self { client, config })
    }
}

#[async_trait]
impl StorageBridge for IpfsBridge {
    fn backend(&self) -> StorageBackend {
        StorageBackend::Ipfs
    }

    async fn upload(&self, content: &[u8]) -> Result<StorageUploadResult> {
        if content.is_empty() || content.len() as u64 > MAX_CONTENT_SIZE {
            return Err(Error::Validation(format!(
                "content size must be 1..={} bytes",
                MAX_CONTENT_SIZE
            )));
        }

        let content_hash = hash_content(content);
        let url = format!(
            "{}/api/v0/add?pin=true&cid-version=1",
            self.config.ipfs_api_url.trim_end_matches('/')
        );

        let part = reqwest::multipart::Part::bytes(content.to_vec())
            .file_name("content")
            .mime_str("application/octet-stream")
            .map_err(|e| Error::Validation(e.to_string()))?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(status, body));
        }

        let body = response.text().await?;
        let cid = parse_ipfs_add_response(&body)?;
        let uri = format!(
            "{}{}",
            self.config.ipfs_gateway.trim_end_matches('/'),
            format!("/{}", cid)
        );

        Ok(StorageUploadResult {
            content_hash,
            cid: cid.clone(),
            uri,
            byte_size: content.len() as u64,
            backend: StorageBackend::Ipfs,
            deduplicated: false,
        })
    }

    async fn fetch(&self, cid: &str) -> Result<Vec<u8>> {
        let url = format!(
            "{}/{}",
            self.config.ipfs_gateway.trim_end_matches('/'),
            cid.trim_start_matches('/')
        );
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(Error::api(
                response.status().as_u16(),
                format!("IPFS gateway fetch failed for {}", cid),
            ));
        }
        let bytes = response.bytes().await?;
        if bytes.len() as u64 > MAX_CONTENT_SIZE {
            return Err(Error::Validation("fetched content exceeds 50 MB limit".into()));
        }
        Ok(bytes.to_vec())
    }

    async fn verify(&self, cid: &str, expected_hash: &str) -> Result<bool> {
        let content = self.fetch(cid).await?;
        Ok(hash_content(&content) == expected_hash.to_lowercase())
    }
}

/// Arweave bridge — uploads via HTTP transaction endpoint, fetches via gateway.
#[derive(Debug, Clone)]
pub struct ArweaveBridge {
    client: Client,
    config: StorageBridgeConfig,
}

impl ArweaveBridge {
    pub fn new(config: StorageBridgeConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(Error::Http)?;
        Ok(Self { client, config })
    }
}

#[async_trait]
impl StorageBridge for ArweaveBridge {
    fn backend(&self) -> StorageBackend {
        StorageBackend::Arweave
    }

    async fn upload(&self, content: &[u8]) -> Result<StorageUploadResult> {
        if content.is_empty() || content.len() as u64 > MAX_CONTENT_SIZE {
            return Err(Error::Validation(format!(
                "content size must be 1..={} bytes",
                MAX_CONTENT_SIZE
            )));
        }

        let content_hash = hash_content(content);

        // Direct upload to Arweave node (zero middleman when self-hosted)
        let response = self
            .client
            .post(&self.config.arweave_upload_url)
            .header("Content-Type", "application/octet-stream")
            .body(content.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(
                status,
                format!("Arweave upload failed: {}", body),
            ));
        }

        let tx_id = if let Some(id) = response
            .headers()
            .get("x-arweave-tx-id")
            .and_then(|v| v.to_str().ok())
        {
            id.to_string()
        } else {
            response.text().await.unwrap_or_default()
        };

        if tx_id.trim().is_empty() {
            return Err(Error::Validation(
                "Arweave upload returned no transaction ID".into(),
            ));
        }

        let uri = format!(
            "{}{}",
            self.config.arweave_gateway.trim_end_matches('/'),
            format!("/{}", tx_id.trim())
        );

        Ok(StorageUploadResult {
            content_hash,
            cid: tx_id.clone(),
            uri,
            byte_size: content.len() as u64,
            backend: StorageBackend::Arweave,
            deduplicated: false,
        })
    }

    async fn fetch(&self, tx_id: &str) -> Result<Vec<u8>> {
        let url = format!(
            "{}/{}",
            self.config.arweave_gateway.trim_end_matches('/'),
            tx_id.trim_start_matches('/')
        );
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(Error::api(
                response.status().as_u16(),
                format!("Arweave gateway fetch failed for {}", tx_id),
            ));
        }
        let bytes = response.bytes().await?;
        if bytes.len() as u64 > MAX_CONTENT_SIZE {
            return Err(Error::Validation("fetched content exceeds 50 MB limit".into()));
        }
        Ok(bytes.to_vec())
    }

    async fn verify(&self, tx_id: &str, expected_hash: &str) -> Result<bool> {
        let content = self.fetch(tx_id).await?;
        Ok(hash_content(&content) == expected_hash.to_lowercase())
    }
}

/// Content store with CAS deduplication across backends.
#[derive(Clone)]
pub struct ContentStore {
    bridges: HashMap<StorageBackend, Arc<dyn StorageBridge>>,
    cas_registry: Arc<Mutex<HashMap<String, StorageUploadResult>>>,
}

impl ContentStore {
    pub fn new(config: StorageBridgeConfig) -> Result<Self> {
        let mut bridges: HashMap<StorageBackend, Arc<dyn StorageBridge>> = HashMap::new();
        bridges.insert(
            StorageBackend::Ipfs,
            Arc::new(IpfsBridge::new(config.clone())?) as Arc<dyn StorageBridge>,
        );
        bridges.insert(
            StorageBackend::Arweave,
            Arc::new(ArweaveBridge::new(config)?) as Arc<dyn StorageBridge>,
        );

        Ok(Self {
            bridges,
            cas_registry: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn with_config(config: StorageBridgeConfig) -> Result<Self> {
        Self::new(config)
    }

    /// Upload content with CAS dedup — identical manuals reuse existing CID.
    pub async fn upload(
        &self,
        content: &[u8],
        backend: StorageBackend,
    ) -> Result<StorageUploadResult> {
        let content_hash = hash_content(content);

        if let Some(existing) = self.cas_registry.lock().unwrap().get(&content_hash) {
            let mut result = existing.clone();
            result.deduplicated = true;
            return Ok(result);
        }

        let bridge = self
            .bridges
            .get(&backend)
            .ok_or_else(|| Error::Validation(format!("unsupported backend: {:?}", backend)))?;

        let result = bridge.upload(content).await?;
        self.cas_registry
            .lock()
            .unwrap()
            .insert(content_hash, result.clone());
        Ok(result)
    }

    pub async fn fetch(&self, backend: StorageBackend, cid: &str) -> Result<Vec<u8>> {
        let bridge = self
            .bridges
            .get(&backend)
            .ok_or_else(|| Error::Validation(format!("unsupported backend: {:?}", backend)))?;
        bridge.fetch(cid).await
    }

    pub async fn verify(
        &self,
        backend: StorageBackend,
        cid: &str,
        expected_hash: &str,
    ) -> Result<bool> {
        let bridge = self
            .bridges
            .get(&backend)
            .ok_or_else(|| Error::Validation(format!("unsupported backend: {:?}", backend)))?;
        bridge.verify(cid, expected_hash).await
    }
}

fn parse_ipfs_add_response(body: &str) -> Result<String> {
    for line in body.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(hash) = v.get("Hash").and_then(|h| h.as_str()) {
                return Ok(hash.to_string());
            }
        }
    }
    Err(Error::Validation(
        "unable to parse IPFS add response for CID".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content_deterministic() {
        let h1 = hash_content(b"manual pdf content");
        let h2 = hash_content(b"manual pdf content");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_cas_dedup_in_memory() {
        let store = ContentStore {
            bridges: HashMap::new(),
            cas_registry: Arc::new(Mutex::new(HashMap::new())),
        };
        let hash = hash_content(b"duplicate manual");
        store.cas_registry.lock().unwrap().insert(
            hash.clone(),
            StorageUploadResult {
                content_hash: hash,
                cid: "bafyTest".into(),
                uri: "ipfs://bafyTest".into(),
                byte_size: 16,
                backend: StorageBackend::Ipfs,
                deduplicated: false,
            },
        );
        let cached = store.cas_registry.lock().unwrap().get(&hash_content(b"duplicate manual")).cloned();
        assert!(cached.is_some());
    }
}

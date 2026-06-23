use crate::client::HttpClient;
use crate::storage::{ContentStore, StorageBackend, StorageBridgeConfig, StorageUploadResult};
use crate::{Config, Result};
use serde::{Deserialize, Serialize};

/// High-level storage service: decentralized upload + API anchor registration.
#[derive(Clone)]
pub struct StorageService {
    http: HttpClient,
    store: ContentStore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRegistration {
    pub product_id: String,
    pub content_hash: String,
    pub cid: String,
    pub storage_scheme: StorageBackend,
    pub byte_size: u64,
    pub storage_uri: String,
    pub on_chain_anchor_id: Option<i64>,
    pub anchored_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRegistrationResponse {
    pub anchor: serde_json::Value,
    pub deduplicated: bool,
}

impl StorageService {
    pub fn new(client: reqwest::Client, config: Config, bridge_config: StorageBridgeConfig) -> Result<Self> {
        Ok(Self {
            http: HttpClient::new(client, config),
            store: ContentStore::new(bridge_config)?,
        })
    }

    pub fn content_store(&self) -> &ContentStore {
        &self.store
    }

    /// Upload manual/PDF to decentralized storage with CAS dedup.
    pub async fn upload_manual(
        &self,
        content: &[u8],
        backend: StorageBackend,
    ) -> Result<StorageUploadResult> {
        self.store.upload(content, backend).await
    }

    /// Upload and register anchor with the ChainLogistics API.
    pub async fn anchor_manual(
        &self,
        product_id: &str,
        content: &[u8],
        backend: StorageBackend,
    ) -> Result<(StorageUploadResult, AnchorRegistrationResponse)> {
        let upload = self.upload_manual(content, backend).await?;

        let registration = AnchorRegistration {
            product_id: product_id.to_string(),
            content_hash: upload.content_hash.clone(),
            cid: upload.cid.clone(),
            storage_scheme: upload.backend,
            byte_size: upload.byte_size,
            storage_uri: upload.uri.clone(),
            on_chain_anchor_id: None,
            anchored_by: None,
        };

        let request = self.http.post("api/v1/admin/storage/anchors");
        let response: AnchorRegistrationResponse =
            self.http.execute_with_body(request, &registration).await?;

        Ok((upload, response))
    }

    /// Verify content integrity against decentralized storage.
    pub async fn verify_content(
        &self,
        backend: StorageBackend,
        cid: &str,
        expected_hash: &str,
    ) -> Result<bool> {
        self.store.verify(backend, cid, expected_hash).await
    }

    pub async fn list_anchors(&self, product_id: &str) -> Result<Vec<serde_json::Value>> {
        let request = self
            .http
            .get(&format!("api/v1/storage/anchors/{}", product_id));
        self.http.execute(request).await
    }

    pub async fn trigger_verification(&self) -> Result<serde_json::Value> {
        let request = self.http.post("api/v1/admin/storage/verify");
        self.http.execute(request).await
    }
}

impl StorageService {
    pub fn from_config(config: Config) -> Result<(reqwest::Client, Self)> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout())
            .user_agent(config.user_agent())
            .build()?;
        let bridge_config = StorageBridgeConfig::default();
        let service = Self::new(client.clone(), config, bridge_config)?;
        Ok((client, service))
    }
}

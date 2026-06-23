use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::services::audit_service::{
    AuditEventCategory, AuditService, AuditSeverity, NewAuditEvent,
};

/// Maximum file size: 50 MB
pub const MAX_CONTENT_SIZE: u64 = 52_428_800;

/// Verification interval for periodic worker (seconds)
pub const DEFAULT_VERIFICATION_INTERVAL_SECS: u64 = 900;

const ANCHOR_SELECT: &str = r#"
    SELECT
        id, on_chain_anchor_id, product_id, content_hash, cid,
        storage_scheme, byte_size, storage_uri, verification_status,
        last_verified_at, failure_reason, failure_count, deduplicated,
        anchored_by, created_at, updated_at
    FROM content_anchors
"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StorageScheme {
    Ipfs,
    Arweave,
}

impl StorageScheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ipfs => "ipfs",
            Self::Arweave => "arweave",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Tampered,
    Unavailable,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Verified => "verified",
            Self::Tampered => "tampered",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentAnchor {
    pub id: Uuid,
    pub on_chain_anchor_id: Option<i64>,
    pub product_id: String,
    pub content_hash: String,
    pub cid: String,
    pub storage_scheme: String,
    pub byte_size: i64,
    pub storage_uri: String,
    pub verification_status: String,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub failure_count: i32,
    pub deduplicated: bool,
    pub anchored_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAnchorRequest {
    pub product_id: String,
    pub content_hash: String,
    pub cid: String,
    pub storage_scheme: StorageScheme,
    pub byte_size: u64,
    pub storage_uri: String,
    pub on_chain_anchor_id: Option<i64>,
    pub anchored_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRunSummary {
    pub checked: usize,
    pub verified: usize,
    pub tampered: usize,
    pub unavailable: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TamperAlert {
    pub id: Uuid,
    pub anchor_id: Uuid,
    pub product_id: String,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub ipfs_gateway: String,
    pub arweave_gateway: String,
    pub verification_interval_secs: u64,
    pub verification_batch_size: i64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            ipfs_gateway: std::env::var("IPFS_GATEWAY")
                .unwrap_or_else(|_| "https://ipfs.io/ipfs/".to_string()),
            arweave_gateway: std::env::var("ARWEAVE_GATEWAY")
                .unwrap_or_else(|_| "https://arweave.net/".to_string()),
            verification_interval_secs: std::env::var("STORAGE_VERIFICATION_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_VERIFICATION_INTERVAL_SECS),
            verification_batch_size: std::env::var("STORAGE_VERIFICATION_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
        }
    }
}

#[derive(Clone)]
pub struct ContentAnchorService {
    pool: PgPool,
}

impl ContentAnchorService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register_anchor(
        &self,
        req: RegisterAnchorRequest,
    ) -> Result<(ContentAnchor, bool), sqlx::Error> {
        if req.byte_size == 0 || req.byte_size > MAX_CONTENT_SIZE {
            return Err(sqlx::Error::Protocol(
                "byte_size must be between 1 and 52428800 (50 MB)".into(),
            ));
        }

        if let Some(existing) = self.get_by_hash(&req.content_hash).await? {
            return Ok((existing, true));
        }

        let anchor = sqlx::query_as::<_, ContentAnchor>(
            r#"
            INSERT INTO content_anchors (
                product_id, content_hash, cid, storage_scheme, byte_size,
                storage_uri, on_chain_anchor_id, anchored_by, deduplicated
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)
            RETURNING
                id, on_chain_anchor_id, product_id, content_hash, cid,
                storage_scheme, byte_size, storage_uri, verification_status,
                last_verified_at, failure_reason, failure_count, deduplicated,
                anchored_by, created_at, updated_at
            "#,
        )
        .bind(&req.product_id)
        .bind(req.content_hash.to_lowercase())
        .bind(&req.cid)
        .bind(req.storage_scheme.as_str())
        .bind(req.byte_size as i64)
        .bind(&req.storage_uri)
        .bind(req.on_chain_anchor_id)
        .bind(&req.anchored_by)
        .fetch_one(&self.pool)
        .await?;

        Ok((anchor, false))
    }

    pub async fn get_by_hash(&self, content_hash: &str) -> Result<Option<ContentAnchor>, sqlx::Error> {
        let query = format!("{} WHERE content_hash = $1", ANCHOR_SELECT);
        sqlx::query_as::<_, ContentAnchor>(&query)
            .bind(content_hash.to_lowercase())
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<ContentAnchor>, sqlx::Error> {
        let query = format!("{} WHERE id = $1", ANCHOR_SELECT);
        sqlx::query_as::<_, ContentAnchor>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_by_product(&self, product_id: &str) -> Result<Vec<ContentAnchor>, sqlx::Error> {
        let query = format!(
            "{} WHERE product_id = $1 ORDER BY created_at DESC",
            ANCHOR_SELECT
        );
        sqlx::query_as::<_, ContentAnchor>(&query)
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn list_unresolved_alerts(&self) -> Result<Vec<TamperAlert>, sqlx::Error> {
        sqlx::query_as::<_, TamperAlert>(
            r#"
            SELECT id, anchor_id, product_id, expected_hash, actual_hash,
                   detected_at, resolved
            FROM content_tamper_alerts
            WHERE resolved = false
            ORDER BY detected_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[derive(Clone)]
pub struct StorageVerificationService {
    pool: PgPool,
    http: Client,
    config: StorageConfig,
    audit_service: AuditService,
}

impl StorageVerificationService {
    pub fn new(pool: PgPool, config: StorageConfig, audit_service: AuditService) -> Self {
        Self {
            pool,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| Client::new()),
            config,
            audit_service,
        }
    }

    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    pub async fn fetch_content(&self, anchor: &ContentAnchor) -> Result<Vec<u8>, String> {
        let url = if anchor.storage_uri.starts_with("http") {
            anchor.storage_uri.clone()
        } else {
            match anchor.storage_scheme.as_str() {
                "ipfs" => format!(
                    "{}/{}",
                    self.config.ipfs_gateway.trim_end_matches('/'),
                    anchor.cid.trim_start_matches('/')
                ),
                "arweave" => format!(
                    "{}/{}",
                    self.config.arweave_gateway.trim_end_matches('/'),
                    anchor.cid.trim_start_matches('/')
                ),
                other => return Err(format!("unsupported storage scheme: {}", other)),
            }
        };

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("fetch failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("gateway returned HTTP {}", response.status()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("read body failed: {}", e))?;

        if bytes.len() as u64 > MAX_CONTENT_SIZE {
            return Err(format!("content exceeds 50 MB limit ({} bytes)", bytes.len()));
        }

        Ok(bytes.to_vec())
    }

    pub fn hash_content(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    async fn record_tamper_alert(
        &self,
        anchor: &ContentAnchor,
        actual_hash: Option<&str>,
        reason: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO content_tamper_alerts (
                anchor_id, product_id, expected_hash, actual_hash, alert_payload
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(anchor.id)
        .bind(&anchor.product_id)
        .bind(&anchor.content_hash)
        .bind(actual_hash)
        .bind(serde_json::json!({ "reason": reason }))
        .execute(&self.pool)
        .await?;

        let _ = self
            .audit_service
            .log(NewAuditEvent {
                correlation_id: Some(anchor.id.to_string()),
                user_id: None,
                actor_api_key_id: None,
                event_category: AuditEventCategory::SecurityEvent,
                event_type: "content_tamper_detected".to_string(),
                severity: AuditSeverity::Error,
                action: "storage_integrity_failure".to_string(),
                resource_type: Some("content_anchor".to_string()),
                target_resource_id: Some(anchor.id.to_string()),
                http_method: None,
                http_path: None,
                http_status: None,
                success: false,
                error_code: Some("TAMPER_DETECTED".to_string()),
                business_context: Some(format!(
                    "product_id={} cid={} reason={}",
                    anchor.product_id, anchor.cid, reason
                )),
                changes: serde_json::json!({
                    "expected_hash": anchor.content_hash,
                    "actual_hash": actual_hash,
                    "product_id": anchor.product_id,
                    "cid": anchor.cid,
                }),
                ip_address: None,
                user_agent: None,
            })
            .await;

        Ok(())
    }

    async fn verify_single(&self, anchor: &ContentAnchor) -> Result<VerificationStatus, sqlx::Error> {
        let (result, failure_reason) = match self.fetch_content(anchor).await {
            Ok(bytes) => {
                let actual = Self::hash_content(&bytes);
                if actual == anchor.content_hash.to_lowercase() {
                    (VerificationStatus::Verified, None)
                } else {
                    let _ = self
                        .record_tamper_alert(anchor, Some(&actual), "hash mismatch")
                        .await;
                    (
                        VerificationStatus::Tampered,
                        Some("content hash mismatch".to_string()),
                    )
                }
            }
            Err(reason) => (
                VerificationStatus::Unavailable,
                Some(format!("unable to fetch from decentralized gateway: {}", reason)),
            ),
        };

        let increment_failure = matches!(
            result,
            VerificationStatus::Tampered | VerificationStatus::Unavailable
        );

        sqlx::query(
            r#"
            UPDATE content_anchors
            SET verification_status = $2,
                last_verified_at = NOW(),
                failure_reason = $3,
                failure_count = CASE WHEN $4 THEN failure_count + 1 ELSE failure_count END,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(anchor.id)
        .bind(result.as_str())
        .bind(&failure_reason)
        .bind(increment_failure)
        .execute(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn verify_due_anchors(&self) -> Result<VerificationRunSummary, sqlx::Error> {
        let interval_secs = self.config.verification_interval_secs as i64;
        let batch_size = self.config.verification_batch_size;

        let query = format!(
            r#"
            {}
            WHERE verification_status IN ('pending', 'verified')
              AND (
                last_verified_at IS NULL
                OR last_verified_at < NOW() - make_interval(secs => $1)
              )
            ORDER BY last_verified_at NULLS FIRST
            LIMIT $2
            "#,
            ANCHOR_SELECT
        );

        let anchors = sqlx::query_as::<_, ContentAnchor>(&query)
            .bind(interval_secs)
            .bind(batch_size)
            .fetch_all(&self.pool)
            .await?;

        let mut summary = VerificationRunSummary {
            checked: 0,
            verified: 0,
            tampered: 0,
            unavailable: 0,
        };

        for anchor in anchors {
            summary.checked += 1;
            match self.verify_single(&anchor).await {
                Ok(VerificationStatus::Verified) => summary.verified += 1,
                Ok(VerificationStatus::Tampered) => summary.tampered += 1,
                Ok(VerificationStatus::Unavailable) => summary.unavailable += 1,
                Ok(VerificationStatus::Pending) => {}
                Err(e) => {
                    tracing::error!("verification failed for anchor {}: {}", anchor.id, e);
                    summary.unavailable += 1;
                }
            }
        }

        if summary.tampered > 0 {
            tracing::warn!(
                "Storage tamper detection: {} anchor(s) failed integrity check",
                summary.tampered
            );
        }

        Ok(summary)
    }

    pub async fn verify_anchor_by_id(&self, id: Uuid) -> Result<VerificationStatus, sqlx::Error> {
        let query = format!("{} WHERE id = $1", ANCHOR_SELECT);
        let anchor = sqlx::query_as::<_, ContentAnchor>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        self.verify_single(&anchor).await
    }
}

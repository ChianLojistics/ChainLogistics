use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::{error, info, warn};

/// Maximum file size supported for decentralized storage (50 MiB).
pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum StorageBackend {
    Ipfs,
    Arweave,
}

impl StorageBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageBackend::Ipfs => "ipfs",
            StorageBackend::Arweave => "arweave",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ipfs" => Some(StorageBackend::Ipfs),
            "arweave" => Some(StorageBackend::Arweave),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Tampered,
    Unavailable,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationStatus::Pending => "pending",
            VerificationStatus::Verified => "verified",
            VerificationStatus::Tampered => "tampered",
            VerificationStatus::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContentAnchor {
    pub content_hash: String,
    pub cid: String,
    pub storage_backend: String,
    pub product_id: Option<String>,
    pub byte_size: i64,
    pub mime_type: Option<String>,
    pub anchored_at: DateTime<Utc>,
    pub anchored_by: Option<String>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub verification_status: String,
    pub tamper_alert_sent: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAnchorRequest {
    pub content_hash: String,
    pub cid: String,
    pub storage_backend: String,
    pub product_id: Option<String>,
    pub byte_size: u64,
    pub mime_type: Option<String>,
    pub anchored_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperAlert {
    pub content_hash: String,
    pub cid: String,
    pub storage_backend: String,
    pub product_id: Option<String>,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub ipfs_gateway: String,
    pub arweave_gateway: String,
    pub verification_batch_size: i64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            ipfs_gateway: std::env::var("IPFS_GATEWAY")
                .unwrap_or_else(|_| "https://ipfs.io/ipfs/".to_string()),
            arweave_gateway: std::env::var("ARWEAVE_GATEWAY")
                .unwrap_or_else(|_| "https://arweave.net/".to_string()),
            verification_batch_size: std::env::var("STORAGE_VERIFY_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
        }
    }
}

enum VerificationOutcome {
    Match,
    Tampered { actual_hash: String },
}

pub struct StorageIntegrityService {
    pool: PgPool,
    redis_client: redis::Client,
    config: StorageConfig,
    http: reqwest::Client,
}

impl StorageIntegrityService {
    pub fn new(pool: PgPool, redis_client: redis::Client, config: StorageConfig) -> Self {
        Self {
            pool,
            redis_client,
            config,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
        }
    }

    /// CAS lookup — returns true when this content hash is already registered.
    pub async fn exists(&self, content_hash: &str) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM content_anchors WHERE content_hash = $1",
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    pub async fn get_anchor(&self, content_hash: &str) -> Result<Option<ContentAnchor>, sqlx::Error> {
        sqlx::query_as::<_, ContentAnchor>(
            "SELECT * FROM content_anchors WHERE content_hash = $1",
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await
    }

    /// Register an anchor (idempotent CAS — same hash + cid is a no-op).
    pub async fn register_anchor(
        &self,
        req: &RegisterAnchorRequest,
    ) -> Result<ContentAnchor, sqlx::Error> {
        if req.byte_size == 0 || req.byte_size > MAX_FILE_BYTES {
            return Err(sqlx::Error::Protocol(
                "byte_size must be between 1 and 52428800".into(),
            ));
        }

        if StorageBackend::from_str(&req.storage_backend).is_none() {
            return Err(sqlx::Error::Protocol(
                "storage_backend must be ipfs or arweave".into(),
            ));
        }

        if let Some(existing) = self.get_anchor(&req.content_hash).await? {
            if existing.cid == req.cid && existing.storage_backend == req.storage_backend {
                return Ok(existing);
            }
            return Err(sqlx::Error::Protocol(
                "content_hash already anchored with different CID".into(),
            ));
        }

        sqlx::query_as::<_, ContentAnchor>(
            r#"
            INSERT INTO content_anchors (
                content_hash, cid, storage_backend, product_id, byte_size,
                mime_type, anchored_by, verification_status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
            RETURNING *
            "#,
        )
        .bind(&req.content_hash)
        .bind(&req.cid)
        .bind(&req.storage_backend.to_lowercase())
        .bind(&req.product_id)
        .bind(req.byte_size as i64)
        .bind(&req.mime_type)
        .bind(&req.anchored_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_product_anchors(
        &self,
        product_id: &str,
    ) -> Result<Vec<ContentAnchor>, sqlx::Error> {
        sqlx::query_as::<_, ContentAnchor>(
            "SELECT * FROM content_anchors WHERE product_id = $1 ORDER BY anchored_at DESC",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Periodic verification: fetch from decentralized storage and compare hashes.
    pub async fn verify_pending_anchors(&self) -> Result<usize, sqlx::Error> {
        let anchors: Vec<ContentAnchor> = sqlx::query_as::<_, ContentAnchor>(
            r#"
            SELECT * FROM content_anchors
            WHERE verification_status IN ('pending', 'verified')
            ORDER BY last_verified_at NULLS FIRST, anchored_at ASC
            LIMIT $1
            "#,
        )
        .bind(self.config.verification_batch_size)
        .fetch_all(&self.pool)
        .await?;

        let mut tamper_count = 0usize;

        for anchor in anchors {
            match self.verify_single(&anchor).await {
                Ok(VerificationOutcome::Match) => {
                    sqlx::query(
                        r#"
                        UPDATE content_anchors
                        SET verification_status = 'verified',
                            last_verified_at = NOW(),
                            updated_at = NOW()
                        WHERE content_hash = $1
                        "#,
                    )
                    .bind(&anchor.content_hash)
                    .execute(&self.pool)
                    .await?;
                }
                Ok(VerificationOutcome::Tampered { actual_hash }) => {
                    tamper_count += 1;
                    self.record_tamper(&anchor, Some(actual_hash)).await?;
                }
                Err(e) => {
                    warn!(
                        content_hash = %anchor.content_hash,
                        error = %e,
                        "Content unavailable for verification"
                    );
                    sqlx::query(
                        r#"
                        UPDATE content_anchors
                        SET verification_status = 'unavailable',
                            last_verified_at = NOW(),
                            updated_at = NOW()
                        WHERE content_hash = $1
                        "#,
                    )
                    .bind(&anchor.content_hash)
                    .execute(&self.pool)
                    .await?;
                }
            }
        }

        if tamper_count > 0 {
            info!("Tamper detection: {} anchor(s) failed verification", tamper_count);
        }

        Ok(tamper_count)
    }

    async fn verify_single(
        &self,
        anchor: &ContentAnchor,
    ) -> Result<VerificationOutcome, reqwest::Error> {
        let bytes = self.fetch_content(anchor).await?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Ok(VerificationOutcome::Tampered {
                actual_hash: hex::encode(Sha256::digest(&bytes)),
            });
        }
        let actual = hex::encode(Sha256::digest(&bytes));
        let expected = anchor.content_hash.trim_start_matches("0x").to_lowercase();
        if actual == expected {
            Ok(VerificationOutcome::Match)
        } else {
            Ok(VerificationOutcome::Tampered { actual_hash: actual })
        }
    }

    async fn fetch_content(&self, anchor: &ContentAnchor) -> Result<Vec<u8>, reqwest::Error> {
        let url = match anchor.storage_backend.as_str() {
            "ipfs" => format!(
                "{}{}",
                self.config.ipfs_gateway.trim_end_matches('/'),
                format!("/{}", anchor.cid.trim_start_matches('/'))
            ),
            "arweave" => format!(
                "{}{}",
                self.config.arweave_gateway.trim_end_matches('/'),
                format!("/{}", anchor.cid.trim_start_matches('/'))
            ),
            _ => return Ok(Vec::new()),
        };

        let response = self.http.get(&url).send().await?;
        response.error_for_status()?.bytes().await.map(|b| b.to_vec())
    }

    async fn record_tamper(
        &self,
        anchor: &ContentAnchor,
        actual_hash: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE content_anchors
            SET verification_status = 'tampered',
                tamper_alert_sent = TRUE,
                last_verified_at = NOW(),
                updated_at = NOW()
            WHERE content_hash = $1
            "#,
        )
        .bind(&anchor.content_hash)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO content_tamper_alerts (
                content_hash, expected_hash, actual_hash, cid,
                storage_backend, product_id, alert_sent
            )
            VALUES ($1, $2, $3, $4, $5, $6, TRUE)
            "#,
        )
        .bind(&anchor.content_hash)
        .bind(&anchor.content_hash)
        .bind(&actual_hash)
        .bind(&anchor.cid)
        .bind(&anchor.storage_backend)
        .bind(&anchor.product_id)
        .execute(&self.pool)
        .await?;

        let alert = TamperAlert {
            content_hash: anchor.content_hash.clone(),
            cid: anchor.cid.clone(),
            storage_backend: anchor.storage_backend.clone(),
            product_id: anchor.product_id.clone(),
            expected_hash: anchor.content_hash.clone(),
            actual_hash,
            detected_at: Utc::now(),
        };

        if let Err(e) = self.publish_tamper_alert(&alert).await {
            error!("Failed to publish tamper alert: {}", e);
        }

        Ok(())
    }

    async fn publish_tamper_alert(&self, alert: &TamperAlert) -> Result<(), redis::RedisError> {
        let payload = serde_json::to_string(alert).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "serialization",
                e.to_string(),
            ))
        })?;

        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let _: () = conn
                .publish("storage:tamper_alerts", payload)
                .await?;
            info!(
                "Tamper alert published for content_hash={} cid={}",
                alert.content_hash, alert.cid
            );
        }
        Ok(())
    }
}

impl Clone for StorageIntegrityService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            redis_client: self.redis_client.clone(),
            config: self.config.clone(),
            http: self.http.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_file_size_is_50mb() {
        assert_eq!(MAX_FILE_BYTES, 52_428_800);
    }

    #[test]
    fn storage_backend_parsing() {
        assert_eq!(StorageBackend::from_str("ipfs"), Some(StorageBackend::Ipfs));
        assert_eq!(
            StorageBackend::from_str("ARWEAVE"),
            Some(StorageBackend::Arweave)
        );
        assert!(StorageBackend::from_str("s3").is_none());
    }
}

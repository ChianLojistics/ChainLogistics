use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use rand::Rng;
use crate::database::ApiKeyRepository;
use crate::models::{ApiKey, NewApiKey, ApiKeyTier};

pub struct ApiKeyService {
    pub(crate) pool: PgPool,
}

impl ApiKeyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// SHA-256 hash of an API key. API keys are long random strings with sufficient
    /// entropy that bcrypt's computational cost is unnecessary and harmful to throughput.
    pub fn hash_api_key(api_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Generates a cryptographically secure API key: `cl_` prefix + 64 hex chars (256 bits entropy).
    pub fn generate_api_key() -> String {
        let bytes: [u8; 32] = rand::thread_rng().gen();
        format!("cl_{}", hex::encode(bytes))
    }

    pub async fn disable_inactive_keys(&self, inactive_days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = false
            WHERE is_active = true
              AND last_used_at IS NOT NULL
              AND last_used_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(inactive_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[async_trait]
impl ApiKeyRepository for ApiKeyService {
    async fn create_api_key(&self, api_key: NewApiKey) -> Result<ApiKey, sqlx::Error> {
        sqlx::query_as::<ApiKey, _>(
            r#"
            INSERT INTO api_keys (user_id, key_hash, name, tier, rate_limit_per_minute, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id, user_id, key_hash, name, tier, rate_limit_per_minute,
                is_active, expires_at, last_used_at, created_at
            "#,
        )
        .bind(api_key.user_id)
        .bind(api_key.key_hash)
        .bind(api_key.name)
        .bind(api_key.tier)
        .bind(api_key.rate_limit_per_minute)
        .bind(api_key.expires_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<ApiKey, _>(
            "SELECT id, user_id, key_hash, name, tier, rate_limit_per_minute, is_active, expires_at, last_used_at, created_at FROM api_keys WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<ApiKey, _>(
            "SELECT id, user_id, key_hash, name, tier, rate_limit_per_minute, is_active, expires_at, last_used_at, created_at FROM api_keys WHERE key_hash = $1 AND is_active = true",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as::<ApiKey, _>(
            "SELECT id, user_id, key_hash, name, tier, rate_limit_per_minute, is_active, expires_at, last_used_at, created_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_api_key(&self, id: Uuid, api_key: ApiKey) -> Result<ApiKey, sqlx::Error> {
        sqlx::query_as::<ApiKey, _>(
            r#"
            UPDATE api_keys SET
                name = $2,
                tier = $3,
                rate_limit_per_minute = $4,
                is_active = $5,
                expires_at = $6
            WHERE id = $1
            RETURNING
                id, user_id, key_hash, name, tier, rate_limit_per_minute,
                is_active, expires_at, last_used_at, created_at
            "#,
        )
        .bind(id)
        .bind(api_key.name)
        .bind(api_key.tier)
        .bind(api_key.rate_limit_per_minute)
        .bind(api_key.is_active)
        .bind(api_key.expires_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_api_key(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

use sqlx::{PgPool, Pool, Postgres};
use std::time::Duration;
use crate::config::DatabaseConfig;
use crate::models::*;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect_with(
            sqlx::postgres::PgConnectOptions::from_str(&config.url)?
                .acquire_timeout(Duration::from_secs(config.connect_timeout))
                .idle_timeout(Duration::from_secs(config.idle_timeout))
        ).await?;

        // Configure pool size
        pool.set_max_connections(config.max_connections);
        pool.set_min_connections(config.min_connections);

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    // Health check
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}

// Repository traits for data access
#[async_trait::async_trait]
pub trait ProductRepository {
    async fn create_product(&self, product: NewProduct) -> Result<Product, sqlx::Error>;
    async fn get_product(&self, id: &str) -> Result<Option<Product>, sqlx::Error>;
    async fn update_product(&self, id: &str, product: Product) -> Result<Product, sqlx::Error>;
    async fn delete_product(&self, id: &str) -> Result<(), sqlx::Error>;
    async fn list_products(
        &self,
        offset: i64,
        limit: i64,
        filters: Option<ProductFilters>,
    ) -> Result<Vec<Product>, sqlx::Error>;
    async fn count_products(&self, filters: Option<ProductFilters>) -> Result<i64, sqlx::Error>;
    async fn search_products(&self, query: &str, limit: i64) -> Result<Vec<Product>, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait EventRepository {
    async fn create_event(&self, event: NewTrackingEvent) -> Result<TrackingEvent, sqlx::Error>;
    async fn get_event(&self, id: i64) -> Result<Option<TrackingEvent>, sqlx::Error>;
    async fn list_events_by_product(
        &self,
        product_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error>;
    async fn count_events_by_product(&self, product_id: &str) -> Result<i64, sqlx::Error>;
    async fn list_events_by_type(
        &self,
        product_id: &str,
        event_type: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error>;
    async fn get_product_stats(&self, product_id: &str) -> Result<Option<ProductStats>, sqlx::Error>;
    async fn get_global_stats(&self) -> Result<GlobalStats, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait UserRepository {
    async fn create_user(&self, user: NewUser) -> Result<User, sqlx::Error>;
    async fn get_user(&self, id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn get_user_by_stellar_address(&self, address: &str) -> Result<Option<User>, sqlx::Error>;
    async fn update_user(&self, id: Uuid, user: User) -> Result<User, sqlx::Error>;
    async fn update_last_login(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

#[async_trait::async_trait]
pub trait ApiKeyRepository {
    async fn create_api_key(&self, api_key: NewApiKey) -> Result<ApiKey, sqlx::Error>;
    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>, sqlx::Error>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error>;
    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error>;
    async fn update_api_key(&self, id: Uuid, api_key: ApiKey) -> Result<ApiKey, sqlx::Error>;
    async fn update_last_used(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn revoke_api_key(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Clone, Default)]
pub struct ProductFilters {
    pub owner_address: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_products: i64,
    pub active_products: i64,
    pub total_events: i64,
    pub total_users: i64,
    pub active_api_keys: i64,
}
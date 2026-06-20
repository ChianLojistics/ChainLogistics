use async_trait::async_trait;
use sqlx::PgPool;
use redis::AsyncCommands;
use crate::database::{EventRepository, GlobalStats};
use crate::models::{TrackingEvent, NewTrackingEvent, ProductStats, AppError};

pub struct EventService {
    pub(crate) pool: PgPool,
    pub(crate) redis_client: redis::Client,
}

impl EventService {
    pub fn new(pool: PgPool, redis_client: redis::Client) -> Self {
        Self { pool, redis_client }
    }

    pub async fn invalidate_global_stats(&self) -> Result<(), AppError> {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let _: Result<(), _> = conn.del("cache:global_stats").await;
        }
        Ok(())
    }
}

#[async_trait]
impl EventRepository for EventService {
    async fn create_event(&self, event: NewTrackingEvent) -> Result<TrackingEvent, sqlx::Error> {
        let created = sqlx::query_as!(
            TrackingEvent,
            r#"
            INSERT INTO tracking_events (
                product_id, actor_address, timestamp, event_type,
                location, data_hash, note, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            event.product_id,
            event.actor_address,
            event.timestamp,
            event.event_type,
            event.location,
            event.data_hash,
            event.note,
            event.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.invalidate_global_stats().await;

        Ok(created)
    }

    async fn get_event(&self, id: i64) -> Result<Option<TrackingEvent>, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            "SELECT * FROM tracking_events WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_events_by_product(
        &self,
        product_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            "SELECT * FROM tracking_events WHERE product_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
            product_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn count_events_by_product(&self, product_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tracking_events WHERE product_id = $1",
            product_id
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0)
    }

    async fn list_events_by_type(
        &self,
        product_id: &str,
        event_type: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            "SELECT * FROM tracking_events WHERE product_id = $1 AND event_type = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4",
            product_id,
            event_type,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn get_product_stats(&self, product_id: &str) -> Result<Option<ProductStats>, sqlx::Error> {
        sqlx::query_as!(
            ProductStats,
            r#"
            SELECT
                p.id as product_id,
                (SELECT COUNT(*) FROM tracking_events WHERE product_id = p.id) as event_count,
                p.is_active,
                (SELECT MAX(timestamp) FROM tracking_events WHERE product_id = p.id) as last_event_at,
                (SELECT event_type FROM tracking_events WHERE product_id = p.id ORDER BY timestamp DESC LIMIT 1) as last_event_type
            FROM products p
            WHERE p.id = $1
            "#,
            product_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_global_stats(&self) -> Result<GlobalStats, sqlx::Error> {
        let cache_key = "cache:global_stats";

        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(cached) = conn.get::<_, String>(cache_key).await {
                if let Ok(stats) = serde_json::from_str::<GlobalStats>(&cached) {
                    return Ok(stats);
                }
            }
        }

        let stats = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM products) as total_products,
                (SELECT COUNT(*) FROM products WHERE is_active = true) as active_products,
                (SELECT COUNT(*) FROM tracking_events) as total_events,
                (SELECT COUNT(*) FROM users) as total_users,
                (SELECT COUNT(*) FROM api_keys WHERE is_active = true) as active_api_keys
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let global_stats = GlobalStats {
            total_products: stats.total_products.unwrap_or(0),
            active_products: stats.active_products.unwrap_or(0),
            total_events: stats.total_events.unwrap_or(0),
            total_users: stats.total_users.unwrap_or(0),
            active_api_keys: stats.active_api_keys.unwrap_or(0),
        };

        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(serialized) = serde_json::to_string(&global_stats) {
                let _: Result<(), _> = conn.set_ex(cache_key, serialized, 300).await;
            }
        }

        Ok(global_stats)
    }
}

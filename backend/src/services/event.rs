use async_trait::async_trait;
use sqlx::{PgPool, Row};
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
        let created = sqlx::query_as::<TrackingEvent, _>(
            r#"
            INSERT INTO tracking_events (
                product_id, actor_address, timestamp, event_type,
                location, data_hash, note, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id,
                product_id,
                actor_address,
                timestamp,
                event_type,
                location,
                data_hash,
                note,
                metadata,
                created_at
            "#,
        )
        .bind(event.product_id)
        .bind(event.actor_address)
        .bind(event.timestamp)
        .bind(event.event_type)
        .bind(event.location)
        .bind(event.data_hash)
        .bind(event.note)
        .bind(event.metadata)
        .fetch_one(&self.pool)
        .await?;

        let _ = self.invalidate_global_stats().await;

        Ok(created)
    }

        sqlx::query_as::<TrackingEvent, _>(
            "SELECT id, product_id, actor_address, timestamp, event_type, location, data_hash, note, metadata, created_at FROM tracking_events WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await

    async fn list_events_by_product(
        &self,
        product_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        sqlx::query_as::<TrackingEvent, _>(
            "SELECT id, product_id, actor_address, timestamp, event_type, location, data_hash, note, metadata, created_at FROM tracking_events WHERE product_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
        )
        .bind(product_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_events_by_product(&self, product_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<i64>(
            "SELECT COUNT(*) FROM tracking_events WHERE product_id = $1",
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
    }

    async fn list_events_by_type(
        &self,
        product_id: &str,
        event_type: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        sqlx::query_as::<TrackingEvent, _>(
            "SELECT id, product_id, actor_address, timestamp, event_type, location, data_hash, note, metadata, created_at FROM tracking_events WHERE product_id = $1 AND event_type = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4",
        )
        .bind(product_id)
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_product_stats(&self, product_id: &str) -> Result<Option<ProductStats>, sqlx::Error> {
        sqlx::query_as::<ProductStats, _>(
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
        )
        .bind(product_id)
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

        let stats = sqlx::query(
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
            total_products: stats.get::<Option<i64>, _>("total_products").unwrap_or(Some(0)).unwrap_or(0),
            active_products: stats.get::<Option<i64>, _>("active_products").unwrap_or(Some(0)).unwrap_or(0),
            total_events: stats.get::<Option<i64>, _>("total_events").unwrap_or(Some(0)).unwrap_or(0),
            total_users: stats.get::<Option<i64>, _>("total_users").unwrap_or(Some(0)).unwrap_or(0),
            active_api_keys: stats.get::<Option<i64>, _>("active_api_keys").unwrap_or(Some(0)).unwrap_or(0),
        };

        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(serialized) = serde_json::to_string(&global_stats) {
                let _: Result<(), _> = conn.set_ex(cache_key, serialized, 300).await;
            }
        }

        Ok(global_stats)
    }
}

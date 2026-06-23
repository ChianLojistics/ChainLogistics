use crate::models::{Recall, RecallAffectedItem, RecallEffectiveness, RecallNotification};
use sqlx::PgPool;
use uuid::Uuid;

pub struct RecallService {
    pool: PgPool,
}

impl RecallService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_recall(
        &self,
        product_id: &str,
        batch_id: Option<&str>,
        title: &str,
        reason: &str,
        severity: &str,
        trigger_type: &str,
        triggered_by: Option<&str>,
        triggered_event_id: Option<i64>,
        metadata: serde_json::Value,
    ) -> Result<Recall, sqlx::Error> {
        let recall = sqlx::query_as::<Recall, _>(
            r#"
            INSERT INTO recalls (
                product_id, batch_id, title, reason, severity, status,
                trigger_type, triggered_by, triggered_event_id, metadata
            ) VALUES ($1, $2, $3, $4, $5, 'open', $6, $7, $8, $9)
            RETURNING
                id,
                product_id,
                batch_id,
                title,
                reason,
                severity,
                status,
                trigger_type,
                triggered_by,
                triggered_event_id,
                started_at,
                closed_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(product_id)
        .bind(batch_id)
        .bind(title)
        .bind(reason)
        .bind(severity)
        .bind(trigger_type)
        .bind(triggered_by)
        .bind(triggered_event_id)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO recall_effectiveness (recall_id)
            VALUES ($1)
            ON CONFLICT (recall_id) DO NOTHING
            "#,
        )
        .bind(recall.id)
        .execute(&self.pool)
        .await?;

        Ok(recall)
    }

    pub async fn identify_affected_items(
        &self,
        recall_id: Uuid,
        product_id: &str,
        batch_id: Option<&str>,
    ) -> Result<Vec<RecallAffectedItem>, sqlx::Error> {
        sqlx::query(
            r#"
            WITH affected_products AS (
                SELECT DISTINCT p.id AS product_id
                FROM products p
                WHERE ($2::TEXT IS NULL)
                   OR (p.custom_fields->>'batch_id') = $2
                UNION
                SELECT DISTINCT e.product_id AS product_id
                FROM tracking_events e
                WHERE ($2::TEXT IS NULL)
                   OR (e.metadata->>'batch_id') = $2
            )
            INSERT INTO recall_affected_items (
                recall_id, product_id, batch_id, stakeholder_role, stakeholder_address, detected_via
            )
            SELECT
                $1,
                ap.product_id,
                $2,
                NULL,
                NULL,
                'metadata'
            FROM affected_products ap
            WHERE ap.product_id = $3
               OR ($2::TEXT IS NOT NULL AND ap.product_id IS NOT NULL)
            ON CONFLICT DO NOTHING
            RETURNING id
            "#,
        )
        .bind(recall_id)
        .bind(batch_id)
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;

        let items = sqlx::query_as::<RecallAffectedItem, _>(
            r#"
            SELECT
                id,
                recall_id,
                product_id,
                batch_id,
                stakeholder_role,
                stakeholder_address,
                detected_via,
                created_at
            FROM recall_affected_items
            WHERE recall_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(recall_id)
        .fetch_all(&self.pool)
        .await?;

        let affected_count = items.len() as i32;
        sqlx::query(
            r#"
            UPDATE recall_effectiveness
            SET affected_count = $2,
                last_updated_at = NOW()
            WHERE recall_id = $1
            "#,
        )
        .bind(recall_id)
        .bind(affected_count)
        .execute(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn queue_notifications(
        &self,
        recall_id: Uuid,
        recipients: Vec<String>,
        channel: &str,
        payload: serde_json::Value,
    ) -> Result<Vec<RecallNotification>, sqlx::Error> {
        for recipient in &recipients {
            sqlx::query(
                r#"
                INSERT INTO recall_notifications (recall_id, recipient, channel, status, payload)
                VALUES ($1, $2, $3, 'queued', $4)
                "#,
            )
            .bind(recall_id)
            .bind(recipient)
            .bind(channel)
            .bind(&payload)
            .execute(&self.pool)
            .await?;
        }

        let notifications = sqlx::query_as::<RecallNotification, _>(
            r#"
            SELECT
                id,
                recall_id,
                recipient,
                channel,
                status,
                sent_at,
                acknowledged_at,
                payload,
                error,
                created_at
            FROM recall_notifications
            WHERE recall_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(recall_id)
        .fetch_all(&self.pool)
        .await?;

        let notified_count = notifications.len() as i32;
        sqlx::query(
            r#"
            UPDATE recall_effectiveness
            SET notified_count = $2,
                last_updated_at = NOW()
            WHERE recall_id = $1
            "#,
        )
        .bind(recall_id)
        .bind(notified_count)
        .execute(&self.pool)
        .await?;

        Ok(notifications)
    }

    pub async fn list_recalls_by_product(
        &self,
        product_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Recall>, sqlx::Error> {
        sqlx::query_as::<Recall, _>(
            r#"
            SELECT
                id,
                product_id,
                batch_id,
                title,
                reason,
                severity,
                status,
                trigger_type,
                triggered_by,
                triggered_event_id,
                started_at,
                closed_at,
                metadata,
                created_at,
                updated_at
            FROM recalls
            WHERE product_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(product_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_recall(&self, recall_id: Uuid) -> Result<Option<Recall>, sqlx::Error> {
        sqlx::query_as::<Recall, _>(
            r#"
            SELECT
                id,
                product_id,
                batch_id,
                title,
                reason,
                severity,
                status,
                trigger_type,
                triggered_by,
                triggered_event_id,
                started_at,
                closed_at,
                metadata,
                created_at,
                updated_at
            FROM recalls
            WHERE id = $1
            "#,
        )
        .bind(recall_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_effectiveness(
        &self,
        recall_id: Uuid,
    ) -> Result<Option<RecallEffectiveness>, sqlx::Error> {
        sqlx::query_as::<RecallEffectiveness, _>(
            r#"
            SELECT
                recall_id,
                affected_count,
                notified_count,
                acknowledged_count,
                recovered_count,
                disposed_count,
                last_updated_at
            FROM recall_effectiveness
            WHERE recall_id = $1
            "#,
        )
        .bind(recall_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_effectiveness(
        &self,
        recall_id: Uuid,
        acknowledged_delta: i32,
        recovered_delta: i32,
        disposed_delta: i32,
    ) -> Result<RecallEffectiveness, sqlx::Error> {
        sqlx::query_as::<RecallEffectiveness, _>(
            r#"
            UPDATE recall_effectiveness
            SET acknowledged_count = GREATEST(0, acknowledged_count + $2),
                recovered_count = GREATEST(0, recovered_count + $3),
                disposed_count = GREATEST(0, disposed_count + $4),
                last_updated_at = NOW()
            WHERE recall_id = $1
            RETURNING
                recall_id,
                affected_count,
                notified_count,
                acknowledged_count,
                recovered_count,
                disposed_count,
                last_updated_at
            "#,
        )
        .bind(recall_id)
        .bind(acknowledged_delta)
        .bind(recovered_delta)
        .bind(disposed_delta)
        .fetch_one(&self.pool)
        .await
    }
}

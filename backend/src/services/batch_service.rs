use crate::models::batch::*;
use async_trait::async_trait;
use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

pub struct BatchService {
    pool: PgPool,
    redis_client: redis::Client,
}

impl BatchService {
    pub fn new(pool: PgPool, redis_client: redis::Client) -> Self {
        Self { pool, redis_client }
    }

    async fn invalidate_batch_cache(&self, batch_id: &Uuid) {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let _: Result<(), _> = conn.del(format!("cache:batch:{}", batch_id)).await;
        }
    }
}

#[async_trait]
pub trait BatchRepository {
    async fn create_batch(&self, batch: NewBatch) -> Result<Batch, sqlx::Error>;
    async fn get_batch(&self, id: Uuid) -> Result<Option<Batch>, sqlx::Error>;
    async fn get_batch_by_number(&self, batch_number: &str) -> Result<Option<Batch>, sqlx::Error>;
    async fn update_batch(&self, id: Uuid, batch: Batch) -> Result<Batch, sqlx::Error>;
    async fn delete_batch(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn list_batches(
        &self,
        offset: i64,
        limit: i64,
        filters: Option<BatchFilters>,
    ) -> Result<Vec<Batch>, sqlx::Error>;
    async fn count_batches(&self, filters: Option<BatchFilters>) -> Result<i64, sqlx::Error>;

    // Genealogy
    async fn create_genealogy(
        &self,
        genealogy: NewBatchGenealogy,
    ) -> Result<BatchGenealogy, sqlx::Error>;
    async fn get_genealogy_tree(&self, batch_id: Uuid) -> Result<BatchGenealogyTree, sqlx::Error>;
    async fn get_batch_parents(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchGenealogyNode>, sqlx::Error>;
    async fn get_batch_children(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchGenealogyNode>, sqlx::Error>;

    // Quality attributes
    async fn create_quality_attribute(
        &self,
        attribute: NewBatchQualityAttribute,
    ) -> Result<BatchQualityAttribute, sqlx::Error>;
    async fn get_quality_attributes(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchQualityAttribute>, sqlx::Error>;
    async fn update_quality_attribute(
        &self,
        id: Uuid,
        attribute: BatchQualityAttribute,
    ) -> Result<BatchQualityAttribute, sqlx::Error>;

    // Recalls
    async fn create_recall(&self, recall: NewBatchRecall) -> Result<BatchRecall, sqlx::Error>;
    async fn get_recall(&self, id: Uuid) -> Result<Option<BatchRecall>, sqlx::Error>;
    async fn get_batch_recalls(&self, batch_id: Uuid) -> Result<Vec<BatchRecall>, sqlx::Error>;
    async fn update_recall(
        &self,
        id: Uuid,
        recall: BatchRecall,
    ) -> Result<BatchRecall, sqlx::Error>;
    async fn list_active_recalls(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<BatchRecall>, sqlx::Error>;

    // Inventory
    async fn create_inventory_transaction(
        &self,
        inventory: NewBatchInventory,
    ) -> Result<BatchInventory, sqlx::Error>;
    async fn get_batch_inventory(&self, batch_id: Uuid)
        -> Result<Vec<BatchInventory>, sqlx::Error>;
    async fn get_inventory_by_location(
        &self,
        location_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<BatchInventory>, sqlx::Error>;
}

#[async_trait]
impl BatchRepository for BatchService {
    async fn create_batch(&self, batch: NewBatch) -> Result<Batch, sqlx::Error> {
        let created = sqlx::query_as::<Batch, _>(
            r#"
            INSERT INTO batches (
                batch_number, product_id, lot_number, production_date, expiry_date,
                quantity_produced, quantity_available, status, production_location,
                quality_grade, quality_score, metadata, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
            RETURNING
                id, batch_number, product_id, lot_number, production_date, expiry_date,
                quantity_produced, quantity_available, status as "status: BatchStatus",
                production_location, quality_grade, quality_score, metadata,
                created_by, updated_by, created_at, updated_at
            "#,
        )
        .bind(batch.batch_number)
        .bind(batch.product_id)
        .bind(batch.lot_number)
        .bind(batch.production_date)
        .bind(batch.expiry_date)
        .bind(batch.quantity_produced)
        .bind(batch.quantity_available)
        .bind(batch.status as BatchStatus)
        .bind(batch.production_location)
        .bind(batch.quality_grade)
        .bind(batch.quality_score)
        .bind(batch.metadata)
        .bind(batch.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn get_batch(&self, id: Uuid) -> Result<Option<Batch>, sqlx::Error> {
        let cache_key = format!("cache:batch:{}", id);

        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(batch) = serde_json::from_str::<Batch>(&cached) {
                    return Ok(Some(batch));
                }
            }
        }

        let batch = sqlx::query_as::<Batch, _>(
            "SELECT id, batch_number, product_id, lot_number, production_date, expiry_date, quantity_produced, quantity_available, status as \"status: BatchStatus\", production_location, quality_grade, quality_score, metadata, created_by, updated_by, created_at, updated_at FROM batches WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref b) = batch {
            if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
                if let Ok(serialized) = serde_json::to_string(b) {
                    let _: Result<(), _> = conn.set_ex(&cache_key, serialized, 3600).await;
                }
            }
        }

        Ok(batch)
    }

    async fn get_batch_by_number(&self, batch_number: &str) -> Result<Option<Batch>, sqlx::Error> {
        sqlx::query_as::<Batch, _>(
            "SELECT id, batch_number, product_id, lot_number, production_date, expiry_date, quantity_produced, quantity_available, status as \"status: BatchStatus\", production_location, quality_grade, quality_score, metadata, created_by, updated_by, created_at, updated_at FROM batches WHERE batch_number = $1",
        )
        .bind(batch_number)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_batch(&self, id: Uuid, batch: Batch) -> Result<Batch, sqlx::Error> {
        let updated = sqlx::query_as::<Batch, _>(
            r#"
            UPDATE batches SET
                batch_number = $2,
                product_id = $3,
                lot_number = $4,
                production_date = $5,
                expiry_date = $6,
                quantity_produced = $7,
                quantity_available = $8,
                status = $9,
                production_location = $10,
                quality_grade = $11,
                quality_score = $12,
                metadata = $13,
                updated_by = $14
            WHERE id = $1
            RETURNING
                id, batch_number, product_id, lot_number, production_date, expiry_date,
                quantity_produced, quantity_available, status as "status: BatchStatus",
                production_location, quality_grade, quality_score, metadata,
                created_by, updated_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(batch.batch_number)
        .bind(batch.product_id)
        .bind(batch.lot_number)
        .bind(batch.production_date)
        .bind(batch.expiry_date)
        .bind(batch.quantity_produced)
        .bind(batch.quantity_available)
        .bind(batch.status as BatchStatus)
        .bind(batch.production_location)
        .bind(batch.quality_grade)
        .bind(batch.quality_score)
        .bind(batch.metadata)
        .bind(batch.updated_by)
        .fetch_one(&self.pool)
        .await?;

        self.invalidate_batch_cache(&id).await;

        Ok(updated)
    }

    async fn delete_batch(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM batches WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.invalidate_batch_cache(&id).await;

        Ok(())
    }

    async fn list_batches(
        &self,
        offset: i64,
        limit: i64,
        filters: Option<BatchFilters>,
    ) -> Result<Vec<Batch>, sqlx::Error> {
        let mut query = "SELECT * FROM batches WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(f) = filters {
            if let Some(product_id) = f.product_id {
                query.push_str(&format!(" AND product_id = ${}", bind_index));
                bindings.push(product_id);
                bind_index += 1;
            }
            if let Some(lot_number) = f.lot_number {
                query.push_str(&format!(" AND lot_number = ${}", bind_index));
                bindings.push(lot_number);
                bind_index += 1;
            }
            if let Some(status) = f.status {
                query.push_str(&format!(" AND status = ${}", bind_index));
                bindings.push(status.to_string());
                bind_index += 1;
            }
            if let Some(quality_grade) = f.quality_grade {
                query.push_str(&format!(" AND quality_grade = ${}", bind_index));
                bindings.push(quality_grade);
                bind_index += 1;
            }
            if let Some(after) = f.production_after {
                query.push_str(&format!(" AND production_date >= ${}", bind_index));
                bindings.push(after.to_rfc3339());
                bind_index += 1;
            }
            if let Some(before) = f.production_before {
                query.push_str(&format!(" AND production_date <= ${}", bind_index));
                bindings.push(before.to_rfc3339());
                bind_index += 1;
            }
            if let Some(after) = f.expiry_after {
                query.push_str(&format!(" AND expiry_date >= ${}", bind_index));
                bindings.push(after.to_rfc3339());
                bind_index += 1;
            }
            if let Some(before) = f.expiry_before {
                query.push_str(&format!(" AND expiry_date <= ${}", bind_index));
                bindings.push(before.to_rfc3339());
                bind_index += 1;
            }
        }

        query.push_str(&format!(
            " ORDER BY production_date DESC LIMIT ${} OFFSET ${}",
            bind_index,
            bind_index + 1
        ));
        bindings.push(limit.to_string());
        bindings.push(offset.to_string());

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<Batch>().fetch_all(&self.pool).await
    }

    async fn count_batches(&self, filters: Option<BatchFilters>) -> Result<i64, sqlx::Error> {
        let mut query = "SELECT COUNT(*) FROM batches WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(f) = filters {
            if let Some(product_id) = f.product_id {
                query.push_str(&format!(" AND product_id = ${}", bind_index));
                bindings.push(product_id);
                bind_index += 1;
            }
            if let Some(lot_number) = f.lot_number {
                query.push_str(&format!(" AND lot_number = ${}", bind_index));
                bindings.push(lot_number);
                bind_index += 1;
            }
            if let Some(status) = f.status {
                query.push_str(&format!(" AND status = ${}", bind_index));
                bindings.push(status.to_string());
                bind_index += 1;
            }
            if let Some(quality_grade) = f.quality_grade {
                query.push_str(&format!(" AND quality_grade = ${}", bind_index));
                bindings.push(quality_grade);
                bind_index += 1;
            }
            if let Some(after) = f.production_after {
                query.push_str(&format!(" AND production_date >= ${}", bind_index));
                bindings.push(after.to_rfc3339());
                bind_index += 1;
            }
            if let Some(before) = f.production_before {
                query.push_str(&format!(" AND production_date <= ${}", bind_index));
                bindings.push(before.to_rfc3339());
                bind_index += 1;
            }
            if let Some(after) = f.expiry_after {
                query.push_str(&format!(" AND expiry_date >= ${}", bind_index));
                bindings.push(after.to_rfc3339());
                bind_index += 1;
            }
            if let Some(before) = f.expiry_before {
                query.push_str(&format!(" AND expiry_date <= ${}", bind_index));
                bindings.push(before.to_rfc3339());
                bind_index += 1;
            }
        }

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_scalar::<i64>().fetch_one(&self.pool).await
    }

    async fn create_genealogy(
        &self,
        genealogy: NewBatchGenealogy,
    ) -> Result<BatchGenealogy, sqlx::Error> {
        let created = sqlx::query_as::<BatchGenealogy, _>(
            r#"
            INSERT INTO batch_genealogy (
                parent_batch_id, child_batch_id, relationship_type,
                quantity_transferred, notes, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, parent_batch_id, child_batch_id, relationship_type, quantity_transferred, notes, metadata, created_at
            "#,
        )
        .bind(genealogy.parent_batch_id)
        .bind(genealogy.child_batch_id)
        .bind(genealogy.relationship_type)
        .bind(genealogy.quantity_transferred)
        .bind(genealogy.notes)
        .bind(genealogy.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn get_genealogy_tree(&self, batch_id: Uuid) -> Result<BatchGenealogyTree, sqlx::Error> {
        let batch = self
            .get_batch(batch_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let parents = self.get_batch_parents(batch_id).await?;
        let children = self.get_batch_children(batch_id).await?;

        Ok(BatchGenealogyTree {
            batch,
            parents,
            children,
        })
    }

    async fn get_batch_parents(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchGenealogyNode>, sqlx::Error> {
        let genealogies = sqlx::query_as::<BatchGenealogy, _>(
            "SELECT id, parent_batch_id, child_batch_id, relationship_type, quantity_transferred, notes, metadata, created_at FROM batch_genealogy WHERE child_batch_id = $1",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await?;

        let mut nodes = Vec::new();
        for genealogy in genealogies {
            if let Some(related_batch) = self.get_batch(genealogy.parent_batch_id).await? {
                nodes.push(BatchGenealogyNode {
                    genealogy,
                    related_batch,
                });
            }
        }

        Ok(nodes)
    }

    async fn get_batch_children(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchGenealogyNode>, sqlx::Error> {
        let genealogies = sqlx::query_as::<BatchGenealogy, _>(
            "SELECT id, parent_batch_id, child_batch_id, relationship_type, quantity_transferred, notes, metadata, created_at FROM batch_genealogy WHERE parent_batch_id = $1",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await?;

        let mut nodes = Vec::new();
        for genealogy in genealogies {
            if let Some(related_batch) = self.get_batch(genealogy.child_batch_id).await? {
                nodes.push(BatchGenealogyNode {
                    genealogy,
                    related_batch,
                });
            }
        }

        Ok(nodes)
    }

    async fn create_quality_attribute(
        &self,
        attribute: NewBatchQualityAttribute,
    ) -> Result<BatchQualityAttribute, sqlx::Error> {
        let is_within_tolerance = if let (Some(min), Some(max), Some(value)) = (
            &attribute.tolerance_min,
            &attribute.tolerance_max,
            &attribute.attribute_value,
        ) {
            if let Ok(parsed_value) = value.parse::<rust_decimal::Decimal>() {
                parsed_value >= *min && parsed_value <= *max
            } else {
                true
            }
        } else {
            true
        };

        let created = sqlx::query_as::<BatchQualityAttribute, _>(
            r#"
            INSERT INTO batch_quality_attributes (
                batch_id, attribute_name, attribute_value, measurement_unit,
                tolerance_min, tolerance_max, measured_by, notes, is_within_tolerance
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, batch_id, attribute_name, attribute_value, measurement_unit, tolerance_min, tolerance_max, measured_at, measured_by, notes, is_within_tolerance
            "#,
        )
        .bind(attribute.batch_id)
        .bind(attribute.attribute_name)
        .bind(attribute.attribute_value)
        .bind(attribute.measurement_unit)
        .bind(attribute.tolerance_min)
        .bind(attribute.tolerance_max)
        .bind(attribute.measured_by)
        .bind(attribute.notes)
        .bind(is_within_tolerance)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn get_quality_attributes(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchQualityAttribute>, sqlx::Error> {
        sqlx::query_as::<BatchQualityAttribute, _>(
            "SELECT id, batch_id, attribute_name, attribute_value, measurement_unit, tolerance_min, tolerance_max, measured_at, measured_by, notes, is_within_tolerance FROM batch_quality_attributes WHERE batch_id = $1 ORDER BY measured_at DESC",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_quality_attribute(
        &self,
        id: Uuid,
        attribute: BatchQualityAttribute,
    ) -> Result<BatchQualityAttribute, sqlx::Error> {
        sqlx::query_as::<BatchQualityAttribute, _>(
            r#"
            UPDATE batch_quality_attributes SET
                attribute_name = $2,
                attribute_value = $3,
                measurement_unit = $4,
                tolerance_min = $5,
                tolerance_max = $6,
                measured_by = $7,
                notes = $8,
                is_within_tolerance = $9
            WHERE id = $1
            RETURNING id, batch_id, attribute_name, attribute_value, measurement_unit, tolerance_min, tolerance_max, measured_at, measured_by, notes, is_within_tolerance
            "#,
        )
        .bind(id)
        .bind(attribute.attribute_name)
        .bind(attribute.attribute_value)
        .bind(attribute.measurement_unit)
        .bind(attribute.tolerance_min)
        .bind(attribute.tolerance_max)
        .bind(attribute.measured_by)
        .bind(attribute.notes)
        .bind(attribute.is_within_tolerance)
        .fetch_one(&self.pool)
        .await
    }

    async fn create_recall(&self, recall: NewBatchRecall) -> Result<BatchRecall, sqlx::Error> {
        let created = sqlx::query_as::<BatchRecall, _>(
            r#"
            INSERT INTO batch_recalls (
                batch_id, recall_type, recall_reason, initiated_by,
                severity, affected_quantity, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, batch_id, recall_date, recall_type, recall_reason, initiated_by, severity, affected_quantity, recovered_quantity, status as "status: RecallStatus", notification_sent, public_announcement, metadata, created_at, updated_at
            "#,
        )
        .bind(recall.batch_id)
        .bind(recall.recall_type)
        .bind(recall.recall_reason)
        .bind(recall.initiated_by)
        .bind(recall.severity)
        .bind(recall.affected_quantity)
        .bind(recall.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn get_recall(&self, id: Uuid) -> Result<Option<BatchRecall>, sqlx::Error> {
        sqlx::query_as::<BatchRecall, _>(
            "SELECT id, batch_id, recall_date, recall_type, recall_reason, initiated_by, severity, affected_quantity, recovered_quantity, status as \"status: RecallStatus\", notification_sent, public_announcement, metadata, created_at, updated_at FROM batch_recalls WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_batch_recalls(&self, batch_id: Uuid) -> Result<Vec<BatchRecall>, sqlx::Error> {
        sqlx::query_as::<BatchRecall, _>(
            "SELECT id, batch_id, recall_date, recall_type, recall_reason, initiated_by, severity, affected_quantity, recovered_quantity, status as \"status: RecallStatus\", notification_sent, public_announcement, metadata, created_at, updated_at FROM batch_recalls WHERE batch_id = $1 ORDER BY recall_date DESC",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_recall(
        &self,
        id: Uuid,
        recall: BatchRecall,
    ) -> Result<BatchRecall, sqlx::Error> {
        sqlx::query_as::<BatchRecall, _>(
            r#"
            UPDATE batch_recalls SET
                status = $2,
                recovered_quantity = $3,
                notification_sent = $4,
                public_announcement = $5
            WHERE id = $1
            RETURNING id, batch_id, recall_date, recall_type, recall_reason, initiated_by, severity, affected_quantity, recovered_quantity, status as "status: RecallStatus", notification_sent, public_announcement, metadata, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(recall.status)
        .bind(recall.recovered_quantity)
        .bind(recall.notification_sent)
        .bind(recall.public_announcement)
        .fetch_one(&self.pool)
        .await
    }

    async fn list_active_recalls(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<BatchRecall>, sqlx::Error> {
        sqlx::query_as::<BatchRecall, _>(
            "SELECT id, batch_id, recall_date, recall_type, recall_reason, initiated_by, severity, affected_quantity, recovered_quantity, status as \"status: RecallStatus\", notification_sent, public_announcement, metadata, created_at, updated_at FROM batch_recalls WHERE status = 'active' ORDER BY recall_date DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn create_inventory_transaction(
        &self,
        inventory: NewBatchInventory,
    ) -> Result<BatchInventory, sqlx::Error> {
        let created = sqlx::query_as::<BatchInventory, _>(
            r#"
            INSERT INTO batch_inventory (
                batch_id, location_id, quantity, transaction_type,
                reference_id, performed_by, notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, batch_id, location_id, quantity, transaction_type, transaction_date, reference_id, performed_by, notes, created_at
            "#,
        )
        .bind(inventory.batch_id)
        .bind(inventory.location_id)
        .bind(inventory.quantity)
        .bind(inventory.transaction_type)
        .bind(inventory.reference_id)
        .bind(inventory.performed_by)
        .bind(inventory.notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn get_batch_inventory(
        &self,
        batch_id: Uuid,
    ) -> Result<Vec<BatchInventory>, sqlx::Error> {
        sqlx::query_as::<BatchInventory, _>(
            "SELECT id, batch_id, location_id, quantity, transaction_type, transaction_date, reference_id, performed_by, notes, created_at FROM batch_inventory WHERE batch_id = $1 ORDER BY transaction_date DESC",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_inventory_by_location(
        &self,
        location_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<BatchInventory>, sqlx::Error> {
        sqlx::query_as::<BatchInventory, _>(
            "SELECT id, batch_id, location_id, quantity, transaction_type, transaction_date, reference_id, performed_by, notes, created_at FROM batch_inventory WHERE location_id = $1 ORDER BY transaction_date DESC LIMIT $2 OFFSET $3",
        )
        .bind(location_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }
}

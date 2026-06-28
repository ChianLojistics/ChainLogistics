use crate::error::AppError;
use crate::saga::state::SagaState;
use async_trait::async_trait;
use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait SagaPersistence: Send + Sync {
    async fn save(&self, saga: &SagaState) -> Result<(), AppError>;
    async fn load(&self, saga_id: Uuid) -> Result<Option<SagaState>, AppError>;
    async fn delete(&self, saga_id: Uuid) -> Result<(), AppError>;
    async fn list_active(&self) -> Result<Vec<SagaState>, AppError>;
}

pub struct PostgresSagaPersistence {
    pool: PgPool,
}

impl PostgresSagaPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaPersistence for PostgresSagaPersistence {
    async fn save(&self, saga: &SagaState) -> Result<(), AppError> {
        let steps_json = serde_json::to_string(&saga.steps)
            .map_err(|e| AppError::Internal(format!("Failed to serialize steps: {}", e)))?;
        let metadata_json = serde_json::to_value(&saga.metadata)
            .map_err(|e| AppError::Internal(format!("Failed to serialize metadata: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO saga_states (id, name, status, steps, current_step_index, created_at, updated_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status,
                steps = EXCLUDED.steps,
                current_step_index = EXCLUDED.current_step_index,
                updated_at = EXCLUDED.updated_at,
                metadata = EXCLUDED.metadata
            "#,
        )
        .bind(saga.id)
        .bind(&saga.name)
        .bind(saga.status.to_string())
        .bind(&steps_json)
        .bind(saga.current_step_index as i64)
        .bind(saga.created_at)
        .bind(saga.updated_at)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load(&self, saga_id: Uuid) -> Result<Option<SagaState>, AppError> {
        let row = sqlx::query_as::<_, (String, String, String, i64, i64, i64, serde_json::Value)>(
            "SELECT name, status, steps, current_step_index, created_at, updated_at, metadata FROM saga_states WHERE id = $1"
        )
        .bind(saga_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((name, status, steps, current_step_index, created_at, updated_at, metadata)) = row {
            let steps: Vec<crate::saga::state::SagaStep> = serde_json::from_str(&steps)
                .map_err(|e| AppError::Internal(format!("Failed to deserialize steps: {}", e)))?;
            let metadata: std::collections::HashMap<String, serde_json::Value> = serde_json::from_value(metadata)
                .map_err(|e| AppError::Internal(format!("Failed to deserialize metadata: {}", e)))?;

            let status = match status.as_str() {
                "Pending" => crate::saga::state::SagaStatus::Pending,
                "InProgress" => crate::saga::state::SagaStatus::InProgress,
                "Completed" => crate::saga::state::SagaStatus::Completed,
                "Failed" => crate::saga::state::SagaStatus::Failed,
                "Compensating" => crate::saga::state::SagaStatus::Compensating,
                "Compensated" => crate::saga::state::SagaStatus::Compensated,
                _ => return Err(AppError::Internal(format!("Invalid status: {}", status))),
            };

            Ok(Some(SagaState {
                id: saga_id,
                name,
                status,
                steps,
                current_step_index: current_step_index as usize,
                created_at,
                updated_at,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, saga_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM saga_states WHERE id = $1")
            .bind(saga_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_active(&self) -> Result<Vec<SagaState>, AppError> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, i64, i64, i64, serde_json::Value)>(
            "SELECT id, name, status, steps, current_step_index, created_at, updated_at, metadata FROM saga_states WHERE status IN ('InProgress', 'Failed', 'Compensating')"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut sagas = Vec::new();
        for (id, name, status, steps, current_step_index, created_at, updated_at, metadata) in rows {
            let steps: Vec<crate::saga::state::SagaStep> = serde_json::from_str(&steps)
                .map_err(|e| AppError::Internal(format!("Failed to deserialize steps: {}", e)))?;
            let metadata: std::collections::HashMap<String, serde_json::Value> = serde_json::from_value(metadata)
                .map_err(|e| AppError::Internal(format!("Failed to deserialize metadata: {}", e)))?;

            let status = match status.as_str() {
                "Pending" => crate::saga::state::SagaStatus::Pending,
                "InProgress" => crate::saga::state::SagaStatus::InProgress,
                "Completed" => crate::saga::state::SagaStatus::Completed,
                "Failed" => crate::saga::state::SagaStatus::Failed,
                "Compensating" => crate::saga::state::SagaStatus::Compensating,
                "Compensated" => crate::saga::state::SagaStatus::Compensated,
                _ => return Err(AppError::Internal(format!("Invalid status: {}", status))),
            };

            sagas.push(SagaState {
                id,
                name,
                status,
                steps,
                current_step_index: current_step_index as usize,
                created_at,
                updated_at,
                metadata,
            });
        }

        Ok(sagas)
    }
}

pub struct RedisSagaPersistence {
    client: redis::Client,
}

impl RedisSagaPersistence {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SagaPersistence for RedisSagaPersistence {
    async fn save(&self, saga: &SagaState) -> Result<(), AppError> {
        let key = format!("saga:{}", saga.id);
        let data = serde_json::to_string(saga)
            .map_err(|e| AppError::Internal(format!("Failed to serialize saga: {}", e)))?;

        let mut conn = self.client.get_multiplexed_tokio_connection().await?;
        conn.set_ex(&key, data, 86400).await?; // 24 hour TTL
        Ok(())
    }

    async fn load(&self, saga_id: Uuid) -> Result<Option<SagaState>, AppError> {
        let key = format!("saga:{}", saga_id);
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        let data: Option<String> = conn.get(&key).await?;
        if let Some(data) = data {
            let saga: SagaState = serde_json::from_str(&data)
                .map_err(|e| AppError::Internal(format!("Failed to deserialize saga: {}", e)))?;
            Ok(Some(saga))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, saga_id: Uuid) -> Result<(), AppError> {
        let key = format!("saga:{}", saga_id);
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;
        conn.del(&key).await?;
        Ok(())
    }

    async fn list_active(&self) -> Result<Vec<SagaState>, AppError> {
        let pattern = "saga:*";
        let mut conn = self.client.get_multiplexed_tokio_connection().await?;

        let keys: Vec<String> = conn.keys(pattern).await?;
        let mut sagas = Vec::new();

        for key in keys {
            let data: Option<String> = conn.get(&key).await?;
            if let Some(data) = data {
                if let Ok(saga) = serde_json::from_str::<SagaState>(&data) {
                    if matches!(saga.status, crate::saga::state::SagaStatus::InProgress | crate::saga::state::SagaStatus::Failed | crate::saga::state::SagaStatus::Compensating) {
                        sagas.push(saga);
                    }
                }
            }
        }

        Ok(sagas)
    }
}

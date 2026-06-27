use crate::error::AppError;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Saga state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SagaState {
    Pending,
    InProgress,
    Compensating,
    Completed,
    Failed,
    Aborted,
}

/// Saga step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub id: String,
    pub name: String,
    pub execute_action: String,
    pub compensate_action: String,
    pub timeout_ms: u64,
    pub retry_policy: RetryPolicy,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    pub exponential_backoff: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_ms: 1000,
            exponential_backoff: true,
        }
    }
}

/// Saga instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    pub id: Uuid,
    pub saga_type: String,
    pub state: SagaState,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
    pub failed_step: Option<String>,
    pub context: serde_json::Value,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Saga execution result
#[derive(Debug, Clone)]
pub struct SagaResult {
    pub saga_id: Uuid,
    pub state: SagaState,
    pub completed_steps: Vec<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Saga step execution result
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

/// Saga action handler trait
#[async_trait::async_trait]
pub trait SagaAction: Send + Sync {
    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError>;
    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError>;
}

/// Saga manager for orchestrating distributed transactions
pub struct SagaManager {
    pool: PgPool,
    redis_client: redis::Client,
    saga_definitions: HashMap<String, SagaDefinition>,
    action_handlers: HashMap<String, Arc<dyn SagaAction>>,
    running_sagas: Arc<RwLock<HashMap<Uuid, Arc<Mutex<SagaInstance>>>>>,
}

#[derive(Debug, Clone)]
pub struct SagaDefinition {
    pub saga_type: String,
    pub steps: Vec<SagaStep>,
    pub timeout_ms: u64,
    pub metadata: serde_json::Value,
}

impl SagaManager {
    pub fn new(pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            pool,
            redis_client,
            saga_definitions: HashMap::new(),
            action_handlers: HashMap::new(),
            running_sagas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a saga definition
    pub fn register_saga(&mut self, definition: SagaDefinition) {
        self.saga_definitions
            .insert(definition.saga_type.clone(), definition);
        info!("Registered saga: {}", definition.saga_type);
    }

    /// Register an action handler
    pub fn register_action(&mut self, action_name: String, handler: Arc<dyn SagaAction>) {
        self.action_handlers.insert(action_name, handler);
    }

    /// Start a new saga instance
    pub async fn start_saga(
        &self,
        saga_type: &str,
        context: serde_json::Value,
    ) -> Result<SagaResult, AppError> {
        let definition = self
            .saga_definitions
            .get(saga_type)
            .ok_or_else(|| AppError::NotFound(format!("Saga type '{}' not found", saga_type)))?;

        let saga_id = Uuid::new_v4();
        let instance = SagaInstance {
            id: saga_id,
            saga_type: saga_type.to_string(),
            state: SagaState::Pending,
            current_step: None,
            completed_steps: Vec::new(),
            failed_step: None,
            context,
            error_message: None,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            metadata: serde_json::json!({}),
        };

        // Store in database
        self.persist_saga(&instance).await?;

        // Store in running sagas
        let instance_arc = Arc::new(Mutex::new(instance));
        let mut running = self.running_sagas.write().await;
        running.insert(saga_id, instance_arc.clone());

        // Start execution
        let manager = self.clone_for_execution();
        tokio::spawn(async move {
            let result = manager.execute_saga(instance_arc).await;
            info!("Saga {} completed with state: {:?}", saga_id, result.state);
        });

        Ok(SagaResult {
            saga_id,
            state: SagaState::InProgress,
            completed_steps: vec![],
            error: None,
            execution_time_ms: 0,
        })
    }

    /// Execute a saga instance
    async fn execute_saga(&self, instance: Arc<Mutex<SagaInstance>>) -> SagaResult {
        let start = std::time::Instant::now();
        let saga_id;

        {
            let mut saga = instance.lock().await;
            saga_id = saga.id;
            saga.state = SagaState::InProgress;
            saga.updated_at = Utc::now();
            let _ = self.persist_saga(&saga).await;
        }

        let saga_type;
        {
            let saga = instance.lock().await;
            saga_type = saga.saga_type.clone();
        }

        let definition = match self.saga_definitions.get(&saga_type) {
            Some(def) => def.clone(),
            None => {
                let mut saga = instance.lock().await;
                saga.state = SagaState::Failed;
                saga.error_message = Some("Saga definition not found".to_string());
                saga.updated_at = Utc::now();
                let _ = self.persist_saga(&saga).await;
                return SagaResult {
                    saga_id,
                    state: SagaState::Failed,
                    completed_steps: vec![],
                    error: Some("Saga definition not found".to_string()),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        // Execute each step
        for step in &definition.steps {
            let step_result = self.execute_step(instance.clone(), step).await;

            if !step_result.success {
                // Step failed, start compensation
                warn!("Step {} failed, starting compensation", step.id);
                self.compensate_saga(instance.clone()).await;

                let mut saga = instance.lock().await;
                saga.state = SagaState::Failed;
                saga.failed_step = Some(step.id.clone());
                saga.error_message = step_result.error;
                saga.updated_at = Utc::now();
                let _ = self.persist_saga(&saga).await;

                return SagaResult {
                    saga_id,
                    state: SagaState::Failed,
                    completed_steps: {
                        let saga = instance.lock().await;
                        saga.completed_steps.clone()
                    },
                    error: step_result.error,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }

            // Mark step as completed
            {
                let mut saga = instance.lock().await;
                saga.completed_steps.push(step.id.clone());
                saga.current_step = Some(step.id.clone());
                saga.updated_at = Utc::now();
                let _ = self.persist_saga(&saga).await;
            }
        }

        // All steps completed successfully
        {
            let mut saga = instance.lock().await;
            saga.state = SagaState::Completed;
            saga.current_step = None;
            saga.completed_at = Some(Utc::now());
            saga.updated_at = Utc::now();
            let _ = self.persist_saga(&saga).await;
        }

        // Remove from running sagas
        let mut running = self.running_sagas.write().await;
        running.remove(&saga_id);

        SagaResult {
            saga_id,
            state: SagaState::Completed,
            completed_steps: {
                let saga = instance.lock().await;
                saga.completed_steps.clone()
            },
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Execute a single saga step
    async fn execute_step(
        &self,
        instance: Arc<Mutex<SagaInstance>>,
        step: &SagaStep,
    ) -> StepResult {
        let start = std::time::Instant::now();
        let mut retry_count = 0;

        loop {
            let context;
            {
                let saga = instance.lock().await;
                context = saga.context.clone();
            }

            if let Some(handler) = self.action_handlers.get(&step.execute_action) {
                match handler.execute(&context).await {
                    Ok(result) => {
                        // Update context with result
                        let mut saga = instance.lock().await;
                        if let Some(obj) = saga.context.as_object_mut() {
                            obj.insert("step_result".to_string(), result);
                        }
                        saga.updated_at = Utc::now();
                        let _ = self.persist_saga(&saga).await;

                        return StepResult {
                            step_id: step.id.clone(),
                            success: true,
                            error: None,
                            execution_time_ms: start.elapsed().as_millis() as u64,
                            retry_count,
                        };
                    }
                    Err(e) => {
                        retry_count += 1;
                        if retry_count >= step.retry_policy.max_attempts {
                            error!(
                                "Step {} failed after {} attempts: {}",
                                step.id, retry_count, e
                            );
                            return StepResult {
                                step_id: step.id.clone(),
                                success: false,
                                error: Some(e.to_string()),
                                execution_time_ms: start.elapsed().as_millis() as u64,
                                retry_count,
                            };
                        }

                        // Backoff before retry
                        let backoff = if step.retry_policy.exponential_backoff {
                            step.retry_policy.backoff_ms * 2_u64.pow(retry_count - 1)
                        } else {
                            step.retry_policy.backoff_ms
                        };
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff)).await;
                    }
                }
            } else {
                return StepResult {
                    step_id: step.id.clone(),
                    success: false,
                    error: Some(format!("Handler '{}' not found", step.execute_action)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    retry_count,
                };
            }
        }
    }

    /// Compensate a failed saga (execute compensation actions in reverse order)
    async fn compensate_saga(&self, instance: Arc<Mutex<SagaInstance>>) {
        let saga_type;
        let completed_steps;

        {
            let mut saga = instance.lock().await;
            saga.state = SagaState::Compensating;
            saga.updated_at = Utc::now();
            let _ = self.persist_saga(&saga).await;
            saga_type = saga.saga_type.clone();
            completed_steps = saga.completed_steps.clone();
        }

        let definition = match self.saga_definitions.get(&saga_type) {
            Some(def) => def.clone(),
            None => {
                error!("Cannot compensate: saga definition not found");
                return;
            }
        };

        // Execute compensation in reverse order
        for step_id in completed_steps.iter().rev() {
            if let Some(step) = definition.steps.iter().find(|s| &s.id == step_id) {
                info!("Compensating step: {}", step.id);

                let context;
                {
                    let saga = instance.lock().await;
                    context = saga.context.clone();
                }

                if let Some(handler) = self.action_handlers.get(&step.compensate_action) {
                    match handler.compensate(&context).await {
                        Ok(_) => {
                            debug!("Compensation succeeded for step: {}", step.id);
                        }
                        Err(e) => {
                            error!("Compensation failed for step {}: {}", step.id, e);
                            // Continue with other compensations
                        }
                    }
                }
            }
        }

        let mut saga = instance.lock().await;
        saga.state = SagaState::Aborted;
        saga.updated_at = Utc::now();
        let _ = self.persist_saga(&saga).await;
    }

    /// Get saga status
    pub async fn get_saga_status(&self, saga_id: Uuid) -> Result<Option<SagaInstance>, AppError> {
        // Check running sagas first
        let running = self.running_sagas.read().await;
        if let Some(instance) = running.get(&saga_id) {
            let saga = instance.lock().await;
            return Ok(Some(saga.clone()));
        }

        // Check database
        sqlx::query_as::<SagaInstance, _>(
            "SELECT id, saga_type, state, current_step, completed_steps, failed_step, context, error_message, started_at, updated_at, completed_at, metadata FROM saga_instances WHERE id = $1"
        )
        .bind(saga_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    /// Persist saga to database
    async fn persist_saga(&self, instance: &SagaInstance) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO saga_instances (
                id, saga_type, state, current_step, completed_steps, failed_step,
                context, error_message, started_at, updated_at, completed_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                state = EXCLUDED.state,
                current_step = EXCLUDED.current_step,
                completed_steps = EXCLUDED.completed_steps,
                failed_step = EXCLUDED.failed_step,
                context = EXCLUDED.context,
                error_message = EXCLUDED.error_message,
                updated_at = EXCLUDED.updated_at,
                completed_at = EXCLUDED.completed_at,
                metadata = EXCLUDED.metadata
            "#,
        )
        .bind(instance.id)
        .bind(&instance.saga_type)
        .bind(instance.state)
        .bind(&instance.current_step)
        .bind(&instance.completed_steps)
        .bind(&instance.failed_step)
        .bind(&instance.context)
        .bind(&instance.error_message)
        .bind(instance.started_at)
        .bind(instance.updated_at)
        .bind(instance.completed_at)
        .bind(&instance.metadata)
        .execute(&self.pool)
        .await?;

        // Cache in Redis
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let serialized = serde_json::to_string(instance)?;
            let _: Result<(), _> = conn
                .set_ex(format!("saga:{}", instance.id), serialized, 3600)
                .await;
        }

        Ok(())
    }

    /// Clone for execution (shallow clone)
    fn clone_for_execution(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            redis_client: self.redis_client.clone(),
            saga_definitions: self.saga_definitions.clone(),
            action_handlers: self.action_handlers.clone(),
            running_sagas: Arc::clone(&self.running_sagas),
        }
    }

    /// Recover in-progress sagas on startup
    pub async fn recover_sagas(&self) -> Result<u64, AppError> {
        let sagas = sqlx::query_as::<SagaInstance, _>(
            "SELECT id, saga_type, state, current_step, completed_steps, failed_step, context, error_message, started_at, updated_at, completed_at, metadata FROM saga_instances WHERE state IN ('InProgress', 'Compensating')"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut recovered = 0;

        for saga in sagas {
            info!("Recovering saga: {}", saga.id);

            let instance_arc = Arc::new(Mutex::new(saga.clone()));
            let mut running = self.running_sagas.write().await;
            running.insert(saga.id, instance_arc.clone());
            recovered += 1;

            let manager = self.clone_for_execution();
            tokio::spawn(async move {
                // For simplicity, we'll restart compensation for in-progress sagas
                manager.compensate_saga(instance_arc).await;
            });
        }

        info!("Recovered {} in-progress sagas", recovered);
        Ok(recovered)
    }
}

/// Example saga for product registration workflow
pub fn get_product_registration_saga() -> SagaDefinition {
    SagaDefinition {
        saga_type: "product_registration".to_string(),
        steps: vec![
            SagaStep {
                id: "validate_product".to_string(),
                name: "Validate Product Data".to_string(),
                execute_action: "validate_product".to_string(),
                compensate_action: "noop".to_string(),
                timeout_ms: 5000,
                retry_policy: RetryPolicy::default(),
                metadata: serde_json::json!({}),
            },
            SagaStep {
                id: "register_on_blockchain".to_string(),
                name: "Register on Blockchain".to_string(),
                execute_action: "register_blockchain".to_string(),
                compensate_action: "revert_blockchain".to_string(),
                timeout_ms: 30000,
                retry_policy: RetryPolicy {
                    max_attempts: 5,
                    backoff_ms: 2000,
                    exponential_backoff: true,
                },
                metadata: serde_json::json!({}),
            },
            SagaStep {
                id: "update_database".to_string(),
                name: "Update Database".to_string(),
                execute_action: "update_database".to_string(),
                compensate_action: "rollback_database".to_string(),
                timeout_ms: 5000,
                retry_policy: RetryPolicy::default(),
                metadata: serde_json::json!({}),
            },
            SagaStep {
                id: "send_notifications".to_string(),
                name: "Send Notifications".to_string(),
                execute_action: "send_notifications".to_string(),
                compensate_action: "cancel_notifications".to_string(),
                timeout_ms: 10000,
                retry_policy: RetryPolicy {
                    max_attempts: 3,
                    backoff_ms: 1000,
                    exponential_backoff: false,
                },
                metadata: serde_json::json!({}),
            },
        ],
        timeout_ms: 60000,
        metadata: serde_json::json!({"category": "product"}),
    }
}

/// No-op action handler for compensation steps that don't need compensation
pub struct NoopAction;

#[async_trait]
impl SagaAction for NoopAction {
    async fn execute(&self, _context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        Ok(serde_json::json!({}))
    }

    async fn compensate(&self, _context: &serde_json::Value) -> Result<(), AppError> {
        Ok(())
    }
}

use crate::error::AppError;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Worker task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerTask {
    pub id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub queue_name: String,
    pub max_concurrent_tasks: usize,
    pub poll_interval_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub worker_id: String,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            queue_name: "event_processing".to_string(),
            max_concurrent_tasks: 10,
            poll_interval_ms: 100,
            heartbeat_interval_ms: 5000,
            worker_id: Uuid::new_v4().to_string(),
        }
    }
}

/// Worker task handler trait
#[async_trait::async_trait]
pub trait TaskHandler: Send + Sync {
    async fn handle(&self, task: WorkerTask) -> Result<serde_json::Value, AppError>;
    fn task_type(&self) -> &str;
}

/// Redis-based worker pool for horizontal scaling
pub struct RedisWorkerPool {
    config: WorkerConfig,
    redis_client: redis::Client,
    handlers: Vec<Arc<dyn TaskHandler>>,
    semaphore: Arc<Semaphore>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl RedisWorkerPool {
    pub fn new(config: WorkerConfig, redis_client: redis::Client) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        
        Self {
            config,
            redis_client,
            handlers: Vec::new(),
            semaphore,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Register a task handler
    pub fn register_handler(&mut self, handler: Arc<dyn TaskHandler>) {
        self.handlers.push(handler);
        info!("Registered handler for task type: {}", handler.task_type());
    }

    /// Start the worker pool
    pub async fn start(&self) -> Result<(), AppError> {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        info!("Starting worker pool: {}", self.config.worker_id);

        // Start heartbeat
        self.start_heartbeat().await;

        // Start task processing loop
        self.start_processing_loop().await;

        Ok(())
    }

    /// Stop the worker pool
    pub async fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Stopping worker pool: {}", self.config.worker_id);
    }

    /// Enqueue a task
    pub async fn enqueue_task(&self, task: WorkerTask) -> Result<(), AppError> {
        let serialized = serde_json::to_string(&task)?;
        
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            // Use priority queue (sorted set)
            let score = -(task.priority as f64); // Negative for higher priority first
            let member = format!("{}:{}", task.id, task.task_type);
            
            let _: Result<(), _> = conn
                .zadd(self.config.queue_name.clone(), score, &serialized)
                .await;
            
            debug!("Enqueued task: {} (priority: {})", task.id, task.priority);
        }
        
        Ok(())
    }

    /// Start the heartbeat to signal worker is alive
    async fn start_heartbeat(&self) {
        let config = self.config.clone();
        let redis_client = self.redis_client.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
                    let key = format!("worker:heartbeat:{}", config.worker_id);
                    let _: Result<(), _> = conn.set_ex(key.clone(), "alive", config.heartbeat_interval_ms / 1000 + 2).await;
                    
                    // Add to worker set
                    let _: Result<(), _> = conn.sadd("workers:active", &config.worker_id).await;
                }
                
                tokio::time::sleep(Duration::from_millis(config.heartbeat_interval_ms)).await;
            }
            
            // Remove from active workers on shutdown
            if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
                let _: Result<(), _> = conn.srem("workers:active", &config.worker_id).await;
            }
        });
    }

    /// Start the main processing loop
    async fn start_processing_loop(&self) {
        let config = self.config.clone();
        let redis_client = self.redis_client.clone();
        let handlers = self.handlers.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                // Try to acquire a permit (concurrency limit)
                let permit = semaphore.clone().acquire_owned().await;
                
                if permit.is_err() {
                    continue;
                }
                
                let permit = permit.unwrap();
                
                // Poll for task
                if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
                    // Get highest priority task
                    let result: Option<Vec<String>> = conn
                        .zpopmax(config.queue_name.clone())
                        .await
                        .ok();
                    
                    if let Some(items) = result {
                        if !items.is_empty() {
                            let task_data = &items[0];
                            
                            if let Ok(task) = serde_json::from_str::<WorkerTask>(task_data) {
                                let task_handlers = handlers.clone();
                                let task_redis = redis_client.clone();
                                
                                tokio::spawn(async move {
                                    let _ = Self::process_task(task, task_handlers, task_redis).await;
                                    drop(permit); // Release permit
                                });
                                
                                continue;
                            }
                        }
                    }
                }
                
                drop(permit); // Release permit if no task
                tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
            }
        });
    }

    /// Process a single task
    async fn process_task(
        task: WorkerTask,
        handlers: Vec<Arc<dyn TaskHandler>>,
        redis_client: redis::Client,
    ) -> Result<(), AppError> {
        let start = std::time::Instant::now();
        
        debug!("Processing task: {} (type: {})", task.id, task.task_type);
        
        // Find appropriate handler
        let handler = handlers
            .iter()
            .find(|h| h.task_type() == task.task_type);
        
        if let Some(handler) = handler {
            let handler = handler.clone();
            
            // Execute with timeout
            let result = tokio::time::timeout(
                Duration::from_millis(task.timeout_ms),
                handler.handle(task.clone()),
            )
            .await;
            
            match result {
                Ok(Ok(output)) => {
                    info!(
                        "Task {} completed in {}ms",
                        task.id,
                        start.elapsed().as_millis()
                    );
                    
                    // Store result
                    Self::store_task_result(&redis_client, &task.id, Some(output), None).await;
                }
                Ok(Err(e)) => {
                    error!("Task {} failed: {}", task.id, e);
                    
                    // Retry if under max retries
                    if task.retry_count < task.max_retries {
                        let mut retry_task = task.clone();
                        retry_task.retry_count += 1;
                        
                        warn!("Retrying task {} (attempt {}/{})", task.id, retry_task.retry_count, task.max_retries);
                        
                        if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
                            let serialized = serde_json::to_string(&retry_task)?;
                            let score = -(retry_task.priority as f64);
                            let _: Result<(), _> = conn
                                .zadd("event_processing", score, &serialized)
                                .await;
                        }
                    } else {
                        error!("Task {} exceeded max retries", task.id);
                        Self::store_task_result(&redis_client, &task.id, None, Some(e.to_string())).await;
                    }
                }
                Err(_) => {
                    error!("Task {} timed out after {}ms", task.id, task.timeout_ms);
                    Self::store_task_result(&redis_client, &task.id, None, Some("Timeout".to_string())).await;
                }
            }
        } else {
            warn!("No handler found for task type: {}", task.task_type);
            Self::store_task_result(&redis_client, &task.id, None, Some("No handler".to_string())).await;
        }
        
        Ok(())
    }

    /// Store task result in Redis
    async fn store_task_result(
        redis_client: &redis::Client,
        task_id: &Uuid,
        result: Option<serde_json::Value>,
        error: Option<String>,
    ) {
        if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
            let key = format!("task:result:{}", task_id);
            let data = serde_json::json!({
                "result": result,
                "error": error,
                "completed_at": chrono::Utc::now(),
            });
            
            let _: Result<(), _> = conn.set_ex(key, serde_json::to_string(&data).unwrap(), 3600).await;
        }
    }

    /// Get task result
    pub async fn get_task_result(&self, task_id: &Uuid) -> Option<TaskResult> {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let key = format!("task:result:{}", task_id);
            if let Ok(data) = conn.get::<_, String>(key).await {
                if let Ok(result) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Some(TaskResult {
                        result: result.get("result").cloned(),
                        error: result.get("error").and_then(|v| v.as_str()).map(String::from),
                        completed_at: result.get("completed_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc)),
                    });
                }
            }
        }
        None
    }

    /// Get active workers count
    pub async fn get_active_workers_count(&self) -> usize {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(count) = conn.scard("workers:active").await {
                return count;
            }
        }
        0
    }

    /// Get queue length
    pub async fn get_queue_length(&self) -> usize {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(count) = conn.zcard(self.config.queue_name.clone()).await {
                return count;
            }
        }
        0
    }
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Example handler for event processing tasks
pub struct EventProcessingHandler {
    // Could include database pool, services, etc.
}

impl EventProcessingHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskHandler for EventProcessingHandler {
    async fn handle(&self, task: WorkerTask) -> Result<serde_json::Value, AppError> {
        // Process event
        debug!("Processing event task: {}", task.id);
        
        // Simulate processing
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(serde_json::json!({
            "status": "processed",
            "task_id": task.id,
        }))
    }

    fn task_type(&self) -> &str {
        "event_processing"
    }
}

/// Example handler for rule evaluation tasks
pub struct RuleEvaluationHandler {
    // Could include rule engine reference
}

impl RuleEvaluationHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskHandler for RuleEvaluationHandler {
    async fn handle(&self, task: WorkerTask) -> Result<serde_json::Value, AppError> {
        debug!("Evaluating rules for task: {}", task.id);
        
        // Simulate rule evaluation
        tokio::time::sleep(Duration::from_millis(30)).await;
        
        Ok(serde_json::json!({
            "status": "evaluated",
            "matched_rules": vec!["quality-check-failure"],
        }))
    }

    fn task_type(&self) -> &str {
        "rule_evaluation"
    }
}

/// Example	handler for notification tasks
pub struct NotificationHandler {
    // Could include email/SMS service
}

impl NotificationHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskHandler for NotificationHandler {
    async fn handle(&self, task: WorkerTask) -> Result<serde_json::Value, AppError> {
        debug!("Sending notification for task: {}", task.id);
        
        // Simulate notification sending
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(serde_json::json!({
            "status": "sent",
            "recipients": task.payload.get("recipients"),
        }))
    }

    fn task_type(&self) -> &str {
        "notification"
    }
}

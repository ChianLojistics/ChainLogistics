use crate::error::AppError;
use crate::workers::executor::TaskExecutor;
use crate::workers::task::Task;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct WorkerPool {
    redis_client: redis::Client,
    executor: Arc<TaskExecutor>,
    worker_id: String,
    queue_name: String,
    processing_queue: String,
    _handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new(
        redis_client: redis::Client,
        executor: TaskExecutor,
        worker_id: String,
        queue_name: String,
    ) -> Self {
        let processing_queue = format!("{}:processing", queue_name);
        Self {
            redis_client,
            executor: Arc::new(executor),
            worker_id,
            queue_name,
            processing_queue,
            _handles: Vec::new(),
        }
    }

    pub async fn start(&mut self, num_workers: usize) -> Result<(), AppError> {
        tracing::info!("Starting {} workers for queue: {}", num_workers, self.queue_name);

        for i in 0..num_workers {
            let worker_id = format!("{}:{}", self.worker_id, i);
            let redis_client = self.redis_client.clone();
            let executor = self.executor.clone();
            let queue_name = self.queue_name.clone();
            let processing_queue = format!("{}:processing", queue_name);

            let handle = tokio::spawn(async move {
                Self::worker_loop(worker_id, redis_client, executor, queue_name, processing_queue).await;
            });

            self._handles.push(handle);
        }

        Ok(())
    }

    async fn worker_loop(
        worker_id: String,
        redis_client: redis::Client,
        executor: Arc<TaskExecutor>,
        queue_name: String,
        processing_queue: String,
    ) {
        tracing::info!("Worker {} started", worker_id);

        loop {
            match Self::fetch_task(&redis_client, &queue_name, &processing_queue, &worker_id).await {
                Ok(Some(task)) => {
                    let start = std::time::Instant::now();
                    
                    match executor.execute(&task).await {
                        Ok(_) => {
                            let duration = start.elapsed();
                            tracing::debug!("Worker {} completed task {} in {}ms", 
                                worker_id, task.id, duration.as_millis());
                            
                            // Remove from processing queue
                            let _ = Self::remove_from_processing(&redis_client, &processing_queue, &task.id).await;
                        }
                        Err(e) => {
                            tracing::error!("Worker {} failed task {}: {}", worker_id, task.id, e);

                            // Requeue if retryable
                            let task_id = task.id.clone();
                            let _ = Self::requeue_task(&redis_client, &queue_name, task).await;
                            let _ = Self::remove_from_processing(&redis_client, &processing_queue, &task_id).await;
                        }
                    }
                }
                Ok(None) => {
                    // No tasks available, wait a bit
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    tracing::error!("Worker {} error fetching task: {}", worker_id, e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn fetch_task(
        redis_client: &redis::Client,
        queue_name: &str,
        processing_queue: &str,
        worker_id: &str,
    ) -> Result<Option<Task>, AppError> {
        let mut conn = redis_client.get_multiplexed_tokio_connection().await?;

        // Use BRPOPLPUSH for atomic move from queue to processing
        let task_json: Option<String> = conn
            .brpoplpush(queue_name, processing_queue, 1.0)
            .await?;

        if let Some(task_json) = task_json {
            let task: Task = serde_json::from_str(&task_json)
                .map_err(|e| AppError::Internal(format!("Failed to deserialize task: {}", e)))?;

            // Add worker metadata
            let worker_key = format!("{}:worker:{}", processing_queue, task.id);
            let _: Result<(), _> = conn.set(&worker_key, worker_id).await;
            let _: Result<(), _> = conn.expire(&worker_key, 3600).await;

            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    async fn remove_from_processing(
        redis_client: &redis::Client,
        processing_queue: &str,
        task_id: &Uuid,
    ) -> Result<(), AppError> {
        let mut conn = redis_client.get_multiplexed_tokio_connection().await?;
        
        // Remove from processing queue
        let worker_key = format!("{}:worker:{}", processing_queue, task_id);
        let _: Result<(), _> = conn.del(&worker_key).await;
        
        Ok(())
    }

    async fn requeue_task(
        redis_client: &redis::Client,
        queue_name: &str,
        mut task: Task,
    ) -> Result<(), AppError> {
        if task.can_retry() {
            task.increment_retry();
            
            let task_json = serde_json::to_string(&task)
                .map_err(|e| AppError::Internal(format!("Failed to serialize task: {}", e)))?;

            let mut conn = redis_client.get_multiplexed_tokio_connection().await?;
            
            // Add back to queue with lower priority
            let _: Result<(), _> = conn.lpush(queue_name, &task_json).await;
            
            tracing::info!("Requeued task {} (retry {}/{})", task.id, task.retry_count, task.max_retries);
        } else {
            tracing::warn!("Task {} exceeded max retries, discarding", task.id);
        }
        
        Ok(())
    }

    pub async fn submit_task(&self, task: Task) -> Result<(), AppError> {
        let task_json = serde_json::to_string(&task)
            .map_err(|e| AppError::Internal(format!("Failed to serialize task: {}", e)))?;

        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        
        // Add to queue based on priority (higher priority = left side of list)
        if task.priority > 0 {
            conn.lpush(&self.queue_name, &task_json).await?;
        } else {
            conn.rpush(&self.queue_name, &task_json).await?;
        }

        tracing::debug!("Submitted task {} to queue {}", task.id, self.queue_name);
        Ok(())
    }

    pub async fn get_queue_size(&self) -> Result<usize, AppError> {
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        let size: usize = conn.llen(&self.queue_name).await?;
        Ok(size)
    }

    pub async fn get_processing_size(&self) -> Result<usize, AppError> {
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        let size: usize = conn.llen(&self.processing_queue).await?;
        Ok(size)
    }

    pub async fn shutdown(self) {
        for handle in self._handles {
            handle.abort();
        }
    }
}

pub struct TaskDistributor {
    redis_client: redis::Client,
    queue_name: String,
}

impl TaskDistributor {
    pub fn new(redis_client: redis::Client, queue_name: String) -> Self {
        Self {
            redis_client,
            queue_name,
        }
    }

    pub async fn submit(&self, task: Task) -> Result<(), AppError> {
        let task_json = serde_json::to_string(&task)
            .map_err(|e| AppError::Internal(format!("Failed to serialize task: {}", e)))?;

        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        
        if task.priority > 0 {
            conn.lpush(&self.queue_name, &task_json).await?;
        } else {
            conn.rpush(&self.queue_name, &task_json).await?;
        }

        Ok(())
    }
}

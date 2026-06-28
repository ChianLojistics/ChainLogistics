use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub max_retries: u32,
    pub retry_count: u32,
    pub created_at: i64,
    pub scheduled_at: Option<i64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Task {
    pub fn new(task_type: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            payload,
            priority: 0,
            max_retries: 3,
            retry_count: 0,
            created_at: chrono::Utc::now().timestamp_millis(),
            scheduled_at: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_scheduled_at(mut self, scheduled_at: i64) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    pub fn is_ready(&self) -> bool {
        if let Some(scheduled_at) = self.scheduled_at {
            chrono::Utc::now().timestamp_millis() >= scheduled_at
        } else {
            true
        }
    }
}

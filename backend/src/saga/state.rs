use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Compensating,
    Compensated,
}

impl std::fmt::Display for SagaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SagaStatus::Pending => write!(f, "pending"),
            SagaStatus::InProgress => write!(f, "in_progress"),
            SagaStatus::Completed => write!(f, "completed"),
            SagaStatus::Failed => write!(f, "failed"),
            SagaStatus::Compensating => write!(f, "compensating"),
            SagaStatus::Compensated => write!(f, "compensated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub id: String,
    pub name: String,
    pub status: SagaStatus,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SagaStep {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            status: SagaStatus::Pending,
            started_at: None,
            completed_at: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        self.status = SagaStatus::InProgress;
        self.started_at = Some(chrono::Utc::now().timestamp_millis());
    }

    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.completed_at = Some(chrono::Utc::now().timestamp_millis());
    }

    pub fn fail(&mut self, error: String) {
        self.status = SagaStatus::Failed;
        self.completed_at = Some(chrono::Utc::now().timestamp_millis());
        self.error = Some(error);
    }

    pub fn compensate(&mut self) {
        self.status = SagaStatus::Compensating;
    }

    pub fn compensated(&mut self) {
        self.status = SagaStatus::Compensated;
        self.completed_at = Some(chrono::Utc::now().timestamp_millis());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaState {
    pub id: Uuid,
    pub name: String,
    pub status: SagaStatus,
    pub steps: Vec<SagaStep>,
    pub current_step_index: usize,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SagaState {
    pub fn new(name: String) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp_millis();
        
        Self {
            id,
            name,
            status: SagaStatus::Pending,
            steps: Vec::new(),
            current_step_index: 0,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, step: SagaStep) {
        self.steps.push(step);
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn start(&mut self) -> Result<(), AppError> {
        if self.status != SagaStatus::Pending {
            return Err(AppError::BusinessRule("Saga already started".to_string()));
        }
        self.status = SagaStatus::InProgress;
        self.updated_at = chrono::Utc::now().timestamp_millis();
        Ok(())
    }

    pub fn advance_step(&mut self) -> Result<(), AppError> {
        if self.current_step_index >= self.steps.len() {
            return Err(AppError::BusinessRule("No more steps to execute".to_string()));
        }
        self.current_step_index += 1;
        self.updated_at = chrono::Utc::now().timestamp_millis();
        Ok(())
    }

    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn fail(&mut self) {
        self.status = SagaStatus::Failed;
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn start_compensation(&mut self) {
        self.status = SagaStatus::Compensating;
        self.current_step_index = self.steps.len().saturating_sub(1);
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn compensate_step(&mut self) -> Result<(), AppError> {
        if self.current_step_index == 0 {
            return Err(AppError::BusinessRule("No more steps to compensate".to_string()));
        }
        self.current_step_index -= 1;
        self.updated_at = chrono::Utc::now().timestamp_millis();
        Ok(())
    }

    pub fn compensated(&mut self) {
        self.status = SagaStatus::Compensated;
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn current_step(&self) -> Option<&SagaStep> {
        self.steps.get(self.current_step_index)
    }

    pub fn current_step_mut(&mut self) -> Option<&mut SagaStep> {
        self.steps.get_mut(self.current_step_index)
    }
}

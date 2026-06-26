use crate::error::AppError;
use crate::saga::persistence::SagaPersistence;
use crate::saga::state::{SagaState, SagaStep, SagaStatus};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait SagaStepHandler: Send + Sync {
    async fn execute(&self, step: &SagaStep, saga: &SagaState) -> Result<(), AppError>;
    async fn compensate(&self, step: &SagaStep, saga: &SagaState) -> Result<(), AppError>;
}

#[derive(Clone)]
pub enum SagaStepHandlerEnum {
    Default,
}

impl SagaStepHandlerEnum {
    async fn execute(&self, _step: &SagaStep, _saga: &SagaState) -> Result<(), AppError> {
        Ok(())
    }

    async fn compensate(&self, _step: &SagaStep, _saga: &SagaState) -> Result<(), AppError> {
        Ok(())
    }
}

pub struct SagaCoordinator {
    persistence: Arc<dyn SagaPersistence>,
    handlers: Arc<std::collections::HashMap<String, SagaStepHandlerEnum>>,
}

impl SagaCoordinator {
    pub fn new(persistence: Arc<dyn SagaPersistence>) -> Self {
        Self {
            persistence,
            handlers: Arc::new(std::collections::HashMap::new()),
        }
    }

    pub fn register_handler(&mut self, step_name: String, handler: SagaStepHandlerEnum) {
        Arc::make_mut(&mut self.handlers).insert(step_name, handler);
    }

    pub async fn create_saga(&self, name: String) -> Result<SagaState, AppError> {
        let saga = SagaState::new(name);
        self.persistence.save(&saga).await?;
        Ok(saga)
    }

    pub async fn add_step(&self, saga_id: Uuid, step: SagaStep) -> Result<(), AppError> {
        let mut saga = self.persistence.load(saga_id).await?
            .ok_or_else(|| AppError::NotFound("Saga not found".to_string()))?;
        
        if saga.status != SagaStatus::Pending {
            return Err(AppError::BusinessRule("Cannot add step to started saga".to_string()));
        }

        saga.add_step(step);
        self.persistence.save(&saga).await?;
        Ok(())
    }

    pub async fn start(&self, saga_id: Uuid) -> Result<(), AppError> {
        let mut saga = self.persistence.load(saga_id).await?
            .ok_or_else(|| AppError::NotFound("Saga not found".to_string()))?;

        saga.start()?;
        self.persistence.save(&saga).await?;

        self.execute_next_step(saga).await?;
        Ok(())
    }

    async fn execute_next_step(&self, mut saga: SagaState) -> Result<(), AppError> {
        loop {
            let step_name = if let Some(step) = saga.current_step() {
                step.name.clone()
            } else {
                break;
            };

            let handler = self.handlers.get(&step_name)
                .ok_or_else(|| AppError::BusinessRule(format!("No handler for step: {}", step_name)))?;

            if let Some(step) = saga.current_step_mut() {
                step.start();
            }
            self.persistence.save(&saga).await?;

            let saga_clone = saga.clone();
            if let Some(step) = saga.current_step_mut() {
                match handler.execute(step, &saga_clone).await {
                    Ok(_) => {
                        step.complete();
                        self.persistence.save(&saga).await?;
                        saga.advance_step()?;
                    }
                    Err(e) => {
                        step.fail(e.to_string());
                        self.persistence.save(&saga).await?;
                        saga.fail();
                        self.persistence.save(&saga).await?;
                        self.start_compensation(saga).await?;
                        return Err(e);
                    }
                }
            }
        }

        saga.complete();
        self.persistence.save(&saga).await?;
        Ok(())
    }

    async fn start_compensation(&self, mut saga: SagaState) -> Result<(), AppError> {
        saga.start_compensation();
        self.persistence.save(&saga).await?;

        self.compensate_steps(saga).await?;
        Ok(())
    }

    async fn compensate_steps(&self, mut saga: SagaState) -> Result<(), AppError> {
        loop {
            let step_name = if let Some(step) = saga.current_step() {
                step.name.clone()
            } else {
                break;
            };

            let step_status = if let Some(step) = saga.current_step() {
                step.status
            } else {
                break;
            };

            if step_status == SagaStatus::Completed {
                if let Some(step) = saga.current_step_mut() {
                    step.compensate();
                }
                self.persistence.save(&saga).await?;

                let handler = self.handlers.get(&step_name)
                    .ok_or_else(|| AppError::BusinessRule(format!("No handler for step: {}", step_name)))?;

                let saga_clone = saga.clone();
                if let Some(step) = saga.current_step_mut() {
                    match handler.compensate(step, &saga_clone).await {
                        Ok(_) => {
                            step.compensated();
                            self.persistence.save(&saga).await?;
                            saga.compensate_step()?;
                        }
                        Err(e) => {
                            tracing::error!("Compensation failed for step {}: {}", step_name, e);
                            // Continue compensation despite errors
                            saga.compensate_step()?;
                        }
                    }
                }
            } else {
                saga.compensate_step()?;
            }
        }

        saga.compensated();
        self.persistence.save(&saga).await?;
        Ok(())
    }

    pub async fn get_saga(&self, saga_id: Uuid) -> Result<Option<SagaState>, AppError> {
        self.persistence.load(saga_id).await
    }

    pub async fn retry_failed_saga(&self, saga_id: Uuid) -> Result<(), AppError> {
        let mut saga = self.persistence.load(saga_id).await?
            .ok_or_else(|| AppError::NotFound("Saga not found".to_string()))?;

        match saga.status {
            SagaStatus::Failed => {
                // Find the failed step and retry from there
                saga.status = SagaStatus::InProgress;
                saga.current_step_index = saga.steps.iter()
                    .position(|s| s.status == SagaStatus::Failed)
                    .ok_or_else(|| AppError::BusinessRule("No failed step found".to_string()))?;
                
                self.persistence.save(&saga).await?;
                self.execute_next_step(saga).await?;
                Ok(())
            }
            _ => Err(AppError::BusinessRule("Can only retry failed sagas".to_string()))
        }
    }
}

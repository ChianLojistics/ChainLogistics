use crate::workers::task::Task;

#[derive(Clone)]
pub enum TaskHandlerEnum {
    Default,
}

impl TaskHandlerEnum {
    async fn handle(&self, _task: &Task) -> Result<(), crate::error::AppError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct TaskExecutor {
    handlers: std::collections::HashMap<String, TaskHandlerEnum>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, task_type: String, handler: TaskHandlerEnum) {
        self.handlers.insert(task_type, handler);
    }

    pub async fn execute(&self, task: &Task) -> Result<(), crate::error::AppError> {
        if let Some(handler) = self.handlers.get(&task.task_type) {
            handler.handle(task).await
        } else {
            Err(crate::error::AppError::Internal(format!("No handler for task type: {}", task.task_type)))
        }
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

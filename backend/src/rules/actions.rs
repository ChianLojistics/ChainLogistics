use crate::error::AppError;
use crate::rules::dsl::Action;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action_name: String,
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub enum ActionHandlerEnum {
    Webhook(WebhookHandler),
    State(StateHandler),
}

impl ActionHandlerEnum {
    async fn handle(&self, action: &Action, context: &HashMap<String, serde_json::Value>) -> Result<ActionResult, AppError> {
        match self {
            ActionHandlerEnum::Webhook(handler) => handler.handle(action, context).await,
            ActionHandlerEnum::State(handler) => handler.handle(action, context).await,
        }
    }
}

pub struct ActionExecutor {
    handlers: Arc<RwLock<HashMap<String, ActionHandlerEnum>>>,
}

impl ActionExecutor {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_handler(&self, name: String, handler: ActionHandlerEnum) {
        let handlers = Arc::clone(&self.handlers);
        tokio::spawn(async move {
            let mut handlers = handlers.write().await;
            handlers.insert(name, handler);
        });
    }

    pub async fn execute(&self, action: &Action, context: &HashMap<String, serde_json::Value>) -> Result<ActionResult, AppError> {
        let handler_name = match action {
            Action::SendWebhook { .. } => "send_webhook",
            Action::SendEmail { .. } => "send_email",
            Action::SendSms { .. } => "send_sms",
            Action::UpdateField { .. } => "update_field",
            Action::CreateRecord { .. } => "create_record",
            Action::DeleteRecord { .. } => "delete_record",
            Action::CallContract { .. } => "call_contract",
            Action::TransferAsset { .. } => "transfer_asset",
            Action::TriggerWorkflow { .. } => "trigger_workflow",
            Action::SetState { .. } => "set_state",
            Action::Custom { name, .. } => name,
        };

        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(handler_name) {
            handler.handle(action, context).await
        } else {
            Ok(ActionResult {
                action_name: handler_name.to_string(),
                success: false,
                message: format!("No handler registered for action: {}", handler_name),
                data: None,
            })
        }
    }
}

// Default action handlers
pub struct WebhookHandler {
    http_client: reqwest::Client,
}

impl WebhookHandler {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    async fn handle(&self, action: &Action, _context: &HashMap<String, serde_json::Value>) -> Result<ActionResult, AppError> {
        if let Action::SendWebhook { url, payload } = action {
            let start = std::time::Instant::now();
            
            let response = self.http_client.post(url)
                .json(payload)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    Ok(ActionResult {
                        action_name: "send_webhook".to_string(),
                        success: true,
                        message: format!("Webhook sent successfully in {}ms", start.elapsed().as_millis()),
                        data: Some(serde_json::json!({ "status": resp.status().as_u16() })),
                    })
                }
                Ok(resp) => {
                    Ok(ActionResult {
                        action_name: "send_webhook".to_string(),
                        success: false,
                        message: format!("Webhook failed with status: {}", resp.status()),
                        data: Some(serde_json::json!({ "status": resp.status().as_u16() })),
                    })
                }
                Err(e) => {
                    Ok(ActionResult {
                        action_name: "send_webhook".to_string(),
                        success: false,
                        message: format!("Webhook error: {}", e),
                        data: None,
                    })
                }
            }
        } else {
            Err(AppError::ValidationError("Invalid action type for WebhookHandler".to_string()))
        }
    }
}

pub struct StateHandler {
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl StateHandler {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_state(&self, key: &str) -> Option<serde_json::Value> {
        let state = self.state.read().await;
        state.get(key).cloned()
    }

    async fn handle(&self, action: &Action, _context: &HashMap<String, serde_json::Value>) -> Result<ActionResult, AppError> {
        if let Action::SetState { key, value } = action {
            let mut state = self.state.write().await;
            state.insert(key.clone(), value.clone());
            
            Ok(ActionResult {
                action_name: "set_state".to_string(),
                success: true,
                message: format!("State updated: {}", key),
                data: Some(value.clone()),
            })
        } else {
            Err(AppError::ValidationError("Invalid action type for StateHandler".to_string()))
        }
    }
}

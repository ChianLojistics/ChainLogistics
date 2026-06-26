use crate::error::AppError;
use crate::models::TrackingEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Rule definition for event processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub priority: i32,
    pub conditions: ConditionGroup,
    pub actions: Vec<Action>,
    pub metadata: serde_json::Value,
}

/// Condition group with logical operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub operator: LogicalOperator,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    And,
    Or,
}

/// Individual condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    Matches, // Regex
    In,
    NotIn,
}

/// Action to execute when rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Action {
    #[serde(rename = "webhook")]
    Webhook { url: String, method: String, headers: HashMap<String, String> },
    #[serde(rename = "email")]
    Email { to: Vec<String>, subject: String, template: String },
    #[serde(rename = "tag")]
    Tag { tags: Vec<String> },
    #[serde(rename = "alert")]
    Alert { severity: String, message: String },
    #[serde(rename = "transform")]
    Transform { script: String },
    #[serde(rename = "block")]
    Block { reason: String },
    #[serde(rename = "custom")]
    Custom { handler: String, params: serde_json::Value },
}

/// Rule evaluation context
#[derive(Debug, Clone)]
pub struct RuleContext {
    pub event: TrackingEvent,
    pub product: Option<ProductContext>,
    pub variables: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProductContext {
    pub id: String,
    pub owner_address: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
}

/// Rule evaluation result
#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    pub rule_id: String,
    pub matched: bool,
    pub actions_executed: Vec<String>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Rule engine for complex event processing
pub struct RuleEngine {
    rules: Vec<Rule>,
    action_handlers: HashMap<String, Arc<dyn ActionHandler>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            action_handlers: HashMap::new(),
        }
    }

    /// Add a rule to the engine
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        // Sort by priority (higher priority first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> Option<Rule> {
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule_id) {
            Some(self.rules.remove(pos))
        } else {
            None
        }
    }

    /// Register an action handler
    pub fn register_handler(&mut self, action_type: String, handler: Arc<dyn ActionHandler>) {
        self.action_handlers.insert(action_type, handler);
    }

    /// Evaluate all rules against context
    pub async fn evaluate(&self, context: &RuleContext) -> Vec<RuleEvaluationResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let start = std::time::Instant::now();
            let matched = self.evaluate_condition_group(&rule.conditions, context);

            let mut result = RuleEvaluationResult {
                rule_id: rule.id.clone(),
                matched,
                actions_executed: Vec::new(),
                execution_time_ms: start.elapsed().as_millis() as u64,
                error: None,
            };

            if matched {
                debug!("Rule '{}' matched event {}", rule.name, context.event.id);
                
                for action in &rule.actions {
                    match self.execute_action(action, context).await {
                        Ok(action_name) => {
                            result.actions_executed.push(action_name);
                        }
                        Err(e) => {
                            error!("Failed to execute action: {}", e);
                            result.error = Some(e.to_string());
                        }
                    }
                }
            }

            results.push(result);
        }

        results
    }

    /// Evaluate a condition group
    fn evaluate_condition_group(&self, group: &ConditionGroup, context: &RuleContext) -> bool {
        let results: Vec<bool> = group
            .conditions
            .iter()
            .map(|cond| self.evaluate_condition(cond, context))
            .collect();

        match group.operator {
            LogicalOperator::And => results.iter().all(|&r| r),
            LogicalOperator::Or => results.iter().any(|&r| r),
        }
    }

    /// Evaluate a single condition
    fn evaluate_condition(&self, condition: &Condition, context: &RuleContext) -> bool {
        let field_value = self.get_field_value(&condition.field, context);

        match condition.operator {
            ComparisonOperator::Equals => self.compare_values(&field_value, &condition.value) == 0,
            ComparisonOperator::NotEquals => self.compare_values(&field_value, &condition.value) != 0,
            ComparisonOperator::GreaterThan => self.compare_values(&field_value, &condition.value) > 0,
            ComparisonOperator::LessThan => self.compare_values(&field_value, &condition.value) < 0,
            ComparisonOperator::GreaterThanOrEqual => {
                self.compare_values(&field_value, &condition.value) >= 0
            }
            ComparisonOperator::LessThanOrEqual => {
                self.compare_values(&field_value, &condition.value) <= 0
            }
            ComparisonOperator::Contains => self.string_contains(&field_value, &condition.value),
            ComparisonOperator::NotContains => !self.string_contains(&field_value, &condition.value),
            ComparisonOperator::Matches => self.regex_matches(&field_value, &condition.value),
            ComparisonOperator::In => self.value_in_array(&field_value, &condition.value),
            ComparisonOperator::NotIn => !self.value_in_array(&field_value, &condition.value),
        }
    }

    /// Get field value from context
    fn get_field_value(&self, field: &str, context: &RuleContext) -> serde_json::Value {
        let parts: Vec<&str> = field.split('.').collect();

        match parts.as_slice() {
            ["event", rest @ ..] => self.get_nested_value(&serde_json::to_value(&context.event).unwrap(), rest),
            ["product", rest @ ..] => {
                if let Some(product) = &context.product {
                    self.get_nested_value(&serde_json::to_value(product).unwrap(), rest)
                } else {
                    serde_json::Value::Null
                }
            }
            ["variable", name] => context.variables.get(*name).cloned().unwrap_or(serde_json::Value::Null),
            _ => serde_json::Value::Null,
        }
    }

    /// Get nested value from JSON
    fn get_nested_value(&self, value: &serde_json::Value, path: &[&str]) -> serde_json::Value {
        let mut current = value;
        for key in path {
            current = match current {
                serde_json::Value::Object(map) => map.get(*key).unwrap_or(&serde_json::Value::Null),
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = key.parse::<usize>() {
                        arr.get(index).unwrap_or(&serde_json::Value::Null)
                    } else {
                        &serde_json::Value::Null
                    }
                }
                _ => &serde_json::Value::Null,
            };
        }
        current.clone()
    }

    /// Compare two JSON values
    fn compare_values(&self, a: &serde_json::Value, b: &serde_json::Value) -> std::cmp::Ordering {
        match (a, b) {
            (serde_json::Value::String(a), serde_json::Value::String(b)) => a.cmp(b),
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                    a.cmp(&b)
                } else if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            (serde_json::Value::Bool(a), serde_json::Value::Bool(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }

    /// Check if string contains substring
    fn string_contains(&self, value: &serde_json::Value, pattern: &serde_json::Value) -> bool {
        if let (Some(s), Some(p)) = (value.as_str(), pattern.as_str()) {
            s.to_lowercase().contains(&p.to_lowercase())
        } else {
            false
        }
    }

    /// Check regex match
    fn regex_matches(&self, value: &serde_json::Value, pattern: &serde_json::Value) -> bool {
        if let (Some(s), Some(p)) = (value.as_str(), pattern.as_str()) {
            if let Ok(re) = regex::Regex::new(p) {
                re.is_match(s)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if value is in array
    fn value_in_array(&self, value: &serde_json::Value, array: &serde_json::Value) -> bool {
        if let serde_json::Value::Array(arr) = array {
            arr.contains(value)
        } else {
            false
        }
    }

    /// Execute an action
    async fn execute_action(&self, action: &Action, context: &RuleContext) -> Result<String, AppError> {
        let action_type = match action {
            Action::Webhook { .. } => "webhook",
            Action::Email { .. } => "email",
            Action::Tag { .. } => "tag",
            Action::Alert { .. } => "alert",
            Action::Transform { .. } => "transform",
            Action::Block { .. } => "block",
            Action::Custom { handler, .. } => handler,
        };

        if let Some(handler) = self.action_handlers.get(action_type) {
            handler.execute(action, context).await?;
            Ok(action_type.to_string())
        } else {
            warn!("No handler registered for action type: {}", action_type);
            Ok(action_type.to_string())
        }
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Get rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Action handler trait
#[async_trait::async_trait]
pub trait ActionHandler: Send + Sync {
    async fn execute(&self, action: &Action, context: &RuleContext) -> Result<(), AppError>;
}

/// Default alert handler
pub struct AlertHandler {
    // Could include notification service, etc.
}

impl AlertHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ActionHandler for AlertHandler {
    async fn execute(&self, action: &Action, context: &RuleContext) -> Result<(), AppError> {
        if let Action::Alert { severity, message } = action {
            info!(
                "ALERT [{}]: {} for event {}",
                severity,
                message,
                context.event.id
            );
            // In production, this would send to monitoring/alerting system
        }
        Ok(())
    }
}

/// Default webhook handler
pub struct WebhookHandler {
    client: reqwest::Client,
}

impl WebhookHandler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ActionHandler for WebhookHandler {
    async fn execute(&self, action: &Action, context: &RuleContext) -> Result<(), AppError> {
        if let Action::Webhook { url, method, headers } = action {
            let mut request = match method.to_lowercase().as_str() {
                "post" => self.client.post(url),
                "put" => self.client.put(url),
                "patch" => self.client.patch(url),
                _ => self.client.post(url),
            };

            for (key, value) in headers {
                request = request.header(key, value);
            }

            let payload = serde_json::json!({
                "event": context.event,
                "timestamp": context.timestamp,
            });

            let response = request.json(&payload).send().await?;

            if !response.status().is_success() {
                return Err(AppError::BadRequest(format!(
                    "Webhook failed with status: {}",
                    response.status()
                )));
            }

            info!("Webhook executed successfully: {}", url);
        }
        Ok(())
    }
}

/// Predefined rules for common supply chain scenarios
pub fn get_default_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "quality-check-failure".to_string(),
            name: "Quality Check Failure Alert".to_string(),
            description: "Alert when quality check fails".to_string(),
            enabled: true,
            priority: 100,
            conditions: ConditionGroup {
                operator: LogicalOperator::And,
                conditions: vec![
                    Condition {
                        field: "event.event_type".to_string(),
                        operator: ComparisonOperator::Equals,
                        value: serde_json::json!("QUALITY_CHECK"),
                    },
                    Condition {
                        field: "event.metadata.passed".to_string(),
                        operator: ComparisonOperator::Equals,
                        value: serde_json::json!(false),
                    },
                ],
            },
            actions: vec![Action::Alert {
                severity: "high".to_string(),
                message: "Quality check failed - immediate attention required".to_string(),
            }],
            metadata: serde_json::json!({"category": "quality"}),
        },
        Rule {
            id: "shipment-delay".to_string(),
            name: "Shipment Delay Detection".to_string(),
            description: "Detect shipments delayed beyond expected time".to_string(),
            enabled: true,
            priority: 80,
            conditions: ConditionGroup {
                operator: LogicalOperator::And,
                conditions: vec![
                    Condition {
                        field: "event.event_type".to_string(),
                        operator: ComparisonOperator::Equals,
                        value: serde_json::json!("SHIP"),
                    },
                    Condition {
                        field: "event.metadata.delayed".to_string(),
                        operator: ComparisonOperator::Equals,
                        value: serde_json::json!(true),
                    },
                ],
            },
            actions: vec![
                Action::Alert {
                    severity: "medium".to_string(),
                    message: "Shipment delayed beyond expected time".to_string(),
                },
                Action::Tag {
                    tags: vec!["delayed".to_string(), "attention-required".to_string()],
                },
            ],
            metadata: serde_json::json!({"category": "logistics"}),
        },
        Rule {
            id: "temperature-excursion".to_string(),
            name: "Temperature Excursion Alert".to_string(),
            description: "Alert when temperature exceeds safe range".to_string(),
            enabled: true,
            priority: 95,
            conditions: ConditionGroup {
                operator: LogicalOperator::Or,
                conditions: vec![
                    Condition {
                        field: "event.metadata.temperature".to_string(),
                        operator: ComparisonOperator::GreaterThan,
                        value: serde_json::json!(25),
                    },
                    Condition {
                        field: "event.metadata.temperature".to_string(),
                        operator: ComparisonOperator::LessThan,
                        value: serde_json::json!(2),
                    },
                ],
            },
            actions: vec![Action::Alert {
                severity: "critical".to_string(),
                message: "Temperature excursion detected - product safety at risk".to_string(),
            }],
            metadata: serde_json::json!({"category": "safety"}),
        },
    ]
}

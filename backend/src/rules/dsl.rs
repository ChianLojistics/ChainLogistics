use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    // Simple conditions
    Equals { field: String, value: serde_json::Value },
    NotEquals { field: String, value: serde_json::Value },
    GreaterThan { field: String, value: serde_json::Value },
    LessThan { field: String, value: serde_json::Value },
    Contains { field: String, value: String },
    Matches { field: String, pattern: String },
    
    // Logical operators
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
    
    // Time-based conditions
    TimeAfter { field: String, timestamp: i64 },
    TimeBefore { field: String, timestamp: i64 },
    TimeBetween { field: String, start: i64, end: i64 },
    
    // Event-based conditions
    EventCount { event_type: String, operator: String, threshold: i64 },
    Sequence { events: Vec<String> },
}

impl Condition {
    pub fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> Result<bool, AppError> {
        match self {
            Condition::Equals { field, value } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                Ok(field_value == value)
            }
            Condition::NotEquals { field, value } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                Ok(field_value != value)
            }
            Condition::GreaterThan { field, value } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let (Some(fv), Some(v)) = (field_value.as_i64(), value.as_i64()) {
                    Ok(fv > v)
                } else if let (Some(fv), Some(v)) = (field_value.as_f64(), value.as_f64()) {
                    Ok(fv > v)
                } else {
                    Err(AppError::ValidationError("Cannot compare non-numeric values".to_string()))
                }
            }
            Condition::LessThan { field, value } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let (Some(fv), Some(v)) = (field_value.as_i64(), value.as_i64()) {
                    Ok(fv < v)
                } else if let (Some(fv), Some(v)) = (field_value.as_f64(), value.as_f64()) {
                    Ok(fv < v)
                } else {
                    Err(AppError::ValidationError("Cannot compare non-numeric values".to_string()))
                }
            }
            Condition::Contains { field, value } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let Some(s) = field_value.as_str() {
                    Ok(s.contains(value))
                } else if let Some(arr) = field_value.as_array() {
                    Ok(arr.iter().any(|v| v.as_str().map(|s| s.contains(value)).unwrap_or(false)))
                } else {
                    Err(AppError::ValidationError("Field must be string or array".to_string()))
                }
            }
            Condition::Matches { field, pattern } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let Some(s) = field_value.as_str() {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| AppError::ValidationError(format!("Invalid regex: {}", e)))?;
                    Ok(regex.is_match(s))
                } else {
                    Err(AppError::ValidationError("Field must be string".to_string()))
                }
            }
            Condition::And { conditions } => {
                for condition in conditions {
                    if !condition.evaluate(context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or { conditions } => {
                for condition in conditions {
                    if condition.evaluate(context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not { condition } => {
                Ok(!condition.evaluate(context)?)
            }
            Condition::TimeAfter { field, timestamp } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let Some(ts) = field_value.as_i64() {
                    Ok(ts > *timestamp)
                } else {
                    Err(AppError::ValidationError("Field must be timestamp".to_string()))
                }
            }
            Condition::TimeBefore { field, timestamp } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let Some(ts) = field_value.as_i64() {
                    Ok(ts < *timestamp)
                } else {
                    Err(AppError::ValidationError("Field must be timestamp".to_string()))
                }
            }
            Condition::TimeBetween { field, start, end } => {
                let field_value = context.get(field)
                    .ok_or_else(|| AppError::ValidationError(format!("Field not found: {}", field)))?;
                if let Some(ts) = field_value.as_i64() {
                    Ok(ts >= *start && ts <= *end)
                } else {
                    Err(AppError::ValidationError("Field must be timestamp".to_string()))
                }
            }
            Condition::EventCount { event_type: _, operator: _, threshold: _ } => {
                // This would need event history context
                Ok(true) // Placeholder
            }
            Condition::Sequence { events: _ } => {
                // This would need event history context
                Ok(true) // Placeholder
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // Notification actions
    SendWebhook { url: String, payload: serde_json::Value },
    SendEmail { to: String, subject: String, body: String },
    SendSms { to: String, message: String },
    
    // Data actions
    UpdateField { field: String, value: serde_json::Value },
    CreateRecord { table: String, data: serde_json::Value },
    DeleteRecord { table: String, id: String },
    
    // Blockchain actions
    CallContract { contract: String, method: String, args: serde_json::Value },
    TransferAsset { from: String, to: String, amount: String },
    
    // Workflow actions
    TriggerWorkflow { workflow_id: String, params: serde_json::Value },
    SetState { key: String, value: serde_json::Value },
    
    // Custom actions
    Custom { name: String, params: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub condition: Condition,
    pub actions: Vec<Action>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Rule {
    pub fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> Result<bool, AppError> {
        if !self.enabled {
            return Ok(false);
        }
        self.condition.evaluate(context)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub id: String,
    pub name: String,
    pub rules: Vec<Rule>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RuleSet {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> Result<Vec<&Rule>, AppError> {
        let mut matched = Vec::new();
        for rule in &self.rules {
            if rule.evaluate(context)? {
                matched.push(rule);
            }
        }
        Ok(matched)
    }
}

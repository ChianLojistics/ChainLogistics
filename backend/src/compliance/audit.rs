use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub changes: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: i64,
}

pub struct AuditLogger;

impl AuditLogger {
    pub fn log_action(
        user_id: Option<String>,
        action: String,
        resource_type: String,
        resource_id: String,
        changes: Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            action,
            resource_type,
            resource_id,
            changes,
            ip_address,
            user_agent,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn log_data_access(user_id: String, resource_id: String, ip_address: Option<String>) -> AuditLogEntry {
        Self::log_action(
            Some(user_id),
            "data_access".to_string(),
            "product".to_string(),
            resource_id,
            serde_json::json!({}),
            ip_address,
            None,
        )
    }

    pub fn log_data_modification(
        user_id: String,
        resource_id: String,
        changes: Value,
    ) -> AuditLogEntry {
        Self::log_action(
            Some(user_id),
            "data_modification".to_string(),
            "product".to_string(),
            resource_id,
            changes,
            None,
            None,
        )
    }

    pub fn log_data_deletion(user_id: String, resource_id: String) -> AuditLogEntry {
        Self::log_action(
            Some(user_id),
            "data_deletion".to_string(),
            "product".to_string(),
            resource_id,
            serde_json::json!({"deleted": true}),
            None,
            None,
        )
    }

    pub fn log_compliance_check(
        compliance_type: String,
        resource_id: String,
        result: Value,
    ) -> AuditLogEntry {
        Self::log_action(
            None,
            "compliance_check".to_string(),
            "compliance".to_string(),
            resource_id,
            result,
            None,
            None,
        )
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RegulatoryRequirement {
    pub id: Uuid,
    pub requirement_id: String,
    pub name: String,
    pub description: Option<String>,
    pub regulation_type: String,
    pub category: String,
    pub severity: String,
    pub required_fields: serde_json::Value,
    pub validation_logic: Option<String>,
    pub is_active: bool,
    pub effective_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewRegulatoryRequirement {
    pub requirement_id: String,
    pub name: String,
    pub description: Option<String>,
    pub regulation_type: String,
    pub category: String,
    pub severity: String,
    pub required_fields: serde_json::Value,
    pub validation_logic: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProductCompliance {
    pub id: Uuid,
    pub product_id: String,
    pub requirement_id: String,
    pub status: String,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_check_result: Option<serde_json::Value>,
    pub violations: serde_json::Value,
    pub warnings: serde_json::Value,
    pub evidence_documents: Vec<String>,
    pub notes: Option<String>,
    pub checked_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewProductCompliance {
    pub product_id: String,
    pub requirement_id: String,
    pub status: String,
    pub notes: Option<String>,
    pub checked_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ComplianceAuditTrail {
    pub id: Uuid,
    pub product_id: String,
    pub requirement_id: Option<String>,
    pub action_type: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub action_details: serde_json::Value,
    pub performed_by: String,
    pub performed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewComplianceAuditTrail {
    pub product_id: String,
    pub requirement_id: Option<String>,
    pub action_type: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub action_details: serde_json::Value,
    pub performed_by: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ComplianceReport {
    pub id: Uuid,
    pub report_id: String,
    pub report_type: String,
    pub scope: serde_json::Value,
    pub generated_by: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub total_products_checked: i32,
    pub compliant_count: i32,
    pub non_compliant_count: i32,
    pub pending_count: i32,
    pub compliance_rate: Option<rust_decimal::Decimal>,
    pub report_data: serde_json::Value,
    pub file_path: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewComplianceReport {
    pub report_type: String,
    pub scope: serde_json::Value,
    pub generated_by: String,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComplianceCheckRequest {
    pub product_id: String,
    pub requirement_ids: Option<Vec<String>>,
    pub performed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComplianceCheckResult {
    pub product_id: String,
    pub total_requirements: i32,
    pub compliant: i32,
    pub non_compliant: i32,
    pub pending: i32,
    pub details: Vec<ComplianceDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComplianceDetail {
    pub requirement_id: String,
    pub requirement_name: String,
    pub regulation_type: String,
    pub status: String,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

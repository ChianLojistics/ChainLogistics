use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QCCheckpoint {
    pub id: Uuid,
    pub checkpoint_id: String,
    pub name: String,
    pub description: Option<String>,
    pub checkpoint_type: String,
    pub category: String,
    pub product_category: Option<String>,
    pub required_fields: serde_json::Value,
    pub acceptance_criteria: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewQCCheckpoint {
    pub checkpoint_id: String,
    pub name: String,
    pub description: Option<String>,
    pub checkpoint_type: String,
    pub category: String,
    pub product_category: Option<String>,
    pub required_fields: serde_json::Value,
    pub acceptance_criteria: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QCWorkflow {
    pub id: Uuid,
    pub workflow_id: String,
    pub name: String,
    pub description: Option<String>,
    pub product_category: Option<String>,
    pub checkpoint_ids: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewQCWorkflow {
    pub workflow_id: String,
    pub name: String,
    pub description: Option<String>,
    pub product_category: Option<String>,
    pub checkpoint_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QCInspection {
    pub id: Uuid,
    pub inspection_id: String,
    pub product_id: String,
    pub checkpoint_id: String,
    pub workflow_id: Option<String>,
    pub status: String,
    pub inspector_id: Option<String>,
    pub inspection_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub results: serde_json::Value,
    pub quality_metrics: serde_json::Value,
    pub notes: Option<String>,
    pub evidence_documents: Vec<String>,
    pub is_passed: Option<bool>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewQCInspection {
    pub inspection_id: String,
    pub product_id: String,
    pub checkpoint_id: String,
    pub workflow_id: Option<String>,
    pub inspector_id: Option<String>,
    pub location: Option<String>,
    pub results: serde_json::Value,
    pub quality_metrics: serde_json::Value,
    pub notes: Option<String>,
    pub evidence_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct NonConformance {
    pub id: Uuid,
    pub nc_id: String,
    pub inspection_id: Option<Uuid>,
    pub product_id: String,
    pub severity: String,
    pub category: String,
    pub description: String,
    pub root_cause: Option<String>,
    pub correction_action: Option<String>,
    pub correction_status: String,
    pub responsible_party: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub verified_by: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewNonConformance {
    pub nc_id: String,
    pub inspection_id: Option<Uuid>,
    pub product_id: String,
    pub severity: String,
    pub category: String,
    pub description: String,
    pub root_cause: Option<String>,
    pub correction_action: Option<String>,
    pub responsible_party: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QualityMetric {
    pub id: Uuid,
    pub metric_id: String,
    pub product_id: Option<String>,
    pub metric_type: String,
    pub metric_value: Decimal,
    pub unit: Option<String>,
    pub measurement_period_start: DateTime<Utc>,
    pub measurement_period_end: DateTime<Utc>,
    pub target_value: Option<Decimal>,
    pub threshold_min: Option<Decimal>,
    pub threshold_max: Option<Decimal>,
    pub is_within_threshold: Option<bool>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewQualityMetric {
    pub metric_id: String,
    pub product_id: Option<String>,
    pub metric_type: String,
    pub metric_value: Decimal,
    pub unit: Option<String>,
    pub measurement_period_start: DateTime<Utc>,
    pub measurement_period_end: DateTime<Utc>,
    pub target_value: Option<Decimal>,
    pub threshold_min: Option<Decimal>,
    pub threshold_max: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkflowExecutionRequest {
    pub product_id: String,
    pub workflow_id: String,
    pub inspector_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkflowExecutionResult {
    pub workflow_id: String,
    pub product_id: String,
    pub total_checkpoints: i32,
    pub completed: i32,
    pub passed: i32,
    pub failed: i32,
    pub skipped: i32,
    pub overall_status: String,
    pub inspections: Vec<QCInspection>,
}

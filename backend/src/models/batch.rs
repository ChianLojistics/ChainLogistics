use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "batch_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BatchStatus {
    Pending,
    InProduction,
    Completed,
    QualityCheck,
    Quarantined,
    Shipped,
    Recalled,
    Disposed,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Batch {
    pub id: Uuid,
    pub batch_number: String,
    pub product_id: String,
    pub lot_number: Option<String>,
    pub production_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub quantity_produced: i32,
    pub quantity_available: i32,
    pub status: BatchStatus,
    pub production_location: String,
    pub quality_grade: Option<String>,
    pub quality_score: Option<rust_decimal::Decimal>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewBatch {
    pub batch_number: String,
    pub product_id: String,
    pub lot_number: Option<String>,
    pub production_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub quantity_produced: i32,
    pub quantity_available: i32,
    pub status: BatchStatus,
    pub production_location: String,
    pub quality_grade: Option<String>,
    pub quality_score: Option<rust_decimal::Decimal>,
    pub metadata: serde_json::Value,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BatchGenealogy {
    pub id: Uuid,
    pub parent_batch_id: Uuid,
    pub child_batch_id: Uuid,
    pub relationship_type: String,
    pub quantity_transferred: i32,
    pub transfer_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewBatchGenealogy {
    pub parent_batch_id: Uuid,
    pub child_batch_id: Uuid,
    pub relationship_type: String,
    pub quantity_transferred: i32,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BatchQualityAttribute {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub attribute_name: String,
    pub attribute_value: String,
    pub measurement_unit: Option<String>,
    pub tolerance_min: Option<rust_decimal::Decimal>,
    pub tolerance_max: Option<rust_decimal::Decimal>,
    pub measured_at: DateTime<Utc>,
    pub measured_by: String,
    pub is_within_tolerance: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewBatchQualityAttribute {
    pub batch_id: Uuid,
    pub attribute_name: String,
    pub attribute_value: String,
    pub measurement_unit: Option<String>,
    pub tolerance_min: Option<rust_decimal::Decimal>,
    pub tolerance_max: Option<rust_decimal::Decimal>,
    pub measured_by: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BatchRecall {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub recall_type: String,
    pub recall_reason: String,
    pub recall_date: DateTime<Utc>,
    pub initiated_by: String,
    pub severity: String,
    pub status: String,
    pub affected_quantity: i32,
    pub recovered_quantity: i32,
    pub notification_sent: bool,
    pub public_announcement: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewBatchRecall {
    pub batch_id: Uuid,
    pub recall_type: String,
    pub recall_reason: String,
    pub initiated_by: String,
    pub severity: String,
    pub affected_quantity: i32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BatchInventory {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub location_id: String,
    pub quantity: i32,
    pub transaction_type: String,
    pub reference_id: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub performed_by: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewBatchInventory {
    pub batch_id: Uuid,
    pub location_id: String,
    pub quantity: i32,
    pub transaction_type: String,
    pub reference_id: Option<String>,
    pub performed_by: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchFilters {
    pub product_id: Option<String>,
    pub lot_number: Option<String>,
    pub status: Option<BatchStatus>,
    pub quality_grade: Option<String>,
    pub production_after: Option<DateTime<Utc>>,
    pub production_before: Option<DateTime<Utc>>,
    pub expiry_after: Option<DateTime<Utc>>,
    pub expiry_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchGenealogyTree {
    pub batch: Batch,
    pub parents: Vec<BatchGenealogyNode>,
    pub children: Vec<BatchGenealogyNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchGenealogyNode {
    pub genealogy: BatchGenealogy,
    pub related_batch: Batch,
}

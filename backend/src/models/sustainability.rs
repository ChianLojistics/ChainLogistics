use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IotReading {
    pub id: Uuid,
    pub product_id: String,
    pub sensor_id: String,
    pub metric_type: String,
    pub value: rust_decimal::Decimal,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SustainabilityMetric {
    pub id: Uuid,
    pub product_id: String,
    pub metric_type: String,
    pub value: rust_decimal::Decimal,
    pub unit: String,
    pub verified: bool,
    pub last_updated: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SustainabilityVerification {
    pub id: Uuid,
    pub product_id: String,
    pub metric_type: String,
    pub verifier_name: String,
    pub status: String,
    pub certificate_hash: Option<String>,
    pub certificate_url: Option<String>,
    pub blockchain_tx_hash: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddIotReadingRequest {
    pub product_id: String,
    pub sensor_id: String,
    pub metric_type: String,
    pub value: rust_decimal::Decimal,
    pub unit: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyMetricRequest {
    pub metric_type: String,
    pub verifier_name: String,
    pub certificate_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SustainabilityReport {
    pub id: Uuid,
    pub product_id: String,
    pub report_type: String,
    pub content: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
}

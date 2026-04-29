use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct IoTDevice {
    pub id: Uuid,
    pub device_id: String,
    pub device_type: String,
    pub product_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub calibration_date: Option<DateTime<Utc>>,
    pub next_calibration_date: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewIoTDevice {
    pub device_id: String,
    pub device_type: String,
    pub product_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub location: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TemperatureReading {
    pub id: Uuid,
    pub device_id: String,
    pub product_id: Option<String>,
    pub temperature_celsius: Decimal,
    pub humidity_percent: Option<Decimal>,
    pub unit: String,
    pub reading_timestamp: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub location: Option<String>,
    pub quality_score: Option<Decimal>,
    pub is_anomaly: bool,
    pub anomaly_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewTemperatureReading {
    pub device_id: String,
    pub product_id: Option<String>,
    pub temperature_celsius: Decimal,
    pub humidity_percent: Option<Decimal>,
    pub unit: String,
    pub reading_timestamp: DateTime<Utc>,
    pub location: Option<String>,
    pub quality_score: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TemperatureThreshold {
    pub id: Uuid,
    pub product_id: String,
    pub device_id: Option<String>,
    pub threshold_type: String,
    pub min_temperature_celsius: Option<Decimal>,
    pub max_temperature_celsius: Option<Decimal>,
    pub duration_minutes: Option<i32>,
    pub alert_level: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewTemperatureThreshold {
    pub product_id: String,
    pub device_id: Option<String>,
    pub threshold_type: String,
    pub min_temperature_celsius: Option<Decimal>,
    pub max_temperature_celsius: Option<Decimal>,
    pub duration_minutes: Option<i32>,
    pub alert_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TemperatureAlert {
    pub id: Uuid,
    pub device_id: String,
    pub product_id: String,
    pub threshold_id: Option<Uuid>,
    pub alert_type: String,
    pub alert_level: String,
    pub temperature_celsius: Option<Decimal>,
    pub threshold_value: Option<Decimal>,
    pub message: String,
    pub is_resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewTemperatureAlert {
    pub device_id: String,
    pub product_id: String,
    pub threshold_id: Option<Uuid>,
    pub alert_type: String,
    pub alert_level: String,
    pub temperature_celsius: Option<Decimal>,
    pub threshold_value: Option<Decimal>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TemperatureSummary {
    pub id: Uuid,
    pub device_id: String,
    pub product_id: Option<String>,
    pub summary_period: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub avg_temperature_celsius: Option<Decimal>,
    pub min_temperature_celsius: Option<Decimal>,
    pub max_temperature_celsius: Option<Decimal>,
    pub total_readings: Option<i32>,
    pub anomaly_count: Option<i32>,
    pub alert_count: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemperatureHistoryQuery {
    pub device_id: Option<String>,
    pub product_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub include_anomalies_only: Option<bool>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

pub mod analytics;
pub mod carbon;
pub mod digital_twin;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
    pub media_hashes: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TrackingEvent {
    pub id: i64,
    pub product_id: String,
    pub actor_address: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub location: String,
    pub data_hash: String,
    pub note: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Recall {
    pub id: Uuid,
    pub product_id: String,
    pub batch_id: Option<String>,
    pub title: String,
    pub reason: String,
    pub severity: String,
    pub status: String,
    pub trigger_type: String,
    pub triggered_by: Option<String>,
    pub triggered_event_id: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RecallAffectedItem {
    pub id: Uuid,
    pub recall_id: Uuid,
    pub product_id: String,
    pub batch_id: Option<String>,
    pub stakeholder_role: Option<String>,
    pub stakeholder_address: Option<String>,
    pub detected_via: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RecallNotification {
    pub id: Uuid,
    pub recall_id: Uuid,
    pub recipient: String,
    pub channel: String,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub payload: serde_json::Value,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RecallEffectiveness {
    pub recall_id: Uuid,
    pub affected_count: i32,
    pub notified_count: i32,
    pub acknowledged_count: i32,
    pub recovered_count: i32,
    pub disposed_count: i32,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Supplier,
    Carrier,
    Inspector,
    Customer,
    Administrator,
    Auditor,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub stellar_address: Option<String>,
    pub role: UserRole,
    pub api_key: Option<String>,
    pub api_key_hash: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub tier: ApiKeyTier,
    pub rate_limit_per_minute: i32,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "text")]
pub enum ApiKeyTier {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Webhook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProductStats {
    pub product_id: String,
    pub event_count: i64,
    pub is_active: bool,
    pub last_event_at: Option<DateTime<Utc>>,
    pub last_event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewProduct {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
    pub media_hashes: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_address: String,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewTrackingEvent {
    pub product_id: String,
    pub actor_address: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub location: String,
    pub data_hash: String,
    pub note: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub stellar_address: Option<String>,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewApiKey {
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub tier: ApiKeyTier,
    pub rate_limit_per_minute: i32,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewWebhook {
    pub user_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
}
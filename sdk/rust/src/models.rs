use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents an API key tier with different rate limits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ApiKeyTier {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

impl Default for ApiKeyTier {
    fn default() -> Self {
        ApiKeyTier::Basic
    }
}

/// Product model representing a supply chain product
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// New product for creation requests
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Product update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub origin_location: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub certifications: Option<Vec<String>>,
    pub media_hashes: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub updated_by: String,
}

/// Tracking event model
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// New tracking event for creation requests
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub stellar_address: Option<String>,
    pub api_key: Option<String>,
    pub api_key_hash: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// New user for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub stellar_address: Option<String>,
}

/// API key model
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// New API key for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewApiKey {
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub tier: ApiKeyTier,
    pub rate_limit_per_minute: i32,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Webhook model
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Product statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductStats {
    pub product_id: String,
    pub event_count: i64,
    pub is_active: bool,
    pub last_event_at: Option<DateTime<Utc>>,
    pub last_event_type: Option<String>,
}

/// Global statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_products: i64,
    pub active_products: i64,
    pub total_events: i64,
    pub total_users: i64,
    pub active_api_keys: i64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub service: String,
}

/// Database health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbHealthResponse {
    pub status: String,
    pub database: String,
    pub timestamp: DateTime<Utc>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Product list query parameters
#[derive(Debug, Clone, Default)]
pub struct ProductListQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub owner_address: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

/// Event list query parameters
#[derive(Debug, Clone, Default)]
pub struct EventListQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub product_id: Option<String>,
    pub event_type: Option<String>,
}

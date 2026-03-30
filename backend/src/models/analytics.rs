use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Dashboard Analytics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub total_products: i64,
    pub active_products: i64,
    pub inactive_products: i64,
    pub total_events: i64,
    pub total_users: i64,
    pub events_last_24h: i64,
    pub events_last_7d: i64,
    pub events_last_30d: i64,
    pub products_registered_last_30d: i64,
    pub top_event_types: Vec<EventTypeCount>,
    pub top_categories: Vec<CategoryCount>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
    pub active_count: i64,
}

// --- Product Analytics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAnalytics {
    pub product_id: String,
    pub product_name: String,
    pub category: String,
    pub is_active: bool,
    pub total_events: i64,
    pub unique_actors: i64,
    pub unique_locations: i64,
    pub first_event_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub lifecycle_days: Option<i64>,
    pub event_type_breakdown: Vec<EventTypeCount>,
    pub event_time_series: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: String, // ISO date string YYYY-MM-DD
    pub count: i64,
}

// --- Event Analytics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAnalytics {
    pub total_events: i64,
    pub events_by_type: Vec<EventTypeCount>,
    pub events_by_location: Vec<LocationCount>,
    pub events_by_actor: Vec<ActorCount>,
    pub hourly_distribution: Vec<HourlyCount>,
    pub daily_time_series: Vec<TimeSeriesPoint>,
    pub avg_events_per_product: f64,
    pub most_active_products: Vec<ProductEventCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCount {
    pub location: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorCount {
    pub actor_address: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyCount {
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductEventCount {
    pub product_id: String,
    pub product_name: String,
    pub event_count: i64,
}

// --- User Analytics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub total_users: i64,
    pub active_users: i64,
    pub users_with_stellar: i64,
    pub new_users_last_30d: i64,
    pub total_api_keys: i64,
    pub active_api_keys: i64,
    pub api_keys_by_tier: Vec<ApiKeyTierCount>,
    pub user_registration_series: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyTierCount {
    pub tier: String,
    pub count: i64,
}

// --- Time Series Query Params ---

#[derive(Debug, Clone, Deserialize)]
pub struct TimeSeriesQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub granularity: Option<String>, // "day", "week", "month"
    pub product_id: Option<String>,
    pub event_type: Option<String>,
    pub category: Option<String>,
}

// --- Export ---

#[derive(Debug, Clone, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>, // "csv" or "json"
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub product_id: Option<String>,
    pub event_type: Option<String>,
    pub limit: Option<i64>,
}

// --- Cache key helpers ---

#[derive(Debug, Clone)]
pub struct CacheKey;

impl CacheKey {
    pub fn dashboard() -> &'static str {
        "analytics:dashboard"
    }

    pub fn event_analytics(start: &str, end: &str) -> String {
        format!("analytics:events:{}:{}", start, end)
    }

    pub fn user_analytics() -> &'static str {
        "analytics:users"
    }

    pub fn product_analytics(product_id: &str) -> String {
        format!("analytics:product:{}", product_id)
    }
}

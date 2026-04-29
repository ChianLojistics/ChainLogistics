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

// --- Resilience Analytics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceAnalytics {
    pub risk_score: f64, // 0.0 to 1.0
    pub disruption_predictions: Vec<DisruptionPrediction>,
    pub risk_assessment: RiskAssessment,
    pub alternative_sources: Vec<AlternativeSource>,
    pub safety_stock_recommendations: Vec<InventoryRecommendation>,
    pub scenarios: Vec<DisruptionScenario>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionPrediction {
    pub factor: String, // e.g., "Weather", "Geopolitical", "Supplier"
    pub probability: f64,
    pub severity: String, // "Low", "Medium", "High", "Critical"
    pub description: String,
    pub estimated_impact_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub supplier_risk: f64,
    pub geographic_risk: f64,
    pub logistics_risk: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeSource {
    pub original_supplier_id: String,
    pub backup_supplier_id: String,
    pub backup_supplier_name: String,
    pub reliability_score: f64,
    pub cost_difference_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryRecommendation {
    pub product_id: String,
    pub product_name: String,
    pub current_safety_stock: i32,
    pub recommended_safety_stock: i32,
    pub reasoning: String,
}

// --- Fraud Analytics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAnalytics {
    pub overall_fraud_score: f64,
    pub anomaly_reports: Vec<AnomalyReport>,
    pub behavioral_analysis: BehavioralAnalysis,
    pub supplier_graph: SupplierGraph,
    pub recent_alerts: Vec<FraudAlert>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub product_id: String,
    pub anomaly_type: String, // "Location Spoofing", "Speed Violation", "Duplicate Registration"
    pub confidence_score: f64,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysis {
    pub actor_reputation_scores: Vec<ActorScore>,
    pub unusual_activity_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorScore {
    pub actor_address: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAlert {
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

// --- Scenario Modeling ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionScenario {
    pub name: String,
    pub description: String,
    pub probability: f64,
    pub impacted_products_count: i32,
    pub estimated_revenue_loss: f64,
    pub recovery_time_days: i32,
    pub mitigation_strategies: Vec<String>,
}

// --- Relationship Graph ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "Supplier", "Product", "Hub"
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relationship: String, // "Supplies", "Transports", "OwnedBy"
    pub criticality: f64,
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

    pub fn resilience_analytics() -> &'static str {
        "analytics:resilience"
    }

    pub fn fraud_analytics() -> &'static str {
        "analytics:fraud"
    }
}

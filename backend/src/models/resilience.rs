use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub risk_score: f64,
    pub risk_level: String,
    pub factors: serde_json::Value,
    pub last_assessment_at: DateTime<Utc>,
    pub next_assessment_due: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DisruptionAlert {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub alert_type: String,
    pub severity: String,
    pub description: String,
    pub probability: f64,
    pub estimated_impact_usd: Option<f64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ResiliencePlan {
    pub id: Uuid,
    pub alert_id: Option<Uuid>,
    pub product_id: String,
    pub mitigation_strategies: Vec<String>,
    pub backup_suppliers: serde_json::Value,
    pub alternative_routes: serde_json::Value,
    pub safety_stock_recommendation: Option<f64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventoryOptimization {
    pub product_id: String,
    pub current_stock: f64,
    pub safety_stock_level: f64,
    pub reorder_point: f64,
    pub lead_time_days: f64,
    pub variability_factor: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScenarioRequest {
    pub scenario_type: String,
    pub focus_entities: Vec<String>,
    pub duration_days: i32,
    pub custom_parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScenarioReport {
    pub scenario_id: Uuid,
    pub name: String,
    pub description: String,
    pub total_impact_score: f64,
    pub high_risk_entities: Vec<String>,
    pub critical_disruption_points: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

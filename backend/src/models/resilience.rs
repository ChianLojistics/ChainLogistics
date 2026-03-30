"""use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DisruptionPrediction {
    pub id: Uuid,
    pub product_id: String,
    pub predicted_at: DateTime<Utc>,
    pub probability: f64,
    pub impact_level: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SupplierRisk {
    pub id: Uuid,
    pub supplier_name: String,
    pub risk_score: f64,
    pub risk_factors: serde_json::Value,
    pub last_assessed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GeographicRisk {
    pub id: Uuid,
    pub location: String,
    pub risk_score: f64,
    pub risk_factors: serde_json::Value,
    pub last_assessed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlternativeSource {
    pub id: Uuid,
    pub product_id: String,
    pub alternative_supplier: String,
    pub viability_score: f64,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InventoryRecommendation {
    pub id: Uuid,
    pub product_id: String,
    pub recommended_safety_stock: i32,
    pub rationale: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResilienceMetrics {
    pub disruption_predictions: Vec<DisruptionPrediction>,
    pub supplier_risks: Vec<SupplierRisk>,
    pub geographic_risks: Vec<GeographicRisk>,
    pub alternative_sources: Vec<AlternativeSource>,
    pub inventory_recommendations: Vec<InventoryRecommendation>,
}
""
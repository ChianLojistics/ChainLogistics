use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Digital Twin represents a virtual replica of a physical supply chain entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DigitalTwin {
    pub id: Uuid,
    pub product_id: String,
    pub twin_type: TwinType,
    pub name: String,
    pub description: String,
    pub current_state: serde_json::Value,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum TwinType {
    Product,
    Warehouse,
    Vehicle,
    Container,
    Facility,
}

/// Simulation represents a what-if scenario for supply chain optimization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Simulation {
    pub id: Uuid,
    pub twin_id: Uuid,
    pub name: String,
    pub description: String,
    pub simulation_type: SimulationType,
    pub parameters: serde_json::Value,
    pub status: SimulationStatus,
    pub results: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SimulationType {
    RouteOptimization,
    DemandForecasting,
    InventoryOptimization,
    RiskAssessment,
    CostAnalysis,
    TimelineProjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SimulationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// TwinState captures the state of a digital twin at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TwinState {
    pub id: Uuid,
    pub twin_id: Uuid,
    pub state_data: serde_json::Value,
    pub metrics: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Prediction represents AI/ML-based predictions for supply chain events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prediction {
    pub id: Uuid,
    pub twin_id: Uuid,
    pub prediction_type: PredictionType,
    pub predicted_value: serde_json::Value,
    pub confidence_score: f64,
    pub prediction_horizon: i32, // hours
    pub created_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub actual_value: Option<serde_json::Value>,
    pub accuracy_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum PredictionType {
    ArrivalTime,
    Delay,
    QualityIssue,
    DemandSpike,
    SupplyDisruption,
    CostOverrun,
}

/// Optimization represents optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Optimization {
    pub id: Uuid,
    pub twin_id: Uuid,
    pub optimization_type: OptimizationType,
    pub current_metrics: serde_json::Value,
    pub optimized_metrics: serde_json::Value,
    pub recommendations: Vec<String>,
    pub estimated_savings: Option<f64>,
    pub implementation_complexity: ComplexityLevel,
    pub created_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum OptimizationType {
    Route,
    Inventory,
    Cost,
    Time,
    Carbon,
    Risk,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
}

/// TwinMetrics aggregates key performance indicators for a digital twin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinMetrics {
    pub twin_id: Uuid,
    pub total_events: i64,
    pub avg_transit_time: Option<f64>,
    pub on_time_delivery_rate: Option<f64>,
    pub quality_score: Option<f64>,
    pub cost_efficiency: Option<f64>,
    pub carbon_footprint: Option<f64>,
    pub risk_score: Option<f64>,
    pub last_updated: DateTime<Utc>,
}

// Request/Response DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDigitalTwinRequest {
    pub product_id: String,
    pub twin_type: TwinType,
    pub name: String,
    pub description: String,
    pub initial_state: serde_json::Value,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTwinStateRequest {
    pub state_data: serde_json::Value,
    pub metrics: serde_json::Value,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSimulationRequest {
    pub twin_id: Uuid,
    pub name: String,
    pub description: String,
    pub simulation_type: SimulationType,
    pub parameters: serde_json::Value,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub simulation_id: Uuid,
    pub status: SimulationStatus,
    pub results: Option<serde_json::Value>,
    pub execution_time_ms: Option<i64>,
    pub insights: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinAnalytics {
    pub twin_id: Uuid,
    pub metrics: TwinMetrics,
    pub recent_predictions: Vec<Prediction>,
    pub active_optimizations: Vec<Optimization>,
    pub simulation_count: i64,
    pub state_history_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRequest {
    pub twin_id: Uuid,
    pub optimization_type: OptimizationType,
    pub constraints: serde_json::Value,
    pub objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub twin_id: Uuid,
    pub prediction_type: PredictionType,
    pub prediction_horizon: i32,
    pub input_features: serde_json::Value,
}

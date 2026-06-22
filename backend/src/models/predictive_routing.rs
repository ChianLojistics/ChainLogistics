use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Graph topology ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RouteNode {
    pub id: Uuid,
    pub node_key: String,
    pub display_name: String,
    pub node_type: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub avg_dwell_time_hours: f64,
    pub historical_delay_rate: f64,
    pub total_shipments_through: i32,
    /// Aggregated anomaly signal propagated from graph neighbours (GNN message pass)
    pub anomaly_signal: f64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RouteEdge {
    pub id: Uuid,
    pub source_node_id: Uuid,
    pub dest_node_id: Uuid,
    pub avg_transit_hours: f64,
    pub stddev_transit_hours: f64,
    pub historical_delay_rate: f64,
    pub total_shipments: i32,
    pub is_disrupted: bool,
    pub disruption_reason: Option<String>,
    pub disruption_started_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Shipment routes ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShipmentRoute {
    pub id: Uuid,
    pub product_id: String,
    pub batch_id: Option<String>,
    pub planned_path: serde_json::Value,
    pub actual_path: serde_json::Value,
    pub current_node_id: Option<Uuid>,
    pub origin_node_id: Option<Uuid>,
    pub destination_node_id: Option<Uuid>,
    pub status: String,
    pub planned_arrival_at: Option<DateTime<Utc>>,
    pub actual_arrival_at: Option<DateTime<Utc>>,
    pub departed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Anomaly scores ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnomalyScore {
    pub id: Uuid,
    pub shipment_route_id: Uuid,
    pub product_id: String,
    pub composite_score: f64,
    pub temporal_score: f64,
    pub path_score: f64,
    pub env_score: f64,
    pub graph_score: f64,
    pub risk_level: String,
    pub feature_vector: serde_json::Value,
    pub model_version: String,
    pub computed_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScoreSummary {
    pub shipment_route_id: Uuid,
    pub product_id: String,
    pub composite_score: f64,
    pub risk_level: RiskLevel,
    pub sub_scores: SubScores,
    pub computed_at: DateTime<Utc>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubScores {
    pub temporal: f64,
    pub path: f64,
    pub environmental: f64,
    pub graph: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.85 => RiskLevel::Critical,
            s if s >= 0.60 => RiskLevel::High,
            s if s >= 0.35 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── Routing alerts ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoutingAlert {
    pub id: Uuid,
    pub shipment_route_id: Uuid,
    pub anomaly_score_id: Option<Uuid>,
    pub product_id: String,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub recommendations: serde_json::Value,
    pub status: String,
    pub triggered_score: f64,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    DelayPredicted,
    RouteDisruption,
    EnvAnomaly,
    CriticalRisk,
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertType::DelayPredicted => "delay_predicted",
            AlertType::RouteDisruption => "route_disruption",
            AlertType::EnvAnomaly => "env_anomaly",
            AlertType::CriticalRisk => "critical_risk",
        }
    }
}

// ── In-memory graph representation (used by the scoring engine) ───────────────

#[derive(Debug, Clone)]
pub struct RouteGraph {
    pub nodes: Vec<RouteNode>,
    pub edges: Vec<RouteEdge>,
}

impl RouteGraph {
    pub fn neighbours(&self, node_id: Uuid) -> Vec<&RouteNode> {
        self.edges
            .iter()
            .filter(|e| e.source_node_id == node_id || e.dest_node_id == node_id)
            .flat_map(|e| {
                let peer_id = if e.source_node_id == node_id {
                    e.dest_node_id
                } else {
                    e.source_node_id
                };
                self.nodes.iter().find(|n| n.id == peer_id)
            })
            .collect()
    }

    pub fn edge(&self, src: Uuid, dst: Uuid) -> Option<&RouteEdge> {
        self.edges
            .iter()
            .find(|e| e.source_node_id == src && e.dest_node_id == dst)
    }
}

// ── Feature vector ────────────────────────────────────────────────────────────

/// Features extracted for a single shipment, fed into the scoring functions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipmentFeatures {
    /// Hours elapsed since departure
    pub elapsed_hours: f64,
    /// Planned transit hours (0 if unknown)
    pub planned_hours: f64,
    /// Z-score of elapsed vs. historical mean for this route
    pub transit_z_score: f64,
    /// Fraction of planned route already visited
    pub path_completion_ratio: f64,
    /// Number of planned nodes skipped / deviated
    pub path_deviation_count: i32,
    /// Max IoT anomaly signal seen on path so far (0-1)
    pub max_iot_anomaly: f64,
    /// Average neighbour anomaly_signal from current node (GNN)
    pub neighbour_signal: f64,
    /// Whether any edge on the remaining path is marked disrupted
    pub downstream_disruption: bool,
    /// Historical delay rate of the highest-risk remaining edge
    pub worst_edge_delay_rate: f64,
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CreateShipmentRouteRequest {
    pub product_id: String,
    pub batch_id: Option<String>,
    pub origin: String,
    pub destination: String,
    pub waypoints: Vec<String>,
    pub planned_arrival_at: Option<DateTime<Utc>>,
    pub departed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateShipmentPositionRequest {
    pub current_location: String,
    pub iot_anomaly_signal: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub acknowledged_by: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportTrainingDataRequest {
    pub format: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub fl_partition_key: Option<String>,
    pub exported_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportTrainingDataResponse {
    pub export_id: Uuid,
    pub record_count: i64,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveAlertsResponse {
    pub total: i64,
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
    pub alerts: Vec<RoutingAlert>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnomalyScoreListResponse {
    pub total: i64,
    pub high_risk_count: i64,
    pub scores: Vec<AnomalyScoreSummary>,
}

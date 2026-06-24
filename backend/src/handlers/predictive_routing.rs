use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::predictive_routing::{
        AcknowledgeAlertRequest, CreateShipmentRouteRequest, ExportTrainingDataRequest,
        UpdateShipmentPositionRequest,
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
}

/// POST /api/v1/routing/shipments
/// Register a new shipment route (upserts graph nodes/edges).
pub async fn create_shipment_route(
    State(state): State<AppState>,
    Json(req): Json<CreateShipmentRouteRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let route = state
        .predictive_routing_service
        .create_shipment_route(req)
        .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "data": route }))))
}

/// GET /api/v1/routing/scores
/// Latest anomaly scores across all active shipments.
pub async fn list_anomaly_scores(
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = q.limit.unwrap_or(100).min(1000);
    let resp = state
        .predictive_routing_service
        .get_anomaly_scores(limit)
        .await?;

    Ok(Json(serde_json::json!({ "data": resp })))
}

/// GET /api/v1/routing/shipments/:id/score
/// Score (or re-score) a single shipment on demand.
pub async fn score_shipment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let summary = state.predictive_routing_service.score_shipment(id).await?;
    Ok(Json(serde_json::json!({ "data": summary })))
}

/// PUT /api/v1/routing/shipments/:id/position
/// Update the current position of a shipment and re-score.
pub async fn update_position(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateShipmentPositionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let summary = state
        .predictive_routing_service
        .update_position(id, req)
        .await?;

    Ok(Json(serde_json::json!({ "data": summary })))
}

/// GET /api/v1/routing/alerts
/// Active routing alerts (open + acknowledged).
pub async fn list_alerts(
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = q.limit.unwrap_or(50).min(500);
    let resp = state
        .predictive_routing_service
        .get_active_alerts(limit)
        .await?;

    Ok(Json(serde_json::json!({ "data": resp })))
}

/// POST /api/v1/routing/alerts/:id/acknowledge
/// Acknowledge an alert.
pub async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AcknowledgeAlertRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let alert = state
        .predictive_routing_service
        .acknowledge_alert(id, req)
        .await?;

    Ok(Json(serde_json::json!({ "data": alert })))
}

/// POST /api/v1/routing/alerts/:id/resolve
/// Resolve an alert.
pub async fn resolve_alert(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let alert = state
        .predictive_routing_service
        .resolve_alert(id)
        .await?;

    Ok(Json(serde_json::json!({ "data": alert })))
}

/// POST /api/v1/routing/export
/// Export training data for offline / federated ML retraining.
pub async fn export_training_data(
    State(state): State<AppState>,
    Json(req): Json<ExportTrainingDataRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let resp = state
        .predictive_routing_service
        .export_training_data(req)
        .await?;

    Ok(Json(serde_json::json!({ "data": resp })))
}

/// POST /api/v1/routing/score-all
/// Trigger a full scoring pass across all in-transit shipments (for cron use).
pub async fn score_all(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let summaries = state
        .predictive_routing_service
        .score_all_in_transit()
        .await?;

    Ok(Json(serde_json::json!({
        "data": {
            "scored": summaries.len(),
            "high_risk": summaries.iter().filter(|s| s.composite_score >= 0.60).count()
        }
    })))
}

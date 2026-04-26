use axum::{
    extract::{Path, State},
    response::Json,
};
use crate::AppState;
use crate::error::AppError;
use crate::models::resilience::{RiskAssessment, DisruptionAlert, InventoryOptimization, ScenarioRequest, ScenarioReport};

/// Get all risk assessments
#[utoipa::path(
    get,
    path = "/api/v1/resilience/risk-scores",
    responses(
        (status = 200, description = "List of risk assessments", body = [RiskAssessment]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("jwt" = []),
        ("api_key" = [])
    )
)]
pub async fn get_risk_scores(
    State(state): State<AppState>,
) -> Result<Json<Vec<RiskAssessment>>, AppError> {
    let assessments = state.resilience_service.get_risk_assessments().await?;
    Ok(Json(assessments))
}

/// Get all active disruption alerts
#[utoipa::path(
    get,
    path = "/api/v1/resilience/alerts",
    responses(
        (status = 200, description = "List of disruption alerts", body = [DisruptionAlert]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("jwt" = []),
        ("api_key" = [])
    )
)]
pub async fn get_disruption_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<DisruptionAlert>>, AppError> {
    let alerts = state.resilience_service.get_disruption_alerts().await?;
    Ok(Json(alerts))
}

/// Get inventory optimization recommendations for a product
#[utoipa::path(
    get,
    path = "/api/v1/resilience/inventory-optimization/{product_id}",
    params(
        ("product_id" = String, Path, description = "Product ID to optimize")
    ),
    responses(
        (status = 200, description = "Inventory optimization details", body = InventoryOptimization),
        (status = 404, description = "Product history not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("jwt" = []),
        ("api_key" = [])
    )
)]
pub async fn optimize_inventory(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<InventoryOptimization>, AppError> {
    let optimization = state.resilience_service.optimize_inventory(&product_id).await?;
    Ok(Json(optimization))
}

/// Generate a scenario planning report
#[utoipa::path(
    post,
    path = "/api/v1/resilience/scenario-plan",
    request_body = ScenarioRequest,
    responses(
        (status = 200, description = "Scenario report generated", body = ScenarioReport),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("jwt" = []),
        ("api_key" = [])
    )
)]
pub async fn generate_scenario_plan(
    State(state): State<AppState>,
    Json(request): Json<ScenarioRequest>,
) -> Result<Json<ScenarioReport>, AppError> {
    let report = state.resilience_service.generate_scenario_report(request).await?;
    Ok(Json(report))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::digital_twin::*,
    AppState,
};

/// Calculate health score for a digital twin
#[derive(Debug, Deserialize)]
pub struct HealthScoreRequest {
    pub current_temp: f64,
    pub current_humidity: f64,
    pub elapsed_hours: f64,
}

pub async fn calculate_health_score(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(request): Json<HealthScoreRequest>,
) -> Result<impl IntoResponse, AppError> {
    let result = state
        .physics_model_service
        .calculate_health_score(
            twin_id,
            request.current_temp,
            request.current_humidity,
            request.elapsed_hours,
        )
        .await?;

    Ok(Json(result))
}

/// Run Monte Carlo simulation
pub async fn run_monte_carlo(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(config): Json<MonteCarloSimulationConfig>,
) -> Result<impl IntoResponse, AppError> {
    let result = state
        .physics_model_service
        .run_monte_carlo_simulation(twin_id, config)
        .await?;

    Ok(Json(result))
}

/// Audit prediction accuracy
#[derive(Debug, Deserialize)]
pub struct AccuracyAuditRequest {
    pub prediction_id: Uuid,
    pub actual_value: serde_json::Value,
}

pub async fn audit_prediction_accuracy(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(request): Json<AccuracyAuditRequest>,
) -> Result<impl IntoResponse, AppError> {
    let audit = state
        .physics_model_service
        .audit_prediction_accuracy(twin_id, request.prediction_id, request.actual_value)
        .await?;

    Ok((StatusCode::CREATED, Json(audit)))
}

/// Get accuracy statistics
pub async fn get_accuracy_statistics(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let stats = state
        .physics_model_service
        .get_accuracy_statistics(twin_id)
        .await?;

    Ok(Json(stats))
}

/// Get health metrics for a twin
pub async fn get_health_metrics(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let metrics = sqlx::query_as::<_, TwinHealthMetric>(
        "SELECT * FROM twin_health_metrics WHERE twin_id = $1 ORDER BY calculated_at DESC LIMIT 100"
    )
    .bind(twin_id)
    .fetch_all(&state.db.pool())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(metrics))
}

/// Configure IoT to Twin sync
#[derive(Debug, Deserialize)]
pub struct IoTSyncConfig {
    pub device_id: String,
    pub sync_type: String,
    pub sync_frequency_seconds: i32,
    pub sync_parameters: serde_json::Value,
}

pub async fn configure_iot_sync(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(config): Json<IoTSyncConfig>,
) -> Result<impl IntoResponse, AppError> {
    let sync = sqlx::query_as::<_, IoTTwinSync>(
        r#"
        INSERT INTO iot_twin_sync (
            id, device_id, twin_id, sync_type, sync_frequency_seconds,
            sync_parameters, is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
        ON CONFLICT (device_id, twin_id, sync_type)
        DO UPDATE SET
            sync_frequency_seconds = EXCLUDED.sync_frequency_seconds,
            sync_parameters = EXCLUDED.sync_parameters,
            is_active = true,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&config.device_id)
    .bind(twin_id)
    .bind(&config.sync_type)
    .bind(config.sync_frequency_seconds)
    .bind(&config.sync_parameters)
    .fetch_one(&state.db.pool())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(sync)))
}

/// Get IoT sync configurations for a twin
pub async fn get_iot_syncs(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let syncs = sqlx::query_as::<_, IoTTwinSync>(
        "SELECT * FROM iot_twin_sync WHERE twin_id = $1 AND is_active = true"
    )
    .bind(twin_id)
    .fetch_all(&state.db.pool())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(syncs))
}

/// Update decay model parameters for a twin
#[derive(Debug, Deserialize)]
pub struct DecayModelUpdate {
    pub decay_model_params: DecayModelParameters,
}

pub async fn update_decay_model(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(request): Json<DecayModelUpdate>,
) -> Result<impl IntoResponse, AppError> {
    let params_json = serde_json::to_value(&request.decay_model_params)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    sqlx::query(
        "UPDATE digital_twins SET decay_model_params = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(&params_json)
    .bind(twin_id)
    .execute(&state.db.pool())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::OK, Json(serde_json::json!({"status": "updated"}))))
}

/// Health check for physics model service
pub async fn physics_model_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "physics_model",
        "status": "healthy",
        "features": [
            "decay_model_calculation",
            "health_scoring",
            "monte_carlo_simulation",
            "accuracy_auditing",
            "iot_integration"
        ]
    }))
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::digital_twin::*,
    AppState,
    validation::{validate_string, sanitize_input},
};

/// Create a new digital twin
pub async fn create_digital_twin(
    State(state): State<AppState>,
    Json(mut request): Json<CreateDigitalTwinRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_string("name", &request.name, 128)?;
    validate_string("description", &request.description, 1024)?;
    
    request.name = sanitize_input(&request.name);
    request.description = sanitize_input(&request.description);

    let twin = state
        .digital_twin_service
        .create_twin(request)
        .await?;

    Ok((StatusCode::CREATED, Json(twin)))
}

/// Get digital twin by ID
pub async fn get_digital_twin(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let twin = state
        .digital_twin_service
        .get_twin(twin_id)
        .await?;

    Ok(Json(twin))
}

/// Get all digital twins for a product
pub async fn get_product_twins(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let twins = state
        .digital_twin_service
        .get_twins_by_product(&product_id)
        .await?;

    Ok(Json(twins))
}

/// Update digital twin state
pub async fn update_twin_state(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(request): Json<UpdateTwinStateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let state_record = state
        .digital_twin_service
        .update_twin_state(twin_id, request)
        .await?;

    Ok(Json(state_record))
}

/// Get state history for a twin
#[derive(Debug, Deserialize)]
pub struct StateHistoryQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    100
}

pub async fn get_state_history(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Query(query): Query<StateHistoryQuery>,
) -> Result<impl IntoResponse, AppError> {
    let states = state
        .digital_twin_service
        .get_state_history(twin_id, query.limit)
        .await?;

    Ok(Json(states))
}

/// Create a simulation
pub async fn create_simulation(
    State(state): State<AppState>,
    Json(request): Json<CreateSimulationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let simulation = state
        .digital_twin_service
        .create_simulation(request)
        .await?;

    Ok((StatusCode::CREATED, Json(simulation)))
}

/// Run a simulation
pub async fn run_simulation(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = state
        .digital_twin_service
        .run_simulation(simulation_id)
        .await?;

    Ok(Json(result))
}

/// Get simulation by ID
pub async fn get_simulation(
    State(state): State<AppState>,
    Path(simulation_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let simulation = state
        .digital_twin_service
        .get_simulation(simulation_id)
        .await?;

    Ok(Json(simulation))
}

/// List simulations for a twin
pub async fn list_simulations(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let simulations = state
        .digital_twin_service
        .list_simulations(twin_id)
        .await?;

    Ok(Json(simulations))
}

/// Create a prediction
pub async fn create_prediction(
    State(state): State<AppState>,
    Json(request): Json<PredictionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let prediction = state
        .digital_twin_service
        .create_prediction(request)
        .await?;

    Ok((StatusCode::CREATED, Json(prediction)))
}

/// Get twin analytics
pub async fn get_twin_analytics(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let analytics = state
        .digital_twin_service
        .get_twin_analytics(twin_id)
        .await?;

    Ok(Json(analytics))
}

/// Health check for digital twin service
pub async fn digital_twin_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "digital_twin",
        "status": "healthy",
        "features": [
            "twin_management",
            "state_tracking",
            "simulations",
            "predictions",
            "optimizations",
            "analytics"
        ]
    }))
}

#[derive(Debug, Serialize)]
pub struct DigitalTwinStats {
    pub total_twins: i64,
    pub active_twins: i64,
    pub total_simulations: i64,
    pub completed_simulations: i64,
    pub total_predictions: i64,
    pub active_optimizations: i64,
}

/// Get overall digital twin statistics
pub async fn get_digital_twin_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // This would query the database for actual stats
    // Simplified implementation for now
    let stats = DigitalTwinStats {
        total_twins: 0,
        active_twins: 0,
        total_simulations: 0,
        completed_simulations: 0,
        total_predictions: 0,
        active_optimizations: 0,
    };

    Ok(Json(stats))
}

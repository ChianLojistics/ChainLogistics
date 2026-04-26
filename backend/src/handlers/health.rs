use axum::{extract::State, response::Json};
use serde_json::json;
use utoipa::ToSchema;

use crate::{AppState, error::AppError};

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "chainlogistics-backend"
    }))
}

#[utoipa::path(
    get,
    path = "/health/db",
    tag = "health",
    responses(
        (status = 200, description = "Database connection is healthy"),
        (status = 503, description = "Database connection failed")
    )
)]
pub async fn db_health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    state.db.health_check().await?;
    
    Ok(Json(json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": chrono::Utc::now()
    })))
}

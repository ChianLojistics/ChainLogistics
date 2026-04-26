use axum::{extract::State, response::Json};
use serde_json::json;

use crate::{AppState, error::AppError};

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "System is healthy", body = Object)
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
    path = "/api/v1/health/db",
    tag = "health",
    responses(
        (status = 200, description = "Database is healthy", body = Object),
        (status = 503, description = "Database is unhealthy")
    ),
    security(
        ("api_key" = [])
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

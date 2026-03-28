use axum::{extract::{State, Path}, response::Json};
use super::super::{AppState, error::AppError};

pub async fn list_events(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn create_event(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn get_event(State(_state): State<AppState>, Path(_id): Path<i64>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

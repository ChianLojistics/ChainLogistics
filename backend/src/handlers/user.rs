use axum::{extract::State, response::Json};
use super::super::{AppState, error::AppError};

pub async fn create_user(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn get_current_user(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

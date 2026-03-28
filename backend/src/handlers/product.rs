use axum::{extract::{State, Path}, response::Json};
use super::super::{AppState, error::AppError};

// Placeholder handlers - will be implemented in REST API phase

pub async fn list_products(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn create_product(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn get_product(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn update_product(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

pub async fn delete_product(State(_state): State<AppState>, Path(_id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"message": "Not implemented yet"})))
}

use axum::{extract::State, response::Json};
use serde_json::json;

use crate::{AppState, error::AppError};

pub async fn get_stats(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let global_stats = state.event_service.get_global_stats().await?;
    
    Ok(Json(json!({
        "total_products": global_stats.total_products,
        "active_products": global_stats.active_products,
        "total_events": global_stats.total_events,
        "total_users": global_stats.total_users,
        "active_api_keys": global_stats.active_api_keys,
        "timestamp": chrono::Utc::now()
    })))
}

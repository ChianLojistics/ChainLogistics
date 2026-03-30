"""use axum::{
    extract::{State, Path},
    routing::get,
    Json,
    Router,
};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::resilience::ResilienceMetrics;
use crate::AppState;

pub fn resilience_routes() -> Router<AppState> {
    Router::new().route("/resilience/:product_id", get(get_resilience_metrics))
}

async fn get_resilience_metrics(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<ResilienceMetrics>, AppError> {
    let metrics = state.resilience_service.get_resilience_metrics(&product_id).await?;
    Ok(Json(metrics))
}
"""
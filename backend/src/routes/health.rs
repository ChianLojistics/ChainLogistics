use axum::{Router, routing::get};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .route("/health/db", get(crate::handlers::health::db_health_check))
}

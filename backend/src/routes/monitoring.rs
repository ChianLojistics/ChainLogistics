use axum::{Router, routing::{get, post}, middleware};
use crate::{AppState, models::UserRole, middleware::auth::{jwt_auth, require_role}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(crate::handlers::monitoring::get_dashboard))
        .route("/errors", get(crate::handlers::monitoring::get_error_stats))
        .route("/errors/recent", get(crate::handlers::monitoring::get_recent_errors))
        .route("/performance", get(crate::handlers::monitoring::get_performance_metrics))
        .route("/infrastructure", get(crate::handlers::monitoring::get_infrastructure_metrics))
        .route("/alerts/check", post(crate::handlers::monitoring::check_alerts))
        .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

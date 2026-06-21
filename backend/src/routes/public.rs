use axum::{Router, routing::{get, post}, middleware};
use crate::{AppState, models::UserRole, middleware::auth::{api_key_auth, require_role}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/products", get(crate::handlers::product::list_products))
        .route("/products/:id", get(crate::handlers::product::get_product))
        .route("/events", get(crate::handlers::event::list_events))
        .route("/events/:id", get(crate::handlers::event::get_event))
        .route("/recalls", get(crate::handlers::recall::list_recalls))
        .route("/recalls/:id", get(crate::handlers::recall::get_recall))
        .route("/recalls/:id/affected", get(crate::handlers::recall::list_affected_items))
        .route("/stats", get(crate::handlers::stats::get_stats))
        .route("/transactions", get(crate::handlers::financial::list_transactions))
        .route("/transactions/:id", get(crate::handlers::financial::get_transaction))
        .route(
            "/compliance/check",
            post(crate::handlers::compliance::check_compliance)
                .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator]))),
        )
        .route(
            "/compliance/report/:product_id",
            get(crate::handlers::compliance::get_compliance_report)
                .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator]))),
        )
        .route(
            "/audit/report",
            get(crate::handlers::compliance::generate_audit_report)
                .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator]))),
        )
        .layer(middleware::from_fn(api_key_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

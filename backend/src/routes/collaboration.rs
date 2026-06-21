use axum::{Router, routing::{get, post, put}, middleware};
use crate::middleware::auth::jwt_auth;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/share", post(crate::handlers::collaboration::share_product))
        .route("/shares/:product_id", get(crate::handlers::collaboration::list_shares))
        .route("/requests", post(crate::handlers::collaboration::create_collaboration_request))
        .route("/requests/:id", put(crate::handlers::collaboration::update_collaboration_request))
        .route("/audit/:entity_type/:entity_id", get(crate::handlers::collaboration::list_audit_trail))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

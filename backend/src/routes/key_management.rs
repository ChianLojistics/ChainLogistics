use axum::{Router, routing::{get, post}, middleware};
use crate::{AppState, middleware::auth::jwt_auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(crate::handlers::api_keys::list_keys).post(crate::handlers::api_keys::create_key))
        .route("/:id/revoke", post(crate::handlers::api_keys::revoke_key))
        .route("/:id/rotate", post(crate::handlers::api_keys::rotate_key))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

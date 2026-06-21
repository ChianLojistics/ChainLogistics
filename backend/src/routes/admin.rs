use axum::{Router, routing::{get, post, put}, middleware};
use crate::{AppState, models::UserRole, middleware::auth::{jwt_auth, require_role, require_admin}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/products", post(crate::handlers::product::create_product))
        .route(
            "/products/:id",
            put(crate::handlers::product::update_product).delete(crate::handlers::product::delete_product),
        )
        .route(
            "/events",
            post(crate::handlers::event::create_event)
                .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Carrier, UserRole::Administrator]))),
        )
        .route(
            "/recalls",
            post(crate::handlers::recall::create_recall)
                .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator]))),
        )
        .route(
            "/recalls/:id/notify",
            post(crate::handlers::recall::notify_recall)
                .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator]))),
        )
        .route(
            "/recalls/:id/effectiveness",
            post(crate::handlers::recall::update_effectiveness)
                .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator]))),
        )
        .route(
            "/transactions",
            post(crate::handlers::financial::create_transaction)
                .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Administrator]))),
        )
        .route(
            "/invoices",
            post(crate::handlers::financial::create_invoice)
                .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Administrator]))),
        )
        .route(
            "/financing/request",
            post(crate::handlers::financial::request_financing)
                .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Administrator]))),
        )
        .route("/users", post(crate::handlers::user::create_user))
        .route("/users/me", get(crate::handlers::user::get_current_user))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

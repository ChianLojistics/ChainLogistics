use axum::{Router, routing::{get, post, put, delete}, middleware};
use super::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", public_api_routes())
        .nest("/api/v1/admin", admin_api_routes())
}

fn public_api_routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(crate::handlers::product::list_products))
        .route("/products/:id", get(crate::handlers::product::get_product))
        .route("/events", get(crate::handlers::event::list_events))
        .route("/events/:id", get(crate::handlers::event::get_event))
        .route("/stats", get(crate::handlers::stats::get_stats))
        .route("/transactions", get(crate::handlers::financial::list_transactions))
        .route("/transactions/:id", get(crate::handlers::financial::get_transaction))
        .route("/compliance/check", post(crate::handlers::compliance::check_compliance))
        .route("/compliance/report/:product_id", get(crate::handlers::compliance::get_compliance_report))
        .route("/audit/report", get(crate::handlers::compliance::generate_audit_report))
        .layer(middleware::from_fn(crate::middleware::auth::api_key_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn admin_api_routes() -> Router<AppState> {
    Router::new()
        .route("/products", post(crate::handlers::product::create_product))
        .route("/products/:id", put(crate::handlers::product::update_product).delete(crate::handlers::product::delete_product))
        .route("/events", post(crate::handlers::event::create_event))
        .route("/transactions", post(crate::handlers::financial::create_transaction))
        .route("/invoices", post(crate::handlers::financial::create_invoice))
        .route("/financing/request", post(crate::handlers::financial::request_financing))
        .route("/users", post(crate::handlers::user::create_user))
        .route("/users/me", get(crate::handlers::user::get_current_user))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .layer(middleware::from_fn(crate::middleware::auth::require_admin))
        .layer(middleware::from_fn(crate::middleware::auth::api_key_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

// Public routes that don't require authentication
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .route("/health/db", get(crate::handlers::health::db_health_check))
}
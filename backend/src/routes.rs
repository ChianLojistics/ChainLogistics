use axum::{Router, routing::{get, post, put, delete}, middleware};
use super::AppState;
use crate::models::UserRole;
use crate::middleware::auth::{jwt_auth, api_key_auth, require_role, require_admin};

pub mod analytics;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", public_api_routes())
        .nest("/api/v1/admin", admin_api_routes())
        .nest("/api/v1/analytics", analytics_routes())
        .nest("/api/v1/carbon", carbon_routes())
        .nest("/api/v1/keys", key_management_routes())
        .nest("/api/v1/monitoring", monitoring_routes())
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
        .route("/compliance/check", post(crate::handlers::compliance::check_compliance)
            .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator]))))
        .route("/compliance/report/:product_id", get(crate::handlers::compliance::get_compliance_report)
            .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator]))))
        .route("/audit/report", get(crate::handlers::compliance::generate_audit_report)
            .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator]))))
        .layer(middleware::from_fn(api_key_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn admin_api_routes() -> Router<AppState> {
    Router::new()
        .route("/products", post(crate::handlers::product::create_product))
        .route("/products/:id", put(crate::handlers::product::update_product).delete(crate::handlers::product::delete_product))
        .route("/events", post(crate::handlers::event::create_event)
            .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Carrier, UserRole::Administrator]))))
        .route("/transactions", post(crate::handlers::financial::create_transaction)
            .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Administrator]))))
        .route("/invoices", post(crate::handlers::financial::create_invoice)
            .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Administrator]))))
        .route("/financing/request", post(crate::handlers::financial::request_financing)
            .layer(middleware::from_fn(require_role(vec![UserRole::Supplier, UserRole::Administrator]))))
        .route("/users", post(crate::handlers::user::create_user))
        .route("/users/me", get(crate::handlers::user::get_current_user))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

// Public routes that don't require authentication
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .route("/health/db", get(crate::handlers::health::db_health_check))
}

fn analytics_routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(crate::routes::analytics::dashboard))
        .route("/products/:id", get(crate::routes::analytics::product_analytics))
        .route("/events", get(crate::routes::analytics::event_analytics))
        .route("/users", get(crate::routes::analytics::user_analytics))
        .route("/export", get(crate::routes::analytics::export))
        .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn key_management_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(crate::handlers::api_keys::list_keys).post(crate::handlers::api_keys::create_key))
        .route("/:id/revoke", post(crate::handlers::api_keys::revoke_key))
        .route("/:id/rotate", post(crate::handlers::api_keys::rotate_key))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn carbon_routes() -> Router<AppState> {
    Router::new()
        // Footprint
        .route("/footprint/calculate", post(crate::handlers::carbon::calculate_footprint))
        .route("/footprint/preview", post(crate::handlers::carbon::preview_footprint))
        .route("/footprint/:product_id", get(crate::handlers::carbon::list_footprints))
        // Credits
        .route("/credits", get(crate::handlers::carbon::list_credits))
        .route("/credits/:id", get(crate::handlers::carbon::get_credit))
        .route("/credits/generate", post(crate::handlers::carbon::generate_credit))
        .route("/credits/retire", post(crate::handlers::carbon::retire_credit))
        // Marketplace
        .route("/market", get(crate::handlers::carbon::market_summary))
        .route("/market/trades", get(crate::handlers::carbon::list_trades))
        .route("/market/list", post(crate::handlers::carbon::list_credit_for_sale))
        .route("/market/purchase", post(crate::handlers::carbon::purchase_credit))
        // Verification
        .route("/verify", post(crate::handlers::carbon::request_verification))
        .route("/verify/:credit_id", get(crate::handlers::carbon::list_verifications))
        // Reports
        .route("/reports", get(crate::handlers::carbon::list_reports).post(crate::handlers::carbon::generate_report))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn monitoring_routes() -> Router<AppState> {
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

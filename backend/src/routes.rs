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
        .nest("/api/regulatory", regulatory_routes())
        .nest("/api/iot", iot_routes())
        .nest("/api/quality", quality_routes())
        .nest("/api/suppliers", supplier_routes())
        .nest("/api/v1/batches", batch_routes())
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

fn regulatory_routes() -> Router<AppState> {
    Router::new()
        // Requirements
        .route("/requirements", get(crate::handlers::regulatory::list_requirements).post(crate::handlers::regulatory::create_requirement))
        .route("/requirements/:requirement_id", get(crate::handlers::regulatory::get_requirement).put(crate::handlers::regulatory::update_requirement))
        // Product Compliance
        .route("/compliance", post(crate::handlers::regulatory::create_product_compliance))
        .route("/compliance/:product_id", get(crate::handlers::regulatory::list_product_compliance))
        .route("/compliance/:product_id/:requirement_id", get(crate::handlers::regulatory::get_product_compliance))
        // Automated Checks
        .route("/check", post(crate::handlers::regulatory::run_compliance_check))
        // Audit Trail
        .route("/audit/:product_id", get(crate::handlers::regulatory::get_audit_trail))
        // Reports
        .route("/reports", get(crate::handlers::regulatory::list_reports).post(crate::handlers::regulatory::generate_report))
        .route("/reports/:report_id", get(crate::handlers::regulatory::get_report))
        .layer(middleware::from_fn(require_role(vec![UserRole::Auditor, UserRole::Administrator, UserRole::Inspector])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn iot_routes() -> Router<AppState> {
    Router::new()
        // Devices
        .route("/devices", get(crate::handlers::iot::list_devices).post(crate::handlers::iot::create_device))
        .route("/devices/:device_id", get(crate::handlers::iot::get_device))
        // Readings
        .route("/readings", get(crate::handlers::iot::get_readings).post(crate::handlers::iot::create_reading))
        // Thresholds
        .route("/thresholds", post(crate::handlers::iot::create_threshold))
        .route("/thresholds/:product_id", get(crate::handlers::iot::get_thresholds))
        // Alerts
        .route("/alerts", get(crate::handlers::iot::get_alerts))
        .route("/alerts/:alert_id/acknowledge", post(crate::handlers::iot::acknowledge_alert))
        .route("/alerts/:alert_id/resolve", post(crate::handlers::iot::resolve_alert))
        // Summaries
        .route("/summaries/:device_id", get(crate::handlers::iot::get_summaries))
        .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator, UserRole::Supplier])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn quality_routes() -> Router<AppState> {
    Router::new()
        // Checkpoints
        .route("/checkpoints", get(crate::handlers::quality::list_checkpoints).post(crate::handlers::quality::create_checkpoint))
        .route("/checkpoints/:checkpoint_id", get(crate::handlers::quality::get_checkpoint))
        // Workflows
        .route("/workflows", get(crate::handlers::quality::list_workflows).post(crate::handlers::quality::create_workflow))
        .route("/workflows/:workflow_id", get(crate::handlers::quality::get_workflow))
        .route("/workflows/execute", post(crate::handlers::quality::execute_workflow))
        // Inspections
        .route("/inspections", get(crate::handlers::quality::list_inspections).post(crate::handlers::quality::create_inspection))
        .route("/inspections/:inspection_id", get(crate::handlers::quality::get_inspection))
        .route("/inspections/:inspection_id/status", put(crate::handlers::quality::update_inspection_status))
        // Non-Conformances
        .route("/non-conformances", get(crate::handlers::quality::list_non_conformances).post(crate::handlers::quality::create_non_conformance))
        .route("/non-conformances/:nc_id", put(crate::handlers::quality::update_non_conformance))
        .route("/non-conformances/:nc_id/verify", post(crate::handlers::quality::verify_non_conformance))
        // Metrics
        .route("/metrics", get(crate::handlers::quality::list_metrics).post(crate::handlers::quality::create_metric))
        .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator, UserRole::Auditor])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn supplier_routes() -> Router<AppState> {
    Router::new()
        // Suppliers
        .route("/", get(crate::handlers::supplier::list_suppliers).post(crate::handlers::supplier::create_supplier))
        .route("/:supplier_id", get(crate::handlers::supplier::get_supplier))
        .route("/:supplier_id/verification", put(crate::handlers::supplier::update_verification))
        .route("/:supplier_id/summary", get(crate::handlers::supplier::get_supplier_summary))
        // Ratings
        .route("/ratings", post(crate::handlers::supplier::create_rating))
        .route("/:supplier_id/ratings", get(crate::handlers::supplier::get_ratings))
        // Performance
        .route("/performance", post(crate::handlers::supplier::create_performance))
        .route("/:supplier_id/performance", get(crate::handlers::supplier::get_performance))
        // Compliance
        .route("/compliance", post(crate::handlers::supplier::create_compliance))
        .route("/:supplier_id/compliance", get(crate::handlers::supplier::get_compliance))
        .route("/compliance/:compliance_id/verify", put(crate::handlers::supplier::verify_compliance))
        // Products
        .route("/products", post(crate::handlers::supplier::add_supplier_product))
        .route("/:supplier_id/products", get(crate::handlers::supplier::get_supplier_products))
        // Audit Trail
        .route("/:supplier_id/audit", get(crate::handlers::supplier::get_audit_trail))
        .layer(middleware::from_fn(require_role(vec![UserRole::Administrator, UserRole::Auditor])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

fn batch_routes() -> Router<AppState> {
    Router::new()
        // Batch CRUD
        .route("/", get(crate::handlers::batch::list_batches).post(crate::handlers::batch::create_batch))
        .route("/:id", get(crate::handlers::batch::get_batch).put(crate::handlers::batch::update_batch).delete(crate::handlers::batch::delete_batch))
        .route("/number/:batch_number", get(crate::handlers::batch::get_batch_by_number))
        // Genealogy
        .route("/genealogy", post(crate::handlers::batch::create_genealogy))
        .route("/:id/genealogy", get(crate::handlers::batch::get_genealogy_tree))
        // Quality attributes
        .route("/quality-attributes", post(crate::handlers::batch::create_quality_attribute))
        .route("/:id/quality-attributes", get(crate::handlers::batch::get_quality_attributes))
        // Recalls
        .route("/recalls", post(crate::handlers::batch::create_recall))
        .route("/recalls/active", get(crate::handlers::batch::list_active_recalls))
        .route("/recalls/:id", get(crate::handlers::batch::get_recall).put(crate::handlers::batch::update_recall))
        .route("/:id/recalls", get(crate::handlers::batch::get_batch_recalls))
        // Inventory
        .route("/inventory", post(crate::handlers::batch::create_inventory_transaction))
        .route("/:id/inventory", get(crate::handlers::batch::get_batch_inventory))
        .layer(middleware::from_fn(require_role(vec![UserRole::Inspector, UserRole::Administrator, UserRole::Auditor, UserRole::Supplier])))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

use axum::{Router, routing::{get, post}, middleware};
use crate::{AppState, middleware::auth::jwt_auth};

pub fn router() -> Router<AppState> {
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
        // Dashboard and Supplier Scoring
        .route("/dashboard", get(crate::handlers::carbon::get_sustainability_dashboard))
        .route("/supplier-score/:supplier_address", get(crate::handlers::carbon::get_supplier_score))
        .layer(middleware::from_fn(jwt_auth))
        .layer(middleware::from_fn(crate::middleware::rate_limit::rate_limit_middleware))
}

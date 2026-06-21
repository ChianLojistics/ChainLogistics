use axum::Router;
use crate::AppState;

pub mod analytics;
pub mod admin;
pub mod carbon;
pub mod collaboration;
pub mod health;
pub mod key_management;
pub mod monitoring;
pub mod public;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", public::router())
        .nest("/api/v1/admin", admin::router())
        .nest("/api/v1/analytics", analytics::routes())
        .nest("/api/v1/carbon", carbon::router())
        .nest("/api/v1/keys", key_management::router())
        .nest("/api/v1/monitoring", monitoring::router())
        .nest("/api/v1/collaboration", collaboration::router())
}

pub fn health_routes() -> Router<AppState> {
    health::router()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_routes_compiles() {
        let router = api_routes();
        let _ = router;
    }

    #[test]
    fn health_routes_compiles() {
        let router = health_routes();
        let _ = router;
    }
}

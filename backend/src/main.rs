use axum::{Router, routing::{get, post}, middleware};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use std::net::SocketAddr;

mod config;
mod middleware;
mod routes;
mod handlers;
mod services;
mod models;
mod database;
mod utils;
mod error;

#[derive(Clone)]
pub struct AppState {
    // pool: PgPool, Add state here
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }
}

async fn health_check() -> &'static str {
    "OK"
}

// Dummy middlewares
async fn auth_middleware(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    next.run(req).await
}

async fn rate_limit_middleware(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    next.run(req).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();
    
    // Create application state
    let app_state = AppState::new().await?;
    
    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        // .nest("/api/v1", routes::api_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(auth_middleware))
                .layer(middleware::from_fn(rate_limit_middleware))
        )
        .with_state(app_state);
    
    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

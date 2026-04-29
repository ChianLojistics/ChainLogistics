use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod blockchain;
mod compliance;
mod config;
mod database;
mod docs;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;
mod websocket;

use config::Config;
use database::Database;
use error::AppError;
use services::{
    AnalyticsService, ApiKeyService, EventService, FinancialService, ProductService, SyncService,
    UserService,
};
use utils::CronService;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub product_service: Arc<ProductService>,
    pub event_service: Arc<EventService>,
    pub user_service: Arc<UserService>,
    pub api_key_service: Arc<ApiKeyService>,
    pub sync_service: Arc<SyncService>,
    pub financial_service: Arc<FinancialService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub config: Config,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::from_env()?;

        // Initialize database
        let db = Database::new(&config.database).await?;

        // Run migrations
        db.migrate().await?;

        // Create services
        let product_service = Arc::new(ProductService::new(db.pool().clone()));
        let event_service = Arc::new(EventService::new(db.pool().clone()));
        let user_service = Arc::new(UserService::new(db.pool().clone()));
        let api_key_service = Arc::new(ApiKeyService::new(db.pool().clone()));
        let sync_service = Arc::new(SyncService::new(db.pool().clone()));
        let financial_service = Arc::new(FinancialService::new(db.pool().clone()));
        let analytics_service = Arc::new(AnalyticsService::new(
            db.pool().clone(),
            config.redis.url.clone(),
        ));

        Ok(Self {
            db,
            product_service,
            event_service,
            user_service,
            api_key_service,
            sync_service,
            financial_service,
            analytics_service,
            config,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create application state
    let app_state = AppState::new().await?;

    // Start cron scheduler
    let cron_service = CronService::new(app_state.db.pool().clone());
    cron_service.start_scheduler().await;

    // Build router
    let app = Router::new()
        .merge(crate::routes::health_routes())
        .merge(crate::routes::api_routes())
        .merge(crate::docs::create_swagger_ui())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(
                    crate::middleware::security::security_headers,
                )),
        )
        .with_state(app_state);

    // Run server
    let config = Config::from_env()?;
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));

    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

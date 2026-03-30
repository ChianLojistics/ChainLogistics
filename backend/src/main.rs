use axum::{Router, routing::{get, post}, middleware};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use std::net::SocketAddr;
use std::sync::Arc;

mod config;
mod middleware;
mod routes;
mod handlers;
mod services;
mod models;
mod database;
mod utils;
mod error;
mod docs;
mod blockchain;
mod websocket;
mod compliance;

use config::Config;
use database::Database;
use services::{ProductService, EventService, UserService, ApiKeyService, SyncService, FinancialService, AnalyticsService, CarbonService};
use utils::CronService;
use error::AppError;

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
    pub carbon_service: Arc<CarbonService>,
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
        let carbon_service = Arc::new(CarbonService::new(db.pool().clone()));

        Ok(Self {
            db,
            product_service,
            event_service,
            user_service,
            api_key_service,
            sync_service,
            financial_service,
            analytics_service,
            carbon_service,
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
        )
        .with_state(app_state);
    
    // Run server
    let config = Config::from_env()?;
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port
    ));
    
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

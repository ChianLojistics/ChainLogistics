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
mod validation;
mod monitoring;

use config::Config;
use database::Database;
use services::{ProductService, EventService, UserService, ApiKeyService, SyncService, FinancialService, AnalyticsService, CarbonService};
use utils::CronService;
use error::AppError;
use monitoring::MonitoringSystem;

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
    pub redis_client: redis::Client,
    pub config: Config,
    pub monitoring_system: MonitoringSystem,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::from_env()?;
        
        // Initialize database
        let db = Database::new(&config.database).await?;
        
        // Run migrations
        db.migrate().await?;
        
        // Initialize Redis client
        let redis_client = redis::Client::open(config.redis.url.as_str())?;
        
        // Create services
        let product_service = Arc::new(ProductService::new(db.pool().clone(), redis_client.clone()));
        let event_service = Arc::new(EventService::new(db.pool().clone(), redis_client.clone()));
        let user_service = Arc::new(UserService::new(db.pool().clone(), config.encryption_key.clone()));
        let api_key_service = Arc::new(ApiKeyService::new(db.pool().clone()));
        let sync_service = Arc::new(SyncService::new(db.pool().clone(), redis_client.clone()));
        let financial_service = Arc::new(FinancialService::new(db.pool().clone()));
        let analytics_service = Arc::new(AnalyticsService::new(
            db.pool().clone(),
            config.redis.url.clone(),
        ));
        let carbon_service = Arc::new(CarbonService::new(db.pool().clone()));
        
        // Initialize comprehensive monitoring system
        let monitoring_system = MonitoringSystem::new();
        
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
            redis_client,
            config,
            monitoring_system,
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
    
    // Start background services
    let cron_service = CronService::new(app_state.db.pool().clone(), app_state.redis_client.clone());
    cron_service.start_scheduler().await;
    
    // Build router with security middleware
    let app = Router::new()
        .merge(crate::routes::health_routes())
        .merge(crate::routes::api_routes())
        .merge(crate::docs::create_swagger_ui())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(
                    middleware::error_handler::request_logger,
                ))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::error_handler::global_error_handler,
                ))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::security::enforce_https,
                ))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::security::security_headers,
                ))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::security::cors_policy,
                ))
        )
        .with_state(app_state.clone());
    
    // Run server
    let config = Config::from_env()?;
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port
    ));
    
    tracing::info!("Server listening on {}", addr);
    tracing::info!("HTTPS enforcement: {}", config.security.enforce_https);
    tracing::info!("TLS enabled: {}", config.server.tls_enabled);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

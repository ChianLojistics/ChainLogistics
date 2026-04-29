use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::env;
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
mod monitoring;
mod routes;
mod services;
mod utils;
mod validation;
mod websocket;

use config::Config;
use database::Database;
use services::{ProductService, EventService, UserService, ApiKeyService, SyncService, FinancialService, AnalyticsService, CarbonService, AuditService, BatchService};
use utils::CronService;
use error::AppError;
use monitoring::MonitoringSystem;
use services::{ProductService, EventService, UserService, ApiKeyService, SyncService, FinancialService, AnalyticsService, CarbonService, RegulatoryService, IoTService, QualityService, SupplierService};

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
    pub regulatory_service: Arc<RegulatoryService>,
    pub iot_service: Arc<IoTService>,
    pub quality_service: Arc<QualityService>,
    pub supplier_service: Arc<SupplierService>,
    pub audit_service: Arc<AuditService>,
    pub batch_service: Arc<BatchService>,
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
        let product_service =
            Arc::new(ProductService::new(db.pool().clone(), redis_client.clone()));
        let event_service = Arc::new(EventService::new(db.pool().clone(), redis_client.clone()));
        let user_service = Arc::new(UserService::new(
            db.pool().clone(),
            config.encryption_key.clone(),
        ));
        let api_key_service = Arc::new(ApiKeyService::new(db.pool().clone()));
        let sync_service = Arc::new(SyncService::new(db.pool().clone(), redis_client.clone()));
        let financial_service = Arc::new(FinancialService::new(db.pool().clone()));
        let analytics_service = Arc::new(AnalyticsService::new(
            db.pool().clone(),
            config.redis.url.clone(),
        ));
        let carbon_service = Arc::new(CarbonService::new(db.pool().clone()));
        let regulatory_service = Arc::new(RegulatoryService::new(db.pool().clone()));
        let iot_service = Arc::new(IoTService::new(db.pool().clone()));
        let quality_service = Arc::new(QualityService::new(db.pool().clone()));
        let supplier_service = Arc::new(SupplierService::new(db.pool().clone()));
        let batch_service = Arc::new(BatchService::new(db.pool().clone(), redis_client.clone()));

        let audit_service = Arc::new(AuditService::new(
            db.pool().clone(),
            config.audit.enabled,
            config.audit.hmac_key.clone(),
            config.audit.retention_days,
        ));
        
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
            regulatory_service,
            iot_service,
            quality_service,
            supplier_service,
            audit_service,
            batch_service,
            redis_client,
            config,
            monitoring_system,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());
    let env_filter = tracing_subscriber::EnvFilter::from_default_env();
    if log_format.eq_ignore_ascii_case("pretty") {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .flatten_event(true)
            .init();
    }

    // Create application state
    let app_state = AppState::new().await?;

    // Start background services
    let cron_service =
        CronService::new(app_state.db.pool().clone(), app_state.redis_client.clone());
    cron_service.start_scheduler().await;

    // Build router with security middleware
    let app = Router::new()
        .merge(crate::routes::health_routes())
        .merge(crate::routes::api_routes())
        .merge(crate::docs::create_swagger_ui())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum_middleware::from_fn(
                    middleware::error_handler::request_logger,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::error_handler::global_error_handler,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::security::enforce_https,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::security::security_headers,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::security::cors_policy,
                )),
        )
        .with_state(app_state.clone());

    // Run server
    let config = Config::from_env()?;
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));

    tracing::info!("Server listening on {}", addr);
    tracing::info!("HTTPS enforcement: {}", config.security.enforce_https);
    tracing::info!("TLS enabled: {}", config.server.tls_enabled);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

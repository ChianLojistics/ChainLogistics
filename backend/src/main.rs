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
// mod compliance; // Temporarily disabled due to missing dependencies
#[cfg(test)]
mod tests;
mod config;
mod database;
mod docs;
mod error;
mod handlers;
mod middleware;
mod models;
mod monitoring;
mod routes;
mod rules;
mod saga;
mod services;
mod streaming;
mod utils;
mod validation;
// mod websocket; // Temporarily disabled due to missing warp dependency
mod workers;

use config::Config;
use database::{Database, ProductRepository, EventRepository};
use monitoring::MonitoringSystem;
use rules::actions::{ActionExecutor, ActionHandlerEnum, StateHandler, WebhookHandler};
use rules::engine::RuleEngine;
use saga::coordinator::SagaCoordinator;
use saga::persistence::PostgresSagaPersistence;
use services::{
    AnalyticsService, ApiKeyService, AuditService, BatchService, CarbonService,
    CollaborationService, EventService, FinancialService, PredictiveRoutingService,
    ProductService, RecallService, SyncService, UserService,
};
use streaming::mercury_client::MercuryConfig;
use streaming::indexer::StreamIndexer;
use utils::CronService;
use workers::executor::TaskExecutor;
use workers::pool::WorkerPool;

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
    pub collaboration_service: Arc<CollaborationService>,
    pub audit_service: Arc<AuditService>,
    pub recall_service: Arc<RecallService>,
    pub batch_service: Arc<BatchService>,
    pub predictive_routing_service: Arc<PredictiveRoutingService>,
    pub redis_client: redis::Client,
    pub config: Config,
    pub monitoring_system: MonitoringSystem,
    pub rule_engine: Arc<RuleEngine>,
    pub saga_coordinator: Arc<SagaCoordinator>,
    pub task_executor: TaskExecutor,
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
        let collaboration_service = Arc::new(CollaborationService::new(db.pool().clone()));
        let audit_service = Arc::new(AuditService::new(db.pool().clone()));
        let recall_service = Arc::new(RecallService::new(db.pool().clone()));
        let batch_service = Arc::new(BatchService::new(db.pool().clone()));
        let predictive_routing_service =
            Arc::new(PredictiveRoutingService::new(db.pool().clone()));

        // Initialize comprehensive monitoring system
        let monitoring_system = MonitoringSystem::new();

        // Initialize Rule Engine
        let action_executor = ActionExecutor::new();
        action_executor.register_handler("send_webhook".to_string(), ActionHandlerEnum::Webhook(WebhookHandler::new()));
        action_executor.register_handler("set_state".to_string(), ActionHandlerEnum::State(StateHandler::new()));
        let rule_engine = Arc::new(RuleEngine::new(action_executor));

        // Initialize Saga Coordinator
        let saga_persistence = Arc::new(PostgresSagaPersistence::new(db.pool().clone()));
        let saga_coordinator = Arc::new(SagaCoordinator::new(saga_persistence));

        // Initialize Task Executor
        let task_executor = TaskExecutor::new();

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
            collaboration_service,
            audit_service,
            recall_service,
            batch_service,
            predictive_routing_service,
            redis_client,
            config,
            monitoring_system,
            rule_engine,
            saga_coordinator,
            task_executor,
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

    // Start streaming indexer (Mercury integration)
    let (product_tx, mut product_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    
    let mercury_config = MercuryConfig {
        websocket_url: std::env::var("MERCURY_WEBSOCKET_URL")
            .unwrap_or_else(|_| "wss://stream.mercurydata.app/v1/stream".to_string()),
        contract_ids: vec![
            std::env::var("CONTRACT_ID").unwrap_or_else(|_| "default".to_string())
        ],
        reconnect_interval: std::time::Duration::from_secs(5),
    };
    
    let stream_indexer = StreamIndexer::new(mercury_config, product_tx, event_tx);
    tracing::info!("Started Mercury streaming indexer");

    // Start Redis-based worker pool
    let mut worker_pool = WorkerPool::new(
        app_state.redis_client.clone(),
        TaskExecutor::new(),
        "worker-1".to_string(),
        "events".to_string(),
    );
    worker_pool.start(4).await?;
    tracing::info!("Started Redis worker pool with 4 workers");

    // Spawn task to handle products from streaming indexer
    let product_service = app_state.product_service.clone();
    tokio::spawn(async move {
        while let Some(product) = product_rx.recv().await {
            if let Err(e) = product_service.create_product(product).await {
                tracing::error!("Failed to create product from stream: {}", e);
            }
        }
    });

    // Spawn task to handle events from streaming indexer
    let event_service = app_state.event_service.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if let Err(e) = event_service.create_event(event).await {
                tracing::error!("Failed to create event from stream: {}", e);
            }
        }
    });

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

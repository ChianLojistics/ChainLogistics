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
mod services;
mod utils;
mod validation;
mod websocket;

use config::Config;
use database::Database;
use error::AppError;
use monitoring::MonitoringSystem;
use services::{
    AnalyticsService, ApiKeyService, AuditService, BatchService, CarbonService,
    CollaborationService, EventService, FinancialService, IoTService, PredictiveRoutingService,
    ProductService, QualityService, RecallService, RegulatoryService, SupplierService,
    SyncService, UserService, MercuryIndexer, MercuryConfig, RuleEngine, SagaManager,
    RedisWorkerPool, WorkerConfig, TrackingEventProcessor, get_default_rules,
    get_product_registration_saga, NoopAction, EventProcessingHandler, RuleEvaluationHandler,
    NotificationHandler, AlertHandler, WebhookHandler,
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
    pub carbon_service: Arc<CarbonService>,
    pub collaboration_service: Arc<CollaborationService>,
    pub audit_service: Arc<AuditService>,
    pub recall_service: Arc<RecallService>,
    pub batch_service: Arc<BatchService>,
    pub regulatory_service: Arc<RegulatoryService>,
    pub iot_service: Arc<IoTService>,
    pub quality_service: Arc<QualityService>,
    pub supplier_service: Arc<SupplierService>,
    pub predictive_routing_service: Arc<PredictiveRoutingService>,
    pub redis_client: redis::Client,
    pub config: Config,
    pub monitoring_system: MonitoringSystem,
    pub mercury_indexer: Arc<MercuryIndexer>,
    pub rule_engine: Arc<RuleEngine>,
    pub saga_manager: Arc<SagaManager>,
    pub worker_pool: Arc<RedisWorkerPool>,
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
        let regulatory_service = Arc::new(RegulatoryService::new(db.pool().clone()));
        let iot_service = Arc::new(IoTService::new(db.pool().clone()));
        let quality_service = Arc::new(QualityService::new(db.pool().clone()));
        let supplier_service = Arc::new(SupplierService::new(db.pool().clone()));
        let predictive_routing_service =
            Arc::new(PredictiveRoutingService::new(db.pool().clone()));

        // Initialize Mercury streaming indexer
        let mercury_config = MercuryConfig::default();
        let (mercury_indexer, _event_rx) =
            MercuryIndexer::new(mercury_config, db.pool().clone(), redis_client.clone());
        mercury_indexer.add_processor(Arc::new(TrackingEventProcessor::new(db.pool().clone())));
        let mercury_indexer = Arc::new(mercury_indexer);

        // Initialize rule engine with default rules
        let mut rule_engine = RuleEngine::new();
        for rule in get_default_rules() {
            rule_engine.add_rule(rule);
        }
        rule_engine.register_handler("alert".to_string(), Arc::new(AlertHandler::new()));
        rule_engine.register_handler("webhook".to_string(), Arc::new(WebhookHandler::new()));
        let rule_engine = Arc::new(rule_engine);

        // Initialize saga manager
        let mut saga_manager = SagaManager::new(db.pool().clone(), redis_client.clone());
        saga_manager.register_saga(get_product_registration_saga());
        saga_manager.register_action("noop".to_string(), Arc::new(NoopAction));
        let saga_manager = Arc::new(saga_manager);

        // Initialize Redis worker pool
        let worker_config = WorkerConfig::default();
        let mut worker_pool = RedisWorkerPool::new(worker_config, redis_client.clone());
        worker_pool.register_handler(Arc::new(EventProcessingHandler::new()));
        worker_pool.register_handler(Arc::new(RuleEvaluationHandler::new()));
        worker_pool.register_handler(Arc::new(NotificationHandler::new()));
        let worker_pool = Arc::new(worker_pool);

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
            collaboration_service,
            audit_service,
            recall_service,
            batch_service,
            regulatory_service,
            iot_service,
            quality_service,
            supplier_service,
            predictive_routing_service,
            redis_client,
            config,
            monitoring_system,
            mercury_indexer,
            rule_engine,
            saga_manager,
            worker_pool,
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

    // Start Mercury streaming indexer
    let mercury_indexer = app_state.mercury_indexer.clone();
    tokio::spawn(async move {
        if let Err(e) = mercury_indexer.start().await {
            tracing::error!("Mercury indexer failed: {}", e);
        }
    });

    // Recover any in-progress sagas
    let saga_manager = app_state.saga_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = saga_manager.recover_sagas().await {
            tracing::error!("Saga recovery failed: {}", e);
        }
    });

    // Start Redis worker pool
    let worker_pool = app_state.worker_pool.clone();
    tokio::spawn(async move {
        if let Err(e) = worker_pool.start().await {
            tracing::error!("Worker pool failed: {}", e);
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

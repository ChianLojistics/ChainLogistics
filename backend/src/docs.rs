use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    product::{ProductResponse, PaginatedProductsResponse, CreateProductRequest, UpdateProductRequest},
    event::{EventResponse, PaginatedEventsResponse, CreateEventRequest, ListEventsQuery},
    auth::{LoginRequest, RegisterRequest, AuthResponse},
    api_keys::{CreateApiKeyRequest, ApiKeyCreatedResponse, ApiKeyResponse},
    financial::{CreateTransactionRequest, CreateInvoiceRequest, FinancingRequestBody},
    compliance::{ComplianceCheckRequest, ComplianceReportResponse},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ChainLogistics API",
        version = "1.0.0",
        description = "API for managing supply chain products, tracking events on blockchain, carbon footprint management, compliance checking, and financial operations",
        contact(
            name = "ChainLogistics Team",
            email = "support@chainlogistics.io"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "https://api.chainlogistics.io", description = "Production server"),
        (url = "https://staging-api.chainlogistics.io", description = "Staging server"),
        (url = "http://localhost:3001", description = "Development server")
    ),
    paths(
        // Product endpoints
        crate::handlers::product::list_products,
        crate::handlers::product::create_product,
        crate::handlers::product::get_product,
        crate::handlers::product::update_product,
        crate::handlers::product::delete_product,
        // Event endpoints
        crate::handlers::event::list_events,
        crate::handlers::event::create_event,
        crate::handlers::event::get_event,
        // Authentication endpoints
        crate::handlers::auth::login,
        crate::handlers::auth::register,
        // Stats endpoints
        crate::handlers::stats::get_stats,
        // Health endpoints
        crate::handlers::health::health_check,
        crate::handlers::health::db_health_check,
        // Carbon endpoints
        crate::handlers::carbon::calculate_footprint,
        crate::handlers::carbon::preview_footprint,
        crate::handlers::carbon::list_footprints,
        crate::handlers::carbon::generate_credit,
        crate::handlers::carbon::list_credits,
        crate::handlers::carbon::get_credit,
        crate::handlers::carbon::retire_credit,
        crate::handlers::carbon::market_summary,
        crate::handlers::carbon::list_trades,
        crate::handlers::carbon::list_credit_for_sale,
        crate::handlers::carbon::purchase_credit,
        crate::handlers::carbon::request_verification,
        crate::handlers::carbon::list_verifications,
        crate::handlers::carbon::generate_report,
        crate::handlers::carbon::list_reports,
        // Financial endpoints
        crate::handlers::financial::create_transaction,
        crate::handlers::financial::get_transaction,
        crate::handlers::financial::list_transactions,
        crate::handlers::financial::create_invoice,
        crate::handlers::financial::request_financing,
        // Compliance endpoints
        crate::handlers::compliance::check_compliance,
        crate::handlers::compliance::get_compliance_report,
        crate::handlers::compliance::generate_audit_report,
        // API Key endpoints
        crate::handlers::api_keys::create_key,
        crate::handlers::api_keys::list_keys,
        crate::handlers::api_keys::revoke_key,
        crate::handlers::api_keys::rotate_key,
    ),
    components(
        schemas(
            // Product schemas
            ProductResponse,
            PaginatedProductsResponse,
            CreateProductRequest,
            UpdateProductRequest,
            // Event schemas
            EventResponse,
            PaginatedEventsResponse,
            CreateEventRequest,
            ListEventsQuery,
            // Auth schemas
            LoginRequest,
            RegisterRequest,
            AuthResponse,
            // API Key schemas
            CreateApiKeyRequest,
            ApiKeyCreatedResponse,
            ApiKeyResponse,
            // Financial schemas
            CreateTransactionRequest,
            CreateInvoiceRequest,
            FinancingRequestBody,
            // Compliance schemas
            ComplianceCheckRequest,
            ComplianceReportResponse,
            // Model schemas
            crate::models::ApiKeyTier,
            crate::models::UserRole,
            crate::models::User,
            crate::models::Product,
            crate::models::TrackingEvent,
        )
    ),
    tags(
        (name = "products", description = "Product management operations"),
        (name = "events", description = "Tracking event operations"),
        (name = "stats", description = "Statistics and analytics"),
        (name = "health", description = "Health check endpoints"),
        (name = "authentication", description = "User authentication and registration"),
        (name = "carbon", description = "Carbon footprint management and trading"),
        (name = "financial", description = "Financial transactions and invoicing"),
        (name = "compliance", description = "Compliance checking and audit reports"),
        (name = "api_keys", description = "API key management")
    ),
    security(
        ("api_key" = []),
        ("jwt" = [])
    )
)]
pub struct ApiDoc;

pub fn create_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}

// API Key security scheme
#[derive(utoipa::ToSchema)]
pub struct ApiKeyAuth {
    #[schema(description = "API key for authentication")]
    api_key: String,
}

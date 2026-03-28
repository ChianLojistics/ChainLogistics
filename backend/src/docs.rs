use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    product::{ProductResponse, PaginatedProductsResponse, CreateProductRequest, UpdateProductRequest},
    event::{EventResponse, PaginatedEventsResponse, CreateEventRequest},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ChainLogistics API",
        version = "1.0.0",
        description = "API for managing supply chain products and tracking events on the blockchain",
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
        crate::handlers::product::list_products,
        crate::handlers::product::create_product,
        crate::handlers::product::get_product,
        crate::handlers::product::update_product,
        crate::handlers::product::delete_product,
        crate::handlers::event::list_events,
        crate::handlers::event::create_event,
        crate::handlers::event::get_event,
        crate::handlers::stats::get_stats,
        crate::handlers::health::health_check,
        crate::handlers::health::db_health_check,
    ),
    components(
        schemas(
            ProductResponse,
            PaginatedProductsResponse,
            CreateProductRequest,
            UpdateProductRequest,
            EventResponse,
            PaginatedEventsResponse,
            CreateEventRequest,
            crate::models::ApiKeyTier,
        )
    ),
    tags(
        (name = "products", description = "Product management operations"),
        (name = "events", description = "Tracking event operations"),
        (name = "stats", description = "Statistics and analytics"),
        (name = "health", description = "Health check endpoints")
    ),
    security(
        ("api_key" = [])
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

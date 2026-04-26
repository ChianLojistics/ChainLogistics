use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::{
    AppState,
    error::AppError,
    models::{Product, NewProduct, ProductFilters},
    validation::{validate_string, sanitize_input, validate_stellar_address, validate_product_id, validate_location, sanitize_json_metadata},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListProductsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub owner_address: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductRequest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
    pub media_hashes: Vec<String>,
    pub custom_fields: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub origin_location: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub certifications: Option<Vec<String>>,
    pub media_hashes: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
    pub media_hashes: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_address: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedProductsResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            name: product.name,
            description: product.description,
            origin_location: product.origin_location,
            category: product.category,
            tags: product.tags,
            certifications: product.certifications,
            media_hashes: product.media_hashes,
            custom_fields: product.custom_fields,
            owner_address: product.owner_address,
            is_active: product.is_active,
            created_at: product.created_at,
            updated_at: product.updated_at,
            created_by: product.created_by,
            updated_by: product.updated_by,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/products",
    tag = "products",
    params(ListProductsQuery),
    responses(
        (status = 200, description = "Products listed successfully", body = PaginatedProductsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<PaginatedProductsResponse>, AppError> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100); // Cap at 100

    // Validate query parameters
    if let Some(ref search) = query.search {
        validate_string("search", search, 100)?;
    }
    if let Some(ref category) = query.category {
        validate_string("category", category, 64)?;
    }
    if let Some(ref owner) = query.owner_address {
        validate_stellar_address(owner)?;
    }

    let products = if let Some(search_query) = query.search {
        state.product_service
            .search_products(&sanitize_input(&search_query), limit)
            .await?
            .into_iter()
            .map(ProductResponse::from)
            .collect()
    } else {
        let filters = ProductFilters {
            owner_address: query.owner_address.clone().map(|s| sanitize_input(&s)),
            category: query.category.clone().map(|s| sanitize_input(&s)),
            is_active: query.is_active,
            created_after: None,
            created_before: None,
        };

        state.product_service
            .list_products(offset, limit, Some(filters))
            .await?
            .into_iter()
            .map(ProductResponse::from)
            .collect()
    };

    let total = if query.search.is_some() {
        products.len() as i64
    } else {
        let filters = ProductFilters {
            owner_address: query.owner_address.map(|s| sanitize_input(&s)),
            category: query.category.map(|s| sanitize_input(&s)),
            is_active: query.is_active,
            created_after: None,
            created_before: None,
        };
        state.product_service
            .count_products(Some(filters))
            .await?
    };

    Ok(Json(PaginatedProductsResponse {
        products,
        total,
        offset,
        limit,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/products",
    tag = "products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created successfully", body = ProductResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn create_product(
    State(state): State<AppState>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    // Validate inputs
    validate_product_id(&request.id)?;
    validate_string("name", &request.name, 128)?;
    validate_string("category", &request.category, 64)?;
    validate_location(&request.origin_location)?;
    if request.description.len() > 2048 {
        return Err(AppError::Validation("description must not exceed 2048 characters".to_string()));
    }

    // Get auth context
    let auth_context = crate::middleware::auth::get_auth_context(&axum::extract::Request::builder().uri("/").body(()).unwrap())?;

    let new_product = NewProduct {
        id: sanitize_input(&request.id),
        name: sanitize_input(&request.name),
        description: sanitize_input(&request.description),
        origin_location: sanitize_input(&request.origin_location),
        category: sanitize_input(&request.category),
        tags: request.tags.iter().map(|t| sanitize_input(t)).collect(),
        certifications: request.certifications.iter().map(|c| sanitize_input(c)).collect(),
        media_hashes: request.media_hashes.iter().map(|m| sanitize_input(m)).collect(),
        custom_fields: {
            let mut fields = request.custom_fields;
            sanitize_json_metadata(&mut fields);
            fields
        },
        owner_address: auth_context.stellar_address.clone().unwrap_or_default(),
        created_by: auth_context.user_id.to_string(),
    };

    let product = state.product_service.create_product(new_product).await?;
    Ok(Json(ProductResponse::from(product)))
}

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    tag = "products",
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product retrieved successfully", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProductResponse>, AppError> {
    validate_product_id(&id)?;
    let product = state
        .product_service
        .get_product(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    Ok(Json(ProductResponse::from(product)))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/products/{id}",
    tag = "products",
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "Product updated successfully", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let auth_context = crate::middleware::auth::get_auth_context(&axum::extract::Request::builder().uri("/").body(()).unwrap())?;
    
    validate_product_id(&id)?;

    let mut product = state
        .product_service
        .get_product(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    // Update fields if provided with validation
    if let Some(name) = request.name {
        validate_string("name", &name, 128)?;
        product.name = sanitize_input(&name);
    }
    if let Some(description) = request.description {
        if description.len() > 2048 {
            return Err(AppError::Validation("description must not exceed 2048 characters".to_string()));
        }
        product.description = sanitize_input(&description);
    }
    if let Some(origin_location) = request.origin_location {
        validate_location(&origin_location)?;
        product.origin_location = sanitize_input(&origin_location);
    }
    if let Some(category) = request.category {
        validate_string("category", &category, 64)?;
        product.category = sanitize_input(&category);
    }
    if let Some(tags) = request.tags {
        product.tags = tags.iter().map(|t| sanitize_input(t)).collect();
    }
    if let Some(certifications) = request.certifications {
        product.certifications = certifications.iter().map(|c| sanitize_input(c)).collect();
    }
    if let Some(media_hashes) = request.media_hashes {
        product.media_hashes = media_hashes.iter().map(|m| sanitize_input(m)).collect();
    }
    if let Some(mut custom_fields) = request.custom_fields {
        sanitize_json_metadata(&mut custom_fields);
        product.custom_fields = custom_fields;
    }
    if let Some(is_active) = request.is_active {
        product.is_active = is_active;
    }

    product.updated_by = auth_context.user_id.to_string();

    let updated = state.product_service.update_product(&id, product).await?;
    Ok(Json(ProductResponse::from(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/products/{id}",
    tag = "products",
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 204, description = "Product deleted successfully"),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    validate_product_id(&id)?;

    // Check if product exists
    state
        .product_service
        .get_product(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    state.product_service.delete_product(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

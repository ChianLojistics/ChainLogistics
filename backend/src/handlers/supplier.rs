use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::supplier::*;
use crate::services::SupplierService;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListSuppliersQuery {
    pub business_type: Option<String>,
    pub tier: Option<String>,
    pub verification_status: Option<String>,
    pub is_verified: Option<bool>,
    pub limit: Option<i64>,
}

// Create supplier
#[utoipa::path(
    post,
    path = "/api/suppliers",
    request_body = NewSupplier,
    responses(
        (status = 201, description = "Supplier created", body = Supplier)
    ),
    tag = "suppliers"
)]
pub async fn create_supplier(
    State(service): State<Arc<SupplierService>>,
    Json(req): Json<NewSupplier>,
) -> Result<Json<Supplier>, AppError> {
    let supplier = service.create_supplier(req).await?;
    Ok(Json(supplier))
}

// Get supplier
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}",
    params(
        ("supplier_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Supplier found", body = Supplier),
        (status = 404, description = "Supplier not found")
    ),
    tag = "suppliers"
)]
pub async fn get_supplier(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
) -> Result<Json<Supplier>, AppError> {
    let supplier = service.get_supplier(&supplier_id).await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;
    Ok(Json(supplier))
}

// List suppliers
#[utoipa::path(
    get,
    path = "/api/suppliers",
    params(ListSuppliersQuery),
    responses(
        (status = 200, description = "List of suppliers", body = Vec<Supplier>)
    ),
    tag = "suppliers"
)]
pub async fn list_suppliers(
    State(service): State<Arc<SupplierService>>,
    Query(query): Query<ListSuppliersQuery>,
) -> Result<Json<Vec<Supplier>>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let suppliers = service.list_suppliers(
        query.business_type,
        query.tier,
        query.verification_status,
        query.is_verified,
        limit,
    ).await?;
    Ok(Json(suppliers))
}

// Update supplier verification
#[utoipa::path(
    put,
    path = "/api/suppliers/{supplier_id}/verification",
    params(
        ("supplier_id" = String, Path)
    ),
    request_body = VerificationRequest,
    responses(
        (status = 200, description = "Verification updated", body = Supplier)
    ),
    tag = "suppliers"
)]
pub async fn update_verification(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
    Json(req): Json<VerificationRequest>,
) -> Result<Json<Supplier>, AppError> {
    let supplier = service.update_supplier_verification(
        &supplier_id,
        req.verification_status,
        req.verified_by,
        req.notes,
    ).await?;
    Ok(Json(supplier))
}

// Get supplier summary
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}/summary",
    params(
        ("supplier_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Supplier summary", body = SupplierSummary),
        (status = 404, description = "Supplier not found")
    ),
    tag = "suppliers"
)]
pub async fn get_supplier_summary(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
) -> Result<Json<SupplierSummary>, AppError> {
    let summary = service.get_supplier_summary(&supplier_id).await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;
    Ok(Json(summary))
}

// Create supplier rating
#[utoipa::path(
    post,
    path = "/api/suppliers/ratings",
    request_body = NewSupplierRating,
    responses(
        (status = 201, description = "Rating created", body = SupplierRating)
    ),
    tag = "suppliers"
)]
pub async fn create_rating(
    State(service): State<Arc<SupplierService>>,
    Json(req): Json<NewSupplierRating>,
) -> Result<Json<SupplierRating>, AppError> {
    let rating = service.create_rating(req).await?;
    Ok(Json(rating))
}

// Get supplier ratings
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}/ratings",
    params(
        ("supplier_id" = String, Path),
        ("limit" = Option<i64>, Query)
    ),
    responses(
        (status = 200, description = "List of ratings", body = Vec<SupplierRating>)
    ),
    tag = "suppliers"
)]
pub async fn get_ratings(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<SupplierRating>>, AppError> {
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let ratings = service.get_ratings(&supplier_id, limit).await?;
    Ok(Json(ratings))
}

// Create supplier performance metric
#[utoipa::path(
    post,
    path = "/api/suppliers/performance",
    request_body = NewSupplierPerformance,
    responses(
        (status = 201, description = "Performance metric created", body = SupplierPerformance)
    ),
    tag = "suppliers"
)]
pub async fn create_performance(
    State(service): State<Arc<SupplierService>>,
    Json(req): Json<NewSupplierPerformance>,
) -> Result<Json<SupplierPerformance>, AppError> {
    let perf = service.create_performance(req).await?;
    Ok(Json(perf))
}

// Get supplier performance
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}/performance",
    params(
        ("supplier_id" = String, Path),
        ("limit" = Option<i64>, Query)
    ),
    responses(
        (status = 200, description = "List of performance metrics", body = Vec<SupplierPerformance>)
    ),
    tag = "suppliers"
)]
pub async fn get_performance(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<SupplierPerformance>>, AppError> {
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let perf = service.get_performance(&supplier_id, limit).await?;
    Ok(Json(perf))
}

// Create supplier compliance record
#[utoipa::path(
    post,
    path = "/api/suppliers/compliance",
    request_body = NewSupplierCompliance,
    responses(
        (status = 201, description = "Compliance record created", body = SupplierCompliance)
    ),
    tag = "suppliers"
)]
pub async fn create_compliance(
    State(service): State<Arc<SupplierService>>,
    Json(req): Json<NewSupplierCompliance>,
) -> Result<Json<SupplierCompliance>, AppError> {
    let compliance = service.create_compliance(req).await?;
    Ok(Json(compliance))
}

// Verify supplier compliance
#[utoipa::path(
    put,
    path = "/api/suppliers/compliance/{compliance_id}/verify",
    params(
        ("compliance_id" = Uuid, Path)
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Compliance verified", body = SupplierCompliance)
    ),
    tag = "suppliers"
)]
pub async fn verify_compliance(
    State(service): State<Arc<SupplierService>>,
    Path(compliance_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SupplierCompliance>, AppError> {
    let verified_by = body.get("verified_by")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("verified_by required".to_string()))?
        .to_string();
    let status = body.get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("status required".to_string()))?
        .to_string();
    
    let compliance = service.verify_compliance(compliance_id, verified_by, status).await?;
    Ok(Json(compliance))
}

// Get supplier compliance
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}/compliance",
    params(
        ("supplier_id" = String, Path)
    ),
    responses(
        (status = 200, description = "List of compliance records", body = Vec<SupplierCompliance>)
    ),
    tag = "suppliers"
)]
pub async fn get_compliance(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
) -> Result<Json<Vec<SupplierCompliance>>, AppError> {
    let compliance = service.get_compliance(&supplier_id).await?;
    Ok(Json(compliance))
}

// Add supplier product
#[utoipa::path(
    post,
    path = "/api/suppliers/products",
    request_body = NewSupplierProduct,
    responses(
        (status = 201, description = "Supplier product added", body = SupplierProduct)
    ),
    tag = "suppliers"
)]
pub async fn add_supplier_product(
    State(service): State<Arc<SupplierService>>,
    Json(req): Json<NewSupplierProduct>,
) -> Result<Json<SupplierProduct>, AppError> {
    let sp = service.add_supplier_product(req).await?;
    Ok(Json(sp))
}

// Get supplier products
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}/products",
    params(
        ("supplier_id" = String, Path)
    ),
    responses(
        (status = 200, description = "List of supplier products", body = Vec<SupplierProduct>)
    ),
    tag = "suppliers"
)]
pub async fn get_supplier_products(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
) -> Result<Json<Vec<SupplierProduct>>, AppError> {
    let products = service.get_supplier_products(&supplier_id).await?;
    Ok(Json(products))
}

// Get supplier audit trail
#[utoipa::path(
    get,
    path = "/api/suppliers/{supplier_id}/audit",
    params(
        ("supplier_id" = String, Path),
        ("limit" = Option<i64>, Query)
    ),
    responses(
        (status = 200, description = "Audit trail", body = Vec<SupplierAuditTrail>)
    ),
    tag = "suppliers"
)]
pub async fn get_audit_trail(
    State(service): State<Arc<SupplierService>>,
    Path(supplier_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<SupplierAuditTrail>>, AppError> {
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let audit = service.get_audit_trail(&supplier_id, limit).await?;
    Ok(Json(audit))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    error::AppError,
    services::storage_integrity_service::{RegisterAnchorRequest, MAX_FILE_BYTES},
    AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct AnchorResponse {
    pub content_hash: String,
    pub cid: String,
    pub storage_backend: String,
    pub product_id: Option<String>,
    pub byte_size: i64,
    pub mime_type: Option<String>,
    pub verification_status: String,
    pub anchored_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExistsResponse {
    pub content_hash: String,
    pub exists: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterAnchorBody {
    pub content_hash: String,
    pub cid: String,
    pub storage_backend: String,
    #[serde(alias = "productId")]
    pub product_id: Option<String>,
    pub byte_size: u64,
    pub mime_type: Option<String>,
    #[serde(alias = "anchoredBy")]
    pub anchored_by: Option<String>,
}

fn to_response(anchor: crate::services::storage_integrity_service::ContentAnchor) -> AnchorResponse {
    AnchorResponse {
        content_hash: anchor.content_hash,
        cid: anchor.cid,
        storage_backend: anchor.storage_backend,
        product_id: anchor.product_id,
        byte_size: anchor.byte_size,
        mime_type: anchor.mime_type,
        verification_status: anchor.verification_status,
        anchored_at: anchor.anchored_at,
    }
}

/// Register a content integrity anchor (CAS — duplicate hash + CID is idempotent).
#[utoipa::path(
    post,
    path = "/api/v1/storage/anchors",
    request_body = RegisterAnchorBody,
    responses(
        (status = 201, description = "Anchor registered", body = AnchorResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Hash mismatch")
    ),
    tag = "storage"
)]
pub async fn register_anchor(
    State(state): State<AppState>,
    Json(body): Json<RegisterAnchorBody>,
) -> Result<(StatusCode, Json<AnchorResponse>), AppError> {
    if body.byte_size == 0 || body.byte_size > MAX_FILE_BYTES {
        return Err(AppError::BadRequest(format!(
            "byte_size must be between 1 and {}",
            MAX_FILE_BYTES
        )));
    }

    if body.content_hash.len() != 64 {
        return Err(AppError::BadRequest(
            "content_hash must be a 64-character hex SHA-256 digest".into(),
        ));
    }

    let req = RegisterAnchorRequest {
        content_hash: body.content_hash.to_lowercase(),
        cid: body.cid,
        storage_backend: body.storage_backend,
        product_id: body.product_id,
        byte_size: body.byte_size,
        mime_type: body.mime_type,
        anchored_by: body.anchored_by,
    };

    let anchor = state
        .storage_integrity_service
        .register_anchor(&req)
        .await
        .map_err(|e| {
            if e.to_string().contains("different CID") {
                AppError::AlreadyExists(e.to_string())
            } else if e.to_string().contains("must be") {
                AppError::BadRequest(e.to_string())
            } else {
                AppError::Database(e)
            }
        })?;

    Ok((StatusCode::CREATED, Json(to_response(anchor))))
}

/// CAS existence check for content-hash deduplication before upload.
#[utoipa::path(
    get,
    path = "/api/v1/storage/exists/{content_hash}",
    params(("content_hash" = String, Path, description = "SHA-256 hex digest")),
    responses((status = 200, description = "Existence result", body = ExistsResponse)),
    tag = "storage"
)]
pub async fn check_exists(
    State(state): State<AppState>,
    Path(content_hash): Path<String>,
) -> Result<Json<ExistsResponse>, AppError> {
    let exists = state
        .storage_integrity_service
        .exists(&content_hash.to_lowercase())
        .await
        .map_err(AppError::Database)?;

    Ok(Json(ExistsResponse {
        content_hash,
        exists,
    }))
}

/// Get anchor metadata by content hash.
#[utoipa::path(
    get,
    path = "/api/v1/storage/anchors/{content_hash}",
    params(("content_hash" = String, Path, description = "SHA-256 hex digest")),
    responses(
        (status = 200, description = "Anchor found", body = AnchorResponse),
        (status = 404, description = "Not found")
    ),
    tag = "storage"
)]
pub async fn get_anchor(
    State(state): State<AppState>,
    Path(content_hash): Path<String>,
) -> Result<Json<AnchorResponse>, AppError> {
    let anchor = state
        .storage_integrity_service
        .get_anchor(&content_hash.to_lowercase())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Anchor not found".into()))?;

    Ok(Json(to_response(anchor)))
}

/// List anchors for a product.
#[utoipa::path(
    get,
    path = "/api/v1/storage/products/{product_id}/anchors",
    params(("product_id" = String, Path, description = "Product identifier")),
    responses((status = 200, description = "Product anchors", body = [AnchorResponse])),
    tag = "storage"
)]
pub async fn list_product_anchors(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<AnchorResponse>>, AppError> {
    let anchors = state
        .storage_integrity_service
        .list_product_anchors(&product_id)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(anchors.into_iter().map(to_response).collect()))
}

/// Trigger on-demand verification (admin/cron hook).
#[utoipa::path(
    post,
    path = "/api/v1/storage/verify",
    responses((status = 200, description = "Verification batch completed")),
    tag = "storage"
)]
pub async fn trigger_verification(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tampered = state
        .storage_integrity_service
        .verify_pending_anchors()
        .await
        .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({
        "status": "completed",
        "tampered_count": tampered
    })))
}

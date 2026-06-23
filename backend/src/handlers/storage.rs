use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::services::storage_service::{
    ContentAnchor, RegisterAnchorRequest, StorageScheme, TamperAlert, VerificationRunSummary,
    VerificationStatus,
};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterAnchorBody {
    pub product_id: String,
    pub content_hash: String,
    pub cid: String,
    pub storage_scheme: StorageScheme,
    pub byte_size: u64,
    pub storage_uri: String,
    pub on_chain_anchor_id: Option<i64>,
    pub anchored_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterAnchorResponse {
    pub anchor: ContentAnchor,
    pub deduplicated: bool,
}

#[derive(Debug, Serialize)]
pub struct VerifyAnchorResponse {
    pub anchor_id: Uuid,
    pub status: VerificationStatus,
}

pub async fn register_anchor(
    State(state): State<AppState>,
    Json(body): Json<RegisterAnchorBody>,
) -> Result<Json<RegisterAnchorResponse>, AppError> {
    let (anchor, deduplicated) = state
        .content_anchor_service
        .register_anchor(RegisterAnchorRequest {
            product_id: body.product_id,
            content_hash: body.content_hash,
            cid: body.cid,
            storage_scheme: body.storage_scheme,
            byte_size: body.byte_size,
            storage_uri: body.storage_uri,
            on_chain_anchor_id: body.on_chain_anchor_id,
            anchored_by: body.anchored_by,
        })
        .await
        .map_err(AppError::from)?;

    Ok(Json(RegisterAnchorResponse {
        anchor,
        deduplicated,
    }))
}

pub async fn list_product_anchors(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ContentAnchor>>, AppError> {
    let anchors = state
        .content_anchor_service
        .list_by_product(&product_id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(anchors))
}

pub async fn get_anchor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ContentAnchor>, AppError> {
    let anchor = state
        .content_anchor_service
        .get_by_id(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Anchor {} not found", id)))?;
    Ok(Json(anchor))
}

pub async fn verify_anchor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VerifyAnchorResponse>, AppError> {
    let status = state
        .storage_verification_service
        .verify_anchor_by_id(id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(VerifyAnchorResponse {
        anchor_id: id,
        status,
    }))
}

pub async fn run_verification(
    State(state): State<AppState>,
) -> Result<Json<VerificationRunSummary>, AppError> {
    let summary = state
        .storage_verification_service
        .verify_due_anchors()
        .await
        .map_err(AppError::from)?;
    Ok(Json(summary))
}

pub async fn list_tamper_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<TamperAlert>>, AppError> {
    let alerts = state
        .content_anchor_service
        .list_unresolved_alerts()
        .await
        .map_err(AppError::from)?;
    Ok(Json(alerts))
}

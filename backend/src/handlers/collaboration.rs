use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use uuid::Uuid;
use crate::{
    error::AppError,
    models::collaboration::{ShareProductRequest, CreateCollaborationRequest, UpdateCollaborationRequest},
    AppState,
};

pub async fn share_product(
    State(state): State<AppState>,
    Json(req): Json<ShareProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real actor_id from JWT auth context
    let actor_id = Uuid::nil();
    let share = state.collaboration_service.share_product(actor_id, req).await?;
    Ok((StatusCode::CREATED, Json(share)))
}

pub async fn list_shares(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let shares = state.collaboration_service.list_shares(&product_id).await?;
    Ok(Json(shares))
}

pub async fn create_collaboration_request(
    State(state): State<AppState>,
    Json(req): Json<CreateCollaborationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real requester_id from JWT auth context
    let requester_id = Uuid::nil();
    let request = state.collaboration_service.create_collaboration_request(requester_id, req).await?;
    Ok((StatusCode::CREATED, Json(request)))
}

pub async fn update_collaboration_request(
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
    Json(req): Json<UpdateCollaborationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real actor_id from JWT auth context
    let actor_id = Uuid::nil();
    let updated = state.collaboration_service.update_collaboration_request(actor_id, request_id, &req.status).await?;
    Ok(Json(updated))
}

pub async fn list_audit_trail(
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let trails = state.collaboration_service.list_audit_trail(&entity_type, &entity_id).await?;
    Ok(Json(trails))
}

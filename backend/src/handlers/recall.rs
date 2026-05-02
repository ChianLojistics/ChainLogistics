use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    AppState,
    error::AppError,
    models::{Recall, RecallAffectedItem, RecallEffectiveness, RecallNotification},
    validation::{sanitize_input, validate_product_id, validate_string, validate_uuid},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListRecallsQuery {
    #[serde(alias = "productId")]
    pub product_id: String,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRecallRequest {
    #[serde(alias = "productId")]
    pub product_id: String,
    #[serde(alias = "batchId")]
    pub batch_id: Option<String>,
    pub title: String,
    pub reason: String,
    pub severity: Option<String>,
    #[serde(alias = "triggerType")]
    pub trigger_type: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NotifyRecallRequest {
    pub recipients: Vec<String>,
    pub channel: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEffectivenessRequest {
    pub acknowledged_delta: Option<i32>,
    pub recovered_delta: Option<i32>,
    pub disposed_delta: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecallWithEffectiveness {
    pub recall: Recall,
    pub effectiveness: Option<RecallEffectiveness>,
}

#[utoipa::path(
    get,
    path = "/api/v1/recalls",
    tag = "recalls",
    params(ListRecallsQuery),
    responses(
        (status = 200, description = "Recalls listed successfully", body = [Recall]),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_recalls(
    State(state): State<AppState>,
    Query(query): Query<ListRecallsQuery>,
) -> Result<Json<Vec<Recall>>, AppError> {
    validate_product_id(&query.product_id)?;
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let recalls = state
        .recall_service
        .list_recalls_by_product(&sanitize_input(&query.product_id), limit, offset)
        .await?;

    Ok(Json(recalls))
}

#[utoipa::path(
    get,
    path = "/api/v1/recalls/{id}",
    tag = "recalls",
    params(
        ("id" = String, Path, description = "Recall ID")
    ),
    responses(
        (status = 200, description = "Recall retrieved successfully", body = RecallWithEffectiveness),
        (status = 404, description = "Recall not found"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_recall(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RecallWithEffectiveness>, AppError> {
    validate_uuid(&id)?;
    let recall_id = uuid::Uuid::parse_str(&id)?;

    let recall = state
        .recall_service
        .get_recall(recall_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Recall {} not found", id)))?;

    let effectiveness = state.recall_service.get_effectiveness(recall_id).await?;

    Ok(Json(RecallWithEffectiveness { recall, effectiveness }))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/recalls",
    tag = "recalls",
    request_body = CreateRecallRequest,
    responses(
        (status = 201, description = "Recall created successfully", body = RecallWithEffectiveness),
        (status = 400, description = "Bad request - invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_recall(
    State(state): State<AppState>,
    Json(request): Json<CreateRecallRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_product_id(&request.product_id)?;
    validate_string("title", &request.title, 128)?;
    validate_string("reason", &request.reason, 512)?;

    if let Some(batch_id) = &request.batch_id {
        validate_string("batch_id", batch_id, 128)?;
    }

    let severity = request.severity.unwrap_or_else(|| "medium".to_string());
    let trigger_type = request.trigger_type.unwrap_or_else(|| "manual".to_string());

    let recall = state
        .recall_service
        .create_recall(
            &sanitize_input(&request.product_id),
            request.batch_id.as_deref(),
            &sanitize_input(&request.title),
            &sanitize_input(&request.reason),
            &sanitize_input(&severity),
            &sanitize_input(&trigger_type),
            None,
            None,
            request.metadata,
        )
        .await?;

    let _ = state
        .recall_service
        .identify_affected_items(recall.id, &request.product_id, request.batch_id.as_deref())
        .await?;

    let effectiveness = state.recall_service.get_effectiveness(recall.id).await?;

    Ok((StatusCode::CREATED, Json(RecallWithEffectiveness { recall, effectiveness })))
}

#[utoipa::path(
    get,
    path = "/api/v1/recalls/{id}/affected",
    tag = "recalls",
    params(
        ("id" = String, Path, description = "Recall ID")
    ),
    responses(
        (status = 200, description = "Affected items listed successfully", body = [RecallAffectedItem]),
        (status = 404, description = "Recall not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_affected_items(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<RecallAffectedItem>>, AppError> {
    validate_uuid(&id)?;
    let recall_id = uuid::Uuid::parse_str(&id)?;

    state
        .recall_service
        .get_recall(recall_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Recall {} not found", id)))?;

    let items = sqlx::query_as!(
        RecallAffectedItem,
        r#"
        SELECT
            id,
            recall_id,
            product_id,
            batch_id,
            stakeholder_role,
            stakeholder_address,
            detected_via,
            created_at
        FROM recall_affected_items
        WHERE recall_id = $1
        ORDER BY created_at ASC
        "#,
        recall_id
    )
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(items))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/recalls/{id}/notify",
    tag = "recalls",
    params(
        ("id" = String, Path, description = "Recall ID")
    ),
    request_body = NotifyRecallRequest,
    responses(
        (status = 200, description = "Notifications queued", body = [RecallNotification]),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Recall not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn notify_recall(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<NotifyRecallRequest>,
) -> Result<Json<Vec<RecallNotification>>, AppError> {
    validate_uuid(&id)?;
    let recall_id = uuid::Uuid::parse_str(&id)?;

    state
        .recall_service
        .get_recall(recall_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Recall {} not found", id)))?;

    if request.recipients.is_empty() {
        return Err(AppError::Validation("recipients must not be empty".to_string()));
    }

    let channel = request.channel.unwrap_or_else(|| "in_app".to_string());
    let recipients = request
        .recipients
        .into_iter()
        .map(|r| sanitize_input(&r))
        .collect::<Vec<_>>();

    let notifications = state
        .recall_service
        .queue_notifications(recall_id, recipients, &sanitize_input(&channel), request.payload)
        .await?;

    Ok(Json(notifications))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/recalls/{id}/effectiveness",
    tag = "recalls",
    params(
        ("id" = String, Path, description = "Recall ID")
    ),
    request_body = UpdateEffectivenessRequest,
    responses(
        (status = 200, description = "Effectiveness updated", body = RecallEffectiveness),
        (status = 404, description = "Recall not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update_effectiveness(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateEffectivenessRequest>,
) -> Result<Json<RecallEffectiveness>, AppError> {
    validate_uuid(&id)?;
    let recall_id = uuid::Uuid::parse_str(&id)?;

    state
        .recall_service
        .get_recall(recall_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Recall {} not found", id)))?;

    let eff = state
        .recall_service
        .update_effectiveness(
            recall_id,
            request.acknowledged_delta.unwrap_or(0),
            request.recovered_delta.unwrap_or(0),
            request.disposed_delta.unwrap_or(0),
        )
        .await?;

    Ok(Json(eff))
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use crate::models::batch::*;
use crate::services::BatchRepository;
use crate::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListBatchesQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub product_id: Option<String>,
    pub lot_number: Option<String>,
    pub status: Option<String>,
    pub quality_grade: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchListResponse {
    pub batches: Vec<Batch>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

#[utoipa::path(
    post,
    path = "/api/v1/batches",
    request_body = NewBatch,
    responses(
        (status = 201, description = "Batch created successfully", body = Batch),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn create_batch(
    State(state): State<AppState>,
    Json(batch): Json<NewBatch>,
) -> Result<Json<Batch>, (StatusCode, String)> {
    state.batch_service
        .create_batch(batch)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}",
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 200, description = "Batch retrieved successfully", body = Batch),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_batch(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Batch>, (StatusCode, String)> {
    state.batch_service
        .get_batch(id)
        .await
        .map(|batch| batch.ok_or((StatusCode::NOT_FOUND, "Batch not found".to_string())))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/number/{batch_number}",
    params(
        ("batch_number" = String, Path, description = "Batch number")
    ),
    responses(
        (status = 200, description = "Batch retrieved successfully", body = Batch),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_batch_by_number(
    State(state): State<AppState>,
    Path(batch_number): Path<String>,
) -> Result<Json<Batch>, (StatusCode, String)> {
    state.batch_service
        .get_batch_by_number(&batch_number)
        .await
        .map(|batch| batch.ok_or((StatusCode::NOT_FOUND, "Batch not found".to_string())))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/batches",
    params(
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
        ("limit" = Option<i64>, Query, description = "Limit for pagination"),
        ("product_id" = Option<String>, Query, description = "Filter by product ID"),
        ("lot_number" = Option<String>, Query, description = "Filter by lot number"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("quality_grade" = Option<String>, Query, description = "Filter by quality grade")
    ),
    responses(
        (status = 200, description = "Batches retrieved successfully", body = BatchListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn list_batches(
    State(state): State<AppState>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<BatchListResponse>, (StatusCode, String)> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    let filters = BatchFilters {
        product_id: query.product_id,
        lot_number: query.lot_number,
        status: query.status.and_then(|s| match s.as_str() {
            "pending" => Some(BatchStatus::Pending),
            "in_production" => Some(BatchStatus::InProduction),
            "completed" => Some(BatchStatus::Completed),
            "quality_check" => Some(BatchStatus::QualityCheck),
            "quarantined" => Some(BatchStatus::Quarantined),
            "shipped" => Some(BatchStatus::Shipped),
            "recalled" => Some(BatchStatus::Recalled),
            "disposed" => Some(BatchStatus::Disposed),
            _ => None,
        }),
        quality_grade: query.quality_grade,
        production_after: None,
        production_before: None,
        expiry_after: None,
        expiry_before: None,
    };

    let batches = state.batch_service
        .list_batches(offset, limit, Some(filters))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total = state.batch_service
        .count_batches(Some(filters))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(BatchListResponse {
        batches,
        total,
        offset,
        limit,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/batches/{id}",
    request_body = Batch,
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 200, description = "Batch updated successfully", body = Batch),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn update_batch(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(batch): Json<Batch>,
) -> Result<Json<Batch>, (StatusCode, String)> {
    state.batch_service
        .update_batch(id, batch)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/batches/{id}",
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 204, description = "Batch deleted successfully"),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn delete_batch(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.batch_service
        .delete_batch(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// Genealogy endpoints

#[utoipa::path(
    post,
    path = "/api/v1/batches/genealogy",
    request_body = NewBatchGenealogy,
    responses(
        (status = 201, description = "Genealogy created successfully", body = BatchGenealogy),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn create_genealogy(
    State(state): State<AppState>,
    Json(genealogy): Json<NewBatchGenealogy>,
) -> Result<Json<BatchGenealogy>, (StatusCode, String)> {
    state.batch_service
        .create_genealogy(genealogy)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/genealogy",
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 200, description = "Genealogy tree retrieved successfully", body = BatchGenealogyTree),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_genealogy_tree(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BatchGenealogyTree>, (StatusCode, String)> {
    state.batch_service
        .get_genealogy_tree(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// Quality attributes endpoints

#[utoipa::path(
    post,
    path = "/api/v1/batches/quality-attributes",
    request_body = NewBatchQualityAttribute,
    responses(
        (status = 201, description = "Quality attribute created successfully", body = BatchQualityAttribute),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn create_quality_attribute(
    State(state): State<AppState>,
    Json(attribute): Json<NewBatchQualityAttribute>,
) -> Result<Json<BatchQualityAttribute>, (StatusCode, String)> {
    state.batch_service
        .create_quality_attribute(attribute)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/quality-attributes",
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 200, description = "Quality attributes retrieved successfully", body = Vec<BatchQualityAttribute>),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_quality_attributes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BatchQualityAttribute>>, (StatusCode, String)> {
    state.batch_service
        .get_quality_attributes(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// Recall endpoints

#[utoipa::path(
    post,
    path = "/api/v1/batches/recalls",
    request_body = NewBatchRecall,
    responses(
        (status = 201, description = "Recall created successfully", body = BatchRecall),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn create_recall(
    State(state): State<AppState>,
    Json(recall): Json<NewBatchRecall>,
) -> Result<Json<BatchRecall>, (StatusCode, String)> {
    state.batch_service
        .create_recall(recall)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/recalls/{id}",
    params(
        ("id" = Uuid, Path, description = "Recall ID")
    ),
    responses(
        (status = 200, description = "Recall retrieved successfully", body = BatchRecall),
        (status = 404, description = "Recall not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_recall(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BatchRecall>, (StatusCode, String)> {
    state.batch_service
        .get_recall(id)
        .await
        .map(|recall| recall.ok_or((StatusCode::NOT_FOUND, "Recall not found".to_string())))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/recalls",
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 200, description = "Batch recalls retrieved successfully", body = Vec<BatchRecall>),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_batch_recalls(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BatchRecall>>, (StatusCode, String)> {
    state.batch_service
        .get_batch_recalls(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/recalls/active",
    params(
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
        ("limit" = Option<i64>, Query, description = "Limit for pagination")
    ),
    responses(
        (status = 200, description = "Active recalls retrieved successfully", body = Vec<BatchRecall>),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn list_active_recalls(
    State(state): State<AppState>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<Vec<BatchRecall>>, (StatusCode, String)> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    state.batch_service
        .list_active_recalls(offset, limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/v1/batches/recalls/{id}",
    request_body = BatchRecall,
    params(
        ("id" = Uuid, Path, description = "Recall ID")
    ),
    responses(
        (status = 200, description = "Recall updated successfully", body = BatchRecall),
        (status = 404, description = "Recall not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn update_recall(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(recall): Json<BatchRecall>,
) -> Result<Json<BatchRecall>, (StatusCode, String)> {
    state.batch_service
        .update_recall(id, recall)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// Inventory endpoints

#[utoipa::path(
    post,
    path = "/api/v1/batches/inventory",
    request_body = NewBatchInventory,
    responses(
        (status = 201, description = "Inventory transaction created successfully", body = BatchInventory),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn create_inventory_transaction(
    State(state): State<AppState>,
    Json(inventory): Json<NewBatchInventory>,
) -> Result<Json<BatchInventory>, (StatusCode, String)> {
    state.batch_service
        .create_inventory_transaction(inventory)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/inventory",
    params(
        ("id" = Uuid, Path, description = "Batch ID")
    ),
    responses(
        (status = 200, description = "Batch inventory retrieved successfully", body = Vec<BatchInventory>),
        (status = 404, description = "Batch not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "batches"
)]
pub async fn get_batch_inventory(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BatchInventory>>, (StatusCode, String)> {
    state.batch_service
        .get_batch_inventory(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

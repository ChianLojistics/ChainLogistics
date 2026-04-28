use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::quality::*;
use crate::services::QualityService;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListCheckpointsQuery {
    pub checkpoint_type: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListInspectionsQuery {
    pub product_id: Option<String>,
    pub checkpoint_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListNonConformancesQuery {
    pub product_id: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListMetricsQuery {
    pub product_id: Option<String>,
    pub metric_type: Option<String>,
    pub limit: Option<i64>,
}

// Create QC checkpoint
#[utoipa::path(
    post,
    path = "/api/quality/checkpoints",
    request_body = NewQCCheckpoint,
    responses(
        (status = 201, description = "Checkpoint created", body = QCCheckpoint)
    ),
    tag = "quality"
)]
pub async fn create_checkpoint(
    State(service): State<Arc<QualityService>>,
    Json(req): Json<NewQCCheckpoint>,
) -> Result<Json<QCCheckpoint>, AppError> {
    let checkpoint = service.create_checkpoint(req).await?;
    Ok(Json(checkpoint))
}

// Get QC checkpoint
#[utoipa::path(
    get,
    path = "/api/quality/checkpoints/{checkpoint_id}",
    params(
        ("checkpoint_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Checkpoint found", body = QCCheckpoint),
        (status = 404, description = "Checkpoint not found")
    ),
    tag = "quality"
)]
pub async fn get_checkpoint(
    State(service): State<Arc<QualityService>>,
    Path(checkpoint_id): Path<String>,
) -> Result<Json<QCCheckpoint>, AppError> {
    let checkpoint = service.get_checkpoint(&checkpoint_id).await?
        .ok_or_else(|| AppError::NotFound("Checkpoint not found".to_string()))?;
    Ok(Json(checkpoint))
}

// List QC checkpoints
#[utoipa::path(
    get,
    path = "/api/quality/checkpoints",
    params(ListCheckpointsQuery),
    responses(
        (status = 200, description = "List of checkpoints", body = Vec<QCCheckpoint>)
    ),
    tag = "quality"
)]
pub async fn list_checkpoints(
    State(service): State<Arc<QualityService>>,
    Query(query): Query<ListCheckpointsQuery>,
) -> Result<Json<Vec<QCCheckpoint>>, AppError> {
    let checkpoints = service.list_checkpoints(
        query.checkpoint_type,
        query.category,
        query.is_active,
    ).await?;
    Ok(Json(checkpoints))
}

// Create QC workflow
#[utoipa::path(
    post,
    path = "/api/quality/workflows",
    request_body = NewQCWorkflow,
    responses(
        (status = 201, description = "Workflow created", body = QCWorkflow)
    ),
    tag = "quality"
)]
pub async fn create_workflow(
    State(service): State<Arc<QualityService>>,
    Json(req): Json<NewQCWorkflow>,
) -> Result<Json<QCWorkflow>, AppError> {
    let workflow = service.create_workflow(req).await?;
    Ok(Json(workflow))
}

// Get QC workflow
#[utoipa::path(
    get,
    path = "/api/quality/workflows/{workflow_id}",
    params(
        ("workflow_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Workflow found", body = QCWorkflow),
        (status = 404, description = "Workflow not found")
    ),
    tag = "quality"
)]
pub async fn get_workflow(
    State(service): State<Arc<QualityService>>,
    Path(workflow_id): Path<String>,
) -> Result<Json<QCWorkflow>, AppError> {
    let workflow = service.get_workflow(&workflow_id).await?
        .ok_or_else(|| AppError::NotFound("Workflow not found".to_string()))?;
    Ok(Json(workflow))
}

// List QC workflows
#[utoipa::path(
    get,
    path = "/api/quality/workflows",
    params(
        ("is_active" = Option<bool>, Query)
    ),
    responses(
        (status = 200, description = "List of workflows", body = Vec<QCWorkflow>)
    ),
    tag = "quality"
)]
pub async fn list_workflows(
    State(service): State<Arc<QualityService>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<QCWorkflow>>, AppError> {
    let is_active = params.get("is_active")
        .and_then(|s| s.parse().ok());
    let workflows = service.list_workflows(is_active).await?;
    Ok(Json(workflows))
}

// Create QC inspection
#[utoipa::path(
    post,
    path = "/api/quality/inspections",
    request_body = NewQCInspection,
    responses(
        (status = 201, description = "Inspection created", body = QCInspection)
    ),
    tag = "quality"
)]
pub async fn create_inspection(
    State(service): State<Arc<QualityService>>,
    Json(req): Json<NewQCInspection>,
) -> Result<Json<QCInspection>, AppError> {
    let inspection = service.create_inspection(req).await?;
    Ok(Json(inspection))
}

// Update inspection status
#[utoipa::path(
    put,
    path = "/api/quality/inspections/{inspection_id}/status",
    params(
        ("inspection_id" = String, Path)
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Inspection updated", body = QCInspection)
    ),
    tag = "quality"
)]
pub async fn update_inspection_status(
    State(service): State<Arc<QualityService>>,
    Path(inspection_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<QCInspection>, AppError> {
    let status = body.get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("status required".to_string()))?
        .to_string();
    let is_passed = body.get("is_passed")
        .and_then(|v| v.as_bool());
    let failure_reason = body.get("failure_reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let inspection = service.update_inspection_status(&inspection_id, status, is_passed, failure_reason).await?;
    Ok(Json(inspection))
}

// Get QC inspection
#[utoipa::path(
    get,
    path = "/api/quality/inspections/{inspection_id}",
    params(
        ("inspection_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Inspection found", body = QCInspection),
        (status = 404, description = "Inspection not found")
    ),
    tag = "quality"
)]
pub async fn get_inspection(
    State(service): State<Arc<QualityService>>,
    Path(inspection_id): Path<String>,
) -> Result<Json<QCInspection>, AppError> {
    let inspection = service.get_inspection(&inspection_id).await?
        .ok_or_else(|| AppError::NotFound("Inspection not found".to_string()))?;
    Ok(Json(inspection))
}

// List QC inspections
#[utoipa::path(
    get,
    path = "/api/quality/inspections",
    params(ListInspectionsQuery),
    responses(
        (status = 200, description = "List of inspections", body = Vec<QCInspection>)
    ),
    tag = "quality"
)]
pub async fn list_inspections(
    State(service): State<Arc<QualityService>>,
    Query(query): Query<ListInspectionsQuery>,
) -> Result<Json<Vec<QCInspection>>, AppError> {
    let limit = query.limit.unwrap_or(100);
    let inspections = service.list_inspections(
        query.product_id,
        query.checkpoint_id,
        query.status,
        limit,
    ).await?;
    Ok(Json(inspections))
}

// Execute workflow
#[utoipa::path(
    post,
    path = "/api/quality/workflows/execute",
    request_body = WorkflowExecutionRequest,
    responses(
        (status = 200, description = "Workflow executed", body = WorkflowExecutionResult)
    ),
    tag = "quality"
)]
pub async fn execute_workflow(
    State(service): State<Arc<QualityService>>,
    Json(req): Json<WorkflowExecutionRequest>,
) -> Result<Json<WorkflowExecutionResult>, AppError> {
    let result = service.execute_workflow(req).await?;
    Ok(Json(result))
}

// Create non-conformance
#[utoipa::path(
    post,
    path = "/api/quality/non-conformances",
    request_body = NewNonConformance,
    responses(
        (status = 201, description = "Non-conformance created", body = NonConformance)
    ),
    tag = "quality"
)]
pub async fn create_non_conformance(
    State(service): State<Arc<QualityService>>,
    Json(req): Json<NewNonConformance>,
) -> Result<Json<NonConformance>, AppError> {
    let nc = service.create_non_conformance(req).await?;
    Ok(Json(nc))
}

// Update non-conformance
#[utoipa::path(
    put,
    path = "/api/quality/non-conformances/{nc_id}",
    params(
        ("nc_id" = String, Path)
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Non-conformance updated", body = NonConformance)
    ),
    tag = "quality"
)]
pub async fn update_non_conformance(
    State(service): State<Arc<QualityService>>,
    Path(nc_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<NonConformance>, AppError> {
    let correction_action = body.get("correction_action")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let correction_status = body.get("correction_status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("correction_status required".to_string()))?
        .to_string();
    let responsible_party = body.get("responsible_party")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let nc = service.update_non_conformance(&nc_id, correction_action, correction_status, responsible_party).await?;
    Ok(Json(nc))
}

// Verify non-conformance
#[utoipa::path(
    post,
    path = "/api/quality/non-conformances/{nc_id}/verify",
    params(
        ("nc_id" = String, Path)
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Non-conformance verified", body = NonConformance)
    ),
    tag = "quality"
)]
pub async fn verify_non_conformance(
    State(service): State<Arc<QualityService>>,
    Path(nc_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<NonConformance>, AppError> {
    let verified_by = body.get("verified_by")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("verified_by required".to_string()))?
        .to_string();
    
    let nc = service.verify_non_conformance(&nc_id, verified_by).await?;
    Ok(Json(nc))
}

// List non-conformances
#[utoipa::path(
    get,
    path = "/api/quality/non-conformances",
    params(ListNonConformancesQuery),
    responses(
        (status = 200, description = "List of non-conformances", body = Vec<NonConformance>)
    ),
    tag = "quality"
)]
pub async fn list_non_conformances(
    State(service): State<Arc<QualityService>>,
    Query(query): Query<ListNonConformancesQuery>,
) -> Result<Json<Vec<NonConformance>>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let ncs = service.get_non_conformances(
        query.product_id,
        query.severity,
        query.status,
        limit,
    ).await?;
    Ok(Json(ncs))
}

// Create quality metric
#[utoipa::path(
    post,
    path = "/api/quality/metrics",
    request_body = NewQualityMetric,
    responses(
        (status = 201, description = "Metric created", body = QualityMetric)
    ),
    tag = "quality"
)]
pub async fn create_metric(
    State(service): State<Arc<QualityService>>,
    Json(req): Json<NewQualityMetric>,
) -> Result<Json<QualityMetric>, AppError> {
    let metric = service.create_metric(req).await?;
    Ok(Json(metric))
}

// List quality metrics
#[utoipa::path(
    get,
    path = "/api/quality/metrics",
    params(ListMetricsQuery),
    responses(
        (status = 200, description = "List of metrics", body = Vec<QualityMetric>)
    ),
    tag = "quality"
)]
pub async fn list_metrics(
    State(service): State<Arc<QualityService>>,
    Query(query): Query<ListMetricsQuery>,
) -> Result<Json<Vec<QualityMetric>>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let metrics = service.get_metrics(
        query.product_id,
        query.metric_type,
        limit,
    ).await?;
    Ok(Json(metrics))
}

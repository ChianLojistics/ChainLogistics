use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::regulatory::*;
use crate::services::RegulatoryService;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListRequirementsQuery {
    pub regulation_type: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RunComplianceCheckRequest {
    pub product_id: String,
    pub requirement_ids: Option<Vec<String>>,
    pub performed_by: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub scope: serde_json::Value,
    pub period_start: Option<chrono::DateTime<chrono::Utc>>,
    pub period_end: Option<chrono::DateTime<chrono::Utc>>,
}

// Create regulatory requirement
#[utoipa::path(
    post,
    path = "/api/regulatory/requirements",
    request_body = NewRegulatoryRequirement,
    responses(
        (status = 201, description = "Requirement created successfully", body = RegulatoryRequirement),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "regulatory"
)]
pub async fn create_requirement(
    State(service): State<Arc<RegulatoryService>>,
    Json(req): Json<NewRegulatoryRequirement>,
) -> Result<Json<RegulatoryRequirement>, AppError> {
    let requirement = service.create_requirement(req).await?;
    Ok(Json(requirement))
}

// Get regulatory requirement
#[utoipa::path(
    get,
    path = "/api/regulatory/requirements/{requirement_id}",
    params(
        ("requirement_id" = String, Path, description = "Requirement ID")
    ),
    responses(
        (status = 200, description = "Requirement found", body = RegulatoryRequirement),
        (status = 404, description = "Requirement not found")
    ),
    tag = "regulatory"
)]
pub async fn get_requirement(
    State(service): State<Arc<RegulatoryService>>,
    Path(requirement_id): Path<String>,
) -> Result<Json<RegulatoryRequirement>, AppError> {
    let requirement = service.get_requirement(&requirement_id).await?
        .ok_or_else(|| AppError::NotFound("Requirement not found".to_string()))?;
    Ok(Json(requirement))
}

// List regulatory requirements
#[utoipa::path(
    get,
    path = "/api/regulatory/requirements",
    params(ListRequirementsQuery),
    responses(
        (status = 200, description = "List of requirements", body = Vec<RegulatoryRequirement>)
    ),
    tag = "regulatory"
)]
pub async fn list_requirements(
    State(service): State<Arc<RegulatoryService>>,
    Query(query): Query<ListRequirementsQuery>,
) -> Result<Json<Vec<RegulatoryRequirement>>, AppError> {
    let requirements = service.list_requirements(
        query.regulation_type,
        query.category,
        query.is_active,
    ).await?;
    Ok(Json(requirements))
}

// Update regulatory requirement
#[utoipa::path(
    put,
    path = "/api/regulatory/requirements/{requirement_id}",
    request_body = NewRegulatoryRequirement,
    responses(
        (status = 200, description = "Requirement updated", body = RegulatoryRequirement),
        (status = 404, description = "Requirement not found")
    ),
    tag = "regulatory"
)]
pub async fn update_requirement(
    State(service): State<Arc<RegulatoryService>>,
    Path(requirement_id): Path<String>,
    Json(req): Json<NewRegulatoryRequirement>,
) -> Result<Json<RegulatoryRequirement>, AppError> {
    let requirement = service.update_requirement(&requirement_id, req).await?;
    Ok(Json(requirement))
}

// Create product compliance record
#[utoipa::path(
    post,
    path = "/api/regulatory/compliance",
    request_body = NewProductCompliance,
    responses(
        (status = 201, description = "Compliance record created", body = ProductCompliance)
    ),
    tag = "regulatory"
)]
pub async fn create_product_compliance(
    State(service): State<Arc<RegulatoryService>>,
    Json(req): Json<NewProductCompliance>,
) -> Result<Json<ProductCompliance>, AppError> {
    let compliance = service.create_product_compliance(req).await?;
    Ok(Json(compliance))
}

// Get product compliance
#[utoipa::path(
    get,
    path = "/api/regulatory/compliance/{product_id}/{requirement_id}",
    params(
        ("product_id" = String, Path),
        ("requirement_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Compliance record found", body = ProductCompliance),
        (status = 404, description = "Compliance record not found")
    ),
    tag = "regulatory"
)]
pub async fn get_product_compliance(
    State(service): State<Arc<RegulatoryService>>,
    Path((product_id, requirement_id)): Path<(String, String)>,
) -> Result<Json<ProductCompliance>, AppError> {
    let compliance = service.get_product_compliance(&product_id, &requirement_id).await?
        .ok_or_else(|| AppError::NotFound("Compliance record not found".to_string()))?;
    Ok(Json(compliance))
}

// List product compliance
#[utoipa::path(
    get,
    path = "/api/regulatory/compliance/{product_id}",
    params(
        ("product_id" = String, Path)
    ),
    responses(
        (status = 200, description = "List of compliance records", body = Vec<ProductCompliance>)
    ),
    tag = "regulatory"
)]
pub async fn list_product_compliance(
    State(service): State<Arc<RegulatoryService>>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ProductCompliance>>, AppError> {
    let compliance = service.list_product_compliance(&product_id).await?;
    Ok(Json(compliance))
}

// Run automated compliance check
#[utoipa::path(
    post,
    path = "/api/regulatory/check",
    request_body = RunComplianceCheckRequest,
    responses(
        (status = 200, description = "Compliance check completed", body = ComplianceCheckResult)
    ),
    tag = "regulatory"
)]
pub async fn run_compliance_check(
    State(service): State<Arc<RegulatoryService>>,
    Json(req): Json<RunComplianceCheckRequest>,
) -> Result<Json<ComplianceCheckResult>, AppError> {
    let check_request = ComplianceCheckRequest {
        product_id: req.product_id,
        requirement_ids: req.requirement_ids,
        performed_by: req.performed_by,
    };
    let result = service.run_compliance_check(check_request).await?;
    Ok(Json(result))
}

// Get audit trail
#[utoipa::path(
    get,
    path = "/api/regulatory/audit/{product_id}",
    params(
        ("product_id" = String, Path),
        ("limit" = Option<i64>, Query, description = "Maximum number of records")
    ),
    responses(
        (status = 200, description = "Audit trail", body = Vec<ComplianceAuditTrail>)
    ),
    tag = "regulatory"
)]
pub async fn get_audit_trail(
    State(service): State<Arc<RegulatoryService>>,
    Path(product_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ComplianceAuditTrail>>, AppError> {
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let audit = service.get_audit_trail(&product_id, limit).await?;
    Ok(Json(audit))
}

// Generate compliance report
#[utoipa::path(
    post,
    path = "/api/regulatory/reports",
    request_body = GenerateReportRequest,
    responses(
        (status = 201, description = "Report generated", body = ComplianceReport)
    ),
    tag = "regulatory"
)]
pub async fn generate_report(
    State(service): State<Arc<RegulatoryService>>,
    Json(req): Json<GenerateReportRequest>,
) -> Result<Json<ComplianceReport>, AppError> {
    let report = NewComplianceReport {
        report_type: req.report_type,
        scope: req.scope,
        generated_by: "system".to_string(), // TODO: Get from auth context
        period_start: req.period_start,
        period_end: req.period_end,
    };
    let result = service.generate_report(report).await?;
    Ok(Json(result))
}

// Get compliance report
#[utoipa::path(
    get,
    path = "/api/regulatory/reports/{report_id}",
    params(
        ("report_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Report found", body = ComplianceReport),
        (status = 404, description = "Report not found")
    ),
    tag = "regulatory"
)]
pub async fn get_report(
    State(service): State<Arc<RegulatoryService>>,
    Path(report_id): Path<String>,
) -> Result<Json<ComplianceReport>, AppError> {
    let report = service.get_report(&report_id).await?
        .ok_or_else(|| AppError::NotFound("Report not found".to_string()))?;
    Ok(Json(report))
}

// List compliance reports
#[utoipa::path(
    get,
    path = "/api/regulatory/reports",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of records")
    ),
    responses(
        (status = 200, description = "List of reports", body = Vec<ComplianceReport>)
    ),
    tag = "regulatory"
)]
pub async fn list_reports(
    State(service): State<Arc<RegulatoryService>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ComplianceReport>>, AppError> {
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let reports = service.list_reports(limit).await?;
    Ok(Json(reports))
}

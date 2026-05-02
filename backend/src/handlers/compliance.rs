use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use crate::{AppState, error::AppError};
use crate::compliance::{ComplianceValidator, ComplianceRule, ComplianceType};
use crate::services::audit_service::AuditLogQuery;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComplianceCheckRequest {
    pub compliance_type: String,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComplianceReportResponse {
    pub product_id: String,
    pub compliance_checks: Vec<Value>,
    pub overall_status: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/compliance/check",
    tag = "compliance",
    request_body = ComplianceCheckRequest,
    responses(
        (status = 200, description = "Compliance check completed successfully"),
        (status = 400, description = "Bad request - unknown compliance type"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn check_compliance(
    State(_state): State<AppState>,
    Json(req): Json<ComplianceCheckRequest>,
) -> impl IntoResponse {
    let compliance_type = match req.compliance_type.as_str() {
        "gdpr" => ComplianceType::GDPR,
        "fda_21_cfr_11" => ComplianceType::FDA21CFR11,
        "fsma" => ComplianceType::FSMA,
        "conflict_minerals" => ComplianceType::ConflictMinerals,
        "organic_certification" => ComplianceType::OrganicCertification,
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Unknown compliance type"}))).into_response(),
    };

    let rule = match compliance_type {
        ComplianceType::GDPR => ComplianceRule::gdpr_data_residency(),
        ComplianceType::FDA21CFR11 => ComplianceRule::fda_electronic_signature(),
        ComplianceType::FSMA => ComplianceRule::fsma_traceability(),
        ComplianceType::ConflictMinerals => ComplianceRule::conflict_minerals_due_diligence(),
        ComplianceType::OrganicCertification => ComplianceRule::organic_certification(),
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Unknown compliance type"}))).into_response(),
    };

    let result = ComplianceValidator::validate(&rule, &req.data);

    (StatusCode::OK, Json(serde_json::json!({
        "is_compliant": result.is_compliant,
        "compliance_type": result.compliance_type.as_str(),
        "violations": result.violations,
        "warnings": result.warnings,
    }))).into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/compliance/report/{product_id}",
    tag = "compliance",
    params(
        ("product_id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Compliance report retrieved successfully", body = ComplianceReportResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_compliance_report(
    State(_state): State<AppState>,
    Path(product_id): Path<String>,
) -> impl IntoResponse {
    // The current backend does not yet persist compliance checks.
    // Returning a structured placeholder keeps the API stable while storage is implemented.
    let report = ComplianceReportResponse {
        product_id,
        compliance_checks: vec![],
        overall_status: "not_implemented".to_string(),
    };

    (StatusCode::NOT_IMPLEMENTED, Json(report)).into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/report",
    tag = "compliance",
    responses(
        (status = 200, description = "Audit report generated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn generate_audit_report(
    State(state): State<AppState>,
    Query(filters): Query<AuditLogQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = filters.limit.unwrap_or(100).clamp(1, 500);
    let offset = filters.offset.unwrap_or(0).max(0);
    let events = state.audit_service.query(filters).await?;

    let report = serde_json::json!({
        "report_type": "audit",
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "limit": limit,
        "offset": offset,
        "returned_events": events.len(),
        "events": events
    });

    Ok(Json(report))
}

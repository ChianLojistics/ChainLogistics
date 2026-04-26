use axum::{
    extract::{Path, State},
    Json,
};
use crate::AppState;
use crate::error::AppError;
use crate::models::sustainability::{
    IotReading, SustainabilityMetric, SustainabilityVerification, 
    AddIotReadingRequest, VerifyMetricRequest, SustainabilityReport, GenerateReportRequest
};

/// POST /api/v1/sustainability/iot
#[utoipa::path(
    post,
    path = "/api/v1/sustainability/iot",
    tag = "sustainability",
    request_body = AddIotReadingRequest,
    responses(
        (status = 200, description = "IoT reading added", body = IotReading),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn add_iot_reading(
    State(state): State<AppState>,
    Json(req): Json<AddIotReadingRequest>,
) -> Result<Json<IotReading>, AppError> {
    let reading = state.sustainability_service.add_iot_reading(req).await?;
    Ok(Json(reading))
}

/// GET /api/v1/sustainability/:product_id
#[utoipa::path(
    get,
    path = "/api/v1/sustainability/{product_id}",
    tag = "sustainability",
    params(
        ("product_id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Sustainability metrics retrieved", body = [SustainabilityMetric]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_product_sustainability(
    Path(product_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<SustainabilityMetric>>, AppError> {
    let metrics = state.sustainability_service.get_product_sustainability(&product_id).await?;
    Ok(Json(metrics))
}

/// POST /api/v1/sustainability/:product_id/verify
#[utoipa::path(
    post,
    path = "/api/v1/sustainability/{product_id}/verify",
    tag = "sustainability",
    params(
        ("product_id" = String, Path, description = "Product ID")
    ),
    request_body = VerifyMetricRequest,
    responses(
        (status = 200, description = "Metric verified", body = SustainabilityVerification),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn verify_metric(
    Path(product_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<VerifyMetricRequest>,
) -> Result<Json<SustainabilityVerification>, AppError> {
    let verification = state.sustainability_service.verify_metric(&product_id, req).await?;
    Ok(Json(verification))
}

/// POST /api/v1/sustainability/:product_id/report
#[utoipa::path(
    post,
    path = "/api/v1/sustainability/{product_id}/report",
    tag = "sustainability",
    params(
        ("product_id" = String, Path, description = "Product ID")
    ),
    request_body = GenerateReportRequest,
    responses(
        (status = 200, description = "Report generated", body = SustainabilityReport),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn generate_report(
    Path(product_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<GenerateReportRequest>,
) -> Result<Json<SustainabilityReport>, AppError> {
    let report = state.sustainability_service.generate_report(&product_id, req).await?;
    Ok(Json(report))
}

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::iot::*;
use crate::services::IoTService;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListDevicesQuery {
    pub product_id: Option<String>,
    pub device_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetReadingsQuery {
    pub device_id: Option<String>,
    pub product_id: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetAlertsQuery {
    pub product_id: Option<String>,
    pub is_resolved: Option<bool>,
    pub limit: Option<i64>,
}

// Create IoT device
#[utoipa::path(
    post,
    path = "/api/iot/devices",
    request_body = NewIoTDevice,
    responses(
        (status = 201, description = "Device created", body = IoTDevice),
        (status = 400, description = "Invalid request")
    ),
    tag = "iot"
)]
pub async fn create_device(
    State(service): State<Arc<IoTService>>,
    Json(req): Json<NewIoTDevice>,
) -> Result<Json<IoTDevice>, AppError> {
    let device = service.create_device(req).await?;
    Ok(Json(device))
}

// Get IoT device
#[utoipa::path(
    get,
    path = "/api/iot/devices/{device_id}",
    params(
        ("device_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Device found", body = IoTDevice),
        (status = 404, description = "Device not found")
    ),
    tag = "iot"
)]
pub async fn get_device(
    State(service): State<Arc<IoTService>>,
    Path(device_id): Path<String>,
) -> Result<Json<IoTDevice>, AppError> {
    let device = service.get_device(&device_id).await?
        .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;
    Ok(Json(device))
}

// List IoT devices
#[utoipa::path(
    get,
    path = "/api/iot/devices",
    params(ListDevicesQuery),
    responses(
        (status = 200, description = "List of devices", body = Vec<IoTDevice>)
    ),
    tag = "iot"
)]
pub async fn list_devices(
    State(service): State<Arc<IoTService>>,
    Query(query): Query<ListDevicesQuery>,
) -> Result<Json<Vec<IoTDevice>>, AppError> {
    let devices = service.list_devices(
        query.product_id,
        query.device_type,
        query.is_active,
    ).await?;
    Ok(Json(devices))
}

// Create temperature reading
#[utoipa::path(
    post,
    path = "/api/iot/readings",
    request_body = NewTemperatureReading,
    responses(
        (status = 201, description = "Reading created", body = TemperatureReading)
    ),
    tag = "iot"
)]
pub async fn create_reading(
    State(service): State<Arc<IoTService>>,
    Json(req): Json<NewTemperatureReading>,
) -> Result<Json<TemperatureReading>, AppError> {
    let reading = service.create_reading(req).await?;
    Ok(Json(reading))
}

// Get temperature readings
#[utoipa::path(
    get,
    path = "/api/iot/readings",
    params(GetReadingsQuery),
    responses(
        (status = 200, description = "List of readings", body = Vec<TemperatureReading>)
    ),
    tag = "iot"
)]
pub async fn get_readings(
    State(service): State<Arc<IoTService>>,
    Query(query): Query<GetReadingsQuery>,
) -> Result<Json<Vec<TemperatureReading>>, AppError> {
    let limit = query.limit.unwrap_or(100);
    let readings = service.get_readings(
        query.device_id,
        query.product_id,
        query.start_time,
        query.end_time,
        limit,
    ).await?;
    Ok(Json(readings))
}

// Create temperature threshold
#[utoipa::path(
    post,
    path = "/api/iot/thresholds",
    request_body = NewTemperatureThreshold,
    responses(
        (status = 201, description = "Threshold created", body = TemperatureThreshold)
    ),
    tag = "iot"
)]
pub async fn create_threshold(
    State(service): State<Arc<IoTService>>,
    Json(req): Json<NewTemperatureThreshold>,
) -> Result<Json<TemperatureThreshold>, AppError> {
    let threshold = service.create_threshold(req).await?;
    Ok(Json(threshold))
}

// Get temperature thresholds
#[utoipa::path(
    get,
    path = "/api/iot/thresholds/{product_id}",
    params(
        ("product_id" = String, Path)
    ),
    responses(
        (status = 200, description = "List of thresholds", body = Vec<TemperatureThreshold>)
    ),
    tag = "iot"
)]
pub async fn get_thresholds(
    State(service): State<Arc<IoTService>>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<TemperatureThreshold>>, AppError> {
    let thresholds = service.get_thresholds(&product_id).await?;
    Ok(Json(thresholds))
}

// Get temperature alerts
#[utoipa::path(
    get,
    path = "/api/iot/alerts",
    params(GetAlertsQuery),
    responses(
        (status = 200, description = "List of alerts", body = Vec<TemperatureAlert>)
    ),
    tag = "iot"
)]
pub async fn get_alerts(
    State(service): State<Arc<IoTService>>,
    Query(query): Query<GetAlertsQuery>,
) -> Result<Json<Vec<TemperatureAlert>>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let alerts = service.get_alerts(
        query.product_id,
        query.is_resolved,
        limit,
    ).await?;
    Ok(Json(alerts))
}

// Acknowledge alert
#[utoipa::path(
    post,
    path = "/api/iot/alerts/{alert_id}/acknowledge",
    params(
        ("alert_id" = Uuid, Path)
    ),
    responses(
        (status = 200, description = "Alert acknowledged", body = TemperatureAlert)
    ),
    tag = "iot"
)]
pub async fn acknowledge_alert(
    State(service): State<Arc<IoTService>>,
    Path(alert_id): Path<Uuid>,
) -> Result<Json<TemperatureAlert>, AppError> {
    let alert = service.acknowledge_alert(alert_id, "system".to_string()).await?;
    Ok(Json(alert))
}

// Resolve alert
#[utoipa::path(
    post,
    path = "/api/iot/alerts/{alert_id}/resolve",
    params(
        ("alert_id" = Uuid, Path)
    ),
    responses(
        (status = 200, description = "Alert resolved", body = TemperatureAlert)
    ),
    tag = "iot"
)]
pub async fn resolve_alert(
    State(service): State<Arc<IoTService>>,
    Path(alert_id): Path<Uuid>,
) -> Result<Json<TemperatureAlert>, AppError> {
    let alert = service.resolve_alert(alert_id, "system".to_string()).await?;
    Ok(Json(alert))
}

// Get temperature summaries
#[utoipa::path(
    get,
    path = "/api/iot/summaries/{device_id}",
    params(
        ("device_id" = String, Path),
        ("summary_period" = Option<String>, Query),
        ("limit" = Option<i64>, Query)
    ),
    responses(
        (status = 200, description = "List of summaries", body = Vec<TemperatureSummary>)
    ),
    tag = "iot"
)]
pub async fn get_summaries(
    State(service): State<Arc<IoTService>>,
    Path(device_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TemperatureSummary>>, AppError> {
    let summary_period = params.get("summary_period").cloned();
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let summaries = service.get_summaries(&device_id, summary_period.as_deref(), limit).await?;
    Ok(Json(summaries))
}

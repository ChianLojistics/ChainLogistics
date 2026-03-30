use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::{Duration, Utc};
use serde_json::json;

use crate::{
    error::AppError,
    models::analytics::{ExportQuery, TimeSeriesQuery},
    AppState,
};

/// GET /api/v1/analytics/dashboard
pub async fn dashboard(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let metrics = state.analytics_service.get_dashboard_metrics().await?;
    Ok(Json(json!(metrics)))
}

/// GET /api/v1/analytics/products/:id
pub async fn product_analytics(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let analytics = state
        .analytics_service
        .get_product_analytics(&product_id)
        .await?;
    Ok(Json(json!(analytics)))
}

/// GET /api/v1/analytics/events?start_date=&end_date=&event_type=
pub async fn event_analytics(
    State(state): State<AppState>,
    Query(params): Query<TimeSeriesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let end = params.end_date.unwrap_or_else(Utc::now);
    let start = params.start_date.unwrap_or_else(|| end - Duration::days(30));

    let analytics = state
        .analytics_service
        .get_event_analytics(start, end, params.event_type.as_deref())
        .await?;
    Ok(Json(json!(analytics)))
}

/// GET /api/v1/analytics/users
pub async fn user_analytics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let analytics = state.analytics_service.get_user_analytics().await?;
    Ok(Json(json!(analytics)))
}

/// GET /api/v1/analytics/export?format=csv&start_date=&end_date=&product_id=&limit=
pub async fn export(
    State(state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let end = params.end_date.unwrap_or_else(Utc::now);
    let start = params.start_date.unwrap_or_else(|| end - Duration::days(30));
    let limit = params.limit.unwrap_or(10_000).min(50_000);
    let format = params.format.as_deref().unwrap_or("json");

    match format {
        "csv" => {
            let csv = state
                .analytics_service
                .export_events_csv(start, end, params.product_id.as_deref(), limit)
                .await?;

            Ok((
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"events_export.csv\"",
                    ),
                ],
                csv,
            )
                .into_response())
        }
        _ => {
            // JSON export — reuse event analytics with full data
            let analytics = state
                .analytics_service
                .get_event_analytics(start, end, params.event_type.as_deref())
                .await?;
            Ok(Json(json!(analytics)).into_response())
        }
    }
}

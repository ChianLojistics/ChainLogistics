use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;

use crate::{
    AppState,
    error::AppError,
    models::{TrackingEvent, NewTrackingEvent},
    validation::{validate_string, validate_stellar_address, sanitize_input, validate_product_id, validate_location, sanitize_json_metadata},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListEventsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    #[serde(alias = "productId")]
    pub product_id: Option<String>,
    #[serde(alias = "eventType")]
    pub event_type: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEventRequest {
    #[serde(alias = "productId")]
    pub product_id: String,
    #[serde(alias = "actorAddress")]
    pub actor_address: String,
    #[serde(deserialize_with = "deserialize_flexible_timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(alias = "eventType")]
    pub event_type: String,
    pub location: String,
    #[serde(alias = "dataHash")]
    pub data_hash: String,
    #[serde(default)]
    pub note: String,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

fn deserialize_flexible_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(n) => {
            let raw = n
                .as_i64()
                .ok_or_else(|| serde::de::Error::custom("timestamp must be an integer"))?;
            let seconds = if raw > 10_000_000_000 { raw / 1000 } else { raw };
            Utc.timestamp_opt(seconds, 0)
                .single()
                .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))
        }
        serde_json::Value::String(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| serde::de::Error::custom("timestamp must be RFC3339 or unix time")),
        _ => Err(serde::de::Error::custom(
            "timestamp must be RFC3339 string or unix number",
        )),
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
    pub id: i64,
    pub product_id: String,
    pub actor_address: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub location: String,
    pub data_hash: String,
    pub note: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedEventsResponse {
    pub events: Vec<EventResponse>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

impl From<TrackingEvent> for EventResponse {
    fn from(event: TrackingEvent) -> Self {
        Self {
            id: event.id,
            product_id: event.product_id,
            actor_address: event.actor_address,
            timestamp: event.timestamp,
            event_type: event.event_type,
            location: event.location,
            data_hash: event.data_hash,
            note: event.note,
            metadata: event.metadata,
            created_at: event.created_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/events",
    tag = "events",
    params(ListEventsQuery),
    responses(
        (status = 200, description = "Events listed successfully", body = PaginatedEventsResponse),
        (status = 400, description = "Bad request - product_id is required"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<PaginatedEventsResponse>, AppError> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100);

    let (events, total) = if let Some(product_id) = query.product_id {
        validate_product_id(&product_id)?;
        let sanitized_product_id = sanitize_input(&product_id);

        let events = if let Some(event_type) = query.event_type {
            validate_string("event_type", &event_type, 64)?;
            state.event_service
                .list_events_by_type(&sanitized_product_id, &sanitize_input(&event_type), offset, limit)
                .await?
        } else {
            state.event_service
                .list_events_by_product(&sanitized_product_id, offset, limit)
                .await?
        };

        let total = if query.event_type.is_some() {
            events.len() as i64
        } else {
            state.event_service
                .count_events_by_product(&sanitized_product_id)
                .await?
        };

        (events, total)
    } else {
        return Err(AppError::BadRequest("product_id is required".to_string()));
    };

    Ok(Json(PaginatedEventsResponse {
        events: events.into_iter().map(EventResponse::from).collect(),
        total,
        offset,
        limit,
    }))
}

const ALLOWED_EVENT_TYPES: &[&str] = &[
    "HARVEST", "PROCESS", "PACKAGE", "SHIP", "RECEIVE",
    "QUALITY_CHECK", "TRANSFER", "REGISTER", "CHECKPOINT",
];

#[utoipa::path(
    post,
    path = "/api/v1/admin/events",
    tag = "events",
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Event created successfully", body = EventResponse),
        (status = 400, description = "Bad request - invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_event(
    State(state): State<AppState>,
    Json(request): Json<CreateEventRequest>,
) -> Result<Json<EventResponse>, AppError> {
    // Validate inputs
    validate_product_id(&request.product_id)?;
    validate_stellar_address(&request.actor_address)?;
    validate_location(&request.location)?;
    if request.note.len() > 256 {
        return Err(AppError::Validation("note must not exceed 256 characters".to_string()));
    }
    if !ALLOWED_EVENT_TYPES.contains(&request.event_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid event_type '{}'. Allowed: {}",
            request.event_type,
            ALLOWED_EVENT_TYPES.join(", ")
        )));
    }
    // Reject future timestamps
    if request.timestamp > chrono::Utc::now() {
        return Err(AppError::Validation("timestamp must not be in the future".to_string()));
    }

    let new_event = NewTrackingEvent {
        product_id: sanitize_input(&request.product_id),
        actor_address: request.actor_address,
        timestamp: request.timestamp,
        event_type: request.event_type,
        location: sanitize_input(&request.location),
        data_hash: request.data_hash,
        note: sanitize_input(&request.note),
        metadata: {
            let mut meta = request.metadata;
            sanitize_json_metadata(&mut meta);
            meta
        },
    };

    let event = state.event_service.create_event(new_event).await?;
    Ok(Json(EventResponse::from(event)))
}

#[utoipa::path(
    get,
    path = "/api/v1/events/{id}",
    tag = "events",
    params(
        ("id" = i64, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Event retrieved successfully", body = EventResponse),
        (status = 404, description = "Event not found"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<EventResponse>, AppError> {
    let event = state
        .event_service
        .get_event(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Event {} not found", id)))?;

    Ok(Json(EventResponse::from(event)))
}

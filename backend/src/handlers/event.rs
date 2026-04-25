use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    error::AppError,
    models::{TrackingEvent, NewTrackingEvent},
    validation::{validate_string, validate_stellar_address, sanitize_input},
};

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub product_id: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub product_id: String,
    pub actor_address: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub location: String,
    pub data_hash: String,
    pub note: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

pub async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<PaginatedEventsResponse>, AppError> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100);

    let (events, total) = if let Some(product_id) = query.product_id {
        validate_string("product_id", &product_id, 64)?;
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

pub async fn create_event(
    State(state): State<AppState>,
    Json(request): Json<CreateEventRequest>,
) -> Result<Json<EventResponse>, AppError> {
    // Validate inputs
    validate_string("product_id", &request.product_id, 64)?;
    validate_stellar_address(&request.actor_address)?;
    validate_string("location", &request.location, 256)?;
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
        metadata: request.metadata,
    };

    let event = state.event_service.create_event(new_event).await?;
    Ok(Json(EventResponse::from(event)))
}

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

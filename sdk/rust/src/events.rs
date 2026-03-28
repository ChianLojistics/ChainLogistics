use crate::{
    client::HttpClient,
    models::{TrackingEvent, NewTrackingEvent, EventListQuery, PaginationMeta},
    Config, Result,
};

/// Service for managing tracking events
#[derive(Debug, Clone)]
pub struct EventsService {
    client: HttpClient,
}

impl EventsService {
    pub(crate) fn new(client: reqwest::Client, config: Config) -> Self {
        Self {
            client: HttpClient::new(client, config),
        }
    }

    /// List events with optional filtering
    pub async fn list(&self, query: Option<EventListQuery>) -> Result<(Vec<TrackingEvent>, PaginationMeta)> {
        let mut request = self.client.get("api/v1/events");

        if let Some(q) = query {
            if let Some(offset) = q.offset {
                request = request.query(&[("offset", offset)]);
            }
            if let Some(limit) = q.limit {
                request = request.query(&[("limit", limit)]);
            }
            if let Some(product_id) = q.product_id {
                request = request.query(&[("product_id", product_id)]);
            }
            if let Some(event_type) = q.event_type {
                request = request.query(&[("event_type", event_type)]);
            }
        } else {
            // product_id is required for event listing
            return Err(crate::Error::Validation("product_id is required for event listing".to_string()));
        }

        #[derive(serde::Deserialize)]
        struct EventListResponse {
            events: Vec<TrackingEvent>,
            total: i64,
            offset: i64,
            limit: i64,
        }

        let response: EventListResponse = self.client.execute(request).await?;
        let pagination = PaginationMeta {
            total: response.total,
            offset: response.offset,
            limit: response.limit,
        };

        Ok((response.events, pagination))
    }

    /// Get a specific event by ID
    pub async fn get(&self, id: i64) -> Result<TrackingEvent> {
        let request = self.client.get(&format!("api/v1/events/{}", id));
        self.client.execute(request).await
    }

    /// Create a new tracking event
    pub async fn create(&self, event: &NewTrackingEvent) -> Result<TrackingEvent> {
        let request = self.client.post("api/v1/admin/events");
        self.client.execute_with_body(request, event).await
    }

    /// List events for a specific product
    pub async fn list_by_product(
        &self,
        product_id: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<TrackingEvent>, PaginationMeta)> {
        let query = EventListQuery {
            offset,
            limit,
            product_id: Some(product_id.to_string()),
            event_type: None,
        };
        self.list(Some(query)).await
    }

    /// List events for a specific product by event type
    pub async fn list_by_product_and_type(
        &self,
        product_id: &str,
        event_type: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<TrackingEvent>, PaginationMeta)> {
        let query = EventListQuery {
            offset,
            limit,
            product_id: Some(product_id.to_string()),
            event_type: Some(event_type.to_string()),
        };
        self.list(Some(query)).await
    }

    /// Get all events for a product (convenience method)
    pub async fn get_all_for_product(&self, product_id: &str) -> Result<Vec<TrackingEvent>> {
        let (events, _) = self.list_by_product(product_id, None, None).await?;
        Ok(events)
    }

    /// Get events of a specific type for a product
    pub async fn get_by_type_for_product(
        &self,
        product_id: &str,
        event_type: &str,
    ) -> Result<Vec<TrackingEvent>> {
        let (events, _) = self.list_by_product_and_type(product_id, event_type, None, None).await?;
        Ok(events)
    }
}

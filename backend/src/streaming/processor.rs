use crate::error::AppError;
use crate::models::{NewProduct, NewTrackingEvent};
use crate::streaming::mercury_client::MercuryEvent;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &MercuryEvent) -> Result<(), AppError>;
}

pub struct EventProcessor {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventProcessor {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    pub async fn process_event(&self, event: MercuryEvent) -> Result<(), AppError> {
        let start = std::time::Instant::now();
        
        tracing::debug!("Processing event: {} from contract: {}", 
            event.function_name, event.contract_id);

        for handler in &self.handlers {
            if let Err(e) = handler.handle(&event).await {
                tracing::error!("Handler failed for event {}: {}", event.id, e);
                return Err(e);
            }
        }

        let duration = start.elapsed();
        if duration.as_millis() > 500 {
            tracing::warn!("Event processing took {}ms (target: <500ms)", duration.as_millis());
        } else {
            tracing::debug!("Event processed in {}ms", duration.as_millis());
        }

        Ok(())
    }
}

pub struct ProductRegistrationHandler {
    product_tx: mpsc::UnboundedSender<NewProduct>,
}

impl ProductRegistrationHandler {
    pub fn new(product_tx: mpsc::UnboundedSender<NewProduct>) -> Self {
        Self { product_tx }
    }
}

#[async_trait]
impl EventHandler for ProductRegistrationHandler {
    async fn handle(&self, event: &MercuryEvent) -> Result<(), AppError> {
        if event.function_name != "register_product" {
            return Ok(());
        }

        let product = NewProduct {
            id: event.args["id"].as_str().ok_or(AppError::ValidationError(
                "Missing product id".to_string()
            ))?.to_string(),
            name: event.args["name"].as_str().ok_or(AppError::ValidationError(
                "Missing product name".to_string()
            ))?.to_string(),
            description: event.args["description"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            origin_location: event.args["origin"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            category: event.args["category"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            tags: event.args["tags"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }).unwrap_or_default(),
            certifications: event.args["certifications"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }).unwrap_or_default(),
            media_hashes: event.args["media_hashes"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }).unwrap_or_default(),
            custom_fields: event.args["custom_fields"].as_object()
                .map(|o| serde_json::Value::Object(o.clone()))
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            owner_address: event.args["owner"].as_str().ok_or(AppError::ValidationError(
                "Missing owner address".to_string()
            ))?.to_string(),
            created_by: event.args["owner"].as_str().map(|s| s.to_string()).unwrap_or_default(),
        };

        self.product_tx.send(product)
            .map_err(|e| AppError::StreamingError(format!("Failed to send product: {}", e)))?;

        Ok(())
    }
}

pub struct TrackingEventHandler {
    event_tx: mpsc::UnboundedSender<NewTrackingEvent>,
}

impl TrackingEventHandler {
    pub fn new(event_tx: mpsc::UnboundedSender<NewTrackingEvent>) -> Self {
        Self { event_tx }
    }
}

#[async_trait]
impl EventHandler for TrackingEventHandler {
    async fn handle(&self, event: &MercuryEvent) -> Result<(), AppError> {
        if event.function_name != "add_tracking_event" {
            return Ok(());
        }

        let tracking_event = NewTrackingEvent {
            product_id: event.args["product_id"].as_str().ok_or(AppError::ValidationError(
                "Missing product_id".to_string()
            ))?.to_string(),
            actor_address: event.args["actor"].as_str().ok_or(AppError::ValidationError(
                "Missing actor address".to_string()
            ))?.to_string(),
            timestamp: chrono::Utc::now(),
            event_type: event.args["event_type"].as_str().ok_or(AppError::ValidationError(
                "Missing event_type".to_string()
            ))?.to_string(),
            location: event.args["location"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            data_hash: event.args["data_hash"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            note: event.args["note"].as_str().map(|s| s.to_string()).unwrap_or_default(),
            metadata: event.args["metadata"].as_object()
                .map(|o| serde_json::Value::Object(o.clone()))
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        };

        self.event_tx.send(tracking_event)
            .map_err(|e| AppError::StreamingError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }
}

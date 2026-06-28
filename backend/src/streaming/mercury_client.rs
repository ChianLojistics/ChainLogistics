use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MercuryEvent {
    pub id: String,
    pub ledger: u32,
    pub timestamp: u64,
    pub transaction_hash: String,
    pub contract_id: String,
    pub function_name: String,
    pub args: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MercuryConfig {
    pub websocket_url: String,
    pub contract_ids: Vec<String>,
    pub reconnect_interval: Duration,
}

impl Default for MercuryConfig {
    fn default() -> Self {
        Self {
            websocket_url: "wss://stream.mercurydata.app/v1/stream".to_string(),
            contract_ids: vec![],
            reconnect_interval: Duration::from_secs(5),
        }
    }
}

pub struct MercuryClient {
    config: MercuryConfig,
    event_tx: mpsc::UnboundedSender<MercuryEvent>,
}

impl MercuryClient {
    pub fn new(config: MercuryConfig, event_tx: mpsc::UnboundedSender<MercuryEvent>) -> Self {
        Self { config, event_tx }
    }

    pub async fn start(&self) -> Result<(), AppError> {
        let mut retry_count = 0;
        let max_retries = 10;

        loop {
            match self.connect_and_stream().await {
                Ok(_) => {
                    retry_count = 0;
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(AppError::StreamingError(
                            format!("Max retries exceeded: {}", e)
                        ));
                    }
                    tracing::warn!("Mercury connection failed (attempt {}/{}): {}", 
                        retry_count, max_retries, e);
                    tokio::time::sleep(self.config.reconnect_interval).await;
                }
            }
        }
    }

    async fn connect_and_stream(&self) -> Result<(), AppError> {
        let url = &self.config.websocket_url;
        tracing::info!("Connecting to Mercury at: {}", url);

        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| AppError::StreamingError(format!("WebSocket connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to contract events
        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "contracts": self.config.contract_ids,
            "filters": {
                "functions": ["register_product", "add_tracking_event", "transfer_ownership"]
            }
        });

        write.send(Message::Text(subscribe_msg.to_string())).await
            .map_err(|e| AppError::StreamingError(format!("Failed to send subscribe message: {}", e)))?;

        tracing::info!("Subscribed to Mercury events for contracts: {:?}", self.config.contract_ids);

        // Process incoming events
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(mercury_event) = serde_json::from_str::<MercuryEvent>(&text) {
                        let _ = self.event_tx.send(mercury_event);
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = write.send(Message::Pong(data)).await;
                }
                Ok(Message::Close(_)) => {
                    tracing::warn!("Mercury connection closed");
                    return Err(AppError::StreamingError("Connection closed".to_string()));
                }
                Err(e) => {
                    tracing::error!("Mercury stream error: {}", e);
                    return Err(AppError::StreamingError(format!("Stream error: {}", e)));
                }
                _ => {}
            }
        }

        Ok(())
    }
}

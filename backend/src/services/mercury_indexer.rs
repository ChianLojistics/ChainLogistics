use crate::error::AppError;
use crate::models::TrackingEvent;
use async_trait::async_trait;
use chrono::Utc;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Mercury streaming indexer configuration
#[derive(Debug, Clone)]
pub struct MercuryConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub contract_id: String,
    pub stream_events: bool,
    pub batch_size: usize,
    pub poll_interval_ms: u64,
}

impl Default for MercuryConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            contract_id: std::env::var("CONTRACT_ID")
                .unwrap_or_else(|_| "CBUWSKT2UGOAXK4ZREVDJV5XHSYB42PZ3CERU2ZFUTUMAZLJEHNZIECA".to_string()),
            stream_events: true,
            batch_size: 100,
            poll_interval_ms: 100,
        }
    }
}

/// Stellar blockchain event from Mercury
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarEvent {
    pub event_id: String,
    pub contract_id: String,
    pub event_type: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
    pub transaction_hash: String,
}

/// Processed event ready for downstream handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEvent {
    pub stellar_event: StellarEvent,
    pub tracking_event: Option<TrackingEvent>,
    pub processing_metadata: ProcessingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub processed_at: chrono::DateTime<Utc>,
    pub processing_duration_ms: u64,
    pub source: String,
    pub retry_count: u32,
}

/// Event processor trait for custom processing logic
#[async_trait]
pub trait EventProcessor: Send + Sync {
    async fn process(&self, event: StellarEvent) -> Result<ProcessedEvent, AppError>;
}

/// Mercury streaming indexer service
pub struct MercuryIndexer {
    config: MercuryConfig,
    pool: PgPool,
    redis_client: redis::Client,
    processors: Vec<Arc<dyn EventProcessor>>,
    event_tx: mpsc::UnboundedSender<StellarEvent>,
    metrics: IndexerMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct IndexerMetrics {
    pub events_processed: u64,
    pub events_failed: u64,
    pub avg_processing_time_ms: u64,
    pub last_event_at: Option<chrono::DateTime<Utc>>,
}

impl MercuryIndexer {
    pub fn new(
        config: MercuryConfig,
        pool: PgPool,
        redis_client: redis::Client,
    ) -> (Self, mpsc::UnboundedReceiver<StellarEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let indexer = Self {
            config,
            pool,
            redis_client,
            processors: Vec::new(),
            event_tx,
            metrics: IndexerMetrics::default(),
        };

        (indexer, event_rx)
    }

    pub fn add_processor(&mut self, processor: Arc<dyn EventProcessor>) {
        self.processors.push(processor);
    }

    /// Start the streaming indexer
    pub async fn start(&self) -> Result<(), AppError> {
        info!("Starting Mercury indexer for contract: {}", self.config.contract_id);

        if self.config.stream_events {
            self.start_event_stream().await?;
        } else {
            self.start_polling().await?;
        }

        Ok(())
    }

    /// Stream events from Mercury (real-time)
    async fn start_event_stream(&self) -> Result<(), AppError> {
        info!("Starting Mercury event stream");

        let config = self.config.clone();
        let pool = self.pool.clone();
        let redis_client = self.redis_client.clone();
        let processors = self.processors.clone();

        tokio::spawn(async move {
            let mut last_ledger = self::get_last_ledger(&redis_client).await.unwrap_or(0);

            loop {
                match Self::fetch_events(&config, last_ledger).await {
                    Ok(events) => {
                        if !events.is_empty() {
                            info!("Fetched {} events from Mercury", events.len());

                            for event in events {
                                let ledger = event.timestamp;
                                last_ledger = ledger.max(last_ledger);

                                // Process through all registered processors
                                for processor in &processors {
                                    match processor.process(event.clone()).await {
                                        Ok(processed) => {
                                            // Store processed event
                                            if let Some(tracking_event) = processed.tracking_event {
                                                let _ = Self::store_tracking_event(
                                                    &pool,
                                                    &redis_client,
                                                    tracking_event,
                                                )
                                                .await;
                                            }
                                        }
                                        Err(e) => {
                                            error!("Event processing failed: {}", e);
                                            // Queue for retry
                                            let _ = Self::queue_retry(&redis_client, event).await;
                                        }
                                    }
                                }
                            }

                            let _ = self::update_last_ledger(&redis_client, last_ledger).await;
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch events: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            config.poll_interval_ms,
                        ))
                        .await;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(config.poll_interval_ms)).await;
            }
        });

        Ok(())
    }

    /// Poll for events (fallback mode)
    async fn start_polling(&self) -> Result<(), AppError> {
        warn!("Using polling mode instead of streaming");

        let config = self.config.clone();
        let pool = self.pool.clone();
        let redis_client = self.redis_client.clone();
        let processors = self.processors.clone();

        tokio::spawn(async move {
            loop {
                match Self::fetch_events(&config, 0).await {
                    Ok(events) => {
                        for event in events {
                            for processor in &processors {
                                let _ = processor.process(event.clone()).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Polling failed: {}", e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(config.poll_interval_ms)).await;
            }
        });

        Ok(())
    }

    /// Fetch events from Mercury indexer
    async fn fetch_events(config: &MercuryConfig, from_ledger: i64) -> Result<Vec<StellarEvent>, AppError> {
        // In production, this would connect to actual Mercury API
        // For now, simulating event fetch
        
        // TODO: Replace with actual Mercury API call
        // let client = reqwest::Client::new();
        // let response = client
        //     .post(&format!("{}/events", config.rpc_url))
        //     .json(&serde_json::json!({
        //         "contract_id": config.contract_id,
        //         "from_ledger": from_ledger,
        //         "limit": config.batch_size
        //     }))
        //     .send()
        //     .await?
        //     .json::<Vec<StellarEvent>>()
        //     .await?;

        // Simulated response for development
        Ok(vec![])
    }

    /// Store tracking event to database
    async fn store_tracking_event(
        pool: &PgPool,
        redis_client: &redis::Client,
        event: TrackingEvent,
    ) -> Result<(), AppError> {
        // Store in PostgreSQL
        sqlx::query(
            r#"
            INSERT INTO tracking_events (
                product_id, actor_address, timestamp, event_type,
                location, data_hash, note, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&event.product_id)
        .bind(&event.actor_address)
        .bind(event.timestamp)
        .bind(&event.event_type)
        .bind(&event.location)
        .bind(&event.data_hash)
        .bind(&event.note)
        .bind(&event.metadata)
        .execute(pool)
        .await?;

        // Invalidate cache
        if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
            let _: Result<(), _> = conn.del(format!("cache:events:{}", event.product_id)).await;
        }

        Ok(())
    }

    /// Queue event for retry processing
    async fn queue_retry(
        redis_client: &redis::Client,
        event: StellarEvent,
    ) -> Result<(), AppError> {
        if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
            let serialized = serde_json::to_string(&event)?;
            let _: Result<(), _> = conn.lpush("retry:events", serialized).await;
        }
        Ok(())
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> IndexerMetrics {
        self.metrics.clone()
    }
}

async fn get_last_ledger(redis_client: &redis::Client) -> Result<i64, AppError> {
    if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
        let ledger: Option<i64> = conn.get("indexer:last_ledger").await?;
        Ok(ledger.unwrap_or(0))
    } else {
        Ok(0)
    }
}

async fn update_last_ledger(redis_client: &redis::Client, ledger: i64) -> Result<(), AppError> {
    if let Ok(mut conn) = redis_client.get_multiplexed_tokio_connection().await {
        let _: Result<(), _> = conn.set("indexer:last_ledger", ledger).await;
    }
    Ok(())
}

/// Default event processor for tracking events
pub struct TrackingEventProcessor {
    pool: PgPool,
}

impl TrackingEventProcessor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventProcessor for TrackingEventProcessor {
    async fn process(&self, event: StellarEvent) -> Result<ProcessedEvent, AppError> {
        let start = std::time::Instant::now();

        // Convert Stellar event to tracking event
        let tracking_event = if event.event_type == "tracking" {
            Some(TrackingEvent {
                id: 0, // Will be assigned by database
                product_id: event
                    .data
                    .get("product_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                actor_address: event
                    .data
                    .get("actor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                timestamp: chrono::DateTime::from_timestamp(event.timestamp, 0)
                    .unwrap_or_else(Utc::now),
                event_type: event
                    .data
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN")
                    .to_string(),
                location: event
                    .data
                    .get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data_hash: event.transaction_hash.clone(),
                note: event
                    .data
                    .get("note")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                metadata: event.data.clone(),
                created_at: Utc::now(),
            })
        } else {
            None
        };

        Ok(ProcessedEvent {
            stellar_event: event,
            tracking_event,
            processing_metadata: ProcessingMetadata {
                processed_at: Utc::now(),
                processing_duration_ms: start.elapsed().as_millis() as u64,
                source: "mercury_indexer".to_string(),
                retry_count: 0,
            },
        })
    }
}

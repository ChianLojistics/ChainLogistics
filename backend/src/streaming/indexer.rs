use crate::streaming::mercury_client::{MercuryClient, MercuryConfig};
use crate::streaming::processor::{EventProcessor, ProductRegistrationHandler, TrackingEventHandler};
use crate::models::{NewProduct, NewTrackingEvent};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct StreamIndexer {
    mercury_client: Arc<MercuryClient>,
    processor: Arc<EventProcessor>,
    product_tx: mpsc::UnboundedSender<NewProduct>,
    event_tx: mpsc::UnboundedSender<NewTrackingEvent>,
    _handle: JoinHandle<()>,
}

impl StreamIndexer {
    pub fn new(
        config: MercuryConfig,
        product_tx: mpsc::UnboundedSender<NewProduct>,
        event_tx: mpsc::UnboundedSender<NewTrackingEvent>,
    ) -> Self {
        let (mercury_event_tx, mut mercury_event_rx) = mpsc::unbounded_channel();

        let mercury_client = Arc::new(MercuryClient::new(config.clone(), mercury_event_tx));
        
        let mut processor = EventProcessor::new();
        processor.register_handler(Arc::new(ProductRegistrationHandler::new(product_tx.clone())));
        processor.register_handler(Arc::new(TrackingEventHandler::new(event_tx.clone())));
        let processor = Arc::new(processor);

        let client_clone = mercury_client.clone();
        let processor_clone = processor.clone();
        
        let handle = tokio::spawn(async move {
            // Start Mercury client
            let client_task = tokio::spawn(async move {
                if let Err(e) = client_clone.start().await {
                    tracing::error!("Mercury client error: {}", e);
                }
            });

            // Process events
            let process_task = tokio::spawn(async move {
                while let Some(event) = mercury_event_rx.recv().await {
                    if let Err(e) = processor_clone.process_event(event).await {
                        tracing::error!("Failed to process event: {}", e);
                    }
                }
            });

            // Wait for both tasks
            let _ = tokio::try_join!(client_task, process_task);
        });

        Self {
            mercury_client,
            processor,
            product_tx,
            event_tx,
            _handle: handle,
        }
    }

    pub fn is_running(&self) -> bool {
        !self._handle.is_finished()
    }

    pub async fn shutdown(self) {
        self._handle.abort();
    }
}

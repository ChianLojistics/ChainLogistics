use super::event::EventService;
use super::product::ProductService;
use crate::database::{EventRepository, ProductRepository};
use crate::models::{NewProduct, NewTrackingEvent, Product, TrackingEvent};
use sqlx::PgPool;

pub struct SyncService {
    pool: PgPool,
    redis_client: redis::Client,
    product_service: ProductService,
    event_service: EventService,
}

impl SyncService {
    pub fn new(pool: PgPool, redis_client: redis::Client) -> Self {
        Self {
            pool: pool.clone(),
            redis_client: redis_client.clone(),
            product_service: ProductService::new(pool.clone(), redis_client.clone()),
            event_service: EventService::new(pool, redis_client),
        }
    }

    pub async fn sync_product_from_contract(
        &self,
        product: NewProduct,
    ) -> Result<Product, sqlx::Error> {
        let existing = self.product_service.get_product(&product.id).await?;

        if let Some(mut existing_product) = existing {
            existing_product.name = product.name.clone();
            existing_product.description = product.description.clone();
            existing_product.origin_location = product.origin_location.clone();
            existing_product.category = product.category.clone();
            existing_product.tags = product.tags.clone();
            existing_product.certifications = product.certifications.clone();
            existing_product.media_hashes = product.media_hashes.clone();
            existing_product.custom_fields = product.custom_fields.clone();
            existing_product.owner_address = product.owner_address.clone();
            existing_product.updated_by = product.created_by.clone();

            self.product_service
                .update_product(&product.id, existing_product)
                .await
        } else {
            self.product_service.create_product(product).await
        }
    }

    pub async fn sync_event_from_contract(
        &self,
        event: NewTrackingEvent,
    ) -> Result<TrackingEvent, sqlx::Error> {
        self.event_service.create_event(event).await
    }

    pub async fn sync_batch_products(
        &self,
        products: Vec<NewProduct>,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let mut results = Vec::new();
        for product in products {
            results.push(self.sync_product_from_contract(product).await?);
        }
        Ok(results)
    }

    pub async fn sync_batch_events(
        &self,
        events: Vec<NewTrackingEvent>,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        let mut results = Vec::new();
        for event in events {
            results.push(self.sync_event_from_contract(event).await?);
        }
        Ok(results)
    }
}

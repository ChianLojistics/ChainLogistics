"""use sqlx::PgPool;

use crate::error::AppError;
use crate::models::resilience::*;
use crate::models::product::Product;
use reqwest::Client;
use serde_json::Value;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

pub struct ResilienceService {
    pool: PgPool,
    http_client: Client,
}

impl ResilienceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool, http_client: Client::new() }
    }

    pub async fn get_resilience_metrics(&self, product_id: &str) -> Result<ResilienceMetrics, AppError> {
        let product = self.get_product_details(product_id).await?;

        let news_data = self.get_news_data(&product.name).await?;
        let weather_data = self.get_weather_data(product.latitude, product.longitude).await?;
        let political_risk_data = self.get_political_risk_data(&product.country_code).await?;

        let predictions = self.generate_disruption_predictions(&news_data, &weather_data, &political_risk_data).await?;
        let suppliers = self.assess_supplier_risks(&product).await?;
        let locations = self.assess_geographic_risks(&product, &weather_data, &political_risk_data).await?;
        let alternatives = self.identify_alternative_sources(&product).await?;
        let inventory = self.recommend_inventory_levels(&product, &predictions).await?;

        Ok(ResilienceMetrics {
            disruption_predictions: predictions,
            supplier_risks: suppliers,
            geographic_risks: locations,
            alternative_sources: alternatives,
            inventory_recommendations: inventory,
        })
    }

    async fn get_disruption_predictions(&self, product_id: &str) -> Result<Vec<DisruptionPrediction>, AppError> {
        let rows = sqlx::query_as!(DisruptionPrediction, "SELECT * FROM disruption_predictions WHERE product_id = $1", product_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_supplier_risks(&self, _product_id: &str) -> Result<Vec<SupplierRisk>, AppError> {
        let rows = sqlx::query_as!(SupplierRisk, "SELECT * FROM supplier_risks")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_geographic_risks(&self, _product_id: &str) -> Result<Vec<GeographicRisk>, AppError> {
        let rows = sqlx::query_as!(GeographicRisk, "SELECT * FROM geographic_risks")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_alternative_sources(&self, product_id: &str) -> Result<Vec<AlternativeSource>, AppError> {
        let rows = sqlx::query_as!(AlternativeSource, "SELECT * FROM alternative_sources WHERE product_id = $1", product_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_inventory_recommendations(&self, product_id: &str) -> Result<Vec<InventoryRecommendation>, AppError> {
        let rows = sqlx::query_as!(InventoryRecommendation, "SELECT * FROM inventory_recommendations WHERE product_id = $1", product_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get_external_data(&self, url: &str) -> Result<Value, AppError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);

        let response = Retry::spawn(retry_strategy, || async {
            self.http_client.get(url).send().await
        })
        .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }

    async fn get_news_data(&self, product_name: &str) -> Result<Value, AppError> {
        let url = format!("https://newsapi.ai/api/v1/article/getArticles?query={}&apiKey={}", product_name, "YOUR_NEWSAPI_KEY");
        self.get_external_data(&url).await
    }

    async fn get_product_details(&self, product_id: &str) -> Result<Product, AppError> {
        let product = sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", product_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(product)
    }

    async fn generate_disruption_predictions(&self, _news_data: &Value, _weather_data: &Value, _political_risk_data: &Value) -> Result<Vec<DisruptionPrediction>, AppError> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn recommend_inventory_levels(&self, _product: &Product, _predictions: &[DisruptionPrediction]) -> Result<Vec<InventoryRecommendation>, AppError> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn identify_alternative_sources(&self, _product: &Product) -> Result<Vec<AlternativeSource>, AppError> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn assess_geographic_risks(&self, _product: &Product, _weather_data: &Value, _political_risk_data: &Value) -> Result<Vec<GeographicRisk>, AppError> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn assess_supplier_risks(&self, _product: &Product) -> Result<Vec<SupplierRisk>, AppError> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn get_political_risk_data(&self, country_code: &str) -> Result<Value, AppError> {
        let url = format!("https://api.prsgroup.com/v2/country/{}?api_key={}", country_code, "YOUR_PRSGROUP_KEY");
        self.get_external_data(&url).await
    }

    async fn get_weather_data(&self, latitude: f64, longitude: f64) -> Result<Value, AppError> {
        let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,relativehumidity_2m,precipitation,windspeed_10m", latitude, longitude);
        self.get_external_data(&url).await
    }
"""
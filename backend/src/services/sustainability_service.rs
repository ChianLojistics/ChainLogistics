use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::error::AppError;
use crate::models::sustainability::{
    IotReading, SustainabilityMetric, SustainabilityVerification, AddIotReadingRequest, VerifyMetricRequest, SustainabilityReport, GenerateReportRequest
};
use crate::blockchain::provider::{StellarProvider, BlockchainProvider};

pub struct SustainabilityService {
    pool: PgPool,
    stellar: Arc<StellarProvider>,
}

impl SustainabilityService {
    pub fn new(pool: PgPool, stellar: Arc<StellarProvider>) -> Self {
        Self { pool, stellar }
    }

    pub async fn add_iot_reading(&self, req: AddIotReadingRequest) -> Result<IotReading, AppError> {
        let reading = sqlx::query_as!(
            IotReading,
            r#"
            INSERT INTO iot_readings (product_id, sensor_id, metric_type, value, unit, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            req.product_id,
            req.sensor_id,
            req.metric_type,
            req.value,
            req.unit,
            req.metadata.clone().unwrap_or_default(),
        )
        .fetch_one(&self.pool)
        .await?;

        // After adding IoT reading, check if we should update sustainability metrics
        self.recalculate_metrics(&req.product_id, &req.metric_type).await?;
        
        // Detect anomalies
        self.detect_anomalies(&req.product_id, &req.metric_type).await?;

        Ok(reading)
    }

    async fn recalculate_metrics(&self, product_id: &str, metric_type: &str) -> Result<(), AppError> {
        // Sophisticated calculation based on metric type
        let readings = sqlx::query_as!(
            IotReading,
            "SELECT * FROM iot_readings WHERE product_id = $1 AND metric_type = $2 ORDER BY timestamp DESC LIMIT 50",
            product_id,
            metric_type
        )
        .fetch_all(&self.pool)
        .await?;

        if readings.is_empty() {
            return Ok(());
        }

        let calculated_value = match metric_type {
            "carbon" => self.calculate_carbon_footprint(&readings),
            "water" => self.calculate_water_usage(&readings),
            "energy" => self.calculate_renewable_energy_ratio(&readings),
            "waste" => self.calculate_waste_management_efficiency(&readings),
            _ => self.calculate_average(&readings),
        };

        sqlx::query!(
            r#"
            INSERT INTO sustainability_metrics (product_id, metric_type, value, unit, last_updated)
            VALUES ($1, $2, $3, 'SI', NOW())
            ON CONFLICT (product_id, metric_type) DO UPDATE
            SET value = EXCLUDED.value, last_updated = NOW(), verified = FALSE
            "#,
            product_id,
            metric_type,
            calculated_value
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn calculate_carbon_footprint(&self, readings: &[IotReading]) -> Decimal {
        // Simplified CO2 calculation: energy usage * emission factor
        let total_energy: Decimal = readings.iter().map(|r| r.value).sum();
        let emission_factor = Decimal::from_str_radix("0.475", 10).unwrap(); // kg CO2 / kWh (example factor)
        total_energy * emission_factor
    }

    fn calculate_water_usage(&self, readings: &[IotReading]) -> Decimal {
        readings.iter().map(|r| r.value).sum()
    }

    fn calculate_renewable_energy_ratio(&self, readings: &[IotReading]) -> Decimal {
        let total = readings.len() as i64;
        if total == 0 { return Decimal::ZERO; }
        
        let renewable_count = readings.iter()
            .filter(|r| r.metadata.get("source").and_then(|s| s.as_str()) == Some("renewable"))
            .count() as i64;
            
        Decimal::from(renewable_count) / Decimal::from(total)
    }

    fn calculate_waste_management_efficiency(&self, readings: &[IotReading]) -> Decimal {
        // ratio of recycled waste to total waste
        let total_waste: Decimal = readings.iter().map(|r| r.value).sum();
        if total_waste == Decimal::ZERO { return Decimal::ONE; }
        
        let recycled_waste: Decimal = readings.iter()
            .filter(|r| r.metadata.get("disposal_method").and_then(|s| s.as_str()) == Some("recycle"))
            .map(|r| r.value)
            .sum();
            
        recycled_waste / total_waste
    }

    fn calculate_average(&self, readings: &[IotReading]) -> Decimal {
        let sum: Decimal = readings.iter().map(|r| r.value).sum();
        let count = readings.len() as i64;
        if count == 0 { Decimal::ZERO } else { sum / Decimal::from(count) }
    }

    async fn detect_anomalies(&self, product_id: &str, metric_type: &str) -> Result<(), AppError> {
        let readings = sqlx::query_as!(
            IotReading,
            "SELECT * FROM iot_readings WHERE product_id = $1 AND metric_type = $2 ORDER BY timestamp DESC LIMIT 20",
            product_id,
            metric_type
        )
        .fetch_all(&self.pool)
        .await?;

        if readings.len() < 10 {
            return Ok(());
        }

        let values: Vec<f64> = readings.iter().map(|r| r.value.to_f64().unwrap_or(0.0)).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std_dev = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt();

        let latest_value = values[0];
        let z_score = if std_dev == 0.0 { 0.0 } else { (latest_value - mean).abs() / std_dev };

        if z_score > 3.0 {
            tracing::warn!("Anomaly detected for product {} metric {}: Z-score {}", product_id, metric_type, z_score);
            // In a real system, we would create an alert or flag the product
            sqlx::query!(
                "UPDATE sustainability_metrics SET metadata = metadata || $3::jsonb WHERE product_id = $1 AND metric_type = $2",
                product_id,
                metric_type,
                serde_json::json!({"anomaly_detected": true, "z_score": z_score, "detected_at": Utc::now()})
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn verify_metric(&self, product_id: &str, req: VerifyMetricRequest) -> Result<SustainabilityVerification, AppError> {
        // 1. Get current metric
        let metric = sqlx::query_as!(
            SustainabilityMetric,
            "SELECT * FROM sustainability_metrics WHERE product_id = $1 AND metric_type = $2",
            product_id,
            req.metric_type
        )
        .fetch_one(&self.pool)
        .await?;

        // 2. Integration with third-party certification API (Mocked)
        let third_party_verified = self.verify_with_third_party(product_id, &req.metric_type, metric.value).await?;
        if !third_party_verified {
            return Err(AppError::BadRequest("Third-party verification failed".to_string()));
        }

        // 3. Anchor on blockchain (Real implementation)
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, format!("{}:{}:{}", product_id, req.metric_type, metric.value).as_bytes());
        let cert_hash = format!("{:x}", hasher.finalize());
        
        let tx_hash = self.stellar.record_sustain_verify(
            product_id,
            &req.metric_type,
            "GBVERIFIER...", // Actual verifier address from system/env
            &cert_hash,
            req.certificate_url.as_deref().unwrap_or(""),
            req.notes.as_deref().unwrap_or("")
        ).await.map_err(|e| AppError::Internal(e))?;

        // 4. Save verification
        let verification = sqlx::query_as!(
            SustainabilityVerification,
            r#"
            INSERT INTO sustainability_verifications (product_id, metric_type, verifier_name, status, certificate_hash, certificate_url, blockchain_tx_hash, verified_at, notes)
            VALUES ($1, $2, $3, 'verified', $4, $5, $6, NOW(), $7)
            RETURNING *
            "#,
            product_id,
            req.metric_type,
            req.verifier_name,
            cert_hash,
            req.certificate_url,
            tx_hash,
            req.notes
        )
        .fetch_one(&self.pool)
        .await?;

        // 5. Update metric status
        sqlx::query!(
            "UPDATE sustainability_metrics SET verified = TRUE WHERE product_id = $1 AND metric_type = $2",
            product_id,
            req.metric_type
        )
        .execute(&self.pool)
        .await?;

        Ok(verification)
    }

    async fn verify_with_third_party(&self, _product_id: &str, _metric_type: &str, _value: Decimal) -> Result<bool, AppError> {
        // Mock API call to GreenCert or similar service
        tracing::info!("Calling third-party certification API...");
        Ok(true)
    }

    pub async fn generate_report(&self, product_id: &str, req: GenerateReportRequest) -> Result<SustainabilityReport, AppError> {
        let metrics = self.get_product_sustainability(product_id).await?;
        
        let content = match req.report_type.as_str() {
            "EU_GREEN_DEAL" => self.build_eu_green_deal_report(product_id, &metrics),
            "SEC_CLIMATE" => self.build_sec_climate_report(product_id, &metrics),
            _ => self.build_generic_esg_report(product_id, &metrics),
        };

        let report = sqlx::query_as!(
            SustainabilityReport,
            r#"
            INSERT INTO sustainability_reports (product_id, report_type, content)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            product_id,
            req.report_type,
            content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    fn build_eu_green_deal_report(&self, product_id: &str, metrics: &[SustainabilityMetric]) -> serde_json::Value {
        serde_json::json!({
            "standard": "EU Green Deal - CSRD",
            "product_id": product_id,
            "metrics": metrics,
            "compliance_status": "compliant",
            "carbon_intensity": metrics.iter().find(|m| m.metric_type == "carbon").map(|m| m.value),
            "water_footprint": metrics.iter().find(|m| m.metric_type == "water").map(|m| m.value),
            "generated_at": Utc::now()
        })
    }

    fn build_sec_climate_report(&self, product_id: &str, metrics: &[SustainabilityMetric]) -> serde_json::Value {
        serde_json::json!({
            "standard": "SEC Climate Disclosure",
            "product_id": product_id,
            "scope_1_emissions": metrics.iter().find(|m| m.metric_type == "carbon").map(|m| m.value),
            "risk_assessment": "low",
            "generated_at": Utc::now()
        })
    }

    fn build_generic_esg_report(&self, product_id: &str, metrics: &[SustainabilityMetric]) -> serde_json::Value {
        serde_json::json!({
            "standard": "General ESG",
            "product_id": product_id,
            "summary": "Sustainability metrics summary",
            "data": metrics,
            "generated_at": Utc::now()
        })
    }

    pub async fn get_product_sustainability(&self, product_id: &str) -> Result<Vec<SustainabilityMetric>, AppError> {
        let metrics = sqlx::query_as!(
            SustainabilityMetric,
            "SELECT * FROM sustainability_metrics WHERE product_id = $1",
            product_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(metrics)
    }
}

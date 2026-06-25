use crate::error::AppError;
use crate::models::digital_twin::*;
use crate::models::iot::*;
use chrono::Utc;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use uuid::Uuid;

pub struct IoTTwinSyncService {
    pool: PgPool,
}

impl IoTTwinSyncService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Start the live sync loop
    pub async fn start_sync_loop(&self) {
        info!("Starting IoT to Digital Twin sync loop");
        
        let mut ticker = interval(Duration::from_secs(60)); // Check every minute
        
        loop {
            ticker.tick().await;
            
            if let Err(e) = self.process_active_syncs().await {
                error!("Error processing IoT syncs: {}", e);
            }
        }
    }

    /// Process all active IoT to Twin sync configurations
    async fn process_active_syncs(&self) -> Result<(), AppError> {
        let syncs = sqlx::query_as::<_, IoTTwinSync>(
            "SELECT * FROM iot_twin_sync WHERE is_active = true"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for sync in syncs {
            if let Err(e) = self.process_sync(&sync).await {
                error!("Error processing sync for device {}: {}", sync.device_id, e);
            }
        }

        Ok(())
    }

    /// Process a single sync configuration
    async fn process_sync(&self, sync: &IoTTwinSync) -> Result<(), AppError> {
        // Check if enough time has passed since last sync
        let now = Utc::now();
        let elapsed = (now - sync.last_sync_at).num_seconds();
        
        if elapsed < sync.sync_frequency_seconds as i64 {
            return Ok(());
        }

        // Get latest IoT reading based on sync type
        match sync.sync_type.as_str() {
            "temperature" => self.sync_temperature(sync).await?,
            "humidity" => self.sync_humidity(sync).await?,
            "quality" => self.sync_quality(sync).await?,
            "decay" => self.sync_decay(sync).await?,
            _ => {
                error!("Unknown sync type: {}", sync.sync_type);
            }
        }

        // Update last sync timestamp
        sqlx::query(
            "UPDATE iot_twin_sync SET last_sync_at = NOW() WHERE id = $1"
        )
        .bind(sync.id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Sync temperature readings to twin state
    async fn sync_temperature(&self, sync: &IoTTwinSync) -> Result<(), AppError> {
        // Get latest temperature reading
        let reading = sqlx::query_as::<_, TemperatureReading>(
            r#"
            SELECT * FROM temperature_readings 
            WHERE device_id = $1 
            ORDER BY reading_timestamp DESC 
            LIMIT 1
            "#
        )
        .bind(&sync.device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some(reading) = reading {
            // Update twin state with temperature data
            let state_data = serde_json::json!({
                "temperature": reading.temperature_celsius,
                "humidity": reading.humidity_percent,
                "location": reading.location,
                "quality_score": reading.quality_score,
                "last_reading_at": reading.reading_timestamp
            });

            let metrics = serde_json::json!({
                "temperature_stress": self.calculate_temperature_stress(reading.temperature_celsius),
                "sync_source": "iot_sensor",
                "device_id": sync.device_id
            });

            // Update twin state
            sqlx::query(
                r#"
                UPDATE digital_twins 
                SET current_state = current_state || $1,
                    updated_at = NOW(),
                    last_sync_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(&state_data)
            .bind(sync.twin_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            // Record state history
            sqlx::query(
                r#"
                INSERT INTO twin_states (id, twin_id, state_data, metrics, timestamp, source)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(sync.twin_id)
            .bind(&state_data)
            .bind(&metrics)
            .bind(Utc::now())
            .bind("iot_sync")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            info!("Synced temperature data for twin {} from device {}", sync.twin_id, sync.device_id);
        }

        Ok(())
    }

    /// Sync humidity readings to twin state
    async fn sync_humidity(&self, sync: &IoTTwinSync) -> Result<(), AppError> {
        let reading = sqlx::query_as::<_, TemperatureReading>(
            r#"
            SELECT * FROM temperature_readings 
            WHERE device_id = $1 
            ORDER BY reading_timestamp DESC 
            LIMIT 1
            "#
        )
        .bind(&sync.device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some(reading) = reading {
            let state_data = serde_json::json!({
                "humidity": reading.humidity_percent,
                "last_humidity_reading_at": reading.reading_timestamp
            });

            let metrics = serde_json::json!({
                "humidity_stress": self.calculate_humidity_stress(reading.humidity_percent),
                "sync_source": "iot_sensor"
            });

            sqlx::query(
                r#"
                UPDATE digital_twins 
                SET current_state = current_state || $1,
                    updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(&state_data)
            .bind(sync.twin_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            info!("Synced humidity data for twin {} from device {}", sync.twin_id, sync.device_id);
        }

        Ok(())
    }

    /// Sync quality readings to twin state
    async fn sync_quality(&self, sync: &IoTTwinSync) -> Result<(), AppError> {
        let reading = sqlx::query_as::<_, TemperatureReading>(
            r#"
            SELECT * FROM temperature_readings 
            WHERE device_id = $1 
            ORDER BY reading_timestamp DESC 
            LIMIT 1
            "#
        )
        .bind(&sync.device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some(reading) = reading {
            if let Some(quality_score) = reading.quality_score {
                let state_data = serde_json::json!({
                    "quality_score": quality_score,
                    "last_quality_reading_at": reading.reading_timestamp
                });

                let metrics = serde_json::json!({
                    "quality_trend": "stable",
                    "sync_source": "iot_sensor"
                });

                sqlx::query(
                    r#"
                    UPDATE digital_twins 
                    SET current_state = current_state || $1,
                        updated_at = NOW()
                    WHERE id = $2
                    "#
                )
                .bind(&state_data)
                .bind(sync.twin_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                info!("Synced quality data for twin {} from device {}", sync.twin_id, sync.device_id);
            }
        }

        Ok(())
    }

    /// Sync decay model calculations to twin state
    async fn sync_decay(&self, sync: &IoTTwinSync) -> Result<(), AppError> {
        // Get twin with decay parameters
        let twin = sqlx::query_as::<_, DigitalTwin>(
            "SELECT * FROM digital_twins WHERE id = $1"
        )
        .bind(sync.twin_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get current temperature and humidity from state
        let current_temp = twin.current_state.get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(20.0);

        let current_humidity = twin.current_state.get("humidity")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);

        // Calculate elapsed time since creation
        let elapsed_hours = (Utc::now() - twin.created_at).num_seconds() as f64 / 3600.0;

        // Import physics model service to calculate health score
        use crate::services::physics_model_service::PhysicsModelService;
        let physics_service = PhysicsModelService::new(self.pool.clone());

        let health_result = physics_service
            .calculate_health_score(
                sync.twin_id,
                current_temp,
                current_humidity,
                elapsed_hours,
            )
            .await?;

        // Update twin state with decay information
        let state_data = serde_json::json!({
            "health_score": health_result.health_score,
            "decay_rate": health_result.decay_rate,
            "predicted_expiry": health_result.predicted_expiry,
            "risk_factors": health_result.risk_factors,
            "last_decay_calculation": Utc::now().to_rfc3339()
        });

        sqlx::query(
            r#"
            UPDATE digital_twins 
            SET current_state = current_state || $1,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(&state_data)
        .bind(sync.twin_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        info!("Synced decay calculation for twin {} from device {}", sync.twin_id, sync.device_id);

        Ok(())
    }

    /// Calculate temperature stress level
    fn calculate_temperature_stress(&self, temp: rust_decimal::Decimal) -> f64 {
        let temp_f = temp.to_f64().unwrap_or(20.0);
        // Optimal range is 15-25°C
        if temp_f >= 15.0 && temp_f <= 25.0 {
            0.0
        } else if temp_f < 15.0 {
            (15.0 - temp_f) / 15.0
        } else {
            (temp_f - 25.0) / 25.0
        }
    }

    /// Calculate humidity stress level
    fn calculate_humidity_stress(&self, humidity: rust_decimal::Decimal) -> f64 {
        let humidity_f = humidity.to_f64().unwrap_or(50.0);
        // Optimal range is 40-60%
        if humidity_f >= 40.0 && humidity_f <= 60.0 {
            0.0
        } else if humidity_f < 40.0 {
            (40.0 - humidity_f) / 40.0
        } else {
            (humidity_f - 60.0) / 60.0
        }
    }

    /// Manually trigger sync for a specific device
    pub async fn trigger_sync(&self, device_id: &str) -> Result<(), AppError> {
        let syncs = sqlx::query_as::<_, IoTTwinSync>(
            "SELECT * FROM iot_twin_sync WHERE device_id = $1 AND is_active = true"
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for sync in syncs {
            self.process_sync(&sync).await?;
        }

        Ok(())
    }
}

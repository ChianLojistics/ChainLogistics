use sqlx::PgPool;
use uuid::Uuid;
use crate::models::iot::*;
use rust_decimal::Decimal;

pub struct IoTService {
    pool: PgPool,
}

impl IoTService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // IoT Device Management
    pub async fn create_device(&self, device: NewIoTDevice) -> Result<IoTDevice, sqlx::Error> {
        sqlx::query_as!(
            IoTDevice,
            r#"
            INSERT INTO iot_devices (
                device_id, device_type, product_id, name, description,
                manufacturer, model, serial_number, firmware_version,
                location, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
            device.device_id,
            device.device_type,
            device.product_id,
            device.name,
            device.description,
            device.manufacturer,
            device.model,
            device.serial_number,
            device.firmware_version,
            device.location,
            device.metadata.unwrap_or(serde_json::json!({}))
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_device(&self, device_id: &str) -> Result<Option<IoTDevice>, sqlx::Error> {
        sqlx::query_as!(
            IoTDevice,
            "SELECT * FROM iot_devices WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_devices(
        &self,
        product_id: Option<String>,
        device_type: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<IoTDevice>, sqlx::Error> {
        match (product_id, device_type, is_active) {
            (Some(pid), Some(dt), Some(active)) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE product_id = $1 AND device_type = $2 AND is_active = $3 ORDER BY created_at DESC",
                    pid, dt, active
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), Some(dt), None) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE product_id = $1 AND device_type = $2 ORDER BY created_at DESC",
                    pid, dt
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, Some(active)) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE product_id = $1 AND is_active = $2 ORDER BY created_at DESC",
                    pid, active
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(dt), Some(active)) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE device_type = $1 AND is_active = $2 ORDER BY created_at DESC",
                    dt, active
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, None) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE product_id = $1 ORDER BY created_at DESC",
                    pid
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(dt), None) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE device_type = $1 ORDER BY created_at DESC",
                    dt
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(active)) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices WHERE is_active = $1 ORDER BY created_at DESC",
                    active
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as!(
                    IoTDevice,
                    "SELECT * FROM iot_devices ORDER BY created_at DESC"
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn update_device_last_seen(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE iot_devices SET last_seen_at = NOW() WHERE device_id = $1",
            device_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Temperature Readings
    pub async fn create_reading(&self, reading: NewTemperatureReading) -> Result<TemperatureReading, sqlx::Error> {
        // Update device last seen
        let _ = self.update_device_last_seen(&reading.device_id).await;

        // Check for anomalies and thresholds
        let (is_anomaly, anomaly_reason) = self.check_anomaly(&reading).await;
        
        let reading_with_anomaly = sqlx::query_as!(
            TemperatureReading,
            r#"
            INSERT INTO temperature_readings (
                device_id, product_id, temperature_celsius, humidity_percent,
                unit, reading_timestamp, location, quality_score,
                is_anomaly, anomaly_reason, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
            reading.device_id,
            reading.product_id,
            reading.temperature_celsius,
            reading.humidity_percent,
            reading.unit,
            reading.reading_timestamp,
            reading.location,
            reading.quality_score,
            is_anomaly,
            anomaly_reason,
            reading.metadata.unwrap_or(serde_json::json!({}))
        )
        .fetch_one(&self.pool)
        .await?;

        // Check thresholds and create alerts if needed
        if let Some(ref pid) = reading.product_id {
            let _ = self.check_thresholds(&reading.device_id, pid, reading.temperature_celsius).await;
        }

        Ok(reading_with_anomaly)
    }

    async fn check_anomaly(&self, reading: &NewTemperatureReading) -> (bool, Option<String>) {
        // Simple anomaly detection: check if temperature is outside reasonable range
        // In production, this would use statistical methods or ML models
        let temp = reading.temperature_celsius;
        
        if temp < Decimal::from(-50) || temp > Decimal::from(100) {
            return (true, Some("Temperature outside reasonable range".to_string()));
        }

        // Check quality score if provided
        if let Some(score) = reading.quality_score {
            if score < Decimal::from(50) {
                return (true, Some("Low quality score".to_string()));
            }
        }

        (false, None)
    }

    async fn check_thresholds(&self, device_id: &str, product_id: &str, temperature: Decimal) -> Result<(), sqlx::Error> {
        let thresholds = sqlx::query_as!(
            TemperatureThreshold,
            r#"
            SELECT * FROM temperature_thresholds
            WHERE (product_id = $1 OR device_id = $2)
            AND is_active = true
            "#,
            product_id,
            device_id
        )
        .fetch_all(&self.pool)
        .await?;

        for threshold in thresholds {
            let breached = match threshold.threshold_type.as_str() {
                "min" => {
                    if let Some(min_temp) = threshold.min_temperature_celsius {
                        temperature < min_temp
                    } else {
                        false
                    }
                }
                "max" => {
                    if let Some(max_temp) = threshold.max_temperature_celsius {
                        temperature > max_temp
                    } else {
                        false
                    }
                }
                "critical_min" => {
                    if let Some(min_temp) = threshold.min_temperature_celsius {
                        temperature < min_temp
                    } else {
                        false
                    }
                }
                "critical_max" => {
                    if let Some(max_temp) = threshold.max_temperature_celsius {
                        temperature > max_temp
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if breached {
                let _ = self.create_alert(NewTemperatureAlert {
                    device_id: device_id.to_string(),
                    product_id: product_id.to_string(),
                    threshold_id: Some(threshold.id),
                    alert_type: "threshold_breach".to_string(),
                    alert_level: threshold.alert_level.clone(),
                    temperature_celsius: Some(temperature),
                    threshold_value: if threshold.threshold_type.contains("min") {
                        threshold.min_temperature_celsius
                    } else {
                        threshold.max_temperature_celsius
                    },
                    message: format!(
                        "{} threshold breached: {}°C",
                        threshold.threshold_type, temperature
                    ),
                }).await;
            }
        }

        Ok(())
    }

    pub async fn get_readings(
        &self,
        device_id: Option<String>,
        product_id: Option<String>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
    ) -> Result<Vec<TemperatureReading>, sqlx::Error> {
        match (device_id, product_id, start_time, end_time) {
            (Some(did), Some(pid), Some(start), Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND product_id = $2 AND reading_timestamp >= $3 AND reading_timestamp <= $4 ORDER BY reading_timestamp DESC LIMIT $5",
                    did, pid, start, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), Some(pid), Some(start), None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND product_id = $2 AND reading_timestamp >= $3 ORDER BY reading_timestamp DESC LIMIT $4",
                    did, pid, start, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), Some(pid), None, Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND product_id = $2 AND reading_timestamp <= $3 ORDER BY reading_timestamp DESC LIMIT $4",
                    did, pid, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), None, Some(start), Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND reading_timestamp >= $2 AND reading_timestamp <= $3 ORDER BY reading_timestamp DESC LIMIT $4",
                    did, start, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(pid), Some(start), Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE product_id = $1 AND reading_timestamp >= $2 AND reading_timestamp <= $3 ORDER BY reading_timestamp DESC LIMIT $4",
                    pid, start, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), Some(pid), None, None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND product_id = $2 ORDER BY reading_timestamp DESC LIMIT $3",
                    did, pid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), None, Some(start), None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND reading_timestamp >= $2 ORDER BY reading_timestamp DESC LIMIT $3",
                    did, start, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), None, None, Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 AND reading_timestamp <= $2 ORDER BY reading_timestamp DESC LIMIT $3",
                    did, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(pid), Some(start), None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE product_id = $1 AND reading_timestamp >= $2 ORDER BY reading_timestamp DESC LIMIT $3",
                    pid, start, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(pid), None, Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE product_id = $1 AND reading_timestamp <= $2 ORDER BY reading_timestamp DESC LIMIT $3",
                    pid, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(start), Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE reading_timestamp >= $1 AND reading_timestamp <= $2 ORDER BY reading_timestamp DESC LIMIT $3",
                    start, end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(did), None, None, None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE device_id = $1 ORDER BY reading_timestamp DESC LIMIT $2",
                    did, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(pid), None, None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE product_id = $1 ORDER BY reading_timestamp DESC LIMIT $2",
                    pid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(start), None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE reading_timestamp >= $1 ORDER BY reading_timestamp DESC LIMIT $2",
                    start, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None, Some(end)) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings WHERE reading_timestamp <= $1 ORDER BY reading_timestamp DESC LIMIT $2",
                    end, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None, None) => {
                sqlx::query_as!(
                    TemperatureReading,
                    "SELECT * FROM temperature_readings ORDER BY reading_timestamp DESC LIMIT $1",
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // Temperature Thresholds
    pub async fn create_threshold(&self, threshold: NewTemperatureThreshold) -> Result<TemperatureThreshold, sqlx::Error> {
        sqlx::query_as!(
            TemperatureThreshold,
            r#"
            INSERT INTO temperature_thresholds (
                product_id, device_id, threshold_type, min_temperature_celsius,
                max_temperature_celsius, duration_minutes, alert_level
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            threshold.product_id,
            threshold.device_id,
            threshold.threshold_type,
            threshold.min_temperature_celsius,
            threshold.max_temperature_celsius,
            threshold.duration_minutes,
            threshold.alert_level
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_thresholds(&self, product_id: &str) -> Result<Vec<TemperatureThreshold>, sqlx::Error> {
        sqlx::query_as!(
            TemperatureThreshold,
            "SELECT * FROM temperature_thresholds WHERE product_id = $1",
            product_id
        )
        .fetch_all(&self.pool)
        .await
    }

    // Temperature Alerts
    pub async fn create_alert(&self, alert: NewTemperatureAlert) -> Result<TemperatureAlert, sqlx::Error> {
        sqlx::query_as!(
            TemperatureAlert,
            r#"
            INSERT INTO temperature_alerts (
                device_id, product_id, threshold_id, alert_type, alert_level,
                temperature_celsius, threshold_value, message
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            alert.device_id,
            alert.product_id,
            alert.threshold_id,
            alert.alert_type,
            alert.alert_level,
            alert.temperature_celsius,
            alert.threshold_value,
            alert.message
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_alerts(
        &self,
        product_id: Option<String>,
        is_resolved: Option<bool>,
        limit: i64,
    ) -> Result<Vec<TemperatureAlert>, sqlx::Error> {
        match (product_id, is_resolved) {
            (Some(pid), Some(resolved)) => {
                sqlx::query_as!(
                    TemperatureAlert,
                    "SELECT * FROM temperature_alerts WHERE product_id = $1 AND is_resolved = $2 ORDER BY created_at DESC LIMIT $3",
                    pid, resolved, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None) => {
                sqlx::query_as!(
                    TemperatureAlert,
                    "SELECT * FROM temperature_alerts WHERE product_id = $1 ORDER BY created_at DESC LIMIT $2",
                    pid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(resolved)) => {
                sqlx::query_as!(
                    TemperatureAlert,
                    "SELECT * FROM temperature_alerts WHERE is_resolved = $1 ORDER BY created_at DESC LIMIT $2",
                    resolved, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as!(
                    TemperatureAlert,
                    "SELECT * FROM temperature_alerts ORDER BY created_at DESC LIMIT $1",
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid, acknowledged_by: String) -> Result<TemperatureAlert, sqlx::Error> {
        sqlx::query_as!(
            TemperatureAlert,
            r#"
            UPDATE temperature_alerts SET
                acknowledged_by = $2,
                acknowledged_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            alert_id,
            acknowledged_by
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn resolve_alert(&self, alert_id: Uuid, resolved_by: String) -> Result<TemperatureAlert, sqlx::Error> {
        sqlx::query_as!(
            TemperatureAlert,
            r#"
            UPDATE temperature_alerts SET
                is_resolved = true,
                resolved_at = NOW(),
                resolved_by = $2
            WHERE id = $1
            RETURNING *
            "#,
            alert_id,
            resolved_by
        )
        .fetch_one(&self.pool)
        .await
    }

    // Temperature Summaries (for reporting)
    pub async fn generate_summary(
        &self,
        device_id: &str,
        summary_period: &str,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<TemperatureSummary, sqlx::Error> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                AVG(temperature_celsius) as avg_temp,
                MIN(temperature_celsius) as min_temp,
                MAX(temperature_celsius) as max_temp,
                COUNT(*) as total_readings,
                SUM(CASE WHEN is_anomaly = true THEN 1 ELSE 0 END) as anomaly_count
            FROM temperature_readings
            WHERE device_id = $1
            AND reading_timestamp >= $2
            AND reading_timestamp <= $3
            "#,
            device_id,
            period_start,
            period_end
        )
        .fetch_one(&self.pool)
        .await?;

        // Count alerts in this period
        let alert_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM temperature_alerts
            WHERE device_id = $1
            AND created_at >= $2
            AND created_at <= $3
            "#,
            device_id,
            period_start,
            period_end
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        sqlx::query_as!(
            TemperatureSummary,
            r#"
            INSERT INTO temperature_summaries (
                device_id, summary_period, period_start, period_end,
                avg_temperature_celsius, min_temperature_celsius, max_temperature_celsius,
                total_readings, anomaly_count, alert_count
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (device_id, summary_period, period_start, period_end)
            DO UPDATE SET
                avg_temperature_celsius = EXCLUDED.avg_temperature_celsius,
                min_temperature_celsius = EXCLUDED.min_temperature_celsius,
                max_temperature_celsius = EXCLUDED.max_temperature_celsius,
                total_readings = EXCLUDED.total_readings,
                anomaly_count = EXCLUDED.anomaly_count,
                alert_count = EXCLUDED.alert_count
            RETURNING *
            "#,
            device_id,
            summary_period,
            period_start,
            period_end,
            stats.avg_temp,
            stats.min_temp,
            stats.max_temp,
            stats.total_readings.unwrap_or(0) as i32,
            stats.anomaly_count.unwrap_or(0) as i32,
            alert_count as i32
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_summaries(
        &self,
        device_id: &str,
        summary_period: Option<String>,
        limit: i64,
    ) -> Result<Vec<TemperatureSummary>, sqlx::Error> {
        if let Some(period) = summary_period {
            sqlx::query_as!(
                TemperatureSummary,
                "SELECT * FROM temperature_summaries WHERE device_id = $1 AND summary_period = $2 ORDER BY period_start DESC LIMIT $3",
                device_id, period, limit
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as!(
                TemperatureSummary,
                "SELECT * FROM temperature_summaries WHERE device_id = $1 ORDER BY period_start DESC LIMIT $2",
                device_id, limit
            )
            .fetch_all(&self.pool)
            .await
        }
    }
}

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
        sqlx::query_as::<IoTDevice, _>(
            r#"
            INSERT INTO iot_devices (
                device_id, device_type, product_id, name, description,
                manufacturer, model, serial_number, firmware_version,
                location, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                id, device_id, device_type, product_id, name, description,
                manufacturer, model, serial_number, firmware_version,
                location, last_seen_at, is_active, metadata, created_at, updated_at
            "#,
        )
        .bind(device.device_id)
        .bind(device.device_type)
        .bind(device.product_id)
        .bind(device.name)
        .bind(device.description)
        .bind(device.manufacturer)
        .bind(device.model)
        .bind(device.serial_number)
        .bind(device.firmware_version)
        .bind(device.location)
        .bind(device.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_device(&self, device_id: &str) -> Result<Option<IoTDevice>, sqlx::Error> {
        sqlx::query_as::<IoTDevice, _>(
            "SELECT id, device_id, device_type, product_id, name, description, manufacturer, model, serial_number, firmware_version, location, last_seen_at, is_active, metadata, created_at, updated_at FROM iot_devices WHERE device_id = $1",
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_devices(
        &self,
        product_id: Option<String>,
        device_type: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<IoTDevice>, sqlx::Error> {
        let mut query = "SELECT id, device_id, device_type, product_id, name, description, manufacturer, model, serial_number, firmware_version, location, last_seen_at, is_active, metadata, created_at, updated_at FROM iot_devices WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(pid) = product_id {
            query.push_str(&format!(" AND product_id = ${}", bind_index));
            bindings.push(pid);
            bind_index += 1;
        }
        if let Some(dt) = device_type {
            query.push_str(&format!(" AND device_type = ${}", bind_index));
            bindings.push(dt);
            bind_index += 1;
        }
        if let Some(active) = is_active {
            query.push_str(&format!(" AND is_active = ${}", bind_index));
            bindings.push(active.to_string());
            bind_index += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<IoTDevice>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_device_last_seen(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE iot_devices SET last_seen_at = NOW() WHERE device_id = $1")
            .bind(device_id)
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
        
        let reading_with_anomaly = sqlx::query_as::<TemperatureReading, _>(
            r#"
            INSERT INTO temperature_readings (
                device_id, product_id, temperature_celsius, humidity_percent,
                unit, reading_timestamp, location, quality_score,
                is_anomaly, anomaly_reason, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                id, device_id, product_id, temperature_celsius, humidity_percent,
                unit, reading_timestamp, location, quality_score, is_anomaly,
                anomaly_reason, metadata, created_at
            "#,
        )
        .bind(reading.device_id)
        .bind(reading.product_id)
        .bind(reading.temperature_celsius)
        .bind(reading.humidity_percent)
        .bind(reading.unit)
        .bind(reading.reading_timestamp)
        .bind(reading.location)
        .bind(reading.quality_score)
        .bind(is_anomaly)
        .bind(anomaly_reason)
        .bind(reading.metadata.unwrap_or(serde_json::json!({})))
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
        let thresholds = sqlx::query_as::<TemperatureThreshold, _>(
            r#"
            SELECT id, product_id, device_id, threshold_type, min_temperature_celsius, max_temperature_celsius, duration_minutes, alert_level, is_active, created_at, updated_at FROM temperature_thresholds
            WHERE (product_id = $1 OR device_id = $2)
            AND is_active = true
            "#,
        )
        .bind(product_id)
        .bind(device_id)
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
        let mut query = "SELECT id, device_id, product_id, temperature_celsius, humidity_percent, unit, reading_timestamp, location, quality_score, is_anomaly, anomaly_reason, metadata, created_at FROM temperature_readings WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(did) = device_id {
            query.push_str(&format!(" AND device_id = ${}", bind_index));
            bindings.push(did);
            bind_index += 1;
        }
        if let Some(pid) = product_id {
            query.push_str(&format!(" AND product_id = ${}", bind_index));
            bindings.push(pid);
            bind_index += 1;
        }
        if let Some(start) = start_time {
            query.push_str(&format!(" AND reading_timestamp >= ${}", bind_index));
            bindings.push(start.to_rfc3339());
            bind_index += 1;
        }
        if let Some(end) = end_time {
            query.push_str(&format!(" AND reading_timestamp <= ${}", bind_index));
            bindings.push(end.to_rfc3339());
            bind_index += 1;
        }

        query.push_str(&format!(" ORDER BY reading_timestamp DESC LIMIT ${}", bind_index));
        bindings.push(limit.to_string());

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<TemperatureReading>()
            .fetch_all(&self.pool)
            .await
    }

    // Temperature Thresholds
    pub async fn create_threshold(&self, threshold: NewTemperatureThreshold) -> Result<TemperatureThreshold, sqlx::Error> {
        sqlx::query_as::<TemperatureThreshold, _>(
            r#"
            INSERT INTO temperature_thresholds (
                product_id, device_id, threshold_type, min_temperature_celsius,
                max_temperature_celsius, duration_minutes, alert_level
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, product_id, device_id, threshold_type, min_temperature_celsius, max_temperature_celsius, duration_minutes, alert_level, is_active, created_at, updated_at
            "#,
        )
        .bind(threshold.product_id)
        .bind(threshold.device_id)
        .bind(threshold.threshold_type)
        .bind(threshold.min_temperature_celsius)
        .bind(threshold.max_temperature_celsius)
        .bind(threshold.duration_minutes)
        .bind(threshold.alert_level)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_thresholds(&self, product_id: &str) -> Result<Vec<TemperatureThreshold>, sqlx::Error> {
        sqlx::query_as::<TemperatureThreshold, _>(
            "SELECT id, product_id, device_id, threshold_type, min_temperature_celsius, max_temperature_celsius, duration_minutes, alert_level, is_active, created_at, updated_at FROM temperature_thresholds WHERE product_id = $1",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
    }

    // Temperature Alerts
    pub async fn create_alert(&self, alert: NewTemperatureAlert) -> Result<TemperatureAlert, sqlx::Error> {
        sqlx::query_as::<TemperatureAlert, _>(
            r#"
            INSERT INTO temperature_alerts (
                device_id, product_id, threshold_id, alert_type, alert_level,
                temperature_celsius, threshold_value, message
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, device_id, product_id, threshold_id, alert_type, alert_level, temperature_celsius, threshold_value, message, is_resolved, acknowledged_at, acknowledged_by, resolved_at, resolved_by, created_at
            "#,
        )
        .bind(alert.device_id)
        .bind(alert.product_id)
        .bind(alert.threshold_id)
        .bind(alert.alert_type)
        .bind(alert.alert_level)
        .bind(alert.temperature_celsius)
        .bind(alert.threshold_value)
        .bind(alert.message)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_alerts(
        &self,
        product_id: Option<String>,
        is_resolved: Option<bool>,
        limit: i64,
    ) -> Result<Vec<TemperatureAlert>, sqlx::Error> {
        let mut query = "SELECT id, device_id, product_id, threshold_id, alert_type, alert_level, temperature_celsius, threshold_value, message, is_resolved, acknowledged_at, acknowledged_by, resolved_at, resolved_by, created_at FROM temperature_alerts WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(pid) = product_id {
            query.push_str(&format!(" AND product_id = ${}", bind_index));
            bindings.push(pid);
            bind_index += 1;
        }
        if let Some(resolved) = is_resolved {
            query.push_str(&format!(" AND is_resolved = ${}", bind_index));
            bindings.push(resolved.to_string());
            bind_index += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", bind_index));
        bindings.push(limit.to_string());

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<TemperatureAlert>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid, acknowledged_by: String) -> Result<TemperatureAlert, sqlx::Error> {
        sqlx::query_as::<TemperatureAlert, _>(
            r#"
            UPDATE temperature_alerts SET
                acknowledged_by = $2,
                acknowledged_at = NOW()
            WHERE id = $1
            RETURNING id, device_id, product_id, threshold_id, alert_type, alert_level, temperature_celsius, threshold_value, message, is_resolved, acknowledged_at, acknowledged_by, resolved_at, resolved_by, created_at
            "#,
        )
        .bind(alert_id)
        .bind(acknowledged_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn resolve_alert(&self, alert_id: Uuid, resolved_by: String) -> Result<TemperatureAlert, sqlx::Error> {
        sqlx::query_as::<TemperatureAlert, _>(
            r#"
            UPDATE temperature_alerts SET
                is_resolved = true,
                resolved_at = NOW(),
                resolved_by = $2
            WHERE id = $1
            RETURNING id, device_id, product_id, threshold_id, alert_type, alert_level, temperature_celsius, threshold_value, message, is_resolved, acknowledged_at, acknowledged_by, resolved_at, resolved_by, created_at
            "#,
        )
        .bind(alert_id)
        .bind(resolved_by)
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
        let stats = sqlx::query(
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
        )
        .bind(device_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await?;

        // Count alerts in this period
        let alert_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM temperature_alerts
            WHERE device_id = $1
            AND created_at >= $2
            AND created_at <= $3
            "#,
        )
        .bind(device_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        use sqlx::Row;
        sqlx::query_as::<TemperatureSummary, _>(
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
            RETURNING id, device_id, summary_period, period_start, period_end, avg_temperature_celsius, min_temperature_celsius, max_temperature_celsius, total_readings, anomaly_count, alert_count, created_at, updated_at
            "#,
        )
        .bind(device_id)
        .bind(summary_period)
        .bind(period_start)
        .bind(period_end)
        .bind(stats.get::<Option<Decimal>, _>("avg_temp"))
        .bind(stats.get::<Option<Decimal>, _>("min_temp"))
        .bind(stats.get::<Option<Decimal>, _>("max_temp"))
        .bind(stats.get::<Option<i64>, _>("total_readings").unwrap_or(0) as i32)
        .bind(stats.get::<Option<i64>, _>("anomaly_count").unwrap_or(0) as i32)
        .bind(alert_count as i32)
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
            sqlx::query_as::<TemperatureSummary, _>(
                "SELECT id, device_id, summary_period, period_start, period_end, avg_temperature_celsius, min_temperature_celsius, max_temperature_celsius, total_readings, anomaly_count, alert_count, created_at, updated_at FROM temperature_summaries WHERE device_id = $1 AND summary_period = $2 ORDER BY period_start DESC LIMIT $3",
            )
            .bind(device_id)
            .bind(period)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<TemperatureSummary, _>(
                "SELECT id, device_id, summary_period, period_start, period_end, avg_temperature_celsius, min_temperature_celsius, max_temperature_celsius, total_readings, anomaly_count, alert_count, created_at, updated_at FROM temperature_summaries WHERE device_id = $1 ORDER BY period_start DESC LIMIT $2",
            )
            .bind(device_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
    }
}

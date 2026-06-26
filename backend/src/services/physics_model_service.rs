use crate::error::AppError;
use crate::models::digital_twin::*;
use chrono::{Duration, Utc};
use rand::Rng;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PhysicsModelService {
    pool: PgPool,
}

impl PhysicsModelService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate health score based on decay model and current conditions
    pub async fn calculate_health_score(
        &self,
        twin_id: Uuid,
        current_temp: f64,
        current_humidity: f64,
        elapsed_hours: f64,
    ) -> Result<HealthScoreResult, AppError> {
        // Get twin with decay model parameters
        let twin = sqlx::query_as::<_, DigitalTwin>(
            "SELECT * FROM digital_twins WHERE id = $1"
        )
        .bind(twin_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::NotFound(format!("Digital twin not found: {}", e)))?;

        let decay_params: DecayModelParameters = if let Some(params) = twin.decay_model_params {
            serde_json::from_value(params).map_err(|e| {
                AppError::ValidationError(format!("Invalid decay model parameters: {}", e))
            })?
        } else {
            // Default decay parameters
            DecayModelParameters {
                base_decay_rate: 0.01,
                temperature_coefficient: 0.05,
                humidity_coefficient: 0.03,
                quality_threshold: 0.7,
                calibration_factor: 1.0,
                model_type: "exponential_decay".to_string(),
            }
        };

        // Calculate adjusted decay rate based on environmental conditions
        let temp_factor = 1.0 + (current_temp - 20.0) * decay_params.temperature_coefficient;
        let humidity_factor = 1.0 + (current_humidity - 50.0) * decay_params.humidity_coefficient;
        let adjusted_decay_rate = decay_params.base_decay_rate * temp_factor * humidity_factor * decay_params.calibration_factor;

        // Calculate health score using exponential decay model
        let health_score = (-adjusted_decay_rate * elapsed_hours).exp();

        // Calculate predicted expiry based on current health trajectory
        let remaining_health = health_score - decay_params.quality_threshold;
        let hours_until_expiry = if remaining_health > 0.0 {
            -remaining_health.ln() / adjusted_decay_rate
        } else {
            0.0
        };

        let predicted_expiry = Utc::now() + Duration::seconds((hours_until_expiry * 3600.0) as i64);

        // Identify risk factors
        let mut risk_factors = Vec::new();
        if current_temp > 30.0 {
            risk_factors.push("High temperature accelerating decay".to_string());
        }
        if current_humidity > 70.0 {
            risk_factors.push("High humidity promoting degradation".to_string());
        }
        if health_score < decay_params.quality_threshold {
            risk_factors.push("Health score below quality threshold".to_string());
        }

        // Generate recommendations
        let mut recommendations = Vec::new();
        if health_score < 0.8 {
            recommendations.push("Consider expedited shipping to maintain quality".to_string());
        }
        if current_temp > 25.0 {
            recommendations.push("Improve temperature control during transit".to_string());
        }
        if hours_until_expiry < 72.0 {
            recommendations.push("Prioritize processing before expiry".to_string());
        }

        // Update twin with new health score
        self.update_twin_health(twin_id, health_score, &predicted_expiry).await?;

        // Store health metrics
        self.store_health_metrics(
            twin_id,
            health_score,
            adjusted_decay_rate,
            current_temp,
            current_humidity,
        ).await?;

        Ok(HealthScoreResult {
            health_score,
            decay_rate: adjusted_decay_rate,
            predicted_expiry,
            confidence_interval: None,
            risk_factors,
            recommendations,
        })
    }

    /// Run Monte Carlo simulation for confidence intervals
    pub async fn run_monte_carlo_simulation(
        &self,
        twin_id: Uuid,
        config: MonteCarloSimulationConfig,
    ) -> Result<serde_json::Value, AppError> {
        let mut health_scores = Vec::with_capacity(config.num_runs as usize);
        let mut rng = rand::thread_rng();

        // Get current twin state
        let twin = sqlx::query_as::<_, DigitalTwin>(
            "SELECT * FROM digital_twins WHERE id = $1"
        )
        .bind(twin_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::NotFound(format!("Digital twin not found: {}", e)))?;

        let decay_params: DecayModelParameters = if let Some(params) = twin.decay_model_params {
            serde_json::from_value(params).map_err(|e| {
                AppError::ValidationError(format!("Invalid decay model parameters: {}", e))
            })?
        } else {
            DecayModelParameters {
                base_decay_rate: 0.01,
                temperature_coefficient: 0.05,
                humidity_coefficient: 0.03,
                quality_threshold: 0.7,
                calibration_factor: 1.0,
                model_type: "exponential_decay".to_string(),
            }
        };

        // Extract parameter ranges for Monte Carlo
        let temp_range = config.parameter_ranges.get("temperature")
            .and_then(|v| v.as_array())
            .map(|arr| (arr[0].as_f64().unwrap_or(15.0), arr[1].as_f64().unwrap_or(35.0)))
            .unwrap_or((15.0, 35.0));

        let humidity_range = config.parameter_ranges.get("humidity")
            .and_then(|v| v.as_array())
            .map(|arr| (arr[0].as_f64().unwrap_or(30.0), arr[1].as_f64().unwrap_or(80.0)))
            .unwrap_or((30.0, 80.0));

        let elapsed_hours = config.parameter_ranges.get("elapsed_hours")
            .and_then(|v| v.as_f64())
            .unwrap_or(48.0);

        // Run Monte Carlo simulations
        for _ in 0..config.num_runs {
            let temp = rng.gen_range(temp_range.0..=temp_range.1);
            let humidity = rng.gen_range(humidity_range.0..=humidity_range.1);

            let temp_factor = 1.0 + (temp - 20.0) * decay_params.temperature_coefficient;
            let humidity_factor = 1.0 + (humidity - 50.0) * decay_params.humidity_coefficient;
            let adjusted_decay_rate = decay_params.base_decay_rate * temp_factor * humidity_factor * decay_params.calibration_factor;
            let health_score = (-adjusted_decay_rate * elapsed_hours).exp();

            health_scores.push(health_score);
        }

        // Calculate confidence interval
        health_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = health_scores.len();
        let lower_idx = ((1.0 - config.confidence_level) / 2.0 * n as f64) as usize;
        let upper_idx = ((1.0 + config.confidence_level) / 2.0 * n as f64) as usize;

        let lower_bound = health_scores.get(lower_idx).unwrap_or(&0.0);
        let upper_bound = health_scores.get(upper_idx).unwrap_or(&1.0);
        let mean_score: f64 = health_scores.iter().sum::<f64>() / n as f64;

        Ok(json!({
            "mean_health_score": mean_score,
            "confidence_interval": {
                "lower": *lower_bound,
                "upper": *upper_bound,
                "level": config.confidence_level
            },
            "percentiles": {
                "p5": health_scores[(n as f64 * 0.05) as usize],
                "p25": health_scores[(n as f64 * 0.25) as usize],
                "p50": health_scores[(n as f64 * 0.50) as usize],
                "p75": health_scores[(n as f64 * 0.75) as usize],
                "p95": health_scores[(n as f64 * 0.95) as usize],
            },
            "num_runs": config.num_runs
        }))
    }

    /// Audit prediction accuracy vs actual outcomes
    pub async fn audit_prediction_accuracy(
        &self,
        twin_id: Uuid,
        prediction_id: Uuid,
        actual_value: serde_json::Value,
    ) -> Result<PredictionAccuracyAudit, AppError> {
        // Get the prediction
        let prediction = sqlx::query_as::<_, Prediction>(
            "SELECT * FROM predictions WHERE id = $1"
        )
        .bind(prediction_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::NotFound(format!("Prediction not found: {}", e)))?;

        // Calculate accuracy score
        let accuracy_score = self.calculate_accuracy(&prediction.predicted_value, &actual_value)?;

        // Calculate error magnitude
        let error_magnitude = self.calculate_error_magnitude(&prediction.predicted_value, &actual_value);

        // Store audit record
        let audit = sqlx::query_as::<_, PredictionAccuracyAudit>(
            r#"
            INSERT INTO prediction_accuracy_audit (
                id, twin_id, prediction_id, prediction_type, predicted_value,
                actual_value, accuracy_score, error_magnitude, timestamp, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(twin_id)
        .bind(prediction_id)
        .bind(format!("{:?}", prediction.prediction_type))
        .bind(&prediction.predicted_value)
        .bind(&actual_value)
        .bind(accuracy_score)
        .bind(error_magnitude)
        .bind(Utc::now())
        .bind(json!({"audited_at": Utc::now().to_rfc3339()}))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update prediction with actual value and accuracy
        sqlx::query(
            r#"
            UPDATE predictions 
            SET actual_value = $1, accuracy_score = $2
            WHERE id = $3
            "#,
        )
        .bind(&actual_value)
        .bind(accuracy_score)
        .bind(prediction_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(audit)
    }

    /// Get accuracy statistics for a twin
    pub async fn get_accuracy_statistics(
        &self,
        twin_id: Uuid,
    ) -> Result<serde_json::Value, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                prediction_type,
                COUNT(*) as total_predictions,
                AVG(accuracy_score) as avg_accuracy,
                AVG(error_magnitude) as avg_error,
                STDDEV(accuracy_score) as accuracy_stddev
            FROM prediction_accuracy_audit
            WHERE twin_id = $1
            GROUP BY prediction_type
            "#,
            twin_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut result = json!({
            "twin_id": twin_id,
            "by_type": []
        });

        let by_type = result["by_type"].as_array_mut().unwrap();
        for stat in stats {
            by_type.push(json!({
                "prediction_type": stat.prediction_type,
                "total_predictions": stat.total_predictions,
                "average_accuracy": stat.avg_accuracy,
                "average_error": stat.avg_error,
                "accuracy_stddev": stat.accuracy_stddev
            }));
        }

        Ok(result)
    }

    /// Update twin health score and predicted expiry
    async fn update_twin_health(
        &self,
        twin_id: Uuid,
        health_score: f64,
        predicted_expiry: &chrono::DateTime<Utc>,
    ) -> Result<(), AppError> {
        // Get current health history
        let twin = sqlx::query_as::<_, DigitalTwin>(
            "SELECT * FROM digital_twins WHERE id = $1"
        )
        .bind(twin_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut health_history = if let Some(history) = twin.health_history {
            history.as_array().unwrap_or(&vec![]).clone()
        } else {
            vec![]
        };

        // Add new health score entry
        health_history.push(json!({
            "score": health_score,
            "timestamp": Utc::now().to_rfc3339()
        }));

        // Keep only last 100 entries
        if health_history.len() > 100 {
            health_history = health_history.into_iter().rev().take(100).collect();
            health_history.reverse();
        }

        // Update twin
        sqlx::query(
            r#"
            UPDATE digital_twins 
            SET current_health_score = $1,
                predicted_expiry_date = $2,
                health_history = $3,
                updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(health_score)
        .bind(predicted_expiry)
        .bind(serde_json::to_value(health_history).unwrap())
        .bind(Utc::now())
        .bind(twin_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store health metrics for visualization
    async fn store_health_metrics(
        &self,
        twin_id: Uuid,
        health_score: f64,
        decay_rate: f64,
        temperature: f64,
        humidity: f64,
    ) -> Result<(), AppError> {
        // Determine severity based on health score
        let severity = if health_score >= 0.8 {
            Some("normal".to_string())
        } else if health_score >= 0.6 {
            Some("warning".to_string())
        } else {
            Some("critical".to_string())
        };

        // Store overall health metric
        sqlx::query(
            r#"
            INSERT INTO twin_health_metrics (
                id, twin_id, metric_type, metric_value, threshold_min, threshold_max, severity, calculated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(twin_id)
        .bind("overall_health")
        .bind(health_score)
        .bind(0.6)
        .bind(0.8)
        .bind(&severity)
        .bind(Utc::now())
        .bind(json!({"decay_rate": decay_rate}))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Store temperature stress metric
        let temp_severity = if temperature <= 25.0 {
            Some("normal".to_string())
        } else if temperature <= 30.0 {
            Some("warning".to_string())
        } else {
            Some("critical".to_string())
        };

        sqlx::query(
            r#"
            INSERT INTO twin_health_metrics (
                id, twin_id, metric_type, metric_value, threshold_min, threshold_max, severity, calculated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(twin_id)
        .bind("temperature_stress")
        .bind(temperature)
        .bind(15.0)
        .bind(25.0)
        .bind(&temp_severity)
        .bind(Utc::now())
        .bind(json!({}))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Store humidity stress metric
        let humidity_severity = if humidity <= 60.0 {
            Some("normal".to_string())
        } else if humidity <= 70.0 {
            Some("warning".to_string())
        } else {
            Some("critical".to_string())
        };

        sqlx::query(
            r#"
            INSERT INTO twin_health_metrics (
                id, twin_id, metric_type, metric_value, threshold_min, threshold_max, severity, calculated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(twin_id)
        .bind("humidity_stress")
        .bind(humidity)
        .bind(40.0)
        .bind(60.0)
        .bind(&humidity_severity)
        .bind(Utc::now())
        .bind(json!({}))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Calculate accuracy between predicted and actual values
    fn calculate_accuracy(&self, predicted: &serde_json::Value, actual: &serde_json::Value) -> Result<f64, AppError> {
        // Handle different types of predictions
        if let (Some(pred_num), Some(act_num)) = (predicted.as_f64(), actual.as_f64()) {
            // For numeric values, use relative error
            if act_num == 0.0 {
                return Ok(if pred_num == 0.0 { 1.0 } else { 0.0 });
            }
            let relative_error = (pred_num - act_num).abs() / act_num.abs();
            Ok(1.0 - relative_error.min(1.0))
        } else if let (Some(pred_str), Some(act_str)) = (predicted.as_str(), actual.as_str()) {
            // For string values, use exact match
            Ok(if pred_str == act_str { 1.0 } else { 0.0 })
        } else if let (Some(pred_obj), Some(act_obj)) = (predicted.as_object(), actual.as_object()) {
            // For objects, compare fields
            let mut matching_fields = 0;
            let total_fields = pred_obj.keys().count();
            
            for key in pred_obj.keys() {
                if let (Some(pred_val), Some(act_val)) = (pred_obj.get(key), act_obj.get(key)) {
                    if self.calculate_accuracy(pred_val, act_val).unwrap_or(0.0) > 0.5 {
                        matching_fields += 1;
                    }
                }
            }
            
            if total_fields == 0 {
                Ok(1.0)
            } else {
                Ok(matching_fields as f64 / total_fields as f64)
            }
        } else {
            Ok(0.0)
        }
    }

    /// Calculate error magnitude
    fn calculate_error_magnitude(&self, predicted: &serde_json::Value, actual: &serde_json::Value) -> Option<f64> {
        if let (Some(pred_num), Some(act_num)) = (predicted.as_f64(), actual.as_f64()) {
            Some((pred_num - act_num).abs())
        } else {
            None
        }
    }
}

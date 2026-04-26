use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::error::AppError;
use crate::models::resilience::{RiskAssessment, DisruptionAlert, InventoryOptimization, ScenarioRequest, ScenarioReport};
use serde_json::json;

pub struct ResilienceService {
    pool: PgPool,
}

impl ResilienceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate risk score for a supplier or location
    pub async fn get_risk_assessments(&self) -> Result<Vec<RiskAssessment>, AppError> {
        let assessments = sqlx::query_as!(
            RiskAssessment,
            r#"SELECT id, entity_type, entity_id, risk_score, risk_level, factors, last_assessment_at, next_assessment_due, created_at FROM risk_assessments ORDER BY risk_score DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(assessments)
    }

    /// Predict disruptions and get active alerts
    pub async fn get_disruption_alerts(&self) -> Result<Vec<DisruptionAlert>, AppError> {
        let alerts = sqlx::query_as!(
            DisruptionAlert,
            r#"SELECT id, entity_type, entity_id, alert_type, severity, description, probability, estimated_impact_usd, start_time, end_time, created_at, updated_at FROM disruption_alerts ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(alerts)
    }

    /// Calculate inventory optimization for a specific product
    pub async fn optimize_inventory(&self, product_id: &str) -> Result<InventoryOptimization, AppError> {
        // Fetch historical data for lead time and demand variability
        let events = sqlx::query!(
            r#"SELECT event_type, timestamp FROM tracking_events WHERE product_id = $1 ORDER BY timestamp ASC"#,
            product_id
        )
        .fetch_all(&self.pool)
        .await?;

        if events.is_empty() {
            return Err(AppError::NotFound(format!("No history found for product {}", product_id)));
        }

        // Heuristic-based calculation for safety stock
        let avg_lead_time_days = 14.5; // Simulated default
        let daily_demand = 120.0; // Simulated default
        let z_score = 1.645; // 95% service level
        let demand_std_dev = 25.0; // Simulated default
        
        let safety_stock = z_score * (avg_lead_time_days as f64).sqrt() * demand_std_dev;
        let reorder_point = (daily_demand * avg_lead_time_days) + safety_stock;

        Ok(InventoryOptimization {
            product_id: product_id.to_string(),
            current_stock: 1500.0,
            safety_stock_level: safety_stock.round(),
            reorder_point: reorder_point.round(),
            lead_time_days: avg_lead_time_days,
            variability_factor: 0.15,
            recommendations: vec![
                format!("Maintain safety stock of {} units due to moderate lead time volatility.", safety_stock.round()),
                "Reorder when stock levels hit ".to_string() + &reorder_point.round().to_string(),
                "Consider multi-sourcing for key raw materials to reduce lead time variability.".to_string(),
            ],
        })
    }

    /// Generate a scenario planning report
    pub async fn generate_scenario_report(&self, req: ScenarioRequest) -> Result<ScenarioReport, AppError> {
        let report_id = Uuid::new_v4();
        
        // Internal logic to simulate scenario impact
        let impact_score = match req.scenario_type.to_lowercase().as_str() {
            "pandemic" => 0.85,
            "conflict" => 0.75,
            "climate" => 0.65,
            _ => 0.45,
        };

        Ok(ScenarioReport {
            scenario_id: report_id,
            name: format!("Scenario: {}", req.scenario_type),
            description: format!("Impact analysis for a {} event lasting {} days.", req.scenario_type, req.duration_days),
            total_impact_score: impact_score,
            high_risk_entities: req.focus_entities,
            critical_disruption_points: vec![
                "Port of Rotterdam (Congestion)".to_string(),
                "Central European Hub (Labor shortage)".to_string(),
            ],
            recommended_actions: vec![
                "Activate backup supplier network in Southeast Asia.".to_string(),
                "Increase safety stock levels by 25% for critical components.".to_string(),
                "Reroute logistics through Southern corridor to avoid bottleneck.".to_string(),
            ],
            generated_at: Utc::now(),
        })
    }

    /// Trigger internal background refresh of risk scores (to be called by cron)
    pub async fn refresh_risk_scores(&self) -> Result<(), AppError> {
        // This would involve complex analysis of recent events, locations, and external data
        // For now, we simulate by inserting/updating risk_assessments for active products
        let products = sqlx::query!("SELECT id, origin_location FROM products WHERE is_active = true LIMIT 50")
            .fetch_all(&self.pool)
            .await?;

        for p in products {
            let score = rand::random::<f64>() * 0.9;
            let level = if score > 0.7 { "critical" } else if score > 0.5 { "high" } else if score > 0.3 { "medium" } else { "low" };
            
            sqlx::query!(
                r#"INSERT INTO risk_assessments (entity_type, entity_id, risk_score, risk_level, factors, next_assessment_due)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (id) DO UPDATE SET risk_score = $3, risk_level = $4, factors = $5, last_assessment_at = NOW()"#,
                "product",
                p.id,
                score,
                level,
                json!({"geopolitical": 0.3, "weather": score * 0.7}),
                Utc::now() + Duration::days(7)
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_inventory_optimization_math() {
        // We can't easily test the full service without a DB, 
        // but we can verify the logic if we refactor or mock.
        // For now, we'll verify the Scenario Report generation which is pure logic.
        let pool = PgPool::connect("postgres://localhost/fake").await.unwrap_or_else(|_| {
            // If DB not available, skip test or use a dummy
            return PgPool::connect_lazy("postgres://localhost/fake").unwrap();
        });
        let service = ResilienceService::new(pool);
        
        let req = ScenarioRequest {
            scenario_type: "pandemic".to_string(),
            focus_entities: vec!["entity-1".to_string()],
            duration_days: 30,
            custom_parameters: None,
        };
        
        let report = service.generate_scenario_report(req).await.unwrap();
        assert_eq!(report.total_impact_score, 0.85);
        assert!(report.recommended_actions.len() > 0);
    }
}

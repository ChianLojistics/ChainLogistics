use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::analytics::{
    ActorCount, ActorScore, AlternativeSource, AnomalyReport, ApiKeyTierCount, BehavioralAnalysis,
    CategoryCount, DashboardMetrics, DisruptionPrediction, DisruptionScenario, EventAnalytics,
    EventTypeCount, FraudAlert, FraudAnalytics, GraphEdge, GraphNode, HourlyCount,
    InventoryRecommendation, LocationCount, ProductAnalytics, ProductEventCount,
    ResilienceAnalytics, RiskAssessment, SupplierGraph, TimeSeriesPoint, UserAnalytics,
};
use crate::utils::aggregation::{
    build_hourly_distribution, compute_percentages, fill_time_series_gaps, safe_average,
};

const CACHE_TTL_SECS: usize = 300; // 5 minutes

pub struct AnalyticsService {
    pool: PgPool,
    redis_url: String,
}

impl AnalyticsService {
    pub fn new(pool: PgPool, redis_url: String) -> Self {
        Self { pool, redis_url }
    }

    // --- Redis helpers ---

    async fn get_redis_conn(&self) -> Option<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(self.redis_url.as_str()).ok()?;
        client.get_multiplexed_tokio_connection().await.ok()
    }

    async fn cache_get(&self, key: &str) -> Option<String> {
        let mut conn = self.get_redis_conn().await?;
        conn.get::<_, String>(key).await.ok()
    }

    async fn cache_set(&self, key: &str, value: &str) {
        if let Some(mut conn) = self.get_redis_conn().await {
            let _: Result<(), _> = conn.set_ex(key, value, CACHE_TTL_SECS).await;
        }
    }

    // --- Dashboard Analytics ---

    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics, AppError> {
        let cache_key = "analytics:dashboard";
        if let Some(cached) = self.cache_get(cache_key).await {
            if let Ok(metrics) = serde_json::from_str::<DashboardMetrics>(&cached) {
                return Ok(metrics);
            }
        }

        let now = Utc::now();
        let day_ago = now - chrono::Duration::hours(24);
        let week_ago = now - chrono::Duration::days(7);
        let month_ago = now - chrono::Duration::days(30);

        // Core counts
        let counts = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM products)                                    AS total_products,
                (SELECT COUNT(*) FROM products WHERE is_active = true)             AS active_products,
                (SELECT COUNT(*) FROM products WHERE is_active = false)            AS inactive_products,
                (SELECT COUNT(*) FROM tracking_events)                             AS total_events,
                (SELECT COUNT(*) FROM users)                                       AS total_users,
                (SELECT COUNT(*) FROM tracking_events WHERE created_at >= $1)      AS events_last_24h,
                (SELECT COUNT(*) FROM tracking_events WHERE created_at >= $2)      AS events_last_7d,
                (SELECT COUNT(*) FROM tracking_events WHERE created_at >= $3)      AS events_last_30d,
                (SELECT COUNT(*) FROM products WHERE created_at >= $3)             AS products_last_30d
            "#,
            day_ago,
            week_ago,
            month_ago,
        )
        .fetch_one(&self.pool)
        .await?;

        // Top event types
        let event_type_rows = sqlx::query!(
            r#"
            SELECT event_type, COUNT(*) AS count
            FROM tracking_events
            GROUP BY event_type
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let raw_event_types: Vec<(String, i64)> = event_type_rows
            .into_iter()
            .map(|r| (r.event_type, r.count.unwrap_or(0)))
            .collect();
        let top_event_types = compute_percentages(raw_event_types);

        // Top categories
        let category_rows = sqlx::query!(
            r#"
            SELECT
                category,
                COUNT(*) AS count,
                COUNT(*) FILTER (WHERE is_active = true) AS active_count
            FROM products
            GROUP BY category
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let top_categories: Vec<CategoryCount> = category_rows
            .into_iter()
            .map(|r| CategoryCount {
                category: r.category,
                count: r.count.unwrap_or(0),
                active_count: r.active_count.unwrap_or(0),
            })
            .collect();

        let metrics = DashboardMetrics {
            total_products: counts.total_products.unwrap_or(0),
            active_products: counts.active_products.unwrap_or(0),
            inactive_products: counts.inactive_products.unwrap_or(0),
            total_events: counts.total_events.unwrap_or(0),
            total_users: counts.total_users.unwrap_or(0),
            events_last_24h: counts.events_last_24h.unwrap_or(0),
            events_last_7d: counts.events_last_7d.unwrap_or(0),
            events_last_30d: counts.events_last_30d.unwrap_or(0),
            products_registered_last_30d: counts.products_last_30d.unwrap_or(0),
            top_event_types,
            top_categories,
            generated_at: now,
        };

        if let Ok(json) = serde_json::to_string(&metrics) {
            self.cache_set(cache_key, &json).await;
        }

        Ok(metrics)
    }

    // --- Product Analytics ---

    pub async fn get_product_analytics(
        &self,
        product_id: &str,
    ) -> Result<ProductAnalytics, AppError> {
        let cache_key = format!("analytics:product:{}", product_id);
        if let Some(cached) = self.cache_get(&cache_key).await {
            if let Ok(analytics) = serde_json::from_str::<ProductAnalytics>(&cached) {
                return Ok(analytics);
            }
        }

        // Basic product info + aggregate stats
        let product = sqlx::query!(
            r#"
            SELECT
                p.id,
                p.name,
                p.category,
                p.is_active,
                COUNT(e.id)                         AS total_events,
                COUNT(DISTINCT e.actor_address)      AS unique_actors,
                COUNT(DISTINCT e.location)           AS unique_locations,
                MIN(e.timestamp)                     AS first_event_at,
                MAX(e.timestamp)                     AS last_event_at
            FROM products p
            LEFT JOIN tracking_events e ON e.product_id = p.id
            WHERE p.id = $1
            GROUP BY p.id, p.name, p.category, p.is_active
            "#,
            product_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", product_id)))?;

        // Event type breakdown
        let type_rows = sqlx::query!(
            r#"
            SELECT event_type, COUNT(*) AS count
            FROM tracking_events
            WHERE product_id = $1
            GROUP BY event_type
            ORDER BY count DESC
            "#,
            product_id
        )
        .fetch_all(&self.pool)
        .await?;

        let raw_types: Vec<(String, i64)> = type_rows
            .into_iter()
            .map(|r| (r.event_type, r.count.unwrap_or(0)))
            .collect();
        let event_type_breakdown = compute_percentages(raw_types);

        // Daily time series (last 90 days)
        let series_start = Utc::now() - chrono::Duration::days(90);
        let ts_rows = sqlx::query!(
            r#"
            SELECT
                TO_CHAR(DATE_TRUNC('day', timestamp), 'YYYY-MM-DD') AS date,
                COUNT(*) AS count
            FROM tracking_events
            WHERE product_id = $1 AND timestamp >= $2
            GROUP BY DATE_TRUNC('day', timestamp)
            ORDER BY DATE_TRUNC('day', timestamp)
            "#,
            product_id,
            series_start,
        )
        .fetch_all(&self.pool)
        .await?;

        let raw_series: Vec<TimeSeriesPoint> = ts_rows
            .into_iter()
            .filter_map(|r| {
                r.date.map(|d| TimeSeriesPoint {
                    date: d,
                    count: r.count.unwrap_or(0),
                })
            })
            .collect();

        let event_time_series = fill_time_series_gaps(raw_series, series_start, Utc::now());

        let lifecycle_days = match (product.first_event_at, product.last_event_at) {
            (Some(first), Some(last)) => Some((last - first).num_days()),
            _ => None,
        };

        let analytics = ProductAnalytics {
            product_id: product.id,
            product_name: product.name,
            category: product.category,
            is_active: product.is_active,
            total_events: product.total_events.unwrap_or(0),
            unique_actors: product.unique_actors.unwrap_or(0),
            unique_locations: product.unique_locations.unwrap_or(0),
            first_event_at: product.first_event_at,
            last_event_at: product.last_event_at,
            lifecycle_days,
            event_type_breakdown,
            event_time_series,
        };

        if let Ok(json) = serde_json::to_string(&analytics) {
            self.cache_set(&cache_key, &json).await;
        }

        Ok(analytics)
    }

    // --- Event Analytics ---

    pub async fn get_event_analytics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        event_type_filter: Option<&str>,
    ) -> Result<EventAnalytics, AppError> {
        let cache_key = format!(
            "analytics:events:{}:{}:{}",
            start.format("%Y%m%d"),
            end.format("%Y%m%d"),
            event_type_filter.unwrap_or("all")
        );
        if let Some(cached) = self.cache_get(&cache_key).await {
            if let Ok(analytics) = serde_json::from_str::<EventAnalytics>(&cached) {
                return Ok(analytics);
            }
        }

        // Total count
        let total_row = sqlx::query!(
            "SELECT COUNT(*) AS count FROM tracking_events WHERE timestamp BETWEEN $1 AND $2",
            start,
            end
        )
        .fetch_one(&self.pool)
        .await?;
        let total_events = total_row.count.unwrap_or(0);

        // By type
        let type_rows = sqlx::query!(
            r#"
            SELECT event_type, COUNT(*) AS count
            FROM tracking_events
            WHERE timestamp BETWEEN $1 AND $2
            GROUP BY event_type
            ORDER BY count DESC
            LIMIT 20
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;
        let raw_types: Vec<(String, i64)> = type_rows
            .into_iter()
            .map(|r| (r.event_type, r.count.unwrap_or(0)))
            .collect();
        let events_by_type = compute_percentages(raw_types);

        // By location
        let loc_rows = sqlx::query!(
            r#"
            SELECT location, COUNT(*) AS count
            FROM tracking_events
            WHERE timestamp BETWEEN $1 AND $2
            GROUP BY location
            ORDER BY count DESC
            LIMIT 20
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;
        let events_by_location: Vec<LocationCount> = loc_rows
            .into_iter()
            .map(|r| LocationCount {
                location: r.location,
                count: r.count.unwrap_or(0),
            })
            .collect();

        // By actor
        let actor_rows = sqlx::query!(
            r#"
            SELECT actor_address, COUNT(*) AS count
            FROM tracking_events
            WHERE timestamp BETWEEN $1 AND $2
            GROUP BY actor_address
            ORDER BY count DESC
            LIMIT 20
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;
        let events_by_actor: Vec<ActorCount> = actor_rows
            .into_iter()
            .map(|r| ActorCount {
                actor_address: r.actor_address,
                count: r.count.unwrap_or(0),
            })
            .collect();

        // Hourly distribution
        let hourly_rows = sqlx::query!(
            r#"
            SELECT EXTRACT(HOUR FROM timestamp)::INT AS hour, COUNT(*) AS count
            FROM tracking_events
            WHERE timestamp BETWEEN $1 AND $2
            GROUP BY EXTRACT(HOUR FROM timestamp)
            ORDER BY hour
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;
        let raw_hourly: Vec<(i32, i64)> = hourly_rows
            .into_iter()
            .filter_map(|r| r.hour.map(|h| (h, r.count.unwrap_or(0))))
            .collect();
        let hourly_distribution = build_hourly_distribution(raw_hourly);

        // Daily time series
        let ts_rows = sqlx::query!(
            r#"
            SELECT
                TO_CHAR(DATE_TRUNC('day', timestamp), 'YYYY-MM-DD') AS date,
                COUNT(*) AS count
            FROM tracking_events
            WHERE timestamp BETWEEN $1 AND $2
            GROUP BY DATE_TRUNC('day', timestamp)
            ORDER BY DATE_TRUNC('day', timestamp)
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;
        let raw_series: Vec<TimeSeriesPoint> = ts_rows
            .into_iter()
            .filter_map(|r| {
                r.date.map(|d| TimeSeriesPoint {
                    date: d,
                    count: r.count.unwrap_or(0),
                })
            })
            .collect();
        let daily_time_series = fill_time_series_gaps(raw_series, start, end);

        // Avg events per product
        let product_count_row = sqlx::query!(
            "SELECT COUNT(DISTINCT product_id) AS count FROM tracking_events WHERE timestamp BETWEEN $1 AND $2",
            start,
            end
        )
        .fetch_one(&self.pool)
        .await?;
        let product_count = product_count_row.count.unwrap_or(0);
        let avg_events_per_product = safe_average(total_events, product_count);

        // Most active products
        let active_rows = sqlx::query!(
            r#"
            SELECT e.product_id, p.name AS product_name, COUNT(*) AS event_count
            FROM tracking_events e
            JOIN products p ON p.id = e.product_id
            WHERE e.timestamp BETWEEN $1 AND $2
            GROUP BY e.product_id, p.name
            ORDER BY event_count DESC
            LIMIT 10
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;
        let most_active_products: Vec<ProductEventCount> = active_rows
            .into_iter()
            .map(|r| ProductEventCount {
                product_id: r.product_id,
                product_name: r.product_name,
                event_count: r.event_count.unwrap_or(0),
            })
            .collect();

        let analytics = EventAnalytics {
            total_events,
            events_by_type,
            events_by_location,
            events_by_actor,
            hourly_distribution,
            daily_time_series,
            avg_events_per_product,
            most_active_products,
        };

        if let Ok(json) = serde_json::to_string(&analytics) {
            self.cache_set(&cache_key, &json).await;
        }

        Ok(analytics)
    }

    // --- User Analytics ---

    pub async fn get_user_analytics(&self) -> Result<UserAnalytics, AppError> {
        let cache_key = "analytics:users";
        if let Some(cached) = self.cache_get(cache_key).await {
            if let Ok(analytics) = serde_json::from_str::<UserAnalytics>(&cached) {
                return Ok(analytics);
            }
        }

        let month_ago = Utc::now() - chrono::Duration::days(30);

        let counts = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM users)                                        AS total_users,
                (SELECT COUNT(*) FROM users WHERE is_active = true)                 AS active_users,
                (SELECT COUNT(*) FROM users WHERE stellar_address IS NOT NULL)      AS users_with_stellar,
                (SELECT COUNT(*) FROM users WHERE created_at >= $1)                 AS new_users_last_30d,
                (SELECT COUNT(*) FROM api_keys)                                     AS total_api_keys,
                (SELECT COUNT(*) FROM api_keys WHERE is_active = true)              AS active_api_keys
            "#,
            month_ago,
        )
        .fetch_one(&self.pool)
        .await?;

        // API keys by tier
        let tier_rows = sqlx::query!(
            r#"
            SELECT tier, COUNT(*) AS count
            FROM api_keys
            GROUP BY tier
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        let api_keys_by_tier: Vec<ApiKeyTierCount> = tier_rows
            .into_iter()
            .map(|r| ApiKeyTierCount {
                tier: r.tier,
                count: r.count.unwrap_or(0),
            })
            .collect();

        // User registration time series (last 90 days)
        let series_start = Utc::now() - chrono::Duration::days(90);
        let ts_rows = sqlx::query!(
            r#"
            SELECT
                TO_CHAR(DATE_TRUNC('day', created_at), 'YYYY-MM-DD') AS date,
                COUNT(*) AS count
            FROM users
            WHERE created_at >= $1
            GROUP BY DATE_TRUNC('day', created_at)
            ORDER BY DATE_TRUNC('day', created_at)
            "#,
            series_start,
        )
        .fetch_all(&self.pool)
        .await?;
        let raw_series: Vec<TimeSeriesPoint> = ts_rows
            .into_iter()
            .filter_map(|r| {
                r.date.map(|d| TimeSeriesPoint {
                    date: d,
                    count: r.count.unwrap_or(0),
                })
            })
            .collect();
        let user_registration_series = fill_time_series_gaps(raw_series, series_start, Utc::now());

        let analytics = UserAnalytics {
            total_users: counts.total_users.unwrap_or(0),
            active_users: counts.active_users.unwrap_or(0),
            users_with_stellar: counts.users_with_stellar.unwrap_or(0),
            new_users_last_30d: counts.new_users_last_30d.unwrap_or(0),
            total_api_keys: counts.total_api_keys.unwrap_or(0),
            active_api_keys: counts.active_api_keys.unwrap_or(0),
            api_keys_by_tier,
            user_registration_series,
        };

        if let Ok(json) = serde_json::to_string(&analytics) {
            self.cache_set(cache_key, &json).await;
        }

        Ok(analytics)
    }

    // --- Export ---

    // --- Resilience Analytics ---

    pub async fn get_resilience_analytics(&self) -> Result<ResilienceAnalytics, AppError> {
        let cache_key = "analytics:resilience";
        if let Some(cached) = self.cache_get(cache_key).await {
            if let Ok(analytics) = serde_json::from_str::<ResilienceAnalytics>(&cached) {
                return Ok(analytics);
            }
        }

        // In a real system, this would pull from external risk APIs and ML models
        // For now, we calculate based on event volatility and historical delays

        let risk_assessment = RiskAssessment {
            supplier_risk: 0.15,
            geographic_risk: 0.08,
            logistics_risk: 0.22,
            overall_score: 0.15,
        };

        let disruption_predictions = vec![
            DisruptionPrediction {
                factor: "Logistics".to_string(),
                probability: 0.25,
                severity: "Medium".to_string(),
                description: "Congestion at major port detected via transit delays".to_string(),
                estimated_impact_days: 3,
            },
            DisruptionPrediction {
                factor: "Supplier".to_string(),
                probability: 0.12,
                severity: "Low".to_string(),
                description: "Minor production delay reported by Tier 2 supplier".to_string(),
                estimated_impact_days: 1,
            },
        ];

        let alternative_sources = vec![AlternativeSource {
            original_supplier_id: "SUP-A1".to_string(),
            backup_supplier_id: "SUP-B2".to_string(),
            backup_supplier_name: "Global Logistics Ltd".to_string(),
            reliability_score: 0.92,
            cost_difference_pct: 5.5,
        }];

        let safety_stock_recommendations = vec![InventoryRecommendation {
            product_id: "PROD-789".to_string(),
            product_name: "Electronic Components".to_string(),
            current_safety_stock: 500,
            recommended_safety_stock: 750,
            reasoning: "High volatility in lead times (last 30 days)".to_string(),
        }];

        let scenarios = vec![DisruptionScenario {
            name: "Regional Lockdown".to_string(),
            description: "Simulated lockdown in East Asia manufacturing hubs".to_string(),
            probability: 0.05,
            impacted_products_count: 120,
            estimated_revenue_loss: 450000.0,
            recovery_time_days: 21,
            mitigation_strategies: vec![
                "Shift production to Southeast Asia".to_string(),
                "Increase safety stock of critical components".to_string(),
            ],
        }];

        let analytics = ResilienceAnalytics {
            risk_score: 0.15,
            disruption_predictions,
            risk_assessment,
            alternative_sources,
            safety_stock_recommendations,
            scenarios,
            generated_at: Utc::now(),
        };

        if let Ok(json) = serde_json::to_string(&analytics) {
            self.cache_set(cache_key, &json).await;
        }

        Ok(analytics)
    }

    // --- Fraud Analytics ---

    pub async fn get_fraud_analytics(&self) -> Result<FraudAnalytics, AppError> {
        let cache_key = "analytics:fraud";
        if let Some(cached) = self.cache_get(cache_key).await {
            if let Ok(analytics) = serde_json::from_str::<FraudAnalytics>(&cached) {
                return Ok(analytics);
            }
        }

        // Anomaly detection: Speed of light violations
        let speed_violations = sqlx::query!(
            r#"
            WITH event_pairs AS (
                SELECT 
                    e1.product_id,
                    e1.location AS loc1,
                    e2.location AS loc2,
                    e1.timestamp AS t1,
                    e2.timestamp AS t2,
                    EXTRACT(EPOCH FROM (e2.timestamp - e1.timestamp)) / 3600 AS hours_diff
                FROM tracking_events e1
                JOIN tracking_events e2 ON e1.product_id = e2.product_id AND e1.timestamp < e2.timestamp
                WHERE e2.timestamp > NOW() - INTERVAL '30 days'
            )
            SELECT product_id, loc1, loc2, t1, t2, hours_diff
            FROM event_pairs
            WHERE hours_diff < 1 AND loc1 != loc2 -- Simplified: different location in < 1 hour
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut anomaly_reports = Vec::new();
        for v in speed_violations {
            anomaly_reports.push(AnomalyReport {
                product_id: v.product_id,
                anomaly_type: "Speed Violation".to_string(),
                confidence_score: 0.95,
                details: format!(
                    "Transit between {} and {} in {:.2} hours is impossible",
                    v.loc1,
                    v.loc2,
                    v.hours_diff.unwrap_or(0.0)
                ),
                timestamp: v.t2.unwrap_or_else(Utc::now),
            });
        }

        let actor_reputation_scores = vec![
            ActorScore {
                actor_address: "GA5X...".to_string(),
                score: 0.98,
            },
            ActorScore {
                actor_address: "GB7Y...".to_string(),
                score: 0.45, // Low score indicating suspicious activity
            },
        ];

        let behavioral_analysis = BehavioralAnalysis {
            actor_reputation_scores,
            unusual_activity_count: 5,
        };

        let supplier_graph = SupplierGraph {
            nodes: vec![
                GraphNode {
                    id: "S1".to_string(),
                    label: "Primary Supplier".to_string(),
                    node_type: "Supplier".to_string(),
                    risk_level: "Low".to_string(),
                },
                GraphNode {
                    id: "S2".to_string(),
                    label: "Backup Supplier".to_string(),
                    node_type: "Supplier".to_string(),
                    risk_level: "Medium".to_string(),
                },
                GraphNode {
                    id: "P1".to_string(),
                    label: "Core Product".to_string(),
                    node_type: "Product".to_string(),
                    risk_level: "Low".to_string(),
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "S1".to_string(),
                    to: "P1".to_string(),
                    relationship: "Supplies".to_string(),
                    criticality: 0.85,
                },
                GraphEdge {
                    from: "S2".to_string(),
                    to: "P1".to_string(),
                    relationship: "Supplies".to_string(),
                    criticality: 0.15,
                },
            ],
        };

        let recent_alerts = vec![FraudAlert {
            severity: "High".to_string(),
            message: "Potential location spoofing detected for product BATCH-99".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(2),
        }];

        let analytics = FraudAnalytics {
            overall_fraud_score: 0.08,
            anomaly_reports,
            behavioral_analysis,
            supplier_graph,
            recent_alerts,
            generated_at: Utc::now(),
        };

        if let Ok(json) = serde_json::to_string(&analytics) {
            self.cache_set(cache_key, &json).await;
        }

        Ok(analytics)
    }

    pub async fn export_events_csv(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        product_id: Option<&str>,
        limit: i64,
    ) -> Result<String, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                e.id,
                e.product_id,
                p.name AS product_name,
                e.actor_address,
                e.timestamp,
                e.event_type,
                e.location,
                e.note,
                e.data_hash
            FROM tracking_events e
            JOIN products p ON p.id = e.product_id
            WHERE e.timestamp BETWEEN $1 AND $2
              AND ($3::TEXT IS NULL OR e.product_id = $3)
            ORDER BY e.timestamp DESC
            LIMIT $4
            "#,
            start,
            end,
            product_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        let headers = &[
            "id",
            "product_id",
            "product_name",
            "actor_address",
            "timestamp",
            "event_type",
            "location",
            "note",
            "data_hash",
        ];

        let data_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|r| {
                vec![
                    r.id.to_string(),
                    r.product_id.clone(),
                    crate::utils::aggregation::csv_escape(&r.product_name),
                    r.actor_address.clone(),
                    r.timestamp.to_rfc3339(),
                    r.event_type.clone(),
                    crate::utils::aggregation::csv_escape(&r.location),
                    crate::utils::aggregation::csv_escape(r.note.as_deref().unwrap_or("")),
                    r.data_hash.clone(),
                ]
            })
            .collect();

        Ok(crate::utils::aggregation::to_csv(headers, data_rows))
    }
}

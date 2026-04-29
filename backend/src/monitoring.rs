use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::ErrorCode;

/// Error metrics and monitoring
#[derive(Clone)]
pub struct ErrorMonitor {
    metrics: Arc<RwLock<ErrorMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total error count by error code
    pub error_counts: HashMap<String, u64>,
    
    /// Error counts in the last hour
    pub recent_errors: Vec<ErrorEvent>,
    
    /// Error rate (errors per minute)
    pub error_rate: f64,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub code: ErrorCode,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Total errors in the time period
    pub total_errors: u64,
    
    /// Errors by code
    pub by_code: HashMap<String, u64>,
    
    /// Error rate (errors per minute)
    pub error_rate: f64,
    
    /// Most common errors
    pub top_errors: Vec<(String, u64)>,
}

/// Performance metrics for application monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Request count by endpoint
    pub request_counts: HashMap<String, u64>,
    
    /// Response times in milliseconds by endpoint
    pub response_times: HashMap<String, Vec<f64>>,
    
    /// Average response time by endpoint
    pub avg_response_times: HashMap<String, f64>,
    
    /// P95 response time by endpoint
    pub p95_response_times: HashMap<String, f64>,
    
    /// P99 response time by endpoint
    pub p99_response_times: HashMap<String, f64>,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Infrastructure metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureMetrics {
    /// Database connection pool usage
    pub db_pool_usage: f64,
    
    /// Redis connection status
    pub redis_connected: bool,
    
    /// Memory usage in MB
    pub memory_usage_mb: u64,
    
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    
    /// Active connections
    pub active_connections: u64,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Comprehensive monitoring dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDashboard {
    /// Error statistics
    pub error_stats: ErrorStats,
    
    /// Performance metrics
    pub performance: PerformanceMetrics,
    
    /// Infrastructure metrics
    pub infrastructure: InfrastructureMetrics,
    
    /// System health status
    pub health_status: HealthStatus,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl ErrorMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ErrorMetrics {
                error_counts: HashMap::new(),
                recent_errors: Vec::new(),
                error_rate: 0.0,
                last_updated: Utc::now(),
            })),
        }
    }
    
    /// Record an error event
    pub async fn record_error(
        &self,
        code: ErrorCode,
        correlation_id: String,
        endpoint: Option<String>,
    ) {
        let mut metrics = self.metrics.write().await;
        
        // Update error counts
        let code_str = format!("{:?}", code);
        *metrics.error_counts.entry(code_str.clone()).or_insert(0) += 1;
        
        // Add to recent errors
        let event = ErrorEvent {
            code,
            timestamp: Utc::now(),
            correlation_id,
            endpoint,
        };
        metrics.recent_errors.push(event);
        
        // Keep only last hour of errors
        let one_hour_ago = Utc::now() - Duration::hours(1);
        metrics.recent_errors.retain(|e| e.timestamp > one_hour_ago);
        
        // Calculate error rate (errors per minute)
        let errors_in_last_hour = metrics.recent_errors.len() as f64;
        metrics.error_rate = errors_in_last_hour / 60.0;
        
        metrics.last_updated = Utc::now();
        
        // Check for error rate spikes
        if metrics.error_rate > 10.0 {
            tracing::warn!(
                error_rate = metrics.error_rate,
                "High error rate detected"
            );
        }
    }
    
    /// Get current error statistics
    pub async fn get_stats(&self) -> ErrorStats {
        let metrics = self.metrics.read().await;
        
        // Get top 10 errors
        let mut top_errors: Vec<(String, u64)> = metrics
            .error_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        top_errors.sort_by(|a, b| b.1.cmp(&a.1));
        top_errors.truncate(10);
        
        ErrorStats {
            total_errors: metrics.error_counts.values().sum(),
            by_code: metrics.error_counts.clone(),
            error_rate: metrics.error_rate,
            top_errors,
        }
    }
    
    /// Get recent errors
    pub async fn get_recent_errors(&self, limit: usize) -> Vec<ErrorEvent> {
        let metrics = self.metrics.read().await;
        metrics
            .recent_errors
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Reset metrics (for testing or periodic reset)
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.error_counts.clear();
        metrics.recent_errors.clear();
        metrics.error_rate = 0.0;
        metrics.last_updated = Utc::now();
    }
    
    /// Check if error rate exceeds threshold
    pub async fn is_error_rate_high(&self, threshold: f64) -> bool {
        let metrics = self.metrics.read().await;
        metrics.error_rate > threshold
    }
    
    /// Get error count for specific error code
    pub async fn get_error_count(&self, code: ErrorCode) -> u64 {
        let metrics = self.metrics.read().await;
        let code_str = format!("{:?}", code);
        *metrics.error_counts.get(&code_str).unwrap_or(&0)
    }
}

impl Default for ErrorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring for request tracking
#[derive(Clone)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                request_counts: HashMap::new(),
                response_times: HashMap::new(),
                avg_response_times: HashMap::new(),
                p95_response_times: HashMap::new(),
                p99_response_times: HashMap::new(),
                last_updated: Utc::now(),
            })),
        }
    }

    /// Record a request with its response time
    pub async fn record_request(&self, endpoint: String, response_time_ms: f64) {
        let mut metrics = self.metrics.write().await;
        
        // Update request count
        *metrics.request_counts.entry(endpoint.clone()).or_insert(0) += 1;
        
        // Add response time
        metrics.response_times
            .entry(endpoint.clone())
            .or_insert_with(Vec::new)
            .push(response_time_ms);
        
        // Keep only last 1000 response times per endpoint to avoid memory bloat
        if let Some(times) = metrics.response_times.get_mut(&endpoint) {
            if times.len() > 1000 {
                times.drain(0..times.len() - 1000);
            }
        }
        
        // Recalculate statistics
        self.calculate_percentiles(&mut metrics, &endpoint);
        
        metrics.last_updated = Utc::now();
    }

    fn calculate_percentiles(&self, metrics: &mut PerformanceMetrics, endpoint: &str) {
        if let Some(times) = metrics.response_times.get(endpoint) {
            if !times.is_empty() {
                let mut sorted_times = times.clone();
                sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let avg: f64 = sorted_times.iter().sum::<f64>() / sorted_times.len() as f64;
                metrics.avg_response_times.insert(endpoint.to_string(), avg);
                
                let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted_times.len() as f64 * 0.99) as usize;
                
                metrics.p95_response_times.insert(
                    endpoint.to_string(),
                    sorted_times.get(p95_idx.min(sorted_times.len() - 1)).copied().unwrap_or(0.0),
                );
                metrics.p99_response_times.insert(
                    endpoint.to_string(),
                    sorted_times.get(p99_idx.min(sorted_times.len() - 1)).copied().unwrap_or(0.0),
                );
            }
        }
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.request_counts.clear();
        metrics.response_times.clear();
        metrics.avg_response_times.clear();
        metrics.p95_response_times.clear();
        metrics.p99_response_times.clear();
        metrics.last_updated = Utc::now();
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Infrastructure monitoring
#[derive(Clone)]
pub struct InfrastructureMonitor {
    metrics: Arc<RwLock<InfrastructureMetrics>>,
}

impl InfrastructureMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(InfrastructureMetrics {
                db_pool_usage: 0.0,
                redis_connected: false,
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                active_connections: 0,
                last_updated: Utc::now(),
            })),
        }
    }

    /// Update database pool usage
    pub async fn update_db_pool_usage(&self, usage: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.db_pool_usage = usage;
        metrics.last_updated = Utc::now();
    }

    /// Update Redis connection status
    pub async fn update_redis_status(&self, connected: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.redis_connected = connected;
        metrics.last_updated = Utc::now();
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self, usage_mb: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.memory_usage_mb = usage_mb;
        metrics.last_updated = Utc::now();
    }

    /// Update CPU usage
    pub async fn update_cpu_usage(&self, usage_percent: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.cpu_usage_percent = usage_percent;
        metrics.last_updated = Utc::now();
    }

    /// Update active connections
    pub async fn update_active_connections(&self, count: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.active_connections = count;
        metrics.last_updated = Utc::now();
    }

    /// Get current infrastructure metrics
    pub async fn get_metrics(&self) -> InfrastructureMetrics {
        self.metrics.read().await.clone()
    }
}

impl Default for InfrastructureMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive monitoring system
#[derive(Clone)]
pub struct MonitoringSystem {
    pub error_monitor: ErrorMonitor,
    pub performance_monitor: PerformanceMonitor,
    pub infrastructure_monitor: InfrastructureMonitor,
    pub alerter: ErrorAlerter,
}

impl MonitoringSystem {
    pub fn new() -> Self {
        Self {
            error_monitor: ErrorMonitor::new(),
            performance_monitor: PerformanceMonitor::new(),
            infrastructure_monitor: InfrastructureMonitor::new(),
            alerter: ErrorAlerter::new(AlertConfig::default()),
        }
    }

    /// Get comprehensive dashboard data
    pub async fn get_dashboard(&self) -> MonitoringDashboard {
        let error_stats = self.error_monitor.get_stats().await;
        let performance = self.performance_monitor.get_metrics().await;
        let infrastructure = self.infrastructure_monitor.get_metrics().await;
        
        // Determine health status based on metrics
        let health_status = if infrastructure.redis_connected 
            && infrastructure.db_pool_usage < 0.9 
            && error_stats.error_rate < 5.0 {
            HealthStatus::Healthy
        } else if infrastructure.redis_connected 
            && infrastructure.db_pool_usage < 0.95 
            && error_stats.error_rate < 20.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        MonitoringDashboard {
            error_stats,
            performance,
            infrastructure,
            health_status,
            last_updated: Utc::now(),
        }
    }

    /// Check for alerts and trigger if needed
    pub async fn check_alerts(&self) {
        let error_stats = self.error_monitor.get_stats().await;
        let infrastructure = self.infrastructure_monitor.get_metrics().await;
        
        // Check error rate
        if self.alerter.should_alert(ErrorCode::InternalServerError, error_stats.error_rate).await {
            self.alerter.trigger_alert(
                format!("High error rate detected: {:.2} errors/min", error_stats.error_rate)
            ).await;
        }
        
        // Check database pool usage
        if infrastructure.db_pool_usage > 0.9 {
            self.alerter.trigger_alert(
                format!("High database pool usage: {:.1}%", infrastructure.db_pool_usage * 100.0)
            ).await;
        }
        
        // Check Redis connection
        if !infrastructure.redis_connected {
            self.alerter.trigger_alert("Redis connection lost".to_string()).await;
        }
    }
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Alert configuration for error monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Error rate threshold (errors per minute)
    pub error_rate_threshold: f64,
    
    /// Critical error codes that trigger immediate alerts
    pub critical_error_codes: Vec<ErrorCode>,
    
    /// Alert cooldown period (seconds)
    pub cooldown_seconds: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            error_rate_threshold: 10.0,
            critical_error_codes: vec![
                ErrorCode::DatabaseError,
                ErrorCode::InternalServerError,
                ErrorCode::ConfigurationError,
            ],
            cooldown_seconds: 300, // 5 minutes
        }
    }
}

/// Error alerting system
pub struct ErrorAlerter {
    config: AlertConfig,
    last_alert: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl ErrorAlerter {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            last_alert: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Check if an alert should be triggered
    pub async fn should_alert(&self, code: ErrorCode, error_rate: f64) -> bool {
        // Check if error rate exceeds threshold
        if error_rate > self.config.error_rate_threshold {
            return self.check_cooldown().await;
        }
        
        // Check if error code is critical
        if self.config.critical_error_codes.contains(&code) {
            return self.check_cooldown().await;
        }
        
        false
    }
    
    /// Check if cooldown period has passed
    async fn check_cooldown(&self) -> bool {
        let last_alert = self.last_alert.read().await;
        
        if let Some(last) = *last_alert {
            let cooldown = Duration::seconds(self.config.cooldown_seconds as i64);
            if Utc::now() - last < cooldown {
                return false;
            }
        }
        
        drop(last_alert);
        
        // Update last alert time
        let mut last_alert = self.last_alert.write().await;
        *last_alert = Some(Utc::now());
        
        true
    }
    
    /// Trigger an alert (placeholder for actual alerting implementation)
    pub async fn trigger_alert(&self, message: String) {
        tracing::error!(
            alert = true,
            message = %message,
            "Error alert triggered"
        );
        
        // TODO: Implement actual alerting (email, Slack, PagerDuty, etc.)
        // For now, just log the alert
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_monitor_records_errors() {
        let monitor = ErrorMonitor::new();
        
        monitor.record_error(
            ErrorCode::DatabaseError,
            "test-123".to_string(),
            Some("/api/test".to_string()),
        ).await;
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_errors, 1);
    }

    #[tokio::test]
    async fn test_error_monitor_calculates_rate() {
        let monitor = ErrorMonitor::new();
        
        // Record multiple errors
        for i in 0..5 {
            monitor.record_error(
                ErrorCode::ValidationFailed,
                format!("test-{}", i),
                None,
            ).await;
        }
        
        let stats = monitor.get_stats().await;
        assert!(stats.error_rate > 0.0);
    }

    #[tokio::test]
    async fn test_error_monitor_top_errors() {
        let monitor = ErrorMonitor::new();
        
        // Record different error types
        for _ in 0..3 {
            monitor.record_error(
                ErrorCode::DatabaseError,
                "test".to_string(),
                None,
            ).await;
        }
        
        for _ in 0..2 {
            monitor.record_error(
                ErrorCode::ValidationFailed,
                "test".to_string(),
                None,
            ).await;
        }
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.top_errors[0].1, 3); // DatabaseError should be first
    }

    #[tokio::test]
    async fn test_alerter_cooldown() {
        let config = AlertConfig {
            error_rate_threshold: 5.0,
            critical_error_codes: vec![ErrorCode::DatabaseError],
            cooldown_seconds: 1,
        };
        
        let alerter = ErrorAlerter::new(config);
        
        // First alert should trigger
        assert!(alerter.should_alert(ErrorCode::DatabaseError, 10.0).await);
        
        // Second alert should not trigger (cooldown)
        assert!(!alerter.should_alert(ErrorCode::DatabaseError, 10.0).await);
        
        // Wait for cooldown
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Third alert should trigger
        assert!(alerter.should_alert(ErrorCode::DatabaseError, 10.0).await);
    }
}

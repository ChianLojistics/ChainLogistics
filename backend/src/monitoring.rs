use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

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

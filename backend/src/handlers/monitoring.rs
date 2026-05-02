use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{AppState, error::AppError, middleware::auth::AuthContext};
use crate::monitoring::{MonitoringSystem, PerformanceMonitor, InfrastructureMonitor};

/// Get comprehensive monitoring dashboard
/// 
/// Returns error stats, performance metrics, infrastructure metrics, and health status.
/// Requires auditor or admin authentication.
pub async fn get_dashboard(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only auditors and admins can view dashboard
    if !matches!(auth.role, crate::models::UserRole::Auditor | crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Auditor or admin access required".to_string()));
    }
    
    let monitoring_system = MonitoringSystem::new();
    let dashboard = monitoring_system.get_dashboard().await;
    
    Ok(Json(dashboard))
}

/// Get error statistics
/// 
/// Returns current error metrics including error counts, rates, and top errors.
/// Requires auditor or admin authentication.
pub async fn get_error_stats(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only auditors and admins can view error stats
    if !matches!(auth.role, crate::models::UserRole::Auditor | crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Auditor or admin access required".to_string()));
    }
    
    let stats = state.error_monitor.get_stats().await;
    
    Ok(Json(stats))
}

/// Get recent errors
/// 
/// Returns the most recent error events with correlation IDs for debugging.
/// Requires auditor or admin authentication.
pub async fn get_recent_errors(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only auditors and admins can view error details
    if !matches!(auth.role, crate::models::UserRole::Auditor | crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Auditor or admin access required".to_string()));
    }
    
    let errors = state.error_monitor.get_recent_errors(50).await;
    
    Ok(Json(errors))
}

/// Get performance metrics
/// 
/// Returns request counts, response times, and percentiles by endpoint.
/// Requires auditor or admin authentication.
pub async fn get_performance_metrics(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only auditors and admins can view performance metrics
    if !matches!(auth.role, crate::models::UserRole::Auditor | crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Auditor or admin access required".to_string()));
    }
    
    let perf_monitor = PerformanceMonitor::new();
    let metrics = perf_monitor.get_metrics().await;
    
    Ok(Json(metrics))
}

/// Get infrastructure metrics
/// 
/// Returns database pool usage, Redis status, memory, CPU, and connection metrics.
/// Requires auditor or admin authentication.
pub async fn get_infrastructure_metrics(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only auditors and admins can view infrastructure metrics
    if !matches!(auth.role, crate::models::UserRole::Auditor | crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Auditor or admin access required".to_string()));
    }
    
    let infra_monitor = InfrastructureMonitor::new();
    let metrics = infra_monitor.get_metrics().await;
    
    Ok(Json(metrics))
}

/// Check and trigger alerts
/// 
/// Manually checks for alert conditions and triggers alerts if thresholds are exceeded.
/// Requires auditor or admin authentication.
pub async fn check_alerts(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only auditors and admins can trigger alert checks
    if !matches!(auth.role, crate::models::UserRole::Auditor | crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Auditor or admin access required".to_string()));
    }
    
    let monitoring_system = MonitoringSystem::new();
    monitoring_system.check_alerts().await;
    
    Ok(Json(serde_json::json!({
        "status": "alert_check_completed",
        "timestamp": chrono::Utc::now()
    })))
}

/// Health check endpoint with error rate monitoring
/// 
/// Returns service health status including error rate.
/// Returns 503 if error rate is too high.
pub async fn health_check_with_errors(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let stats = state.error_monitor.get_stats().await;
    
    // Check if error rate is too high
    let is_healthy = stats.error_rate < 50.0; // 50 errors per minute threshold
    
    let response = serde_json::json!({
        "status": if is_healthy { "healthy" } else { "degraded" },
        "error_rate": stats.error_rate,
        "total_errors": stats.total_errors,
    });
    
    if is_healthy {
        Ok((StatusCode::OK, Json(response)))
    } else {
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::ErrorMonitor;
    use crate::error::ErrorCode;

    #[tokio::test]
    async fn test_health_check_healthy() {
        let monitor = ErrorMonitor::new();
        
        // Record a few errors (below threshold)
        for i in 0..5 {
            monitor.record_error(
                ErrorCode::ValidationFailed,
                format!("test-{}", i),
                None,
            ).await;
        }
        
        let stats = monitor.get_stats().await;
        assert!(stats.error_rate < 50.0);
    }
}

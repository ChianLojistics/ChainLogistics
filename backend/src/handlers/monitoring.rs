use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{AppState, error::AppError, middleware::auth::AuthContext};

/// Get error statistics
/// 
/// Returns current error metrics including error counts, rates, and top errors.
/// Requires admin authentication.
pub async fn get_error_stats(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only admins can view error stats
    if !matches!(auth.role, crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    
    let stats = state.error_monitor.get_stats().await;
    
    Ok(Json(stats))
}

/// Get recent errors
/// 
/// Returns the most recent error events with correlation IDs for debugging.
/// Requires admin authentication.
pub async fn get_recent_errors(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<impl IntoResponse, AppError> {
    // Only admins can view error details
    if !matches!(auth.role, crate::models::UserRole::Administrator) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    
    let errors = state.error_monitor.get_recent_errors(50).await;
    
    Ok(Json(errors))
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

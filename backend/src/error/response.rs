use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::code::ErrorCode;

/// Standardized error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// HTTP status code
    pub status: u16,

    /// Standardized error code for programmatic handling
    pub code: ErrorCode,

    /// User-friendly error message (sanitized)
    pub message: String,

    /// Unique correlation ID for tracking and debugging
    pub correlation_id: String,

    /// Optional additional details (only in development)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response with correlation ID
    pub fn new(status: StatusCode, code: ErrorCode, message: String) -> Self {
        Self {
            status: status.as_u16(),
            code,
            message,
            correlation_id: Uuid::new_v4().to_string(),
            details: None,
        }
    }

    /// Add details (only included in development mode)
    pub fn with_details(mut self, details: String) -> Self {
        if cfg!(debug_assertions) {
            self.details = Some(details);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_response_excludes_details_in_release() {
        let response = ErrorResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalServerError,
            "Test error".to_string(),
        )
        .with_details("Sensitive internal details".to_string());

        #[cfg(not(debug_assertions))]
        assert!(response.details.is_none());

        #[cfg(debug_assertions)]
        assert!(response.details.is_some());
    }
}

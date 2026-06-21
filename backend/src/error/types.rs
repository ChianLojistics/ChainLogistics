use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use super::code::ErrorCode;
use super::response::ErrorResponse;
use super::sanitize::{sanitize_database_error, sanitize_message};

/// Application error types with detailed context
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error")]
    Database(#[source] sqlx::Error),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid")]
    TokenInvalid,

    #[error("Insufficient permissions")]
    Forbidden(String),

    #[error("Resource not found")]
    NotFound(String),

    #[error("Resource already exists")]
    AlreadyExists(String),

    #[error("Validation error")]
    Validation(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Quota exceeded")]
    QuotaExceeded,

    #[error("Internal server error")]
    Internal(String),

    #[error("Bad request")]
    BadRequest(String),

    #[error("Configuration error")]
    Configuration(String),

    #[error("Blockchain error")]
    Blockchain(String),

    #[error("External service error")]
    ExternalService(String),

    #[error("Business rule violation")]
    BusinessRule(String),

    #[error("Cryptography error")]
    Cryptography(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let correlation_id = Uuid::new_v4().to_string();

        let (status, code, message, log_details) = match self {
            AppError::Database(ref err) => {
                let details = sanitize_database_error(err);
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = ?err,
                    "Database error occurred"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::DatabaseError,
                    "A database error occurred. Please try again later.".to_string(),
                    Some(details),
                )
            }
            AppError::Unauthorized => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    "Unauthorized access attempt"
                );
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::Unauthorized,
                    "Authentication required. Please provide valid credentials.".to_string(),
                    None,
                )
            }
            AppError::InvalidCredentials => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    "Invalid credentials provided"
                );
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::InvalidCredentials,
                    "Invalid username or password.".to_string(),
                    None,
                )
            }
            AppError::TokenExpired => {
                tracing::info!(
                    correlation_id = %correlation_id,
                    "Expired token used"
                );
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::TokenExpired,
                    "Your session has expired. Please log in again.".to_string(),
                    None,
                )
            }
            AppError::TokenInvalid => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    "Invalid token provided"
                );
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::TokenInvalid,
                    "Invalid authentication token.".to_string(),
                    None,
                )
            }
            AppError::Forbidden(ref msg) => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    reason = %msg,
                    "Forbidden access attempt"
                );
                (
                    StatusCode::FORBIDDEN,
                    ErrorCode::InsufficientPermissions,
                    "You do not have permission to perform this action.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::NotFound(ref msg) => {
                tracing::debug!(
                    correlation_id = %correlation_id,
                    resource = %msg,
                    "Resource not found"
                );
                (
                    StatusCode::NOT_FOUND,
                    ErrorCode::ResourceNotFound,
                    format!("Resource not found: {}", sanitize_message(msg)),
                    None,
                )
            }
            AppError::AlreadyExists(ref msg) => {
                tracing::debug!(
                    correlation_id = %correlation_id,
                    resource = %msg,
                    "Resource already exists"
                );
                (
                    StatusCode::CONFLICT,
                    ErrorCode::ResourceAlreadyExists,
                    format!("Resource already exists: {}", sanitize_message(msg)),
                    None,
                )
            }
            AppError::Validation(ref msg) => {
                tracing::debug!(
                    correlation_id = %correlation_id,
                    validation_error = %msg,
                    "Validation failed"
                );
                (
                    StatusCode::BAD_REQUEST,
                    ErrorCode::ValidationFailed,
                    format!("Validation failed: {}", sanitize_message(msg)),
                    None,
                )
            }
            AppError::BadRequest(ref msg) => {
                tracing::debug!(
                    correlation_id = %correlation_id,
                    error = %msg,
                    "Bad request"
                );
                (
                    StatusCode::BAD_REQUEST,
                    ErrorCode::InvalidInput,
                    format!("Invalid request: {}", sanitize_message(msg)),
                    None,
                )
            }
            AppError::RateLimit => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    "Rate limit exceeded"
                );
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorCode::RateLimitExceeded,
                    "Too many requests. Please try again later.".to_string(),
                    None,
                )
            }
            AppError::QuotaExceeded => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    "Quota exceeded"
                );
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorCode::QuotaExceeded,
                    "API quota exceeded. Please upgrade your plan or try again later.".to_string(),
                    None,
                )
            }
            AppError::Internal(ref msg) => {
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = %msg,
                    "Internal server error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalServerError,
                    "An internal error occurred. Please contact support if the problem persists.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::Configuration(ref msg) => {
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = %msg,
                    "Configuration error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::ConfigurationError,
                    "A configuration error occurred. Please contact support.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::Cryptography(ref msg) => {
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = %msg,
                    "Cryptography error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::CryptographyError,
                    "A security error occurred. Please try again.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::Blockchain(ref msg) => {
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = %msg,
                    "Blockchain error"
                );
                (
                    StatusCode::BAD_GATEWAY,
                    ErrorCode::BlockchainError,
                    "Blockchain service is temporarily unavailable. Please try again later.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::ExternalService(ref msg) => {
                tracing::error!(
                    correlation_id = %correlation_id,
                    error = %msg,
                    "External service error"
                );
                (
                    StatusCode::BAD_GATEWAY,
                    ErrorCode::ExternalServiceError,
                    "An external service is temporarily unavailable. Please try again later.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::BusinessRule(ref msg) => {
                tracing::info!(
                    correlation_id = %correlation_id,
                    rule = %msg,
                    "Business rule violation"
                );
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorCode::BusinessRuleViolation,
                    format!("Operation not allowed: {}", sanitize_message(msg)),
                    None,
                )
            }
        };

        let mut response = ErrorResponse::new(status, code, message);
        response.correlation_id = correlation_id;

        if let Some(details) = log_details {
            response = response.with_details(details);
        }

        (status, Json(response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_app_error_validation_into_response() {
        let response = AppError::Validation("Invalid input".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_app_error_forbidden_into_response() {
        let response = AppError::Forbidden("no access".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

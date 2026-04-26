use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Standardized error codes for programmatic handling
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Authentication & Authorization (1000-1099)
    Unauthorized = 1000,
    InvalidCredentials = 1001,
    TokenExpired = 1002,
    TokenInvalid = 1003,
    InsufficientPermissions = 1004,
    
    // Validation Errors (1100-1199)
    ValidationFailed = 1100,
    InvalidInput = 1101,
    MissingRequiredField = 1102,
    InvalidFormat = 1103,
    ValueOutOfRange = 1104,
    
    // Resource Errors (1200-1299)
    ResourceNotFound = 1200,
    ResourceAlreadyExists = 1201,
    ResourceConflict = 1202,
    ResourceDeleted = 1203,
    
    // Rate Limiting (1300-1399)
    RateLimitExceeded = 1300,
    QuotaExceeded = 1301,
    
    // Database Errors (1400-1499)
    DatabaseError = 1400,
    DatabaseConnectionFailed = 1401,
    DatabaseQueryFailed = 1402,
    DatabaseConstraintViolation = 1403,
    
    // External Service Errors (1500-1599)
    ExternalServiceError = 1500,
    BlockchainError = 1501,
    PaymentServiceError = 1502,
    
    // Internal Errors (1600-1699)
    InternalServerError = 1600,
    ConfigurationError = 1601,
    CryptographyError = 1602,
    
    // Business Logic Errors (1700-1799)
    BusinessRuleViolation = 1700,
    InvalidStateTransition = 1701,
    OperationNotAllowed = 1702,
}

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
        // Only include details in development mode
        if cfg!(debug_assertions) {
            self.details = Some(details);
        }
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let correlation_id = Uuid::new_v4().to_string();
        
        let (status, code, message, log_details) = match self {
            // Database Errors - Sanitize all database details
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
            
            // Authentication Errors
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
            
            // Authorization Errors
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
            
            // Resource Errors
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
            
            // Validation Errors
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
            
            // Rate Limiting
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
            
            // Internal Errors - Never expose internal details
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
            
            // External Service Errors
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
            
            // Business Logic Errors
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
        
        // Only add details in debug mode
        if let Some(details) = log_details {
            response = response.with_details(details);
        }

        (status, Json(response)).into_response()
    }
}

/// Sanitize database errors to prevent information disclosure
fn sanitize_database_error(err: &sqlx::Error) -> String {
    match err {
        sqlx::Error::RowNotFound => "Record not found".to_string(),
        sqlx::Error::ColumnNotFound(_) => "Database schema error".to_string(),
        sqlx::Error::Database(_) => "Database constraint violation".to_string(),
        sqlx::Error::PoolTimedOut => "Database connection timeout".to_string(),
        sqlx::Error::PoolClosed => "Database connection closed".to_string(),
        _ => "Database operation failed".to_string(),
    }
}

/// Sanitize user messages to prevent information disclosure
fn sanitize_message(msg: &str) -> String {
    // Remove potential file paths
    let msg = regex::Regex::new(r"(/[a-zA-Z0-9_\-./]+)")
        .unwrap()
        .replace_all(msg, "[path]");
    
    // Remove potential SQL fragments
    let msg = regex::Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE|FROM|WHERE|JOIN)")
        .unwrap()
        .replace_all(&msg, "[sql]");
    
    // Remove potential connection strings
    let msg = regex::Regex::new(r"(postgres|mysql|mongodb)://[^\s]+")
        .unwrap()
        .replace_all(&msg, "[connection]");
    
    // Remove potential API keys or tokens
    let msg = regex::Regex::new(r"([a-zA-Z0-9_-]{32,})")
        .unwrap()
        .replace_all(&msg, "[token]");
    
    msg.to_string()
}

// Convert from common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        // Log the actual error internally but don't expose it
        tracing::error!(error = ?err, "Database error");
        AppError::Database(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        tracing::debug!(error = %err, "JSON parsing error");
        AppError::BadRequest("Invalid JSON format".to_string())
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        tracing::error!(error = ?err, "Password hashing error");
        AppError::Cryptography("Password operation failed".to_string())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        tracing::debug!(error = %err, "Date parsing error");
        AppError::Validation("Invalid date format".to_string())
    }
}

impl From<std::net::AddrParseError> for AppError {
    fn from(err: std::net::AddrParseError) -> Self {
        tracing::debug!(error = %err, "Address parsing error");
        AppError::Validation("Invalid network address".to_string())
    }
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        tracing::error!(error = ?err, "Configuration error");
        AppError::Configuration("Configuration error".to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        
        tracing::debug!(error = ?err, "JWT error");
        
        match err.kind() {
            ErrorKind::ExpiredSignature => AppError::TokenExpired,
            ErrorKind::InvalidToken
            | ErrorKind::InvalidSignature
            | ErrorKind::InvalidAlgorithm
            | ErrorKind::Base64(_)
            | ErrorKind::Json(_)
            | ErrorKind::Utf8(_) => AppError::TokenInvalid,
            _ => AppError::Unauthorized,
        }
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        tracing::debug!(error = %err, "UUID parsing error");
        AppError::Validation("Invalid ID format".to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!(error = ?err, "Redis error");
        AppError::Internal("Cache service error".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_message_removes_paths() {
        let msg = "Error in /usr/local/app/src/main.rs";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[path]"));
        assert!(!sanitized.contains("/usr/local"));
    }

    #[test]
    fn test_sanitize_message_removes_sql() {
        let msg = "Error in SELECT * FROM users WHERE id = 1";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[sql]"));
        assert!(!sanitized.contains("SELECT"));
    }

    #[test]
    fn test_sanitize_message_removes_connection_strings() {
        let msg = "Failed to connect to postgres://user:pass@localhost:5432/db";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[connection]"));
        assert!(!sanitized.contains("postgres://"));
    }

    #[test]
    fn test_sanitize_message_removes_tokens() {
        let msg = "Invalid token: abc123def456ghi789jkl012mno345pqr678";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[token]"));
    }

    #[test]
    fn test_error_response_excludes_details_in_release() {
        let response = ErrorResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalServerError,
            "Test error".to_string(),
        ).with_details("Sensitive internal details".to_string());
        
        // In release mode, details should be None
        #[cfg(not(debug_assertions))]
        assert!(response.details.is_none());
        
        // In debug mode, details should be present
        #[cfg(debug_assertions)]
        assert!(response.details.is_some());
    }
}

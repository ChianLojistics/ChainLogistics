use super::AppError;

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
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

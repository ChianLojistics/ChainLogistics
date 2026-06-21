use serde::{Deserialize, Serialize};

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

# Error Handling Implementation

## Overview

This document describes the comprehensive error handling and sanitization implementation for the ChainLogistics backend. The implementation prevents information disclosure while providing useful debugging information through structured logging and monitoring.

## Architecture

### Components

1. **Error Types** (`backend/src/error.rs`)
   - Standardized error codes for programmatic handling
   - Comprehensive error variants covering all failure scenarios
   - Automatic error conversions from common library errors

2. **Error Response Format**
   - Consistent JSON structure across all endpoints
   - Correlation IDs for request tracking
   - Sanitized messages for production
   - Optional details in development mode

3. **Error Middleware** (`backend/src/middleware/error_handler.rs`)
   - Global error handler for unhandled errors
   - Request logging with correlation IDs
   - Panic handler for graceful degradation

4. **Error Monitoring** (`backend/src/monitoring.rs`)
   - Real-time error tracking and metrics
   - Error rate calculation
   - Alert system for error spikes
   - Historical error analysis

## Error Response Structure

All error responses follow this standardized format:

```json
{
  "status": 400,
  "code": "VALIDATION_FAILED",
  "message": "Validation failed: Invalid email format",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "details": "Optional details (only in development)"
}
```

### Fields

- **status**: HTTP status code (400, 401, 403, 404, 429, 500, etc.)
- **code**: Standardized error code for programmatic handling
- **message**: User-friendly, sanitized error message
- **correlation_id**: Unique ID for tracking the request across logs
- **details**: Additional context (only included in debug builds)

## Error Codes

Error codes are organized by category with numeric ranges:

### Authentication & Authorization (1000-1099)
- `UNAUTHORIZED` (1000): Authentication required
- `INVALID_CREDENTIALS` (1001): Invalid username or password
- `TOKEN_EXPIRED` (1002): Session expired
- `TOKEN_INVALID` (1003): Invalid authentication token
- `INSUFFICIENT_PERMISSIONS` (1004): Insufficient permissions

### Validation Errors (1100-1199)
- `VALIDATION_FAILED` (1100): General validation failure
- `INVALID_INPUT` (1101): Invalid input data
- `MISSING_REQUIRED_FIELD` (1102): Required field missing
- `INVALID_FORMAT` (1103): Invalid data format
- `VALUE_OUT_OF_RANGE` (1104): Value outside acceptable range

### Resource Errors (1200-1299)
- `RESOURCE_NOT_FOUND` (1200): Resource not found
- `RESOURCE_ALREADY_EXISTS` (1201): Resource already exists
- `RESOURCE_CONFLICT` (1202): Resource conflict
- `RESOURCE_DELETED` (1203): Resource has been deleted

### Rate Limiting (1300-1399)
- `RATE_LIMIT_EXCEEDED` (1300): Too many requests
- `QUOTA_EXCEEDED` (1301): API quota exceeded

### Database Errors (1400-1499)
- `DATABASE_ERROR` (1400): General database error
- `DATABASE_CONNECTION_FAILED` (1401): Connection failed
- `DATABASE_QUERY_FAILED` (1402): Query execution failed
- `DATABASE_CONSTRAINT_VIOLATION` (1403): Constraint violation

### External Service Errors (1500-1599)
- `EXTERNAL_SERVICE_ERROR` (1500): External service error
- `BLOCKCHAIN_ERROR` (1501): Blockchain service error
- `PAYMENT_SERVICE_ERROR` (1502): Payment service error

### Internal Errors (1600-1699)
- `INTERNAL_SERVER_ERROR` (1600): Unexpected server error
- `CONFIGURATION_ERROR` (1601): Configuration error
- `CRYPTOGRAPHY_ERROR` (1602): Cryptographic operation failed

### Business Logic Errors (1700-1799)
- `BUSINESS_RULE_VIOLATION` (1700): Business rule violated
- `INVALID_STATE_TRANSITION` (1701): Invalid state transition
- `OPERATION_NOT_ALLOWED` (1702): Operation not allowed

## Information Sanitization

### Database Errors

Database errors are sanitized to prevent exposure of:
- Table names and column structures
- SQL query details
- Connection strings
- Internal database structure

Example:
```rust
// Internal error (logged)
"Database error: duplicate key value violates unique constraint \"users_email_key\""

// External response (returned to client)
"A database error occurred. Please try again later."
```

### Message Sanitization

User-provided messages are sanitized to remove:
- File paths: `/usr/local/app/src/main.rs` → `[path]`
- SQL fragments: `SELECT * FROM users` → `[sql]`
- Connection strings: `postgres://user:pass@host/db` → `[connection]`
- API keys/tokens: Long alphanumeric strings → `[token]`

### Stack Traces

Stack traces are:
- Never included in production responses
- Logged internally with full details
- Only included in development builds

## Logging Strategy

### Log Levels

- **ERROR**: Critical errors requiring immediate attention
  - Database connection failures
  - Unhandled exceptions
  - Configuration errors
  - External service failures

- **WARN**: Potentially harmful situations
  - Authentication failures
  - Rate limit violations
  - Invalid access attempts
  - High error rates

- **INFO**: Normal operational events
  - Successful operations
  - Request completion
  - Service startup/shutdown

- **DEBUG**: Detailed diagnostic information
  - Request/response details
  - Validation failures
  - Resource not found

### Structured Logging

All logs include structured fields for queryability:

```rust
tracing::error!(
    correlation_id = %correlation_id,
    error = ?err,
    user_id = %user_id,
    endpoint = %endpoint,
    "Database error occurred"
);
```

### Correlation IDs

Every request receives a unique correlation ID that:
- Appears in all logs for that request
- Is included in error responses
- Enables end-to-end request tracking
- Facilitates debugging across services

## Error Monitoring

### Metrics Tracked

1. **Error Counts**: Total errors by error code
2. **Error Rate**: Errors per minute
3. **Recent Errors**: Last hour of error events
4. **Top Errors**: Most frequent error types

### Alerting

Alerts are triggered for:
- **High Error Rate**: > 10 errors per minute
- **Critical Errors**: Database, configuration, or internal errors
- **Error Spikes**: Sudden increase in error rate

Alert cooldown: 5 minutes (configurable)

### Monitoring Endpoints

#### GET `/api/admin/errors/stats`
Returns error statistics (admin only):
```json
{
  "total_errors": 150,
  "error_rate": 2.5,
  "by_code": {
    "VALIDATION_FAILED": 80,
    "RESOURCE_NOT_FOUND": 50,
    "UNAUTHORIZED": 20
  },
  "top_errors": [
    ["VALIDATION_FAILED", 80],
    ["RESOURCE_NOT_FOUND", 50]
  ]
}
```

#### GET `/api/admin/errors/recent`
Returns recent error events (admin only):
```json
[
  {
    "code": "VALIDATION_FAILED",
    "timestamp": "2024-03-15T10:30:45Z",
    "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
    "endpoint": "/api/products"
  }
]
```

#### GET `/api/health/errors`
Health check with error rate:
```json
{
  "status": "healthy",
  "error_rate": 2.5,
  "total_errors": 150
}
```

Returns 503 if error rate > 50 errors/minute.

## Usage Examples

### Returning Errors in Handlers

```rust
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Product>, AppError> {
    // Validation error
    if id.is_empty() {
        return Err(AppError::Validation("Product ID is required".to_string()));
    }
    
    // Database error (automatically converted)
    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(&id)
        .fetch_one(state.db.pool())
        .await?; // Converts sqlx::Error to AppError::Database
    
    // Not found error
    if product.deleted_at.is_some() {
        return Err(AppError::NotFound("Product not found".to_string()));
    }
    
    Ok(Json(product))
}
```

### Recording Errors for Monitoring

```rust
pub async fn create_product(
    State(state): State<AppState>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<Product>, AppError> {
    match state.product_service.create_product(request).await {
        Ok(product) => Ok(Json(product)),
        Err(e) => {
            // Record error for monitoring
            state.error_monitor.record_error(
                ErrorCode::DatabaseError,
                Uuid::new_v4().to_string(),
                Some("/api/products".to_string()),
            ).await;
            
            Err(e)
        }
    }
}
```

### Custom Error Conversions

```rust
impl From<MyCustomError> for AppError {
    fn from(err: MyCustomError) -> Self {
        tracing::error!(error = ?err, "Custom error occurred");
        
        match err {
            MyCustomError::NotFound => AppError::NotFound("Resource not found".to_string()),
            MyCustomError::Invalid => AppError::Validation("Invalid data".to_string()),
            _ => AppError::Internal("An error occurred".to_string()),
        }
    }
}
```

## Security Considerations

### What NOT to Expose

❌ **Never expose:**
- Database schema details (table names, columns)
- SQL queries or fragments
- File paths or directory structures
- Connection strings or credentials
- Internal service URLs or endpoints
- Stack traces in production
- Detailed error messages from libraries
- System configuration details

✅ **Safe to expose:**
- HTTP status codes
- Standardized error codes
- User-friendly error messages
- Correlation IDs for support
- Validation error details (sanitized)
- Business rule violations

### Development vs Production

**Development Mode** (debug builds):
- Includes detailed error information
- Shows stack traces
- Logs verbose details
- Includes `details` field in responses

**Production Mode** (release builds):
- Sanitized error messages only
- No stack traces
- Minimal external information
- No `details` field in responses

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_error_sanitization() {
    let msg = "Error in /usr/local/app/src/main.rs";
    let sanitized = sanitize_message(msg);
    assert!(sanitized.contains("[path]"));
    assert!(!sanitized.contains("/usr/local"));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_error_response_format() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/products/invalid")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body: ErrorResponse = serde_json::from_slice(
        &hyper::body::to_bytes(response.into_body()).await.unwrap()
    ).unwrap();
    
    assert_eq!(body.code, ErrorCode::ResourceNotFound);
    assert!(!body.correlation_id.is_empty());
}
```

## Monitoring and Alerting

### Grafana Dashboard

Create dashboards to visualize:
- Error rate over time
- Error distribution by code
- Top error endpoints
- Correlation ID lookup

### Alert Rules

Configure alerts for:
```yaml
- alert: HighErrorRate
  expr: error_rate > 10
  for: 5m
  annotations:
    summary: "High error rate detected"
    
- alert: CriticalError
  expr: critical_error_count > 0
  for: 1m
  annotations:
    summary: "Critical error occurred"
```

## Best Practices

1. **Always use correlation IDs** for request tracking
2. **Log errors at appropriate levels** (error, warn, info, debug)
3. **Sanitize all user-facing messages** to prevent information disclosure
4. **Include context in logs** using structured logging
5. **Monitor error rates** and set up alerts
6. **Use standardized error codes** for programmatic handling
7. **Test error paths** as thoroughly as success paths
8. **Document error conditions** in API documentation
9. **Review error logs regularly** to identify patterns
10. **Never log sensitive data** (passwords, tokens, PII)

## Related Documentation

- [Error Handling Standards](./ERROR_HANDLING_STANDARDS.md)
- [Logging Standards](./LOGGING_STANDARDS.md)
- [Security Implementation](./SECURITY_IMPLEMENTATION.md)
- [API Documentation](./API.md)

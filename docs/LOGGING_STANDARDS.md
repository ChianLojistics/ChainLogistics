# Logging Standards

## Overview

This document defines the logging standards and practices used across the ChainLogistics project. The backend uses the `tracing` crate for structured logging, providing consistent, queryable logs across all services.

## Table of Contents

1. [Logging Levels](#logging-levels)
2. [Structured Logging](#structured-logging)
3. [Log Format](#log-format)
4. [Logging Configuration](#logging-configuration)
5. [Best Practices](#best-practices)
6. [Examples](#examples)
7. [Log Aggregation](#log-aggregation)

---

## Logging Levels

### Level Definitions

| Level | Severity | When to Use | Examples |
|-------|----------|-------------|----------|
| `error!` | Critical | Errors that prevent operation completion | Database connection failures, critical bugs |
| `warn!` | Warning | Potentially harmful situations | Unauthorized access attempts, rate limits |
| `info!` | Informational | Normal operational events | Server start, successful operations |
| `debug!` | Debug | Detailed diagnostic information | Function entry/exit, variable values |
| `trace!` | Trace | Very detailed tracing | Loop iterations, detailed state changes |

### Level Guidelines

- **Use `error!`** for errors that require immediate attention or prevent the application from functioning
- **Use `warn!`** for issues that don't prevent operation but may indicate problems
- **Use `info!`` for significant events that track the application's lifecycle
- **Use `debug!`** for detailed information useful during development and troubleshooting
- **Use `trace!`** for extremely detailed information, typically only during deep debugging

---

## Structured Logging

### Key-Value Fields

Structured logging uses key-value pairs to provide context:

```rust
tracing::info!(
    user_id = %user_id,
    product_id = %product_id,
    action = "created",
    "Product created successfully"
);
```

### Field Types

- **String fields**: Use `key = "value"` syntax
- **Display fields**: Use `key = %value` syntax (uses `Display` trait)
- **Debug fields**: Use `key = ?value` syntax (uses `Debug` trait)

### Field Naming Conventions

- Use `snake_case` for field names
- Use descriptive, meaningful names
- Include units when applicable (e.g., `duration_ms`, `size_bytes`)
- Use consistent field names across the codebase

---

## Log Format

### Standard Format

Logs follow this structured format:

```
TIMESTAMP LEVEL [MODULE] MESSAGE key1=value1 key2=value2
```

### Example

```
2024-03-15T10:30:45.123Z INFO [database] User created user_id=123 email="user@example.com"
2024-03-15T10:30:46.456Z ERROR [api] Failed to process request error=DatabaseError path="/api/products"
2024-03-15T10:30:47.789Z WARN [auth] Rate limit exceeded ip="192.168.1.1" user_id=456
```

### Format Components

- **TIMESTAMP**: ISO 8601 format with timezone
- **LEVEL**: Log level in uppercase (ERROR, WARN, INFO, DEBUG, TRACE)
- **MODULE**: The module or component generating the log
- **MESSAGE**: Human-readable message
- **FIELDS**: Key-value pairs providing additional context

---

## Logging Configuration

### Environment Variables

Configure logging behavior using environment variables:

```bash
# Set log level (default: info)
RUST_LOG=info

# Set log level for specific modules
RUST_LOG=chainlogistics=debug,sqlx=warn

# Enable trace logging
RUST_LOG=trace
```

### Common Configurations

```bash
# Production
RUST_LOG=chainlogistics=info,sqlx=warn

# Development
RUST_LOG=chainlogistics=debug,sqlx=debug

# Debugging
RUST_LOG=chainlogistics=trace,sqlx=trace
```

### Initialization

The backend initializes tracing in `main.rs`:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // ... rest of application
}
```

---

## Best Practices

### Do's

- **Log at appropriate levels**: Use the correct level for the situation
- **Include context**: Add relevant key-value pairs to provide context
- **Be consistent**: Use the same field names and formats across the codebase
- **Log errors with details**: Include error information when logging errors
- **Use structured fields**: Prefer structured fields over string interpolation
- **Log security events**: Log authentication, authorization, and security-related events

### Don'ts

- **Don't log sensitive data**: Never log passwords, API keys, or personal information
- **Don't log excessively**: Avoid logging at trace/debug levels in production
- **Don't use string interpolation**: Use structured fields instead of `format!`
- **Don't log in hot paths**: Avoid logging in performance-critical code paths
- **Don't ignore errors**: Always log errors appropriately

### Sensitive Data

Never log sensitive information:

```rust
// BAD - Logs password
tracing::info!("User login: {}", password);

// GOOD - Logs without password
tracing::info!(
    user_id = %user.id,
    "User logged in"
);
```

### Performance Considerations

- Use lazy evaluation for expensive operations
- Consider using `span!` for tracking operations
- Disable debug/trace logging in production

---

## Examples

### Basic Logging

```rust
// Info level
tracing::info!("Server started on port 8080");

// Error level
tracing::error!("Failed to connect to database");

// Warning level
tracing::warn!("Rate limit exceeded for user {}", user_id);
```

### Structured Logging

```rust
// With fields
tracing::info!(
    user_id = %user.id,
    product_id = %product.id,
    "Product created"
);

// With error
tracing::error!(
    error = %err,
    user_id = %user_id,
    "Failed to create product"
);

// With multiple fields
tracing::debug!(
    request_id = %request_id,
    method = %method,
    path = %path,
    status = status_code,
    duration_ms = duration.as_millis(),
    "Request completed"
);
```

### Error Logging

```rust
// In error.rs
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match self {
            AppError::Database(msg) => {
                tracing::error!(
                    error = %msg,
                    "Database error occurred"
                );
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Unauthorized => {
                tracing::warn!(
                    ip = %ip,
                    "Unauthorized access attempt"
                );
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            // ... other variants
        };
        // ... response construction
    }
}
```

### Using Spans

```rust
use tracing::{span, Level};

// Create a span for an operation
let span = span!(Level::INFO, "process_product", product_id = %product_id);
let _enter = span.enter();

// Logs within this span will include the product_id context
tracing::info!("Starting product processing");
// ... processing logic
tracing::info!("Product processing completed");
```

### Instrumented Functions

```rust
use tracing::instrument;

#[instrument(skip(db))]
async fn get_product(
    db: &PgPool,
    product_id: String,
) -> Result<Product, AppError> {
    tracing::debug!("Fetching product from database");
    
    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(&product_id)
        .fetch_one(db)
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                product_id = %product_id,
                "Database query failed"
            );
            AppError::Database(e.to_string())
        })?;
    
    tracing::info!(product_id = %product.id, "Product retrieved successfully");
    Ok(product)
}
```

---

## Log Aggregation

### Current Setup

The backend uses `tracing` which produces structured logs that can be:

1. **Output to stdout/stderr**: Captured by container orchestration (Docker, Kubernetes)
2. **Written to files**: Using file appenders for persistent storage
3. **Sent to log aggregators**: Compatible with ELK, Splunk, Datadog, etc.

### Recommended Aggregation Tools

- **ELK Stack** (Elasticsearch, Logstash, Kibana): Open-source log aggregation
- **Grafana Loki**: Lightweight log aggregation system
- **Datadog**: Cloud-based monitoring and logging
- **Splunk**: Enterprise log management

### Log Rotation

Configure log rotation to prevent disk exhaustion:

```toml
# Example: log4rs configuration
[appenders.file]
kind = "file"
path = "logs/chainlogistics.log"
append = true
encoder = "pattern"

[appenders.file.encoder]
pattern = "{d(%Y-%m-%d %H:%M:%S)} {l} {M} - {m}{n}"
```

### Monitoring and Alerting

Set up alerts for:

- **Error rate spikes**: Sudden increase in error logs
- **Critical errors**: Any error-level logs in production
- **Warning patterns**: Repeated warnings that may indicate issues
- **Log volume**: Unusual log volume increases

---

## Module-Specific Guidelines

### API Layer

- Log all incoming requests at `info` level
- Log request duration for performance monitoring
- Include request ID for traceability

```rust
tracing::info!(
    method = %method,
    path = %path,
    status = status,
    duration_ms = duration.as_millis(),
    "Request completed"
);
```

### Database Layer

- Log all queries at `debug` level in development
- Log query failures at `error` level
- Include query details for debugging

```rust
tracing::debug!(
    query = %query,
    rows_affected = rows,
    "Query executed successfully"
);
```

### Authentication/Authorization

- Log all authentication attempts at `info` level
- Log failed authentication at `warn` level
- Include user ID and IP address

```rust
tracing::warn!(
    user_id = %user_id,
    ip = %ip,
    "Failed authentication attempt"
);
```

### Blockchain Integration

- Log all contract interactions at `info` level
- Log transaction failures at `error` level
- Include transaction hash for traceability

```rust
tracing::info!(
    tx_hash = %tx_hash,
    contract = %contract_address,
    "Transaction submitted"
);
```

---

## Related Documentation

- [Error Handling Standards](./ERROR_HANDLING_STANDARDS.md)
- [API Documentation](./API.md)
- [Architecture Documentation](./ARCHITECTURE.md)

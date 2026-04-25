# Error Handling Standards

## Overview

This document defines the consistent error handling patterns used across the ChainLogistics codebase, including smart contracts (Rust/Soroban) and backend services (Rust/Axum).

## Table of Contents

1. [Smart Contract Error Handling](#smart-contract-error-handling)
2. [Backend Error Handling](#backend-error-handling)
3. [Error Types and Categories](#error-types-and-categories)
4. [Error Propagation Strategy](#error-propagation-strategy)
5. [Error Logging Standards](#error-logging-standards)
6. [Best Practices](#best-practices)

---

## Smart Contract Error Handling

### Error Definition

Smart contracts use Soroban's `#[contracterror]` macro to define errors:

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Core errors (1-10)
    ProductAlreadyExists = 1,
    ProductNotFound = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    EventNotFound = 5,
    NotInitialized = 6,
    AlreadyInitialized = 7,
    ContractPaused = 8,
    ContractNotPaused = 9,
    
    // Validation errors (10-30)
    InvalidProductId = 10,
    InvalidProductName = 11,
    InvalidOrigin = 12,
    // ... more validation errors
    
    // Lifecycle errors (40-50)
    ProductDeactivated = 40,
    DeactivationReasonRequired = 41,
    ProductAlreadyActive = 42,
    
    // Upgrade errors (50-60)
    InvalidUpgrade = 50,
    UpgradeInProgress = 51,
    // ... more upgrade errors
    
    // Multi-sig errors (60-70)
    MultiSigNotConfigured = 60,
    NotSigner = 61,
    // ... more multi-sig errors
}
```

### Error Ranges

Error codes are organized by category with numeric ranges:

- **1-10**: Core operational errors
- **10-30**: Validation errors
- **30-40**: Batch operation errors
- **40-50**: Product lifecycle errors
- **50-60**: Upgrade/migration errors
- **60-70**: Multi-signature errors

### Error Usage Patterns

#### Returning Errors

All contract functions that can fail should return `Result<T, Error>`:

```rust
pub fn get_product(env: Env, id: String) -> Result<Product, Error> {
    storage::get_product(&env, &id).ok_or(Error::ProductNotFound)
}
```

#### Early Validation

Validate inputs early to fail fast and save gas:

```rust
pub fn register_product(env: Env, owner: Address, config: ProductConfig) -> Result<Product, Error> {
    owner.require_auth();
    
    // Validate inputs early
    ValidationContract::validate_product_config(&config)?;
    
    // Check for duplicates
    if storage::has_product(&env, &config.id) {
        return Err(Error::ProductAlreadyExists);
    }
    
    // ... rest of implementation
}
```

#### Helper Functions

Use helper functions for common error checks:

```rust
fn require_not_paused(env: &Env) -> Result<(), Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    if &admin == caller {
        caller.require_auth();
        return Ok(());
    }
    Err(Error::Unauthorized)
}

fn require_owner(product: &Product, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if &product.owner != caller {
        return Err(Error::Unauthorized);
    }
    Ok(())
}
```

---

## Backend Error Handling

### Error Definition

Backend services use `thiserror` for error definitions:

```rust
use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Authentication failed")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}
```

### HTTP Response Mapping

Implement `IntoResponse` to map errors to HTTP responses:

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match self {
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Unauthorized => {
                tracing::warn!("Unauthorized access attempt");
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AppError::Forbidden(msg) => {
                tracing::warn!("Forbidden access attempt: {}", msg);
                (StatusCode::FORBIDDEN, msg)
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::RateLimit => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
```

### Error Conversions

Implement `From` traits for common error types:

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("JSON error: {}", err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Internal(format!("Password hashing error: {}", err))
    }
}
```

### Error Response Format

All error responses follow this JSON format:

```json
{
  "error": "Error message",
  "status": 400
}
```

---

## Error Types and Categories

### Smart Contract Error Categories

| Category | Range | Description |
|----------|-------|-------------|
| Core | 1-10 | Fundamental operational errors |
| Validation | 10-30 | Input validation failures |
| Batch | 30-40 | Batch operation errors |
| Lifecycle | 40-50 | Product lifecycle errors |
| Upgrade | 50-60 | Contract upgrade errors |
| Multi-Sig | 60-70 | Multi-signature errors |

### Backend Error Categories

| Error Type | HTTP Status | When to Use |
|------------|--------------|-------------|
| `BadRequest` | 400 | Invalid input data |
| `Unauthorized` | 401 | Missing or invalid authentication |
| `Forbidden` | 403 | Authenticated but insufficient permissions |
| `NotFound` | 404 | Resource not found |
| `RateLimit` | 429 | Too many requests |
| `Internal` | 500 | Unexpected server errors |
| `Database` | 500 | Database-related errors |
| `Validation` | 400 | Validation failures |

---

## Error Propagation Strategy

### Smart Contract Propagation

1. **Validate Early**: Check all preconditions at the start of functions
2. **Use `?` Operator**: Propagate errors using the `?` operator
3. **Context Preservation**: Errors maintain their original context
4. **Gas Efficiency**: Fail fast to save gas

```rust
pub fn add_tracking_event(
    env: Env,
    actor: Address,
    product_id: String,
    // ... other parameters
) -> Result<u64, Error> {
    // Early validation
    require_not_paused(&env)?;
    let product = read_product(&env, &product_id)?;
    require_can_add_event(&env, &product_id, &product, &actor)?;
    
    // Validate inputs
    ValidationContract::validate_event_location(&location)?;
    ValidationContract::validate_event_note(&note)?;
    ValidationContract::validate_metadata(&metadata)?;
    
    // ... rest of implementation
}
```

### Backend Propagation

1. **Use `?` Operator**: Propagate errors using the `?` operator
2. **Contextual Errors**: Add context when converting errors
3. **Logging**: Log errors at appropriate levels
4. **User-Friendly Messages**: Return generic messages for internal errors

```rust
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Product>, AppError> {
    let product = state.product_service
        .get_product(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get product {}: {}", id, e);
            AppError::NotFound(format!("Product {} not found", id))
        })?;
    
    Ok(Json(product))
}
```

---

## Error Logging Standards

### Logging Levels

| Level | When to Use | Examples |
|-------|-------------|----------|
| `error!` | Errors that prevent operation | Database failures, critical bugs |
| `warn!` | Potentially harmful situations | Unauthorized attempts, rate limits |
| `info!` | Informational messages | Server start, successful operations |
| `debug!` | Detailed diagnostic information | Function entry/exit, variable values |
| `trace!` | Very detailed tracing | Loop iterations, detailed state |

### Smart Contract Logging

Smart contracts emit events for logging:

```rust
env.events().publish(
    (Symbol::new(&env, "product_registered"), product_id.clone()),
    product.clone(),
);
```

### Backend Logging

Backend uses `tracing` for structured logging:

```rust
// Error logging
tracing::error!("Database error: {}", msg);

// Warning logging
tracing::warn!("Unauthorized access attempt from IP: {}", ip);

// Info logging
tracing::info!("Product {} registered by user {}", product_id, user_id);

// Debug logging
tracing::debug!("Processing request: {:?}", request);

// Structured logging with fields
tracing::error!(
    error = %err,
    user_id = %user_id,
    product_id = %product_id,
    "Failed to register product"
);
```

### Log Format

Logs should follow this structured format:

```
TIMESTAMP LEVEL [MODULE] MESSAGE key1=value1 key2=value2
```

Example:
```
2024-03-15T10:30:45.123Z ERROR [database] Failed to connect to PostgreSQL host=localhost port=5432
```

---

## Best Practices

### Smart Contracts

1. **Define All Errors Upfront**: All possible errors should be defined in the Error enum
2. **Use Descriptive Error Names**: Error names should clearly indicate the problem
3. **Validate Inputs Early**: Fail fast to save gas
4. **Use Helper Functions**: Extract common validation logic
5. **Document Error Conditions**: Document when each error is returned

### Backend

1. **Never Expose Internal Details**: Return generic messages for internal errors
2. **Log at Appropriate Levels**: Use correct log levels for different situations
3. **Include Context**: Add relevant context to log messages
4. **Use Structured Logging**: Use structured logging with key-value pairs
5. **Implement Error Conversions**: Use `From` traits for error conversions

### General

1. **Be Consistent**: Use the same error handling patterns throughout the codebase
2. **Document Everything**: Document error types, when they occur, and how to handle them
3. **Monitor Errors**: Set up monitoring and alerting for errors
4. **Learn from Errors**: Review error logs to identify and fix common issues
5. **Test Error Paths**: Write tests for all error conditions

---

## Examples

### Smart Contract Example

```rust
pub fn register_product(
    env: Env,
    owner: Address,
    config: ProductConfig,
) -> Result<Product, Error> {
    // Authentication
    owner.require_auth();
    
    // Validation
    ValidationContract::validate_product_config(&config)?;
    
    // Duplicate check
    if storage::has_product(&env, &config.id) {
        return Err(Error::ProductAlreadyExists);
    }
    
    // Build product
    let product = Product {
        id: config.id.clone(),
        name: config.name,
        // ... other fields
    };
    
    // Storage
    storage::put_product(&env, &product);
    
    // Emit event
    env.events().publish(
        (Symbol::new(&env, "product_registered"), config.id.clone()),
        product.clone(),
    );
    
    Ok(product)
}
```

### Backend Example

```rust
pub async fn register_product(
    State(state): State<AppState>,
    Json(request): Json<RegisterProductRequest>,
) -> Result<Json<Product>, AppError> {
    // Validate input
    request.validate()?;
    
    // Check for duplicates
    if state.product_service.product_exists(&request.id).await? {
        return Err(AppError::BadRequest(format!(
            "Product with ID {} already exists",
            request.id
        )));
    }
    
    // Create product
    let product = state.product_service
        .create_product(request)
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                product_id = %request.id,
                "Failed to create product"
            );
            AppError::Internal("Failed to create product".to_string())
        })?;
    
    tracing::info!("Product {} created successfully", product.id);
    
    Ok(Json(product))
}
```

---

## Related Documentation

- [Smart Contract Documentation](./SMART_CONTRACTS.md)
- [API Documentation](./API.md)
- [Logging Standards](./LOGGING_STANDARDS.md)
- [Architecture Documentation](./ARCHITECTURE.md)

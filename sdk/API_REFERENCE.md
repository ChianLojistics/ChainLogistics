# SDK API Reference

This reference summarizes the main services and methods available in the Python and Rust SDKs.

## Core Client

### Python

- `ChainLogisticsClient(config)`
- Services:
  - `client.products`
  - `client.events`
  - `client.stats`

### Rust

- `ChainLogisticsClient::new(config)`
- Services:
  - `client.products()`
  - `client.events()`
  - `client.stats()`

## Products Service

### Methods

- `list(query)`
- `get(product_id)`
- `create(new_product)`
- `update(product_id, update_product)`
- `delete(product_id)`
- `search(query, limit)`
- `list_by_owner(owner_address, offset, limit)`
- `list_by_category(category, offset, limit)`
- `list_active(offset, limit)`

### Primary Models

- `Product`
- `NewProduct`
- `UpdateProduct`
- `ProductListQuery`
- `PaginationMeta`

## Events Service

### Methods

- `list(query)`
- `get(event_id)`
- `create(new_tracking_event)`
- `list_by_product(product_id, offset, limit)`
- `list_by_product_and_type(product_id, event_type, offset, limit)`
- `get_all_for_product(product_id)`
- `get_by_type_for_product(product_id, event_type)`

### Primary Models

- `TrackingEvent`
- `NewTrackingEvent`
- `EventListQuery`
- `PaginationMeta`

## Stats Service

### Methods

- `get_global()`
- `health()`
- `db_health()`

### Primary Models

- `GlobalStats`
- `HealthResponse`
- `DbHealthResponse`

## Error Handling

### Python exceptions

- `ApiError`
- `AuthenticationError`
- `RateLimitError`
- `NotFoundError`
- `ValidationError`
- `ConfigError`

### Rust error type

- `chainlogistics_sdk::Error` with variants for API/auth/rate-limit/validation/server errors.

## API Endpoint Mapping

The SDK methods map to REST endpoints under the backend API, for example:

- Products: `api/v1/products`, `api/v1/admin/products`
- Events: `api/v1/events`, `api/v1/admin/events`
- Stats: `api/v1/stats`
- Health: `health`, `health/db`

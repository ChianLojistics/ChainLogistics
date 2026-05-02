# SDK Migration Guide

This guide describes how to migrate between major SDK versions.

## 0.x -> 1.x

## Breaking Changes Summary

- Stable service-oriented API exposed via `products`, `events`, and `stats` services.
- Standardized pagination metadata for list endpoints.
- Standardized error mapping for auth, validation, not-found, rate-limit, and server errors.

## Python SDK Migration

### Before (0.x style)

```python
# Pseudocode from legacy usage patterns
client.get_products()
client.create_event({...})
```

### After (1.x)

```python
products, page = client.products.list()
client.events.create(new_event)
stats = client.stats.get_global()
```

Checklist:

- Replace direct client method calls with service namespaces.
- Use `ProductListQuery` and `EventListQuery` for filters.
- Update error handling to typed exceptions.

## Rust SDK Migration

### Before (0.x style)

```rust
// Pseudocode from legacy usage patterns
client.get_products().await?;
client.create_event(payload).await?;
```

### After (1.x)

```rust
let (products, page) = client.products().list(None).await?;
let created = client.events().create(&event).await?;
let stats = client.stats().get_global().await?;
```

Checklist:

- Use service accessors (`products()`, `events()`, `stats()`).
- Update list calls to consume `(items, pagination)` tuples.
- Handle `chainlogistics_sdk::Error` variants explicitly where needed.

## Validation Steps After Migration

- Run SDK unit tests.
- Validate one product flow end-to-end in staging.
- Validate one event flow end-to-end in staging.
- Confirm error handling paths for 401/404/429.

## Future Migration Notes

For each major release, add:

- Explicit API changes with old/new examples.
- Data model changes and compatibility notes.
- Rollback guidance where applicable.

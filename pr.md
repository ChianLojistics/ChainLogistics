# [BA-01] Refactor: Split services.rs into dedicated service modules

## Summary

- Extracted `ProductService`, `EventService`, `UserService`, `ApiKeyService`, `SyncService`, and `RecallService` from the 1282-line monolithic `services.rs` into individual files under `backend/src/services/`
- Fixed a correctness issue: cache helper methods (`invalidate_product_cache`, `invalidate_global_stats`) were incorrectly placed inside trait `impl` blocks; they are now in inherent `impl` blocks where they belong
- Reduced `services.rs` to 40 lines of `pub mod` declarations and `pub use` re-exports — all existing import paths remain unchanged
- Added missing re-exports for `BatchService`, `BatchRepository`, `SupplierService`, `IoTService`, `QualityService`, and `RegulatoryService`, which were referenced in handlers but not previously re-exported

## New files

| File | Lines | Contents |
|---|---|---|
| `services/product.rs` | 266 | `ProductService` + `ProductRepository` impl |
| `services/event.rs` | 169 | `EventService` + `EventRepository` impl |
| `services/user.rs` | 178 | `UserService` + `UserRepository` impl |
| `services/api_key.rs` | 143 | `ApiKeyService` + `ApiKeyRepository` impl |
| `services/sync.rs` | 64 | `SyncService` |
| `services/recall.rs` | 343 | `RecallService` |

## Test plan

- [ ] `cargo check` passes with no new errors
- [ ] All handler paths that call into services resolve correctly (no broken imports)
- [ ] `crate::services::ApiKeyService::hash_api_key` / `generate_api_key` still callable as static methods
- [ ] `crate::services::UserService::hash_password` still callable as a static method
- [ ] `crate::services::BatchRepository` trait is accessible from `handlers/batch.rs`

## Relates to

Closes #407

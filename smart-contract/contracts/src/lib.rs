#![cfg_attr(test, allow(unused_imports, unused_variables))]
// Test-only allowances: During testing, we often import utilities and declare variables
// that are only used for test setup or debugging. These warnings are suppressed
// for test code to avoid noise while maintaining strict warnings for production code.
#![no_std]

mod authorization;
mod events;
mod contract;
mod error;
mod multisig;
mod storage;
mod storage_contract;
mod types;
mod validation;
mod validation_contract;

// Architecture: Modular Contract Design
// 
// This crate uses a modular architecture where each contract functionality is separated
// into its own module. This approach provides several benefits:
// 
// 1. **Separation of Concerns**: Each module handles a specific domain (admin, tracking, 
//    multisig, etc.) making the codebase easier to understand and maintain.
// 
// 2. **Conditional Compilation**: Modules that are only needed for host-side testing
//    or admin functions are excluded from the WASM artifact to reduce contract size.
// 
// 3. **Symbol Collision Avoidance**: Building a single WASM artifact from a crate 
//    containing multiple `#[contract]` definitions can trigger Soroban macro export 
//    symbol collisions (method names like `init`, `get_stats`, etc). For CI's WASM 
//    build step we compile only the ChainLogisticsContract + dependencies; the full 
//    contract suite is still built during host-side `cargo test`.
#[cfg(not(target_arch = "wasm32"))]
mod admin;
#[cfg(not(target_arch = "wasm32"))]
mod event_query;
#[cfg(not(target_arch = "wasm32"))]
mod product_query;
#[cfg(not(target_arch = "wasm32"))]
mod product_registry;
#[cfg(not(target_arch = "wasm32"))]
mod product_transfer;
#[cfg(not(target_arch = "wasm32"))]
mod stats;
#[cfg(not(target_arch = "wasm32"))]
mod tracking;
#[cfg(not(target_arch = "wasm32"))]
mod upgrade;

#[cfg(test)]
mod load_tests;
#[cfg(test)]
mod test;
#[cfg(test)]
mod test_auth;
#[cfg(test)]
mod test_benchmarks;
#[cfg(test)]
mod test_error_coverage;
#[cfg(test)]
mod test_integration;

// Public API Surface
// 
// These `pub use` statements re-export the public interface of each module, creating
// a clean and unified API surface for contract consumers. This pattern:
// 
// - Provides a single point of entry for all contract functionality
// - Allows internal module refactoring without breaking the public API
// - Enables consumers to import from the crate root rather than specific modules
// - Maintains backward compatibility when internal organization changes

pub use authorization::*;
pub use contract::*;
pub use error::*;
pub use multisig::*;
pub use types::*;

#[cfg(not(target_arch = "wasm32"))]
pub use admin::*;
#[cfg(not(target_arch = "wasm32"))]
pub use event_query::*;
#[cfg(not(target_arch = "wasm32"))]
pub use product_query::*;
#[cfg(not(target_arch = "wasm32"))]
pub use product_registry::*;
#[cfg(not(target_arch = "wasm32"))]
pub use product_transfer::*;
#[cfg(not(target_arch = "wasm32"))]
pub use stats::*;
#[cfg(not(target_arch = "wasm32"))]
pub use tracking::*;
#[cfg(not(target_arch = "wasm32"))]
pub use upgrade::*;

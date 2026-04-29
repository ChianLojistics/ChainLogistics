#![allow(unexpected_cfgs)]
#![allow(ambiguous_glob_reexports)]
#![allow(mismatched_lifetime_syntaxes)]
#![allow(dead_code)]
#![allow(deprecated)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::needless_borrow)]
#![cfg_attr(test, allow(unused_imports, unused_variables))]
#![no_std]

mod authorization;
mod contract;
mod error;
mod multisig;
mod quality_control;
mod storage;
mod storage_contract;
mod types;
mod validation;
mod validation_contract;

// Building a single WASM artifact from a crate that contains multiple `#[contract]`
// definitions can trigger Soroban macro export symbol collisions (method names like
// `init`, `get_stats`, etc). For CI's WASM build step we compile only the
// ChainLogisticsContract + dependencies; the full contract suite is still built
// during host-side `cargo test`.
#[cfg(not(target_arch = "wasm32"))]
mod admin;
#[cfg(not(target_arch = "wasm32"))]
mod event_query;
#[cfg(not(target_arch = "wasm32"))]
mod oracle;
#[cfg(not(target_arch = "wasm32"))]
mod product_query;
#[cfg(not(target_arch = "wasm32"))]
mod product_registry;
#[cfg(not(target_arch = "wasm32"))]
mod product_transfer;
#[cfg(not(target_arch = "wasm32"))]
mod stats;
#[cfg(not(target_arch = "wasm32"))]
mod timelock;
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

#[cfg(test)]
#[path = "test/integration_tests.rs"]
mod integration_tests;

#[cfg(test)]
#[path = "test/supply_chain_scenarios.rs"]
mod supply_chain_scenarios;

pub use authorization::*;
pub use contract::*;
pub use error::*;
pub use multisig::*;
pub use quality_control::*;
pub use types::*;

#[cfg(not(target_arch = "wasm32"))]
pub use admin::*;
#[cfg(not(target_arch = "wasm32"))]
pub use event_query::*;
#[cfg(not(target_arch = "wasm32"))]
pub use oracle::*;
#[cfg(not(target_arch = "wasm32"))]
pub use product_query::*;
#[cfg(not(target_arch = "wasm32"))]
pub use product_registry::*;
#[cfg(not(target_arch = "wasm32"))]
pub use product_transfer::*;
#[cfg(not(target_arch = "wasm32"))]
pub use stats::*;
#[cfg(not(target_arch = "wasm32"))]
pub use timelock::*;
#[cfg(not(target_arch = "wasm32"))]
pub use tracking::*;
#[cfg(not(target_arch = "wasm32"))]
pub use upgrade::*;

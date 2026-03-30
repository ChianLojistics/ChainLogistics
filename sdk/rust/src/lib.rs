//! ChainLogistics Rust SDK
//! 
//! This SDK provides a convenient interface for interacting with the ChainLogistics API.
//! 
//! # Quick Start
//! 
//! ```rust,no_run
//! use chainlogistics_sdk::{ChainLogisticsClient, Config};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::new("your-api-key")
//!         .with_base_url("https://api.chainlogistics.io");
//!     
//!     let client = ChainLogisticsClient::new(config)?;
//!     
//!     // List products
//!     let products = client.products().list(None).await?;
//!     println!("Found {} products", products.len());
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod products;
pub mod events;
pub mod stats;

// Re-export main types for convenience
pub use client::ChainLogisticsClient;
pub use config::Config;
pub use error::{Error, Result};
pub use models::*;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default API base URL
pub const DEFAULT_BASE_URL: &str = "https://api.chainlogistics.io";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_base_url() {
        assert_eq!(DEFAULT_BASE_URL, "https://api.chainlogistics.io");
    }
}

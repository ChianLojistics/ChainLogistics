# ChainLogistics Rust SDK

Rust SDK for the ChainLogistics API.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
chainlogistics-sdk = "1.0.0"
```

## Quick Start

```rust
use chainlogistics_sdk::{ChainLogisticsClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("YOUR_API_KEY")
        .with_base_url("https://api.chainlogistics.io");

    let client = ChainLogisticsClient::new(config)?;

    let (products, page) = client.products().list(None).await?;
    println!("products: {}, total: {}", products.len(), page.total);

    Ok(())
}
```

## Services

- `products()`: product lifecycle operations.
- `events()`: tracking event operations.
- `stats()`: global metrics and health endpoints.

## Additional Documentation

- Shared SDK docs: `sdk/README.md`
- API reference: `sdk/API_REFERENCE.md`
- Examples: `sdk/EXAMPLES.md`
- Migration: `sdk/MIGRATION_GUIDE.md`

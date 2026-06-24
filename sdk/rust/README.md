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

## Ring signatures (privacy-preserving audit trail)

Enable the `ring-signatures` feature to sign audit attestations anonymously as
one member of a ring of auditors (BLS12-381). Signatures verify on-chain via the
`AuditTrailContract` / `RingSignatureVerifier` Soroban contracts unchanged.

```toml
chainlogistics-sdk = { version = "1.0.0", features = ["ring-signatures"] }
```

```rust
use chainlogistics_sdk::ring_signature::{KeyPair, sign, verify};
use rand_core::OsRng;

let auditors: Vec<KeyPair> = (0..8).map(|_| KeyPair::generate(OsRng)).collect();
let ring: Vec<[u8; 96]> = auditors.iter().map(|k| k.public_key()).collect();

let msg = b"custody: warehouse-A -> truck-7";
let sig = sign(&ring, 3, &auditors[3], msg, OsRng).unwrap(); // auditor #3, anonymously
assert!(verify(&ring, msg, &sig));
```

See `smart-contract/RING_SIGNATURE.md` for the full scheme, gas costs and
linkability analysis.

## Additional Documentation

- Shared SDK docs: `sdk/README.md`
- API reference: `sdk/API_REFERENCE.md`
- Examples: `sdk/EXAMPLES.md`
- Migration: `sdk/MIGRATION_GUIDE.md`

# SDK Installation and Setup

## Prerequisites

- Access to ChainLogistics API endpoint.
- Valid API key with required scope.

## Python SDK

Path: `sdk/python`

### Install from package index

```bash
pip install chainlogistics-sdk
```

### Install locally for development

```bash
cd sdk/python
pip install -e .
```

### Verify installation

```python
from chainlogistics_sdk import ChainLogisticsClient, Config

client = ChainLogisticsClient(Config(api_key="YOUR_API_KEY"))
print(client)
```

## Rust SDK

Path: `sdk/rust`

### Add dependency

```toml
[dependencies]
chainlogistics-sdk = "1.0.0"
```

### Local path dependency

```toml
[dependencies]
chainlogistics-sdk = { path = "../sdk/rust" }
```

### Verify installation

```rust
use chainlogistics_sdk::{ChainLogisticsClient, Config};

fn main() {
    let config = Config::new("YOUR_API_KEY");
    let client = ChainLogisticsClient::new(config).expect("client");
    println!("configured for {}", client.config().base_url());
}
```

## Environment and Configuration

Common configuration options:

- `api_key`: authentication token.
- `base_url`: API host (production/staging).
- `timeout`: request timeout.
- `user_agent`: custom client identifier.

Use staging for integration testing and release validation before production rollout.

# SDK Examples

## 1. Initialize a Client

### Python

```python
from chainlogistics_sdk import ChainLogisticsClient, Config

config = Config(
    api_key="YOUR_API_KEY",
    base_url="https://api.chainlogistics.io",
    timeout=30,
)
client = ChainLogisticsClient(config)
```

### Rust

```rust
use chainlogistics_sdk::{ChainLogisticsClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("YOUR_API_KEY")
        .with_base_url("https://api.chainlogistics.io");
    let client = ChainLogisticsClient::new(config)?;
    println!("SDK ready: {}", client.config().base_url());
    Ok(())
}
```

## 2. Create and Fetch a Product

### Python

```python
from chainlogistics_sdk import NewProduct

payload = NewProduct(
    id="PROD-9001",
    name="Premium Coffee",
    description="Traceable single-origin coffee",
    origin_location="Colombia",
    category="Food",
    tags=["coffee", "single-origin"],
    certifications=["Organic"],
    media_hashes=[],
    custom_fields={},
    owner_address="GABC123DEF456",
    created_by="sdk-example",
)

created = client.products.create(payload)
loaded = client.products.get(created.id)
print(loaded.name)
```

### Rust

```rust
use chainlogistics_sdk::{models::NewProduct, ChainLogisticsClient, Config};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let client = ChainLogisticsClient::new(Config::new("YOUR_API_KEY"))?;
let payload = NewProduct {
    id: "PROD-9001".to_string(),
    name: "Premium Coffee".to_string(),
    description: "Traceable single-origin coffee".to_string(),
    origin_location: "Colombia".to_string(),
    category: "Food".to_string(),
    tags: vec!["coffee".into(), "single-origin".into()],
    certifications: vec!["Organic".into()],
    media_hashes: vec![],
    custom_fields: serde_json::json!({}),
    owner_address: "GABC123DEF456".to_string(),
    created_by: "sdk-example".to_string(),
};

let created = client.products().create(&payload).await?;
let loaded = client.products().get(&created.id).await?;
println!("{}", loaded.name);
# Ok(())
# }
```

## 3. Track an Event and Query by Product

### Python

```python
from datetime import datetime
from chainlogistics_sdk import NewTrackingEvent

event = NewTrackingEvent(
    product_id="PROD-9001",
    actor_address="GABC123DEF456",
    timestamp=datetime.utcnow(),
    event_type="shipment",
    location="Port Warehouse",
    data_hash="hash123",
    note="Shipped to regional hub",
    metadata={"batch": "B1"},
)

client.events.create(event)
events, page = client.events.list_by_product("PROD-9001", offset=0, limit=20)
print(page.total)
```

### Rust

```rust
use chainlogistics_sdk::models::{EventListQuery, NewTrackingEvent};

# async fn run(client: &chainlogistics_sdk::ChainLogisticsClient) -> Result<(), Box<dyn std::error::Error>> {
let event = NewTrackingEvent {
    product_id: "PROD-9001".to_string(),
    actor_address: "GABC123DEF456".to_string(),
    timestamp: chrono::Utc::now(),
    event_type: "shipment".to_string(),
    location: "Port Warehouse".to_string(),
    data_hash: "hash123".to_string(),
    note: Some("Shipped to regional hub".to_string()),
    metadata: serde_json::json!({"batch": "B1"}),
};

client.events().create(&event).await?;
let query = EventListQuery {
    offset: Some(0),
    limit: Some(20),
    product_id: Some("PROD-9001".to_string()),
    event_type: None,
};
let (events, pagination) = client.events().list(Some(query)).await?;
println!("events={}, total={}", events.len(), pagination.total);
# Ok(())
# }
```

## 4. Health and Stats

### Python

```python
health = client.stats.health()
stats = client.stats.get_global()
print(health.status, stats.total_products)
```

### Rust

```rust
# async fn run(client: &chainlogistics_sdk::ChainLogisticsClient) -> Result<(), Box<dyn std::error::Error>> {
let health = client.stats().health().await?;
let stats = client.stats().get_global().await?;
println!("{} {}", health.status, stats.total_products);
# Ok(())
# }
```

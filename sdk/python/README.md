# ChainLogistics Python SDK

The official Python SDK for the ChainLogistics API, providing a convenient interface for managing supply chain products and tracking events on the blockchain.

Shared SDK documentation is available in:

- `sdk/README.md`
- `sdk/INSTALLATION.md`
- `sdk/API_REFERENCE.md`
- `sdk/EXAMPLES.md`
- `sdk/MIGRATION_GUIDE.md`

## Installation

```bash
pip install chainlogistics-sdk
```

## Quick Start

```python
from chainlogistics_sdk import ChainLogisticsClient, Config

# Initialize the client
config = Config(api_key="your-api-key")
client = ChainLogisticsClient(config)

# List products
products, pagination = client.products.list()
print(f"Found {pagination.total} products")

for product in products:
    print(f"- {product.name} ({product.id})")

# Create a new product
from chainlogistics_sdk import NewProduct

new_product = NewProduct(
    id="PROD-001",
    name="Organic Coffee Beans",
    description="Premium organic coffee beans from Colombia",
    origin_location="Colombia",
    category="Food & Beverages",
    tags=["organic", "coffee", "premium"],
    certifications=["Fair Trade", "Organic"],
    media_hashes=[],
    custom_fields={},
    owner_address="GABC123DEF456",
    created_by="user@example.com",
)

created = client.products.create(new_product)
print(f"Created product: {created.id}")

# Add tracking event
from chainlogistics_sdk import NewTrackingEvent
from datetime import datetime

event = NewTrackingEvent(
    product_id="PROD-001",
    actor_address="GABC123DEF456",
    timestamp=datetime.utcnow(),
    event_type="harvest",
    location="Colombia Farm",
    data_hash="abc123...",
    note="Harvested premium beans",
    metadata={"batch": "BATCH-001"},
)

created_event = client.events.create(event)
print(f"Created event: {created_event.id}")
```

## Configuration

The SDK can be configured with various options:

```python
from chainlogistics_sdk import Config

config = Config(
    api_key="your-api-key",
    base_url="https://api.chainlogistics.io",  # Default
    timeout=30,  # Default: 30 seconds
    user_agent="my-app/1.0",  # Default: chainlogistics-sdk-python/1.0.0
)

# Or use builder pattern
config = Config("your-api-key").with_timeout(60).with_base_url("https://staging-api.chainlogistics.io")
```

## Products

### List Products

```python
# List all products
products, pagination = client.products.list()

# With filtering
from chainlogistics_sdk import ProductListQuery

query = ProductListQuery(
    limit=20,
    offset=0,
    category="Food & Beverages",
    is_active=True,
    search="coffee"
)

products, pagination = client.products.list(query)
```

### Get Product

```python
product = client.products.get("PROD-001")
print(product.name)
```

### Create Product

```python
new_product = NewProduct(
    id="PROD-002",
    name="Premium Tea",
    description="High-quality tea leaves",
    origin_location="India",
    category="Food & Beverages",
    tags=["tea", "premium"],
    certifications=[],
    media_hashes=[],
    custom_fields={},
    owner_address="GDEF789ABC012",
    created_by="user@example.com",
)

product = client.products.create(new_product)
```

### Update Product

```python
from chainlogistics_sdk import UpdateProduct

update = UpdateProduct(
    name="Premium Tea - Updated",
    description="High-quality tea leaves from Darjeeling",
    updated_by="user@example.com"
)

product = client.products.update("PROD-002", update)
```

### Delete Product

```python
client.products.delete("PROD-002")
```

### Search Products

```python
# Search by text
results = client.products.search("organic coffee", limit=10)

# By owner
products, _ = client.products.list_by_owner("GABC123DEF456")

# By category
products, _ = client.products.list_by_category("Food & Beverages")

# Active products only
products, _ = client.products.list_active()
```

## Events

### List Events

```python
from chainlogistics_sdk import EventListQuery

# Events for a specific product
query = EventListQuery(
    product_id="PROD-001",
    limit=50,
    offset=0
)

events, pagination = client.events.list(query)

# Filter by event type
query = EventListQuery(
    product_id="PROD-001",
    event_type="shipment",
    limit=20
)

events, pagination = client.events.list(query)
```

### Get Event

```python
event = client.events.get(123)
print(event.event_type)
```

### Create Event

```python
from datetime import datetime

event = NewTrackingEvent(
    product_id="PROD-001",
    actor_address="GABC123DEF456",
    timestamp=datetime.utcnow(),
    event_type="shipment",
    location="Shipping Port",
    data_hash="def456...",
    note="Shipped to customer",
    metadata={"tracking_number": "TRACK-123"},
)

created_event = client.events.create(event)
```

### Convenience Methods

```python
# Get all events for a product
events = client.events.get_all_for_product("PROD-001")

# Get specific event type for a product
shipments = client.events.get_by_type_for_product("PROD-001", "shipment")
```

## Statistics

```python
# Global statistics
stats = client.stats.get_global()
print(f"Total products: {stats.total_products}")
print(f"Active products: {stats.active_products}")
print(f"Total events: {stats.total_events}")

# Health checks
health = client.stats.health()
print(f"Service status: {health.status}")

db_health = client.stats.db_health()
print(f"Database status: {db_health.status}")
```

## Error Handling

The SDK provides specific exception types for different error scenarios:

```python
from chainlogistics_sdk import ChainLogisticsClient, Config
from chainlogistics_sdk.exceptions import (
    AuthenticationError,
    RateLimitError,
    NotFoundError,
    ValidationError,
    ApiError
)

try:
    products, _ = client.products.list()
except AuthenticationError:
    print("Invalid API key")
except RateLimitError:
    print("Rate limit exceeded - try again later")
except NotFoundError as e:
    print(f"Resource not found: {e}")
except ValidationError as e:
    print(f"Invalid request: {e}")
except ApiError as e:
    print(f"API error: {e}")
```

## Context Manager

Use the client as a context manager for automatic cleanup:

```python
with ChainLogisticsClient(config) as client:
    products, _ = client.products.list()
    # Client session is automatically closed
```

## Development

### Installation for Development

```bash
git clone https://github.com/chainlogistics/chainlogistics-sdk-python
cd chainlogistics-sdk-python
pip install -e ".[dev]"
```

### Running Tests

```bash
pytest
```

### Code Formatting

```bash
black src tests
isort src tests
```

### Type Checking

```bash
mypy src
```

## License

MIT License - see LICENSE file for details.

## Support

- Documentation: https://chainlogistics-sdk-python.readthedocs.io/
- Issues: https://github.com/chainlogistics/chainlogistics-sdk-python/issues
- Email: support@chainlogistics.io

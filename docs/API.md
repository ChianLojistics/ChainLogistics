# ChainLogistics API Documentation

## Overview

The ChainLogistics API provides comprehensive REST endpoints for managing supply chain products, tracking events, and accessing analytics data. This API is built with Rust/Axum for high performance and type safety.

## Base URL

- **Production**: `https://api.chainlogistics.io`
- **Staging**: `https://staging-api.chainlogistics.io`
- **Development**: `http://localhost:3001`

## Authentication

### API Key Authentication

All API endpoints (except health checks) require authentication using an API key.

**Header**: `X-API-Key: your-api-key-here`

#### Getting an API Key

1. Register at [ChainLogistics Portal](https://portal.chainlogistics.io)
2. Navigate to API Keys section
3. Generate a new key with appropriate tier

#### API Key Tiers

| Tier | Requests/Minute | Rate Limit | Features |
|------|----------------|------------|----------|
| Basic | 100 | 15-minute window | Read access |
| Standard | 1000 | 15-minute window | Read + Write |
| Enterprise | 10000 | 15-minute window | Full access + Webhooks |

## Error Handling

### Standard Error Response Format

```json
{
  "error": "Error message description",
  "code": "ERROR_CODE",
  "statusCode": 400,
  "timestamp": "2024-03-15T10:30:00Z",
  "path": "/api/v1/products"
}
```

### Common Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `INVALID_API_KEY` | API key is invalid or expired | 401 |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 |
| `PRODUCT_NOT_FOUND` | Product does not exist | 404 |
| `UNAUTHORIZED_ACCESS` | Insufficient permissions | 403 |
| `VALIDATION_ERROR` | Invalid request data | 400 |
| `INTERNAL_ERROR` | Server error | 500 |

## API Endpoints

### Health Check Endpoints

#### GET /health
Check API server status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-03-15T10:30:00Z",
  "version": "1.0.0"
}
```

#### GET /health/db
Check database connectivity.

**Response:**
```json
{
  "status": "healthy",
  "database": "connected",
  "latency_ms": 5
}
```

---

### Products API

#### GET /api/v1/products
List all products with optional filtering and pagination.

**Query Parameters:**
- `offset` (integer, optional): Number of items to skip (default: 0)
- `limit` (integer, optional): Maximum items to return (default: 20, max: 100)
- `owner_address` (string, optional): Filter by owner wallet address
- `category` (string, optional): Filter by product category
- `is_active` (boolean, optional): Filter by active status
- `search` (string, optional): Search in name and description

**Example Request:**
```bash
curl -X GET "https://api.chainlogistics.io/api/v1/products?limit=10&category=coffee&is_active=true" \
  -H "X-API-Key: your-api-key"
```

**Response:**
```json
{
  "products": [
    {
      "id": "PROD-12345",
      "name": "Ethiopian Single Origin Coffee",
      "description": "Premium Arabica beans from Sidamo region",
      "origin_location": "Sidamo, Ethiopia",
      "category": "coffee",
      "owner_address": "GABC...XYZ",
      "created_at": "2024-03-15T10:00:00Z",
      "updated_at": "2024-03-15T10:00:00Z",
      "is_active": true,
      "tags": ["organic", "fair-trade", "single-origin"],
      "certifications": ["USDA Organic", "Fair Trade Certified"],
      "media_hashes": ["QmHash1...", "QmHash2..."],
      "custom_fields": {
        "altitude": "1800m",
        "processing_method": "washed"
      }
    }
  ],
  "pagination": {
    "offset": 0,
    "limit": 10,
    "total": 150,
    "has_more": true
  }
}
```

#### POST /api/v1/admin/products
Create a new product (Admin only).

**Request Body:**
```json
{
  "id": "PROD-12345",
  "name": "Ethiopian Single Origin Coffee",
  "description": "Premium Arabica beans from Sidamo region",
  "origin_location": "Sidamo, Ethiopia",
  "category": "coffee",
  "tags": ["organic", "fair-trade", "single-origin"],
  "certifications": ["USDA Organic", "Fair Trade Certified"],
  "media_hashes": ["QmHash1...", "QmHash2..."],
  "custom_fields": {
    "altitude": "1800m",
    "processing_method": "washed"
  }
}
```

**Response:**
```json
{
  "id": "PROD-12345",
  "name": "Ethiopian Single Origin Coffee",
  "description": "Premium Arabica beans from Sidamo region",
  "origin_location": "Sidamo, Ethiopia",
  "category": "coffee",
  "owner_address": "GABC...XYZ",
  "created_at": "2024-03-15T10:00:00Z",
  "updated_at": "2024-03-15T10:00:00Z",
  "is_active": true,
  "tags": ["organic", "fair-trade", "single-origin"],
  "certifications": ["USDA Organic", "Fair Trade Certified"],
  "media_hashes": ["QmHash1...", "QmHash2..."],
  "custom_fields": {
    "altitude": "1800m",
    "processing_method": "washed"
  }
}
```

#### GET /api/v1/products/{id}
Get details of a specific product.

**Path Parameters:**
- `id` (string): Product ID

**Response:** Same as individual product object in list response.

#### PUT /api/v1/admin/products/{id}
Update a product (Admin only).

**Request Body:** All fields are optional.
```json
{
  "name": "Updated Product Name",
  "description": "Updated description",
  "is_active": false
}
```

#### DELETE /api/v1/admin/products/{id}
Delete a product (Admin only).

**Response:**
```json
{
  "message": "Product deleted successfully",
  "id": "PROD-12345"
}
```

---

### Events API

#### GET /api/v1/events
List all tracking events with optional filtering.

**Query Parameters:**
- `offset` (integer, optional): Number of items to skip
- `limit` (integer, optional): Maximum items to return
- `product_id` (string, optional): Filter by product ID
- `event_type` (string, optional): Filter by event type

**Example Request:**
```bash
curl -X GET "https://api.chainlogistics.io/api/v1/events?product_id=PROD-12345&limit=20" \
  -H "X-API-Key: your-api-key"
```

**Response:**
```json
{
  "events": [
    {
      "id": 1001,
      "product_id": "PROD-12345",
      "actor_address": "GDEF...ABC",
      "timestamp": "2024-03-15T10:30:00Z",
      "event_type": "HARVEST",
      "location": "Sidamo, Ethiopia",
      "data_hash": "0xabc123...",
      "note": "Harvested premium Arabica beans",
      "metadata": {
        "quantity_kg": 500,
        "quality_grade": "A",
        "batch_number": "BATCH-001"
      },
      "created_at": "2024-03-15T10:30:00Z"
    }
  ],
  "pagination": {
    "offset": 0,
    "limit": 20,
    "total": 45,
    "has_more": true
  }
}
```

#### POST /api/v1/admin/events
Create a new tracking event (Admin only).

**Request Body:**
```json
{
  "product_id": "PROD-12345",
  "actor_address": "GDEF...ABC",
  "timestamp": "2024-03-15T10:30:00Z",
  "event_type": "HARVEST",
  "location": "Sidamo, Ethiopia",
  "data_hash": "0xabc123...",
  "note": "Harvested premium Arabica beans",
  "metadata": {
    "quantity_kg": 500,
    "quality_grade": "A",
    "batch_number": "BATCH-001"
  }
}
```

#### GET /api/v1/events/{id}
Get details of a specific event.

**Response:** Same as individual event object in list response.

---

### Statistics API

#### GET /api/v1/stats
Get platform statistics and analytics.

**Response:**
```json
{
  "products": {
    "total": 1250,
    "active": 1180,
    "new_this_month": 85
  },
  "events": {
    "total": 15420,
    "today": 45,
    "this_week": 312
  },
  "users": {
    "total": 320,
    "active_this_month": 180
  },
  "blockchain": {
    "contract_address": "CBUWSKT2UGOAXK4ZREVDJV5XHSYB42PZ3CERU2ZFUTUMAZLJEHNZIECA",
    "network": "stellar-testnet",
    "total_transactions": 15420
  }
}
```

---

### Financial API

#### GET /api/v1/transactions
List financial transactions.

**Response:**
```json
{
  "transactions": [
    {
      "id": "TXN-789",
      "product_id": "PROD-12345",
      "type": "payment",
      "amount": 1500.00,
      "currency": "USD",
      "status": "completed",
      "from_address": "GABC...XYZ",
      "to_address": "GDEF...ABC",
      "timestamp": "2024-03-15T10:30:00Z",
      "blockchain_tx_hash": "0x123...abc"
    }
  ],
  "pagination": {
    "offset": 0,
    "limit": 20,
    "total": 850,
    "has_more": true
  }
}
```

#### GET /api/v1/transactions/{id}
Get details of a specific transaction.

#### POST /api/v1/admin/transactions
Create a new transaction (Admin only).

#### POST /api/v1/admin/invoices
Create a new invoice (Admin only).

#### POST /api/v1/admin/financing/request
Request supply chain financing (Admin only).

---

### Compliance API

#### POST /api/v1/compliance/check
Check compliance status for a product or batch.

**Request Body:**
```json
{
  "product_id": "PROD-12345",
  "check_type": "certification",
  "standards": ["USDA Organic", "Fair Trade"]
}
```

**Response:**
```json
{
  "product_id": "PROD-12345",
  "compliance_status": "compliant",
  "checks": [
    {
      "standard": "USDA Organic",
      "status": "compliant",
      "verified_at": "2024-03-15T10:30:00Z",
      "certificate_id": "USDA-2024-12345"
    },
    {
      "standard": "Fair Trade",
      "status": "compliant",
      "verified_at": "2024-03-15T10:30:00Z",
      "certificate_id": "FT-2024-67890"
    }
  ],
  "overall_score": 100
}
```

#### GET /api/v1/compliance/report/{product_id}
Get compliance report for a product.

#### GET /api/v1/audit/report
Generate comprehensive audit report.

**Query Parameters:**
- `start_date` (string): Start date (ISO 8601)
- `end_date` (string): End date (ISO 8601)
- `product_ids` (string, optional): Comma-separated product IDs
- `format` (string, optional): Output format (json, csv, pdf)

---

### Analytics API

#### GET /api/v1/analytics/dashboard
Get analytics dashboard data.

**Response:**
```json
{
  "overview": {
    "total_products": 1250,
    "total_events": 15420,
    "active_users": 180,
    "compliance_rate": 94.5
  },
  "trends": {
    "products_by_month": [
      {"month": "2024-01", "count": 45},
      {"month": "2024-02", "count": 62},
      {"month": "2024-03", "count": 85}
    ],
    "events_by_type": {
      "HARVEST": 3200,
      "PROCESSING": 2800,
      "SHIPPING": 4500,
      "QUALITY_CHECK": 2100,
      "RECEIVING": 2820
    }
  },
  "geographic": {
    "top_origins": [
      {"location": "Ethiopia", "count": 320},
      {"location": "Colombia", "count": 280},
      {"location": "Guatemala", "count": 195}
    ]
  }
}
```

#### GET /api/v1/analytics/products/{id}
Get detailed analytics for a specific product.

#### GET /api/v1/analytics/events
Get event analytics and trends.

#### GET /api/v1/analytics/users
Get user analytics and engagement metrics.

#### GET /api/v1/analytics/export
Export analytics data.

**Query Parameters:**
- `type` (string): Data type to export
- `format` (string): Export format (csv, json, xlsx)
- `start_date` (string): Start date
- `end_date` (string): End date

---

## Rate Limiting

### Rate Limit Headers

All API responses include rate limiting headers:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 950
X-RateLimit-Reset: 1710504000
```

### Rate Limit Response

When rate limit is exceeded:

```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "statusCode": 429,
  "retry_after": 300
}
```

---

## Webhooks

### Setting Up Webhooks

1. Contact support to configure webhook endpoints
2. Provide webhook URL and event types
3. Verify webhook signature using secret key

### Webhook Events

- `product.created`: New product registered
- `event.created`: New tracking event added
- `compliance.checked`: Compliance check completed
- `transaction.completed`: Financial transaction processed

### Webhook Payload Example

```json
{
  "event": "product.created",
  "timestamp": "2024-03-15T10:30:00Z",
  "data": {
    "product_id": "PROD-12345",
    "name": "Ethiopian Coffee"
  },
  "signature": "sha256=5d41402abc4b2a76b9719d911017c592"
}
```

---

## SDKs and Libraries

### Official SDKs

- **Rust**: `cargo install chainlogistics-sdk`
- **Python**: `pip install chainlogistics-sdk`
- **JavaScript**: `npm install @chainlogistics/sdk`

### Example Usage (JavaScript)

```javascript
import { ChainLogisticsAPI } from '@chainlogistics/sdk';

const client = new ChainLogisticsAPI({
  apiKey: 'your-api-key',
  baseURL: 'https://api.chainlogistics.io'
});

// List products
const products = await client.products.list({
  category: 'coffee',
  limit: 10
});

// Create product
const product = await client.products.create({
  id: 'PROD-12345',
  name: 'Ethiopian Coffee',
  category: 'coffee'
});
```

---

## Testing and Sandbox

### Sandbox Environment

- **URL**: `https://sandbox-api.chainlogistics.io`
- **Features**: Full API functionality with test data
- **Rate Limits**: Generous limits for testing

### Test Data

Use the sandbox environment with test API keys to:
- Test product registration
- Simulate supply chain events
- Verify webhook delivery
- Test compliance checks

---

## Support

### Getting Help

- **Documentation**: [docs.chainlogistics.io](https://docs.chainlogistics.io)
- **API Reference**: Interactive docs at `/swagger-ui`
- **Support Email**: api-support@chainlogistics.io
- **Status Page**: [status.chainlogistics.io](https://status.chainlogistics.io)

### Common Issues

1. **401 Unauthorized**: Check API key is valid and not expired
2. **429 Rate Limited**: Implement exponential backoff
3. **404 Not Found**: Verify resource ID is correct
4. **400 Bad Request**: Check request body format and required fields

---

## Changelog

### v1.0.0 (2024-03-15)
- Initial API release
- Product and event management
- Compliance checking
- Analytics dashboard
- Financial transactions

### v1.1.0 (Planned)
- Advanced filtering and search
- Bulk operations
- Enhanced analytics
- Mobile app API endpoints

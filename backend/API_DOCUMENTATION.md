# ChainLogistics API Documentation

## Overview

The ChainLogistics API provides a comprehensive RESTful interface for managing supply chain products, tracking events on blockchain, carbon footprint management, compliance checking, and financial operations.

**Base URL:** `https://api.chainlogistics.io`
**API Version:** v1

## Authentication

The API supports two authentication methods:

### 1. API Key Authentication

API keys are used for programmatic access to the API. Include your API key in the `X-API-Key` header.

**Request Headers:**
```
X-API-Key: your_api_key_here
```

**API Key Tiers:**
- **Basic**: 60 requests per minute
- **Standard**: 300 requests per minute
- **Premium**: 1000 requests per minute
- **Enterprise**: 5000 requests per minute

**Endpoints using API Key:**
- `/api/v1/products` (GET)
- `/api/v1/products/{id}` (GET)
- `/api/v1/events` (GET)
- `/api/v1/events/{id}` (GET)
- `/api/v1/stats` (GET)
- `/api/v1/transactions` (GET)
- `/api/v1/transactions/{id}` (GET)
- `/api/v1/compliance/check` (POST)
- `/api/v1/compliance/report/{product_id}` (GET)

### 2. JWT Authentication

JWT (JSON Web Token) authentication is used for admin and sensitive operations. Include your JWT token in the `Authorization` header.

**Request Headers:**
```
Authorization: Bearer your_jwt_token_here
```

**Obtaining a JWT Token:**
1. Register a user account via `/api/v1/admin/auth/register`
2. Login via `/api/v1/admin/auth/login` to receive your JWT token
3. Include the token in subsequent requests

**Token Expiration:**
- JWT tokens expire after 24 hours
- Refresh your token by logging in again

**Endpoints using JWT:**
- `/api/v1/admin/products` (POST, PUT, DELETE)
- `/api/v1/admin/events` (POST)
- `/api/v1/admin/transactions` (POST)
- `/api/v1/admin/invoices` (POST)
- `/api/v1/admin/financing/request` (POST)
- `/api/v1/admin/users` (POST, GET)
- `/api/v1/admin/auth/login` (POST)
- `/api/v1/admin/auth/register` (POST)
- `/api/v1/carbon/*` (all endpoints)
- `/api/v1/keys/*` (all endpoints)

## Error Responses

The API uses standard HTTP status codes and returns error responses in JSON format.

### Error Response Format

```json
{
  "error": "Error type",
  "message": "Detailed error message",
  "details": {
    "field": "Additional error details"
  }
}
```

### HTTP Status Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request - Invalid input data |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists |
| 422 | Unprocessable Entity - Validation error |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |
| 503 | Service Unavailable - Service temporarily down |

### Common Error Types

#### 400 Bad Request
```json
{
  "error": "Bad Request",
  "message": "Invalid input data",
  "details": {
    "field": "email",
    "reason": "Invalid email format"
  }
}
```

#### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Missing or invalid authentication credentials"
}
```

#### 403 Forbidden
```json
{
  "error": "Forbidden",
  "message": "You do not have permission to access this resource"
}
```

#### 404 Not Found
```json
{
  "error": "Not Found",
  "message": "Resource not found",
  "details": {
    "resource": "Product",
    "id": "123"
  }
}
```

#### 429 Rate Limit Exceeded
```json
{
  "error": "Rate Limit Exceeded",
  "message": "Too many requests",
  "details": {
    "limit": 60,
    "remaining": 0,
    "reset_at": "2024-01-01T00:00:00Z"
  }
}
```

#### 500 Internal Server Error
```json
{
  "error": "Internal Server Error",
  "message": "An unexpected error occurred",
  "details": {
    "request_id": "req_123456789"
  }
}
```

## Rate Limiting

All API endpoints are rate-limited based on your API key tier. Rate limits are applied per API key.

**Rate Limit Headers:**
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1640995200
```

## Pagination

List endpoints support pagination using query parameters.

**Query Parameters:**
- `offset`: Number of items to skip (default: 0)
- `limit`: Number of items to return (default: 20, max: 100)

**Example:**
```
GET /api/v1/products?offset=0&limit=20
```

**Paginated Response Format:**
```json
{
  "products": [...],
  "total": 150,
  "offset": 0,
  "limit": 20
}
```

## Swagger UI

Interactive API documentation is available via Swagger UI at:
```
https://api.chainlogistics.io/swagger-ui
```

The OpenAPI specification is available at:
```
https://api.chainlogistics.io/api-docs/openapi.json
```

## API Endpoints

### Products
- `GET /api/v1/products` - List all products
- `GET /api/v1/products/{id}` - Get a specific product
- `POST /api/v1/admin/products` - Create a new product
- `PUT /api/v1/admin/products/{id}` - Update a product
- `DELETE /api/v1/admin/products/{id}` - Delete a product

### Events
- `GET /api/v1/events` - List tracking events
- `GET /api/v1/events/{id}` - Get a specific event
- `POST /api/v1/admin/events` - Create a tracking event

### Authentication
- `POST /api/v1/admin/auth/login` - User login
- `POST /api/v1/admin/auth/register` - User registration

### Carbon Management
- `POST /api/v1/carbon/footprint/calculate` - Calculate carbon footprint
- `POST /api/v1/carbon/footprint/preview` - Preview carbon footprint
- `GET /api/v1/carbon/footprint/{product_id}` - List footprints for product
- `POST /api/v1/carbon/credits/generate` - Generate carbon credit
- `GET /api/v1/carbon/credits` - List carbon credits
- `GET /api/v1/carbon/credits/{id}` - Get carbon credit
- `POST /api/v1/carbon/credits/retire` - Retire carbon credit
- `GET /api/v1/carbon/market` - Get market summary
- `GET /api/v1/carbon/market/trades` - List marketplace trades
- `POST /api/v1/carbon/market/list` - List credit for sale
- `POST /api/v1/carbon/market/purchase` - Purchase carbon credit
- `POST /api/v1/carbon/verify` - Request verification
- `GET /api/v1/carbon/verify/{credit_id}` - List verifications
- `POST /api/v1/carbon/reports` - Generate report
- `GET /api/v1/carbon/reports` - List reports

### Financial
- `POST /api/v1/admin/transactions` - Create transaction
- `GET /api/v1/transactions` - List transactions
- `GET /api/v1/transactions/{id}` - Get transaction
- `POST /api/v1/admin/invoices` - Create invoice
- `POST /api/v1/admin/financing/request` - Request financing

### Compliance
- `POST /api/v1/compliance/check` - Check compliance
- `GET /api/v1/compliance/report/{product_id}` - Get compliance report
- `GET /api/v1/audit/report` - Generate audit report

### API Keys
- `POST /api/v1/keys` - Create API key
- `GET /api/v1/keys` - List API keys
- `POST /api/v1/keys/{id}/revoke` - Revoke API key
- `POST /api/v1/keys/{id}/rotate` - Rotate API key

### Statistics
- `GET /api/v1/stats` - Get global statistics

### Health
- `GET /health` - Health check
- `GET /health/db` - Database health check

## Example Requests

### Create a Product

**Request:**
```bash
curl -X POST https://api.chainlogistics.io/api/v1/admin/products \
  -H "Authorization: Bearer your_jwt_token" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "prod_123",
    "name": "Organic Coffee Beans",
    "description": "Premium organic coffee beans from Colombia",
    "origin_location": "Colombia",
    "category": "Beverages",
    "tags": ["organic", "fair-trade"],
    "certifications": ["USDA Organic", "Fair Trade"],
    "media_hashes": ["ipfs://QmXxx..."],
    "custom_fields": {"roast_level": "medium"}
  }'
```

**Response:**
```json
{
  "id": "prod_123",
  "name": "Organic Coffee Beans",
  "description": "Premium organic coffee beans from Colombia",
  "origin_location": "Colombia",
  "category": "Beverages",
  "tags": ["organic", "fair-trade"],
  "certifications": ["USDA Organic", "Fair Trade"],
  "media_hashes": ["ipfs://QmXxx..."],
  "custom_fields": {"roast_level": "medium"},
  "owner_address": "GDXXX...",
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "created_by": "user_123",
  "updated_by": "user_123"
}
```

### Login

**Request:**
```bash
curl -X POST https://api.chainlogistics.io/api/v1/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password"
  }'
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "uuid-123",
    "email": "user@example.com",
    "role": "Administrator",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Calculate Carbon Footprint

**Request:**
```bash
curl -X POST https://api.chainlogistics.io/api/v1/carbon/footprint/calculate \
  -H "Authorization: Bearer your_jwt_token" \
  -H "Content-Type: application/json" \
  -d '{
    "product_id": "prod_123",
    "transport_distance": 1000,
    "transport_method": "truck",
    "weight": 500
  }'
```

**Response:**
```json
{
  "id": "footprint_123",
  "product_id": "prod_123",
  "total_emissions": 250.5,
  "unit": "kg CO2e",
  "breakdown": {
    "transport": 200.0,
    "production": 50.5
  },
  "created_at": "2024-01-01T00:00:00Z"
}
```

## Support

For API support, contact:
- Email: support@chainlogistics.io
- Documentation: https://docs.chainlogistics.io

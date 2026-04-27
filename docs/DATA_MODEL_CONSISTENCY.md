# Data Model Consistency Guide

This document defines the data structure standards and consistency requirements across the ChainLogistics system layers: Smart Contracts, Backend, and Frontend.

## Overview

The ChainLogistics system uses three layers with different programming languages and conventions:

- **Smart Contracts** (Rust/Soroban): On-chain data storage with snake_case naming
- **Backend** (Rust): API and database layer with snake_case naming
- **Frontend** (TypeScript): UI layer with camelCase naming

## Naming Conventions

### Smart Contracts & Backend (Rust)
- Use **snake_case** for all field names
- Example: `product_id`, `event_type`, `actor_address`, `created_at`

### Frontend (TypeScript)
- Use **camelCase** for all field names
- Example: `productId`, `eventType`, `actorAddress`, `createdAt`

## Core Data Structures

### Product

**Smart Contract (types.rs):**
```rust
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin: Origin,
    pub owner: Address,
    pub created_at: u64,
    pub active: bool,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<BytesN<32>>,
    pub media_hashes: Vec<BytesN<32>>,
    pub custom: Map<Symbol, String>,
    pub deactivation_info: Vec<DeactInfo>,
}
```

**Backend (models.rs):**
```rust
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
    pub media_hashes: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}
```

**Frontend (types/product.ts):**
```typescript
export type Product = {
  id: ProductId;
  name: string;
  description: string;
  origin: {
    location: string;
  };
  owner: string; // Address as string
  createdAt: number; // Unix timestamp in seconds
  active: boolean;
  category: string;
  tags: string[];
  eventCount?: number; // Client-side computed field
};
```

### TrackingEvent

**Smart Contract (types.rs):**
```rust
pub struct TrackingEvent {
    pub event_id: u64,
    pub product_id: String,
    pub actor: Address,
    pub timestamp: u64,
    pub event_type: Symbol,
    pub location: String,
    pub data_hash: BytesN<32>,
    pub note: String,
    pub metadata: Map<Symbol, String>,
}
```

**Backend (models.rs):**
```rust
pub struct TrackingEvent {
    pub id: i64,
    pub product_id: String,
    pub actor_address: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub location: String,
    pub data_hash: String,
    pub note: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
```

**Frontend (types/tracking.ts):**
```typescript
export type TrackingEvent = {
  productId: ProductId;
  type: TrackingEventType;
  timestamp: number; // Unix timestamp in seconds
  location?: string;
  dataHash?: string;
  note?: string;
  metadata?: EventMetadata;
};
```

## Timestamp Standards

### Smart Contracts
- **Format**: `u64` (unsigned 64-bit integer)
- **Unit**: Unix timestamp in **seconds**
- **Example**: `1704067200` (January 1, 2024 00:00:00 UTC)

### Backend
- **Format**: `DateTime<Utc>` (chrono library)
- **Unit**: Full datetime object with timezone
- **Storage**: PostgreSQL TIMESTAMP WITH TIME ZONE
- **Conversion**: Automatically converts to/from Unix timestamps when interacting with smart contracts

### Frontend
- **Format**: `number` (JavaScript number)
- **Unit**: Unix timestamp in **seconds** (matching smart contract)
- **Example**: `1704067200`
- **Note**: JavaScript `Date` objects can be created via `new Date(timestamp * 1000)` for display

### Conversion Guide

**Smart Contract → Backend:**
```rust
// Convert u64 seconds to DateTime<Utc>
let datetime = DateTime::from_timestamp(timestamp as i64, 0).unwrap();
```

**Backend → Smart Contract:**
```rust
// Convert DateTime<Utc> to u64 seconds
let timestamp = datetime.timestamp() as u64;
```

**Smart Contract → Frontend:**
```typescript
// u64 seconds are directly usable as JavaScript numbers
const timestamp = event.timestamp; // Already in seconds
const date = new Date(timestamp * 1000); // Convert to Date for display
```

**Frontend → Smart Contract:**
```typescript
// JavaScript number (seconds) maps directly to u64
const timestamp = Math.floor(Date.now() / 1000); // Current time in seconds
```

## Field Mapping Reference

### Product Field Mapping

| Smart Contract | Backend | Frontend | Notes |
|---|---|---|---|
| `id` | `id` | `id` | Same across all layers |
| `name` | `name` | `name` | Same across all layers |
| `description` | `description` | `description` | Same across all layers |
| `origin.location` | `origin_location` | `origin.location` | Nested in frontend/contract |
| `owner` | `owner_address` | `owner` | Address format varies |
| `created_at` | `created_at` | `createdAt` | Timestamp format varies |
| `active` | `is_active` | `active` | Boolean naming varies |
| `category` | `category` | `category` | Same across all layers |
| `tags` | `tags` | `tags` | Same across all layers |
| `certifications` | `certifications` | - | Hash format varies |
| `custom` | `custom_fields` | - | Map vs JSON |

### TrackingEvent Field Mapping

| Smart Contract | Backend | Frontend | Notes |
|---|---|---|---|
| `event_id` | `id` | - | Backend uses simpler name |
| `product_id` | `product_id` | `productId` | camelCase in frontend |
| `actor` | `actor_address` | - | Address format varies |
| `timestamp` | `timestamp` | `timestamp` | Format varies (see above) |
| `event_type` | `event_type` | `type` | Shortened in frontend |
| `location` | `location` | `location` | Same across all layers |
| `data_hash` | `data_hash` | `dataHash` | camelCase in frontend |
| `note` | `note` | `note` | Same across all layers |
| `metadata` | `metadata` | `metadata` | Map vs JSON format |

## Required Fields

### Product
- **Required in all layers**: `id`, `name`, `description`, `category`, `owner/owner_address`
- **Optional in frontend**: `eventCount` (computed client-side)

### TrackingEvent
- **Required in smart contract**: `event_id`, `product_id`, `actor`, `timestamp`, `event_type`, `location`, `data_hash`, `note`
- **Required in backend**: `id`, `product_id`, `actor_address`, `timestamp`, `event_type`, `location`, `data_hash`, `note`
- **Optional in frontend**: `location`, `dataHash`, `note`, `metadata`

## Data Validation

### Smart Contract Level
- Validates on-chain before storage
- Type-safe via Soroban SDK
- Authorization checks for write operations

### Backend Level
- Validates incoming API requests
- Type-safe via Rust/SQLx
- Database constraints enforced

### Frontend Level
- Validates user input before API calls
- Type-safe via TypeScript
- UI-level validation feedback

## Best Practices

1. **Always use the correct timestamp format**: Unix seconds for smart contract/frontend, DateTime for backend
2. **Maintain field consistency**: If a field exists in one layer, it should exist in all layers unless explicitly documented
3. **Follow naming conventions**: snake_case for Rust, camelCase for TypeScript
4. **Document deviations**: Any field that differs between layers should be documented here
5. **Test conversions**: Always test data conversion between layers, especially timestamps
6. **Handle optional fields**: Use proper null/undefined checks for optional fields

## Migration Notes

When updating data structures:

1. Update smart contract types first (on-chain)
2. Update backend models and database schema
3. Update frontend TypeScript types
4. Update API handlers to handle both old and new formats during transition
5. Update UI components to use new field names
6. Add data migration scripts if needed
7. Update this documentation

## Future Considerations

- Consider adding a shared schema definition (e.g., OpenAPI/Swagger) to auto-generate types
- Evaluate using a code generation tool to ensure consistency
- Consider adding runtime validation libraries (e.g., Zod for frontend)
- Document any breaking changes in changelog

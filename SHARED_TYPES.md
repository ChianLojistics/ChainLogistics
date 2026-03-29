# Shared Data Types Specification

This document defines the canonical data structures used across the ChainLogistics platform to ensure consistency between smart contracts, backend API, and frontend.

## 🎯 Design Principles

1. **Smart Contract as Source of Truth**: Smart contract types define the core business logic
2. **Backend as Service Layer**: Backend provides API-friendly representations with additional metadata
3. **Frontend as Consumer**: Frontend types are optimized for UI/UX while maintaining compatibility

## 📋 Core Data Types

### Product

#### Smart Contract (Canonical)
```rust
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin: Origin { location: String },
    pub owner: Address,
    pub created_at: u64,        // Unix timestamp (seconds)
    pub active: bool,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<BytesN<32>>,
    pub media_hashes: Vec<BytesN<32>>,
    pub custom: Map<Symbol, String>,
    pub deactivation_info: Vec<DeactInfo>,
}
```

#### Backend API
```rust
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,    // Flattened from origin.location
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>, // Hash strings
    pub media_hashes: Vec<String>,   // Hash strings
    pub custom_fields: serde_json::Value,
    pub owner_address: String,       // Address as string
    pub is_active: bool,             // snake_case naming
    pub created_at: DateTime<Utc>,   // ISO datetime
    pub updated_at: DateTime<Utc>,   // Additional field
    pub created_by: String,          // Additional field
    pub updated_by: String,          // Additional field
}
```

#### Frontend
```typescript
export type Product = {
  id: string;
  name: string;
  description: string;
  origin: { location: string };     // Nested structure
  owner: string;                    // Address as string
  created_at: number;               // Unix timestamp (seconds)
  active: boolean;                  // camelCase naming
  category: string;
  tags: string[];
  certifications: string[];         // Hash strings
  media_hashes: string[];           // Hash strings
  custom_fields?: Record<string, string>;
  eventCount?: number;              // Computed field
  updated_at?: number;
  created_by?: string;
  updated_by?: string;
};
```

### Tracking Event

#### Smart Contract (Canonical)
```rust
pub struct TrackingEvent {
    pub event_id: u64,
    pub product_id: String,
    pub actor: Address,
    pub timestamp: u64,              // Unix timestamp (seconds)
    pub event_type: Symbol,
    pub location: String,
    pub data_hash: BytesN<32>,
    pub note: String,
    pub metadata: Map<Symbol, String>,
}
```

#### Backend API
```rust
pub struct TrackingEvent {
    pub id: i64,                     // Auto-increment primary key
    pub product_id: String,
    pub actor_address: String,       // Address as string
    pub timestamp: DateTime<Utc>,    // ISO datetime
    pub event_type: String,           // String representation of Symbol
    pub location: String,
    pub data_hash: String,            // Hash string
    pub note: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,    // Backend timestamp
}
```

#### Frontend
```typescript
export type TrackingEvent = {
  event_id: number;
  product_id: string;
  actor: string;                    // Address as string
  timestamp: number;                // Unix timestamp (seconds)
  event_type: TrackingEventType;    // Union of valid types
  location: string;
  data_hash: string;                // Hash string
  note: string;
  metadata?: EventMetadata;         // Structured metadata
};
```

## 🔄 Field Mapping

### Product Field Mappings

| Smart Contract | Backend API | Frontend | Notes |
|----------------|-------------|----------|-------|
| `id` | `id` | `id` | ✅ Consistent |
| `name` | `name` | `name` | ✅ Consistent |
| `description` | `description` | `description` | ✅ Consistent |
| `origin.location` | `origin_location` | `origin.location` | 🔄 Structure/naming |
| `owner` | `owner_address` | `owner` | 🔄 Naming |
| `created_at` | `created_at` | `created_at` | 🔄 Format (u64 vs DateTime vs number) |
| `active` | `is_active` | `active` | 🔄 Naming |
| `category` | `category` | `category` | ✅ Consistent |
| `tags` | `tags` | `tags` | ✅ Consistent |
| `certifications` | `certifications` | `certifications` | 🔄 Type (BytesN vs String) |
| `media_hashes` | `media_hashes` | `media_hashes` | 🔄 Type (BytesN vs String) |
| `custom` | `custom_fields` | `custom_fields` | 🔄 Naming/Type |
| `deactivation_info` | ❌ Missing | ❌ Missing | ❌ Backend/Frontend missing |

### Tracking Event Field Mappings

| Smart Contract | Backend API | Frontend | Notes |
|----------------|-------------|----------|-------|
| `event_id` | `id` | `event_id` | 🔄 Naming |
| `product_id` | `product_id` | `product_id` | ✅ Consistent |
| `actor` | `actor_address` | `actor` | 🔄 Naming |
| `timestamp` | `timestamp` | `timestamp` | 🔄 Format (u64 vs DateTime vs number) |
| `event_type` | `event_type` | `event_type` | 🔄 Type (Symbol vs String) |
| `location` | `location` | `location` | ✅ Consistent |
| `data_hash` | `data_hash` | `data_hash` | 🔄 Type (BytesN vs String) |
| `note` | `note` | `note` | ✅ Consistent |
| `metadata` | `metadata` | `metadata` | 🔄 Type (Map vs JSON vs structured) |

## 📅 Timestamp Standards

- **Smart Contract**: Unix timestamp (seconds) as `u64`
- **Backend**: ISO 8601 datetime string as `DateTime<Utc>`
- **Frontend**: Unix timestamp (seconds) as `number`

## 🔗 Address Format

- **Smart Contract**: Stellar `Address` type
- **Backend**: String representation of Stellar address
- **Frontend**: String representation of Stellar address

## 🔐 Hash Format

- **Smart Contract**: 32-byte hash as `BytesN<32>`
- **Backend**: Hex string representation
- **Frontend**: Hex string representation

## 📝 Metadata Format

- **Smart Contract**: `Map<Symbol, String>` - key-value pairs
- **Backend**: `serde_json::Value` - flexible JSON structure
- **Frontend**: `EventMetadata` - structured interface

## 🎛️ Event Types

Standardized event types across all layers:
- `REGISTER` - Product registration
- `TRANSFER` - Ownership transfer
- `CHECKPOINT` - Generic checkpoint
- `HARVEST` - Harvest event
- `PROCESSING` - Processing event
- `PACKAGING` - Packaging event
- `SHIPPING` - Shipping event
- `RECEIVING` - Receiving event
- `QUALITY_CHECK` - Quality check event

## 🔍 Validation Rules

- **Product ID**: Alphanumeric, 1-50 characters
- **Stellar Address**: G-prefixed, 56 characters, alphanumeric
- **Hash**: SHA-256 hex, 64 characters, hexadecimal
- **Timestamp**: Positive integer, reasonable range
- **Event Type**: Must be one of predefined types

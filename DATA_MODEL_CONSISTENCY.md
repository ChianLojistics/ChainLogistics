# Data Model Consistency Analysis & Fixes

## 🚨 Current Inconsistencies

### 1. Product Model Issues

#### **Origin Structure**
- **Smart Contract**: `origin: Origin { location: String }`
- **Backend**: `origin_location: String`
- **Frontend**: `origin: { location: string }`

#### **Owner Field**
- **Smart Contract**: `owner: Address`
- **Backend**: `owner_address: String`
- **Frontend**: `owner: string`

#### **Active Status**
- **Smart Contract**: `active: bool`
- **Backend**: `is_active: bool`
- **Frontend**: `active: boolean`

#### **Timestamp Format**
- **Smart Contract**: `created_at: u64` (Unix timestamp seconds)
- **Backend**: `created_at: DateTime<Utc>` (ISO datetime)
- **Frontend**: `created_at: number` (Unix timestamp milliseconds)

#### **Missing Fields**
- `certifications`: Present in SC/backend, missing in frontend
- `media_hashes`: Present in SC/backend, missing in frontend
- `custom_fields`: Present in backend, different in SC, missing in frontend
- `deactivation_info`: Only in smart contract

### 2. TrackingEvent Model Issues

#### **ID Field**
- **Smart Contract**: `event_id: u64`
- **Backend**: `id: i64`
- **Frontend**: `event_id: number`

#### **Actor Field**
- **Smart Contract**: `actor: Address`
- **Backend**: `actor_address: String`
- **Frontend**: `actor: string`

#### **Event Type**
- **Smart Contract**: `event_type: Symbol`
- **Backend**: `event_type: String`
- **Frontend**: `event_type: string`

#### **Location Field**
- **Smart Contract**: `location: String`
- **Backend**: `location: String`
- **Frontend**: Missing

#### **Data Hash**
- **Smart Contract**: `data_hash: BytesN<32>`
- **Backend**: `data_hash: String`
- **Frontend**: `data_hash?: string`

#### **Metadata**
- **Smart Contract**: `metadata: Map<Symbol, String>`
- **Backend**: `metadata: serde_json::Value`
- **Frontend**: `metadata?: EventMetadata`

## 🎯 Proposed Standardization

### 1. Unified Naming Convention
**Adopt camelCase for frontend, snake_case for backend/contracts**

### 2. Timestamp Standardization
**Use Unix timestamp (seconds) consistently across all layers**

### 3. Address Handling
**Use string representation consistently, validate format in each layer**

### 4. Missing Field Resolution
**Add missing fields to respective models with proper type conversions**

## 📋 Implementation Plan

1. Update frontend types to include missing fields
2. Standardize timestamp formats
3. Add conversion utilities between layers
4. Update API contracts to match
5. Add validation for data consistency

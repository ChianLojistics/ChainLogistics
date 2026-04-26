# Contributing to ChainLojistic 🚀

Thank you for your interest in contributing to ChainLojistic! This comprehensive guide will help you contribute effectively to our open-source supply chain tracking platform.

## 📚 Table of Contentss
- [Quick Start](#quick-start)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
- [Issue Labels](#issue-labels)
- [Detailed GitHub Issues](#detailed-github-issues)
  - [Smart Contract Issues](#smart-contract-issues)
  - [Frontend Issues](#frontend-issues)
  - [Backend Issues](#backend-issues)
  - [Testing Issues](#testing-issues)
  - [Documentation Issues](#documentation-issues)
- [Pull Request Process](#pull-request-process)
- [Code Style Guidelines](#code-style-guidelines)

---

## 🚀 Quick Start

ChainLojistic is a decentralized supply chain tracker built on Stellar's Soroban. It has three components:

1. **Smart Contracts** (Rust/Soroban) - On-chain logic
2. **Frontend** (Next.js 15/React 19/TypeScript) - Web UI
3. **Backend** (Rust/Axum/SQLx) - High-Performance API Server

**New contributors**: Look for issues labeled `good first issue`!

---

## 💻 Development Setup

### Prerequisites

#### Smart Contracts:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli --features opt

# Add WASM target
rustup target add wasm32-unknown-unknown
```

#### Frontend & Backend:
- Node.js 18+ (for frontend)
- Rust 1.70+ (for backend)
- PostgreSQL 14+ (for backend database)
- Redis 6+ (for backend caching)
- npm or yarn
- Git

### Setup Instructions

```bash
# 1. Fork the repo on GitHub

# 2. Clone YOUR fork
git clone https://github.com/ChainLojistics/ChainLogistics.git
cd ChainLogistics

# 3. Add upstream
git remote add upstream https://github.com/ChainLojistics/ChainLogistics.git

# 4. Smart Contracts
cd contracts
cargo build --target wasm32-unknown-unknown --release
cargo test

# 5. Frontend
cd ../frontend
npm install
npm run dev  # http://localhost:3000

# 6. Backend (Rust/Axum)
cd ../backend
cargo build
cargo test
cp .env.example .env
cargo run  # http://localhost:3001
```

---

## 📁 Project Structure
```
ChainLojistic/
├── contracts/                           # Soroban Smart Contracts
│   ├── src/
│   │   ├── lib.rs                      # Contract entry point & exports
│   │   ├── contract.rs                 # Main contract implementation
│   │   ├── types.rs                    # Data structures (Product, Event)
│   │   ├── storage.rs                  # Storage keys & helpers
│   │   ├── error.rs                    # Custom error types
│   │   ├── events.rs                   # Event emission
│   │   ├── validation.rs               # Input validation logic
│   │   └── test/
│   │       ├── mod.rs                  # Test module exports
│   │       ├── setup.rs                # Test utilities & fixtures
│   │       ├── product_tests.rs        # Product function tests
│   │       ├── event_tests.rs          # Event tracking tests
│   │       ├── access_tests.rs         # Authorization tests
│   │       └── integration_tests.rs    # Full workflow tests
│   ├── Cargo.toml
│   └── README.md
│
├── frontend/                            # Next.js Application
│   ├── app/
│   │   ├── layout.tsx                  # Root layout
│   │   ├── page.tsx                    # Homepage (EXISTS)
│   │   ├── globals.css                 # Global styles
│   │   ├── register/
│   │   │   └── page.tsx               # Product registration
│   │   ├── products/
│   │   │   ├── page.tsx               # Products list
│   │   │   └── [id]/
│   │   │       ├── page.tsx           # Product detail
│   │   │       └── add-event/
│   │   │           └── page.tsx       # Add tracking event
│   │   ├── verify/
│   │   │   └── [id]/
│   │   │       └── page.tsx           # QR verification page
│   │   └── analytics/
│   │       └── page.tsx               # Analytics dashboard
│   ├── components/
│   │   ├── wallet/
│   │   │   ├── WalletConnect.tsx      # Wallet connection button
│   │   │   └── WalletStatus.tsx       # Wallet status indicator
│   │   ├── forms/
│   │   │   ├── ProductForm.tsx        # Product registration form
│   │   │   ├── EventForm.tsx          # Event tracking form
│   │   │   └── FormInput.tsx          # Reusable form input
│   │   ├── tracking/
│   │   │   ├── Timeline.tsx           # Event timeline
│   │   │   ├── EventCard.tsx          # Single event display
│   │   │   └── EventFilters.tsx       # Filter events
│   │   ├── products/
│   │   │   ├── ProductCard.tsx        # Product card
│   │   │   ├── ProductList.tsx        # Products grid
│   │   │   └── ProductDetails.tsx     # Product info display
│   │   ├── qr/
│   │   │   ├── QRGenerator.tsx        # Generate QR codes
│   │   │   └── QRScanner.tsx          # Scan QR codes
│   │   ├── charts/
│   │   │   ├── EventsChart.tsx        # Events visualization
│   │   │   └── OriginChart.tsx        # Origin distribution
│   │   └── ui/
│   │       ├── Button.tsx             # Reusable button
│   │       ├── Card.tsx               # Reusable card
│   │       ├── Input.tsx              # Reusable input
│   │       ├── Modal.tsx              # Modal component
│   │       └── LoadingSpinner.tsx     # Loading state
│   ├── lib/
│   │   ├── stellar/
│   │   │   ├── client.ts              # Stellar RPC client
│   │   │   ├── contract.ts            # Contract interaction
│   │   │   ├── wallet.ts              # Wallet utilities
│   │   │   └── types.ts               # Stellar types
│   │   ├── utils/
│   │   │   ├── format.ts              # Formatting helpers
│   │   │   ├── validation.ts          # Client-side validation
│   │   │   └── constants.ts           # Constants
│   │   └── hooks/
│   │       ├── useContract.ts         # Contract interaction hook
│   │       ├── useProducts.ts         # Product data hook
│   │       ├── useEvents.ts           # Events data hook
│   │       └── useWallet.ts           # Wallet hook
│   ├── contexts/
│   │   ├── WalletContext.tsx          # Wallet state
│   │   └── ContractContext.tsx        # Contract state
│   ├── types/
│   │   ├── product.ts                 # Product types
│   │   ├── event.ts                   # Event types
│   │   └── api.ts                     # API types
│   ├── public/
│   │   ├── images/
│   │   └── icons/
│   ├── tests/
│   │   ├── unit/                      # Unit tests
│   │   └── e2e/                       # E2E tests
│   ├── package.json
│   ├── tsconfig.json
│   ├── tailwind.config.ts
│   └── next.config.ts
│
├── backend/                             # Rust API Server (Axum)
│   ├── src/
│   │   ├── main.rs                    # Server entry point
│   │   ├── lib.rs                     # Library exports
│   │   ├── routes/
│   │   │   ├── mod.rs                 # Route module
│   │   │   ├── products.rs            # Product routes
│   │   │   ├── events.rs              # Event routes
│   │   │   ├── analytics.rs           # Analytics routes
│   │   │   └── webhooks.rs            # Webhook routes
│   │   ├── services/
│   │   │   ├── soroban_service.rs     # Contract interactions
│   │   │   ├── cache_service.rs        # Redis caching
│   │   │   └── webhook_service.rs      # Webhook handling
│   │   ├── middleware/
│   │   │   ├── mod.rs                 # Middleware module
│   │   │   ├── auth.rs                # Authentication
│   │   │   └── rate_limit.rs          # Rate limiting
│   │   ├── models/
│   │   │   ├── mod.rs                 # Model exports
│   │   │   ├── product.rs              # Product structs
│   │   │   └── event.rs               # Event structs
│   │   ├── config/
│   │   │   └── mod.rs                 # Configuration
│   │   ├── database/
│   │   │   └── mod.rs                 # Database layer
│   │   └── utils/
│   │       └── mod.rs                 # Utilities
│   ├── migrations/                     # SQLx migrations
│   ├── tests/
│   │   ├── unit/
│   │   └── integration/
│   ├── Cargo.toml                      # Rust dependencies
│   └── .env.example                    # Environment variables
│
├── docs/                                # Documentation
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── DEPLOYMENT.md
│   └── images/
│
├── .github/                             # GitHub configs
│   ├── workflows/
│   │   ├── contracts-ci.yml           # Contract CI/CD
│   │   ├── frontend-ci.yml            # Frontend CI/CD
│   │   └── backend-ci.yml             # Backend CI/CD
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── good_first_issue.md
│   └── pull_request_template.md
│
├── .gitignore
├── README.md
├── CONTRIBUTING.md
├── LICENSE
└── package.json                         # Root workspace config

---

## 🤝 How to Contribute

### Step-by-Step

1. **Find an Issue**
   - Browse [GitHub Issues](link)
   - Look for `good first issue` or `help wanted`
   - Read the issue description carefully

2. **Claim the Issue**
   - Comment: "I'd like to work on this!"
   - Wait for assignment from maintainer
   - Ask questions if unclear

3. **Create Your Branch**
   ```bash
   git checkout main
   git pull upstream main
   git checkout -b feature/issue-23-wallet-connection
   ```

4. **Make Changes**
   - Write clean, documented code
   - Follow style guidelines
   - Add tests if applicable

5. **Test Everything**
   ```bash
   # Contracts
   cd contracts && cargo test
   
   # Frontend
   cd frontend && npm run build
   
   # Backend
   cd backend && cargo test
   ```

6. **Commit & Push**
   ```bash
   git add .
   git commit -m "feat: add wallet connection (#23)"
   git push origin feature/issue-23-wallet-connection
   ```

7. **Open Pull Request**
   - Go to your fork on GitHub
   - Click "Compare & pull request"
   - Fill out PR template
   - Link issue: "Closes #23"
   - Request review

8. **Address Feedback**
   - Respond to comments
   - Make requested changes
   - Push updates

---

## 🏷️ Issue Labels

| Label | Description | Difficulty |
|-------|-------------|------------|
| `good first issue` | Perfect for newcomers | ⭐ Easy |
| `help wanted` | Need contributors | ⭐⭐ Medium |
| `bug` | Something's broken | Varies |
| `enhancement` | New feature | ⭐⭐⭐ Hard |
| `documentation` | Docs work | ⭐ Easy |
| `smart-contract` | Soroban/Rust | ⭐⭐⭐ Hard |
| `frontend` | Next.js/React | ⭐⭐ Medium |
| `backend` | Rust/Axum/API | ⭐⭐ Medium |
| `testing` | Test coverage | ⭐⭐ Medium |
| `design` | UI/UX work | ⭐⭐ Medium |
| `priority: high` | Urgent | - |
| `priority: low` | Nice to have | - |

---

## 📋 Detailed GitHub Issues

Below are ready-to-use GitHub issue templates. Copy these into your GitHub Issues to help contributors.

---

## SMART CONTRACT ISSUES

### Issue #1: Implement Product Storage with Persistent Data
**Labels:** `smart-contract` `enhancement` `good first issue`

#### Description
The `register_product` function currently creates a Product struct but doesn't persist it to storage properly. We need to implement durable storage using Soroban's storage API.

#### What You'll Learn
- Soroban storage patterns (persistent vs temporary)
- Rust struct serialization
- Blockchain data structures

#### Requirements
- [ ] Store products in persistent storage using proper keys
- [ ] Implement `get_product` to retrieve by ID
- [ ] Prevent duplicate product IDs
- [ ] Add error handling for missing products

#### Technical Approach
```rust
use soroban_sdk::storage::Persistent;

// Storage key
let key = symbol_short!("PRODUCT");
let product_key = (key, id.clone());

// Store
env.storage().persistent().set(&product_key, &product);

// Retrieve
env.storage().persistent().get(&product_key)
```

#### Testing
```bash
cd contracts
cargo test test_product_storage
cargo build --target wasm32-unknown-unknown --release
```

#### Acceptance Criteria
- [ ] Products persist across contract calls
- [ ] `get_product` returns correct data
- [ ] Duplicate IDs are rejected with clear error
- [ ] All tests pass
- [ ] Code is documented with `///` comments

#### Files to Modify
- `contracts/src/lib.rs`

#### Resources
- [Soroban Storage Guide](https://soroban.stellar.org/docs/learn/persisting-data)
- [Example: Token Contract Storage](https://github.com/stellar/soroban-examples/tree/main/token)

#### Estimated Time
2-4 hours for someone new to Soroban

---

### Issue #2: Add Access Control for Tracking Events
**Labels:** `smart-contract` `security` `priority: high`

#### Description
Anyone can currently add tracking events to any product. We need role-based access control so only authorized parties can update products.

#### What You'll Learn
- Smart contract security patterns
- Authorization in Soroban
- Address verification

#### Requirements
- [ ] Add `authorized_actors: Vec<Address>` to Product struct
- [ ] Create `add_authorized_actor(owner, actor)` function
- [ ] Verify actor in `add_tracking_event`
- [ ] Create `remove_authorized_actor` function
- [ ] Emit events on authorization changes

#### Security Considerations
```rust
// Verify caller is authorized
actor.require_auth();

// Check if actor is in authorized list
if !product.authorized_actors.contains(&actor) {
    panic_with_error!(&env, Error::Unauthorized);
}
```

#### Testing Scenarios
1. Owner adds authorized actor ✅
2. Authorized actor adds event ✅
3. Unauthorized actor adds event ❌ (should fail)
4. Owner removes actor ✅
5. Removed actor adds event ❌ (should fail)

#### Acceptance Criteria
- [ ] Only owner can authorize actors
- [ ] Only authorized actors can add events
- [ ] Proper error messages
- [ ] No breaking changes to existing API
- [ ] Security tests pass

#### Estimated Time
4-6 hours

---

### Issue #3: Implement Batch Event Addition
**Labels:** `smart-contract` `enhancement` `optimization`

#### Description
Currently, adding multiple events requires multiple transactions. Implement batch operations to reduce costs and improve efficiency.

#### What You'll Learn
- Gas optimization
- Batch processing patterns
- Atomic operations

#### Requirements
- [ ] Create `add_tracking_events_batch` function
- [ ] Accept `Vec<EventInput>`
- [ ] Validate all events before adding any (atomic)
- [ ] Return `Vec<TrackingEvent>`
- [ ] Optimize for gas efficiency

#### API Design
```rust
pub struct EventInput {
    pub product_id: String,
    pub location: String,
    pub event_type: String,
    pub metadata: String,
}

pub fn add_tracking_events_batch(
    env: Env,
    actor: Address,
    events: Vec<EventInput>
) -> Vec<TrackingEvent>
```

#### Gas Optimization Tips
- Single authorization check
- Batch storage writes
- Minimize contract calls

#### Testing
- Test with 1 event
- Test with 10 events
- Test with 100 events
- Test partial failure (should rollback all)

#### Acceptance Criteria
- [ ] All events added atomically
- [ ] More gas-efficient than individual calls
- [ ] Handles up to 100 events
- [ ] Tests verify atomicity

#### Estimated Time
6-8 hours

---

## FRONTEND ISSUES

### Issue #4: Create Wallet Connection Component
**Labels:** `frontend` `enhancement` `good first issue`

#### Description
Build a component that connects to Freighter wallet, displays connection status, and manages wallet state.

#### What You'll Learn
- Wallet integration
- React hooks (useState, useEffect)
- Context API for global state

#### Requirements
- [ ] Detect Freighter wallet installation
- [ ] Connect/disconnect functionality
- [ ] Display connected address (truncated)
- [ ] Store wallet state in Context
- [ ] Handle connection errors gracefully

#### UI Requirements
```
Disconnected:
[🔗 Connect Wallet] button

Connected:
[0x1234...5678] [Disconnect]
```

#### Component Structure
```typescript
// components/wallet/WalletConnect.tsx
import { useState, useEffect } from 'react';
import { isConnected, getPublicKey } from '@stellar/freighter-api';

export function WalletConnect() {
  const [address, setAddress] = useState<string | null>(null);
  
  async function connect() {
    // Connection logic
  }
  
  return (
    // UI
  );
}
```

#### Context Setup
```typescript
// contexts/WalletContext.tsx
const WalletContext = createContext({
  address: null,
  connect: () => {},
  disconnect: () => {},
});
```

#### Testing Checklist
- [ ] Works with Freighter installed
- [ ] Shows error without Freighter
- [ ] Handles user rejection
- [ ] Persists on refresh
- [ ] Mobile responsive

#### Acceptance Criteria
- [ ] Connects successfully
- [ ] Errors handled gracefully
- [ ] State managed with Context
- [ ] Responsive design
- [ ] Clean UI/UX

#### Files to Create
- `frontend/components/wallet/WalletConnect.tsx`
- `frontend/contexts/WalletContext.tsx`
- `frontend/lib/wallet.ts`

#### Dependencies
```bash
npm install @stellar/freighter-api
```

#### Resources
- [Freighter Docs](https://docs.freighter.app/)

#### Estimated Time
3-5 hours

---

### Issue #5: Build Product Registration Form
**Labels:** `frontend` `enhancement` `medium`

#### Description
Create a multi-step form for registering new products on the blockchain.

#### What You'll Learn
- Form validation
- Multi-step workflows
- Smart contract interaction
- Transaction signing

#### Form Steps
1. **Basic Info**: Product ID, Name
2. **Origin Details**: Location, Description, Certifications
3. **Review & Submit**: Preview all data

#### Requirements
- [ ] Multi-step form with progress indicator
- [ ] Form validation (required fields, formats)
- [ ] Connect to smart contract
- [ ] Sign transaction with wallet
- [ ] Show loading state during submission
- [ ] Success page with product link
- [ ] Error handling

#### Form Structure
```typescript
interface ProductFormData {
  id: string;
  name: string;
  origin: string;
  description: string;
  initialLocation: string;
}

function ProductRegistrationForm() {
  const [step, setStep] = useState(1);
  const [formData, setFormData] = useState<ProductFormData>({});
  
  async function handleSubmit() {
    // Call smart contract
  }
}
```

#### Validation Rules
- ID: Required, alphanumeric, max 20 chars
- Name: Required, min 3 chars
- Origin: Required

#### Transaction Flow
```typescript
import { Contract, SorobanRpc } from '@stellar/stellar-sdk';

async function registerProduct(data: ProductFormData) {
  // 1. Build transaction
  const contract = new Contract(CONTRACT_ID);
  const tx = contract.register_product({...});
  
  // 2. Sign with wallet
  const signedTx = await signTransaction(tx);
  
  // 3. Submit to network
  const result = await submitTransaction(signedTx);
  
  // 4. Return product ID
  return result.productId;
}
```

#### UI/UX
- Step indicator: ●○○
- Disabled "Next" until valid
- Back button on steps 2-3
- Loading spinner on submit
- Success message with QR code

#### Acceptance Criteria
- [ ] All steps work
- [ ] Validation prevents invalid data
- [ ] Successfully calls contract
- [ ] Transaction confirmed
- [ ] Redirects to product page
- [ ] Mobile responsive

#### Files to Create
- `frontend/app/register/page.tsx`
- `frontend/components/forms/ProductRegistrationForm.tsx`
- `frontend/lib/contract.ts`

#### Estimated Time
8-12 hours

---

### Issue #6: Create Product Timeline Component
**Labels:** `frontend` `enhancement` `design` `good first issue`

#### Description
Build a visual timeline showing all tracking events for a product in chronological order.

#### What You'll Learn
- Data visualization
- API integration
- Responsive design
- Loading states

#### Requirements
- [ ] Fetch events from smart contract
- [ ] Display in chronological order
- [ ] Visual timeline with connecting lines
- [ ] Event cards with all details
- [ ] Icons for event types
- [ ] Responsive (vertical desktop, horizontal mobile)
- [ ] Loading skeleton
- [ ] Empty state

#### Event Types & Icons
```typescript
const EVENT_ICONS = {
  HARVEST: '🌱',
  PROCESSING: '⚙️',
  PACKAGING: '📦',
  SHIPPING: '🚚',
  RECEIVING: '📥',
  QUALITY_CHECK: '✅',
};
```

#### Component Structure
```typescript
interface TimelineProps {
  productId: string;
}

function Timeline({ productId }: TimelineProps) {
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    loadEvents();
  }, [productId]);
  
  return (
    <div className="timeline">
      {events.map(event => (
        <EventCard key={event.id} event={event} />
      ))}
    </div>
  );
}
```

#### Event Card Design
```
┌─────────────────────────┐
│ 🚚 SHIPPING             │
│ May 15, 2024 2:30 PM    │
│ Port of Seattle         │
│ Shipped via cargo       │
│ Actor: 0x1234...5678    │
└─────────────────────────┘
```

#### Responsive Behavior
- Desktop: Vertical timeline, left-aligned
- Mobile: Horizontal scroll, compact cards

#### Acceptance Criteria
- [ ] Events load from blockchain
- [ ] Timeline displays correctly
- [ ] All event details shown
- [ ] Icons match event types
- [ ] Responsive design
- [ ] Loading state
- [ ] Empty state

#### Files to Create
- `frontend/components/tracking/Timeline.tsx`
- `frontend/components/tracking/EventCard.tsx`
- `frontend/lib/events.ts`

#### Estimated Time
4-6 hours

---

### Issue #7: Implement QR Code Generation
**Labels:** `frontend` `enhancement` `medium`

#### Description
Generate QR codes for products that link to their verification page.

#### What You'll Learn
- QR code generation
- File downloads
- Print layouts

#### Requirements
- [ ] Generate QR code for product verification URL
- [ ] Display on product page
- [ ] Download as PNG
- [ ] Download as SVG
- [ ] Print-friendly layout
- [ ] Copy verification link

#### Implementation
```typescript
import QRCode from 'qrcode';

async function generateQR(productId: string) {
  const url = `${process.env.NEXT_PUBLIC_APP_URL}/verify/${productId}`;
  const qrDataUrl = await QRCode.toDataURL(url, {
    width: 300,
    margin: 2,
  });
  return qrDataUrl;
}
```

#### UI Layout
```
┌─────────────────┐
│   [QR Code]     │
│                 │
│   PROD-12345    │
│                 │
│ [📥 PNG] [📥 SVG]
│ [🖨️ Print] [📋 Copy]
└─────────────────┘
```

#### Download Functionality
```typescript
function downloadQR(dataUrl: string, format: 'png' | 'svg') {
  const link = document.createElement('a');
  link.href = dataUrl;
  link.download = `product-${productId}-qr.${format}`;
  link.click();
}
```

#### Acceptance Criteria
- [ ] QR codes generate correctly
- [ ] Scannable with phone camera
- [ ] Links to verification page
- [ ] PNG download works
- [ ] SVG download works
- [ ] Print layout is clean

#### Files to Create
- `frontend/components/qr/QRCodeGenerator.tsx`
- `frontend/lib/qr.ts`

#### Dependencies
```bash
npm install qrcode
npm install -D @types/qrcode
```

#### Estimated Time
3-4 hours

---

## BACKEND ISSUES

### Issue #8: Create Product API Endpoints
**Labels:** `backend` `enhancement` `good first issue`

#### Description
Build RESTful API endpoints for product CRUD operations.

#### What You'll Learn
- REST API design with Rust/Axum
- Async Rust programming
- Soroban integration from Rust
- Error handling with thiserror

#### Endpoints to Create
```
GET    /api/products          - List all products
POST   /api/products          - Register product
GET    /api/products/:id      - Get product by ID
GET    /api/products/:id/events  - Get events
POST   /api/products/:id/events  - Add event
```

#### Implementation
```rust
// src/routes/products.rs
use axum::{extract::Path, response::Json, routing::get, Router};
use soroban_sdk::{Address, Env};

pub fn product_routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route("/products/:id", get(get_product))
        .route("/products/:id/events", get(get_product_events))
}

async fn get_product(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<Product>, AppError> {
    let product = app_state.soroban_service.get_product(&id).await?;
    Ok(Json(product))
}
```

#### Validation
```rust
// src/middleware/validation.rs
use axum::{extract::Request, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 3, message = "Name must be at least 3 characters"))]
    pub name: String,
    
    #[validate(length(min = 1, message = "Origin is required"))]
    pub origin: String,
    
    #[validate(custom = "validate_stellar_address")]
    pub owner: String,
}

pub async fn validate_product(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Validation logic here
    Ok(next.run(req).await)
}
```
#### Error Responses
```json
{
  "error": "Product not found",
  "code": "PRODUCT_NOT_FOUND",
  "statusCode": 404
}
```

#### Acceptance Criteria
- [ ] All endpoints functional with proper validation
- [ ] Error handling works correctly
- [ ] Contract integration tested
- [ ] API responses follow consistent format
- [ ] Rate limiting applied

#### Files to Create
- `backend/src/routes/products.rs`
- `backend/src/services/soroban_service.rs`
- `backend/src/middleware/validation.rs`

#### Dependencies
```toml
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
validator = "0.16"
thiserror = "1.0"
soroban-sdk = "21.0"
```

#### Estimated Time
4-6 hours

---

### Issue #9: Add Pagination to Events API
**Labels:** `backend` `enhancement` `medium`

#### Description
Implement pagination for tracking events to handle products with many events efficiently.

#### What You'll Learn
- Pagination patterns
- Query parameters
- Performance optimization

#### API Design
```
GET /api/products/:id/events?page=1&limit=20&sort=desc

Response:
{
  "events": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150,
    "totalPages": 8,
    "hasNext": true,
    "hasPrev": false
  }
}
```

#### Implementation
```typescript
router.get('/products/:id/events', async (req, res) => {
  const page = parseInt(req.query.page as string) || 1;
  const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
  const sort = req.query.sort === 'asc' ? 'asc' : 'desc';
  
  const allEvents = await getProductEvents(id);
  const sortedEvents = sortEvents(allEvents, sort);
  const paginatedEvents = paginate(sortedEvents, page, limit);
  
  res.json({
    events: paginatedEvents,
    pagination: buildPaginationMeta(allEvents.length, page, limit)
  });
});
```

#### Helper Functions
```typescript
function paginate<T>(items: T[], page: number, limit: number): T[] {
  const start = (page - 1) * limit;
  const end = start + limit;
  return items.slice(start, end);
}

function buildPaginationMeta(total: number, page: number, limit: number) {
  return {
    page,
    limit,
    total,
    totalPages: Math.ceil(total / limit),
    hasNext: page * limit < total,
    hasPrev: page > 1
  };
}
```

#### Acceptance Criteria
- [ ] Pagination works correctly
- [ ] Handles edge cases (page 0, beyond total)
- [ ] Sorting works
- [ ] Max limit enforced
- [ ] Metadata accurate

#### Files to Modify
- `backend/src/routes/products.ts`
- `backend/src/utils/pagination.ts`

#### Estimated Time
3-4 hours

---

### Issue #10: Implement Rate Limiting
**Labels:** `backend` `security` `priority: high`

#### Description
Add rate limiting to prevent API abuse and ensure fair usage.

#### What You'll Learn
- API security
- Rate limiting strategies
- Middleware patterns

#### Rate Limits
```
Unauthenticated: 100 requests / 15 minutes
Authenticated:   1000 requests / 15 minutes
```

#### Implementation
```typescript
import rateLimit from 'express-rate-limit';

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 100,
  message: 'Too many requests, please try again later.',
  standardHeaders: true,
  legacyHeaders: false,
  handler: (req, res) => {
    res.status(429).json({
      error: 'Rate limit exceeded',
      retryAfter: req.rateLimit.resetTime
    });
  }
});

app.use('/api/', limiter);
```

#### Response Headers
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1234567890
```

#### Acceptance Criteria
- [ ] Rate limits enforced
- [ ] Proper headers returned
- [ ] Clear error messages
- [ ] Different tiers work

#### Dependencies
```bash
npm install express-rate-limit
```

#### Estimated Time
2-3 hours

---

## TESTING ISSUES

### Issue #11: Add Frontend E2E Tests
**Labels:** `testing` `enhancement` `medium`

#### Description
Write end-to-end tests for critical user flows using Playwright.

#### What You'll Learn
- E2E testing
- Playwright
- Test automation

#### Test Scenarios
1. Connect wallet
2. Register product
3. Add tracking event
4. View timeline
5. Generate QR code

#### Example Test
```typescript
// tests/e2e/registration.spec.ts
import { test, expect } from '@playwright/test';

test('user can register a product', async ({ page }) => {
  await page.goto('/register');
  
  await page.fill('[name="productId"]', 'PROD001');
  await page.fill('[name="name"]', 'Organic Coffee');
  await page.fill('[name="origin"]', 'Ethiopia');
  
  await page.click('button[type="submit"]');
  
  await expect(page).toHaveURL(/\/products\/PROD001/);
  await expect(page.locator('h1')).toContainText('Organic Coffee');
});
```

#### Setup
```bash
npm install -D @playwright/test
npx playwright install
```

#### Acceptance Criteria
- [ ] All critical flows tested
- [ ] Tests pass consistently
- [ ] Good coverage
- [ ] CI ready

#### Estimated Time
6-8 hours

---

## DOCUMENTATION ISSUES

### Issue #12: Write Getting Started Guide
**Labels:** `documentation` `good first issue`

#### Description
Create a beginner-friendly guide for new users.

#### What You'll Learn
- Technical writing
- User documentation

#### Content Needed
1. What is ChainLojistic?
2. Why use it?
3. Setting up a wallet
4. Registering your first product
5. Adding tracking events
6. Verifying products
7. Troubleshooting

#### Format
- Markdown with screenshots
- Step-by-step instructions
- Common issues section

#### Acceptance Criteria
- [ ] Clear explanations
- [ ] Screenshots included
- [ ] Covers all basics
- [ ] Proofread

#### Files to Create
- `docs/getting-started.md`

#### Estimated Time
4-6 hours

---

## 📝 Pull Request Process

### Before Submitting

1. **Run Tests**
   ```bash
   # Contracts
   cd contracts
   cargo test
   cargo clippy -- -D warnings
   cargo build --target wasm32-unknown-unknown --release

   # Frontend
   cd frontend
   npm run build
   npm run lint
   npm run type-check
   npm test

   # Backend
   cd backend
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

2. **Update Docs**
   - Add/update README if needed
   - Document new features
   - Update API documentation if endpoints changed
   - Add utoipa annotations for new endpoints

3. **Commit Convention**
   ```
   feat: add feature
   fix: bug fix
   docs: documentation
   style: formatting
   refactor: code restructure
   test: add tests
   chore: maintenance
   ci: CI/CD changes
   ```

### Branch Naming

Use descriptive branch names following this pattern:
- `feature/issue-23-wallet-connection`
- `bugfix/issue-45-auth-token-expiry`
- `hotfix/issue-67-security-patch`
- `docs/update-api-documentation`
- `refactor/optimize-database-queries`

### PR Description Template

```markdown
## Description
[Provide a clear and concise description of what this PR does]

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring

## Related Issue
Closes #[issue number]
Related to #[issue number]

## Changes Made
- [ ] Added tests for new functionality
- [ ] Updated documentation
- [ ] Updated API documentation (if applicable)
- [ ] Added/updated utoipa annotations (if applicable)

## Testing
### Manual Testing Steps
1.
2.
3.

### Automated Tests
- [ ] All existing tests pass
- [ ] New tests added and passing
- [ ] Test coverage maintained or improved

## Screenshots (if applicable)
[Add screenshots for UI changes]

## Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published
```

### PR Review Process

1. **Self-Review**
   - Review your own changes before submitting
   - Ensure all tests pass
   - Check documentation is complete
   - Verify no sensitive data is committed

2. **Request Review**
   - Assign at least one reviewer from the team
   - Tag relevant team members based on the area of change
   - Provide context on what needs focus

3. **Address Feedback**
   - Respond to all review comments
   - Make requested changes or provide justification
   - Push updates to the same branch
   - Request re-review when changes are complete

4. **Merge Requirements**
   - At least one approval from a maintainer
   - All CI checks must pass
   - No merge conflicts
   - PR must be up-to-date with main branch

---

## 🧪 Testing Requirements

### Smart Contracts (Rust/Soroban)

#### Unit Tests
```rust
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_product_creation() {
        let env = Env::default();
        let product_id = String::from_str("PROD001").unwrap();
        
        let product = register_product(env.clone(), product_id.clone(), "Coffee", "Ethiopia");
        
        assert_eq!(product.id, product_id);
        assert_eq!(product.name, "Coffee");
    }
}
```

#### Integration Tests
```rust
#[test]
fn test_full_product_lifecycle() {
    let env = Env::default();
    
    // Register product
    let product = register_product(env.clone(), "PROD001", "Coffee", "Ethiopia");
    
    // Add tracking event
    let event = add_tracking_event(env.clone(), "PROD001", "Harvest", "Farm");
    
    // Retrieve product
    let retrieved = get_product(env, "PROD001");
    
    assert_eq!(retrieved.id, product.id);
}
```

#### Coverage Requirements
- Minimum 80% code coverage for critical paths
- All public functions must have tests
- Error cases must be tested
- Edge cases must be covered

#### Test Commands
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_product_creation

# Run with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

---

### Frontend (Next.js/TypeScript)

#### Unit Tests
```typescript
// components/__tests__/ProductCard.test.tsx
import { render, screen } from '@testing-library/react';
import { ProductCard } from '../ProductCard';

describe('ProductCard', () => {
  it('renders product name', () => {
    const product = { id: '1', name: 'Coffee' };
    render(<ProductCard product={product} />);
    expect(screen.getByText('Coffee')).toBeInTheDocument();
  });
});
```

#### Integration Tests (Playwright)
```typescript
// tests/e2e/product-registration.spec.ts
import { test, expect } from '@playwright/test';

test('user can register a product', async ({ page }) => {
  await page.goto('/register');
  await page.fill('[name="productId"]', 'PROD001');
  await page.fill('[name="name"]', 'Organic Coffee');
  await page.click('button[type="submit"]');
  
  await expect(page).toHaveURL(/\/products\/PROD001/);
});
```

#### Coverage Requirements
- Minimum 70% code coverage for components
- Critical user flows must have E2E tests
- All hooks must have unit tests
- Utility functions must have 100% coverage

#### Test Commands
```bash
# Run unit tests
npm test

# Run with coverage
npm run test:coverage

# Run E2E tests
npx playwright test

# Run specific test file
npm test ProductCard.test.tsx
```

---

### Backend (Rust/Axum)

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_product() {
        let pool = create_test_pool().await;
        let product = create_test_product(&pool).await;
        
        let result = get_product(&pool, &product.id).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, product.id);
    }
}
```

#### Integration Tests
```rust
#[tokio::test]
async fn test_product_api_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/products/PROD001")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

#### Coverage Requirements
- Minimum 80% code coverage for handlers
- All service functions must have tests
- Database operations must be tested
- Error paths must be covered

#### Test Commands
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_get_product

# Run tests in release mode
cargo test --release

# Run with coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

---

### General Testing Guidelines

#### Test Organization
```
contracts/src/test/
├── mod.rs              # Test module exports
├── setup.rs            # Test fixtures and utilities
├── product_tests.rs    # Product-related tests
├── event_tests.rs      # Event-related tests
└── integration_tests.rs # Full workflow tests

frontend/tests/
├── unit/               # Unit tests
│   └── components/
├── integration/        # Integration tests
└── e2e/               # End-to-end tests

backend/tests/
├── unit/               # Unit tests
└── integration/        # Integration tests
```

#### Test Naming
```rust
// Good: Descriptive and specific
#[test]
fn test_product_registration_with_valid_data() {}

// Bad: Vague
#[test]
fn test_product() {}
```

#### Test Data Management
- Use fixtures for common test data
- Clean up test data after each test
- Use transactions for database tests
- Mock external dependencies

#### CI/CD Testing
All tests must pass in CI before merging:
- GitHub Actions runs tests on every PR
- Tests run on multiple OS versions
- Database tests use test containers
- E2E tests run on staging environment

---

## ✅ Code Review Checklist

### Before Submitting for Review

**Code Quality**
- [ ] Code follows project style guidelines
- [ ] No compiler warnings
- [ ] No clippy warnings (Rust)
- [ ] No ESLint warnings (TypeScript)
- [ ] Code is properly formatted
- [ ] Complex logic has comments
- [ ] Magic numbers are replaced with constants

**Functionality**
- [ ] Feature works as specified
- [ ] Edge cases are handled
- [ ] Error handling is comprehensive
- [ ] Input validation is present
- [ ] Security best practices followed
- [ ] Performance considerations addressed

**Testing**
- [ ] Unit tests added
- [ ] Integration tests added (if applicable)
- [ ] E2E tests added (for UI changes)
- [ ] Tests cover happy path
- [ ] Tests cover error cases
- [ ] Test coverage is sufficient
- [ ] All tests pass locally

**Documentation**
- [ ] Public APIs documented
- [ ] Complex algorithms explained
- [ ] README updated (if needed)
- [ ] API documentation updated (if endpoints changed)
- [ ] Changelog updated (if breaking change)
- [ ] Comments are accurate and up-to-date

**Security**
- [ ] No secrets or keys committed
- [ ] User inputs are validated
- [ ] SQL injection prevention in place
- [ ] XSS prevention in place (frontend)
- [ ] Authentication/authorization correct
- [ ] Rate limiting applied (if public API)

**Performance**
- [ ] No obvious performance issues
- [ ] Database queries optimized
- [ ] Caching used where appropriate
- [ ] Large data sets handled efficiently
- [ ] Memory leaks checked

### During Review

**Review Focus Areas**
- [ ] Code is readable and maintainable
- [ ] Logic is correct and efficient
- [ ] Error handling is appropriate
- [ ] Security vulnerabilities identified
- [ ] Performance bottlenecks identified
- [ ] Test coverage is adequate
- [ ] Documentation is clear

**Review Comments Guidelines**
- Be constructive and specific
- Explain the "why" behind suggestions
- Provide code examples for improvements
- Ask questions if something is unclear
- Acknowledge good work

### Before Merging

**Final Checks**
- [ ] All review comments addressed
- [ ] CI checks passing
- [ ] No merge conflicts
- [ ] Up-to-date with main branch
- [ ] At least one approval from maintainer
- [ ] Breaking changes documented
- [ ] Migration guide provided (if needed)

### Post-Merge

**Follow-up Tasks**
- [ ] Monitor production for issues
- [ ] Update issue tracker
- [ ] Close related issues
- [ ] Notify team of deployment
- [ ] Update documentation website (if applicable)
- [ ] Create release notes (if needed)

---

## 🎨 Code Style Guidelines

### Smart Contracts (Rust/Soroban)

#### Naming Conventions
```rust
// Functions: snake_case
pub fn register_product() {} ✅
pub fn reg_prod() {}          ❌

// Structs: PascalCase
pub struct Product {}         ✅
pub struct product {}         ❌

// Constants: SCREAMING_SNAKE_CASE
pub const MAX_PRODUCTS: u32 = 1000;  ✅
pub const max_products: u32 = 1000;  ❌

// Storage keys: SymbolShort
let key = symbol_short!("PRODUCT");  ✅
let key = symbol_short!("product");  ❌
```

#### Documentation
```rust
/// Registers a new product on the blockchain.
///
/// # Arguments
///
/// * `product_id` - Unique identifier for the product
/// * `name` - Human-readable product name
/// * `origin` - Geographic origin location
///
/// # Returns
///
/// Returns the newly created Product struct.
///
/// # Errors
///
/// Returns `Error::AlreadyExists` if product_id is already registered.
///
/// # Example
///
/// ```
/// let product = register_product(env, "PROD001", "Coffee", "Ethiopia");
/// ```
pub fn register_product(
    env: Env,
    product_id: String,
    name: String,
    origin: String,
) -> Product {
    // implementation
}
```

#### Error Handling
```rust
// Use custom error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Product already exists")]
    AlreadyExists,
    #[error("Unauthorized access")]
    Unauthorized,
}

// Panic with errors for contract failures
if product_exists {
    panic_with_error!(&env, Error::AlreadyExists);
}
```

#### Code Organization
```rust
// File structure
src/
├── lib.rs           # Contract entry point
├── contract.rs      # Main contract logic
├── types.rs         # Data structures
├── storage.rs       # Storage helpers
├── error.rs         # Error types
├── events.rs        # Event emission
└── validation.rs    # Input validation

// Use modules for organization
pub mod types;
pub mod storage;
pub mod error;

use crate::types::{Product, Event};
use crate::storage::{get_product, set_product};
```

#### Format & Lint
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run tests
cargo test

# Build for WASM
cargo build --target wasm32-unknown-unknown --release
```

---

### Frontend (Next.js/TypeScript)

#### Naming Conventions
```typescript
// Components: PascalCase
export function ProductCard() {}     ✅
export function productCard() {}     ❌

// Functions: camelCase
function getUserData() {}           ✅
function get_user_data() {}         ❌

// Constants: UPPER_SNAKE_CASE
const MAX_RETRIES = 3;              ✅
const maxRetries = 3;               ❌

// Types/Interfaces: PascalCase
interface UserProfile {}            ✅
interface userProfile {}            ❌
```

#### Component Structure
```typescript
// Use functional components with hooks
export function ProductCard({ product }: ProductCardProps) {
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // effect logic
  }, [product.id]);

  if (loading) return <LoadingSpinner />;

  return <div>{product.name}</div>;
}

// Define props interface
interface ProductCardProps {
  product: Product;
  onEdit?: () => void;
}
```

#### TypeScript Best Practices
```typescript
// Always define types
interface Product {
  id: string;
  name: string;
  origin: string;
}

// Avoid `any`
function processData(data: any) {}      ❌
function processData(data: Product) {}  ✅

// Use union types for variants
type EventStatus = 'pending' | 'confirmed' | 'failed';

// Use generics for reusable types
interface ApiResponse<T> {
  data: T;
  error: string | null;
}
```

#### Import Organization
```typescript
// 1. React and hooks
import { useState, useEffect } from 'react';

// 2. Third-party libraries
import { Contract } from '@stellar/stellar-sdk';
import { useRouter } from 'next/navigation';

// 3. Internal modules
import { Product } from '@/types/product';
import { useWallet } from '@/hooks/useWallet';
import { formatAddress } from '@/lib/utils';
```

#### File Organization
```
app/
├── layout.tsx           # Root layout
├── page.tsx             # Homepage
└── products/
    ├── page.tsx         # Products list
    └── [id]/
        └── page.tsx     # Product detail

components/
├── ui/                  # Reusable UI components
├── forms/               # Form components
├── wallet/              # Wallet-related
└── tracking/            # Tracking components

lib/
├── stellar/             # Stellar SDK utilities
├── utils/               # Helper functions
└── hooks/               # Custom hooks
```

#### Linting & Formatting
```bash
# Format code
npm run format

# Lint code
npm run lint

# Type check
npm run type-check

# Run tests
npm test

# Build for production
npm run build
```

---

### Backend (Rust/Axum)

#### Naming Conventions
```rust
// Structs: PascalCase
pub struct AppState {}        ✅
pub struct app_state {}      ❌

// Functions: snake_case
pub async fn get_product() {}    ✅
pub async fn getProduct() {}     ❌

// Constants: SCREAMING_SNAKE_CASE
pub const API_VERSION: &str = "v1";  ✅
pub const api_version: &str = "v1";  ❌
```

#### Handler Structure
```rust
// Use proper error handling
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProductResponse>, AppError> {
    let product = state.product_service
        .get_product(&id)
        .await
        .map_err(|_| AppError::NotFound("Product not found".to_string()))?;

    Ok(Json(product.into()))
}
```

#### Database Queries
```rust
// Use SQLx for type-safe queries
pub async fn get_product(
    pool: &PgPool,
    id: &str,
) -> Result<Product, sqlx::Error> {
    sqlx::query_as!(
        Product,
        r#"
        SELECT id, name, description, origin_location, category, tags,
               certifications, media_hashes, custom_fields, owner_address,
               is_active, created_at, updated_at, created_by, updated_by
        FROM products
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}
```

#### Error Handling
```rust
// Define custom error types
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    ValidationError(String),
    DatabaseError(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

#### Validation
```rust
// Validate inputs before processing
use crate::validation::{validate_string, validate_stellar_address};

pub async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    // Validate inputs
    validate_string("name", &req.name, 100)?;
    validate_stellar_address(&req.owner_address)?;
    validate_string("origin", &req.origin, 200)?;

    // Process request
    let product = state.product_service.create_product(req).await?;

    Ok(Json(product))
}
```

#### OpenAPI Documentation
```rust
// Add utoipa annotations for all public endpoints
#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    tag = "products",
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product retrieved successfully", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProductResponse>, AppError> {
    // implementation
}
```

#### Format & Lint
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run tests
cargo test

# Run with database
cargo run

# Build for release
cargo build --release
```

---

### General Guidelines

#### Git Commit Messages
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions/changes
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

**Examples:**
```
feat(products): add batch product registration

Implements batch registration for multiple products
in a single transaction to reduce gas costs.

Closes #23
```

```
fix(auth): resolve JWT token expiration issue

Fixed token validation logic that was incorrectly
rejecting valid tokens near expiration time.

Closes #45
```

#### Code Comments
```rust
// GOOD: Explain WHY, not WHAT
// We use a cache here because the blockchain RPC calls
// are expensive and can take up to 2 seconds
let cached_product = cache.get(&product_id).await;

// BAD: Obvious comments
// Get the product from cache
let cached_product = cache.get(&product_id).await;

// GOOD: Document complex logic
// This algorithm calculates the optimal route by considering
// both distance and carbon footprint, with a 60% weight on
// environmental impact
let optimal_route = calculate_route(locations, carbon_weights);

// GOOD: TODO comments with context
// TODO: Add retry logic for network failures
// Currently fails silently on network errors
// Issue: #67
```

#### Security Best Practices
- Never commit secrets or API keys
- Use environment variables for configuration
- Validate all user inputs
- Sanitize data before database operations
- Use parameterized queries to prevent SQL injection
- Implement rate limiting on public endpoints
- Use HTTPS in production
- Keep dependencies updated

---

## 🆘 Getting Help

- **Discussions**: Ask questions
- **Issues**: Report bugs
- **Discord**: Real-time chat
- **Email**: maintainer@chainlojistic.com

---

## 🎉 Recognition

Contributors are featured in:
- README contributors section
- Release notes
- Annual blog post

Thank you for contributing to ChainLojistic! 🌍✨

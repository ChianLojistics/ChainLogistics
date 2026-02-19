# Contributing to ChainLojistic 🚀

Thank you for your interest in contributing to ChainLojistic! This comprehensive guide will help you contribute effectively to our open-source supply chain tracking platform.

## 📚 Table of Contents
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
3. **Backend** (Node.js/Express/TypeScript) - API (optional)

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
- Node.js 18+
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

# 6. Backend (optional)
cd ../backend
npm install
cp .env.example .env
npm run dev  # http://localhost:3001
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
├── backend/                             # API Server (Optional)
│   ├── src/
│   │   ├── index.ts                   # Server entry point
│   │   ├── routes/
│   │   │   ├── products.ts            # Product routes
│   │   │   ├── events.ts              # Event routes
│   │   │   └── analytics.ts           # Analytics routes
│   │   ├── services/
│   │   │   ├── contractService.ts     # Contract interactions
│   │   │   ├── cacheService.ts        # Redis caching
│   │   │   └── webhookService.ts      # Webhook handling
│   │   ├── middleware/
│   │   │   ├── auth.ts                # Authentication
│   │   │   ├── rateLimiter.ts         # Rate limiting
│   │   │   └── validation.ts          # Request validation
│   │   ├── utils/
│   │   │   ├── logger.ts              # Logging
│   │   │   └── errors.ts              # Error handling
│   │   └── types/
│   │       └── index.ts               # TypeScript types
│   ├── tests/
│   │   ├── unit/
│   │   └── integration/
│   ├── package.json
│   ├── tsconfig.json
│   └── .env.example
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
   cd backend && npm test
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
| `backend` | Node.js/API | ⭐⭐ Medium |
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
- REST API design
- Express.js routing
- Soroban integration
- Error handling

#### Endpoints to Create
```
GET    /api/products          - List all products
POST   /api/products          - Register product
GET    /api/products/:id      - Get product by ID
GET    /api/products/:id/events  - Get events
POST   /api/products/:id/events  - Add event
```

#### Implementation
```typescript
// src/routes/products.ts
import { Router } from 'express';
import { Contract } from '@stellar/stellar-sdk';

const router = Router();

router.get('/products/:id', async (req, res) => {
  try {
    const { id } = req.params;
    const contract = new Contract(process.env.CONTRACT_ID);
    const product = await contract.get_product({ id });
    res.json(product);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

export default router;
```

#### Validation
```typescript
// src/middleware/validation.ts
import { body, param, validationResult } from 'express-validator';

export const validateProduct = [
  body('id').isString().trim().notEmpty(),
  body('name').isString().trim().isLength({ min: 3 }),
  body('origin').isString().trim().notEmpty(),
  (req, res, next) => {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ errors: errors.array() });
    }
    next();
  }
];
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
- [ ] All endpoints work
- [ ] Proper validation
- [ ] Error handling
- [ ] Correct HTTP status codes
- [ ] API documented

#### Files to Create
- `backend/src/routes/products.ts`
- `backend/src/services/productService.ts`
- `backend/src/middleware/validation.ts`

#### Dependencies
```bash
npm install express-validator
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
   cargo test && cargo clippy
   
   # Frontend
   npm run build && npm run lint
   
   # Backend
   npm test
   ```

2. **Update Docs**
   - Add/update README if needed
   - Document new features

3. **Commit Convention**
   ```
   feat: add feature
   fix: bug fix
   docs: documentation
   style: formatting
   refactor: code restructure
   test: add tests
   chore: maintenance
   ```

### PR Template

```markdown
## Description
[What does this PR do?]

## Related Issue
Closes #[issue number]

## Type
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation

## Testing
- [ ] Tests pass
- [ ] Manual testing done

## Screenshots
[If applicable]

## Checklist
- [ ] Code follows style guide
- [ ] Self-reviewed
- [ ] Commented complex code
- [ ] Docs updated
- [ ] Tests added
```

---

## 🎨 Code Style Guidelines

### Smart Contracts (Rust)
```rust
// Good naming
pub fn register_product() {} ✅
pub fn reg_prod() {}          ❌

// Document public APIs
/// Registers a new product.
pub fn register_product() {}

// Format & lint
cargo fmt
cargo clippy
```

### Frontend (TypeScript)
```typescript
// Strict TypeScript
interface Product {
  id: string;
  name: string;
}

// Functional components
export function ProductCard({ product }: { product: Product }) {
  return <div>{product.name}</div>;
}

// Organize imports
import { useState } from 'react';          // React
import { Contract } from '@stellar/sdk';   // Packages
import { format } from '@/lib/utils';      // Local
```

### Backend (TypeScript)
```typescript
// Async/await
async function getProduct(id: string) {
  return await contract.get_product({ id });
}

// Error handling
try {
  const product = await getProduct(id);
  res.json(product);
} catch (error) {
  logger.error('Error fetching product', { id, error });
  res.status(500).json({ error: 'Internal error' });
}
```

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
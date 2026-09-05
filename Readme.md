# ChainLogistics

### Blockchain-backed supply chain provenance built on Stellar Soroban

**Track. Verify. Prove.**

ChainLogistics is an open-source supply chain provenance platform that records critical product events on **Stellar's Soroban smart contract platform**, giving producers, processors, logistics providers, retailers, and consumers a shared and verifiable history of a product.

The goal is simple: **make it possible to verify where a product came from, what happened to it along the way, and who recorded each step.**

---

## The Problem

Supply chains are usually fragmented.

A producer may have one database, a processor another, a logistics company its own records, and a retailer yet another system. Documents can be lost, duplicated, altered, or difficult to verify.

This creates problems such as:

* Limited visibility across supply-chain participants
* Difficult product provenance verification
* Counterfeit and fraudulent products
* Fragmented records between organizations
* Manual verification and paperwork
* Difficulty tracing products back to their source
* Unverified claims about product origin and handling

The problem is not simply a lack of data.

**The problem is a lack of shared trust in the data.**

---

# Our Solution

ChainLogistics creates a shared provenance layer for supply chains.

Instead of asking every participant to trust a single organization's database, important supply-chain events can be recorded and verified through Stellar Soroban.

A typical product journey looks like this:

```text
Producer
   │
   │ Register product
   ▼
Stellar / Soroban
   │
   │ Record provenance
   ▼
Processor
   │
   │ Add processing event
   ▼
Logistics Provider
   │
   │ Add shipment event
   ▼
Retailer
   │
   │ Confirm receipt
   ▼
Consumer
   │
   │ Scan QR code
   ▼
Verified Product History
```

Each participant contributes to the product's history while access controls determine who is authorized to record events.

---

# Why Blockchain?

A traditional database is useful for storing and querying information.

But supply chains involve **multiple organizations that do not necessarily share the same systems or trust relationships**.

ChainLogistics uses a hybrid architecture:

### Off-chain

Used for data that benefits from traditional application infrastructure:

* Search
* Analytics
* API requests
* Caching
* Application metadata
* Integrations
* Operational data

### On-chain

Used for information where shared verification and tamper resistance matter:

* Product registration
* Product ownership
* Provenance events
* Authorized actors
* Verification records
* Critical state transitions

This gives ChainLogistics the performance and flexibility of conventional infrastructure while using Soroban as the shared trust layer.

---

# Why Stellar Soroban?

ChainLogistics is built on Stellar Soroban because the platform fits the requirements of a global supply-chain application:

* Low transaction costs
* Fast transaction finality
* Smart contracts written in Rust
* Native support for account-based authorization
* A network designed for global financial and cross-border applications
* A developer ecosystem that makes it practical to build secure, verifiable applications

Most importantly, **Soroban is not being used simply because the project is a blockchain project.**

It is used where multiple supply-chain participants need a shared record that can be independently verified.

---

# Core Product Flow

## 1. Register a Product

A producer registers a product or batch.

Example:

```text
Product:
Ethiopian Coffee

Batch:
COFFEE-2026-001

Origin:
Sidamo, Ethiopia

Owner:
Producer wallet address
```

The product receives a unique identifier that can be used throughout its lifecycle.

---

## 2. Add Supply-Chain Events

Authorized participants can add events as the product moves through the supply chain.

Examples include:

```text
HARVEST
PROCESSING
QUALITY_CHECK
PACKAGING
SHIPPING
WAREHOUSE_RECEIPT
RETAIL
```

Each event can contain information such as:

* Product ID
* Location
* Actor
* Timestamp
* Event type
* Additional metadata

---

## 3. Verify the Product

A consumer or supply-chain participant can retrieve the product's history and verify its recorded journey.

The frontend supports QR-based product verification so that a physical product can be connected to its digital provenance record.

```text
Scan QR
   ↓
Find Product
   ↓
Verify Product Identity
   ↓
Retrieve Provenance Events
   ↓
Display Product Journey
```

---

# Key Features

### Product Registration

* Register products and batches
* Assign unique product identifiers
* Record origin information
* Establish product ownership

### Provenance Tracking

* Record supply-chain events
* Track locations and timestamps
* Maintain product history
* Associate events with authorized actors

### Multi-party Authorization

Supply-chain participants should not have unrestricted access to every product.

ChainLogistics includes authorization mechanisms that allow specific actors to be permitted to interact with a product's provenance record.

### Product Verification

* QR code generation
* Product lookup
* Provenance history
* Verification interface

### API

The backend provides an API layer for applications and integrations that need to interact with ChainLogistics.

### SDK

The project includes SDK components intended to make integration with ChainLogistics easier for external applications.

### Analytics

Application-level analytics and reporting provide visibility into tracked products and supply-chain activity.

### Security

The project includes:

* Contract-level authorization
* Automated tests
* Security-focused development
* Formal verification work
* Separation of on-chain and off-chain responsibilities

---

# Architecture

```text
                         ┌─────────────────────┐
                         │      Users          │
                         │                     │
                         │ Producers           │
                         │ Processors          │
                         │ Logistics           │
                         │ Retailers           │
                         │ Consumers           │
                         └──────────┬──────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │      Frontend       │
                         │                     │
                         │ Next.js             │
                         │ React               │
                         │ TypeScript          │
                         │ Freighter Wallet    │
                         │ QR Verification     │
                         └──────────┬──────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │       Backend       │
                         │                     │
                         │ Rust / Axum         │
                         │ REST API            │
                         │ PostgreSQL          │
                         │ Redis               │
                         │ Webhooks            │
                         └──────────┬──────────┘
                                    │
                       ┌────────────┴────────────┐
                       │                         │
                       ▼                         ▼
              ┌─────────────────┐       ┌─────────────────┐
              │  SDK / External │       │ Stellar Soroban │
              │   Integrations  │       │ Smart Contracts │
              └─────────────────┘       └────────┬────────┘
                                                  │
                                                  ▼
                                      ┌─────────────────────┐
                                      │ Product Provenance  │
                                      │ Authorization       │
                                      │ Ownership            │
                                      │ Tracking Events      │
                                      └─────────────────────┘
```

---

# Technology Stack

## Smart Contracts

* Rust
* Soroban SDK
* Stellar
* WASM
* Contract-level authorization
* Provenance and event storage

## Backend

* Rust
* Axum
* Tokio
* SQLx
* PostgreSQL
* Redis
* REST APIs
* Webhooks
* Caching
* Rate limiting

## Frontend

* Next.js
* React
* TypeScript
* Freighter wallet integration
* QR code generation
* Product dashboards
* Provenance timeline
* Search and analytics

## SDK

* Rust SDK
* Python bindings / integration work
* API and contract integration utilities

## Development & Infrastructure

* Docker
* GitHub Actions
* Automated testing
* Formal verification
* Deployment scripts

---

# Smart Contract Interface

The core contract exposes functionality for managing products and their provenance.

### Register a Product

```text
register_product(
    id,
    name,
    origin,
    owner
) -> Product
```

### Add a Tracking Event

```text
add_tracking_event(
    product_id,
    location,
    event_type,
    metadata
) -> Event
```

### Get Product

```text
get_product(
    id
) -> Product
```

### Get Product History

```text
get_tracking_events(
    product_id
) -> Vec<Event>
```

### Transfer Ownership

```text
transfer_ownership(
    product_id,
    new_owner
) -> Success
```

### Authorize an Actor

```text
add_authorized_actor(
    product_id,
    actor_address
) -> Success
```

The exact contract interface may evolve as the protocol develops. Refer to the smart-contract source and tests for the authoritative implementation.

---

# Data Model

A product is represented conceptually as:

```text
Product
├── ID
├── Name
├── Origin
├── Owner
├── Registration timestamp
└── Authorized actors
```

A provenance event contains:

```text
TrackingEvent
├── Product ID
├── Location
├── Actor
├── Timestamp
├── Event type
└── Metadata
```

This allows a product to build a chronological history throughout its lifecycle.

---

# Example: Agricultural Supply Chain

Agriculture is one of the clearest applications for ChainLogistics because products can pass through many independent actors before reaching the consumer.

For example:

```text
Farm
 │
 │ Harvest
 ▼
Processing Facility
 │
 │ Processing + Quality Check
 ▼
Export Warehouse
 │
 │ Packaging
 ▼
Logistics Provider
 │
 │ Shipment
 ▼
Roaster / Distributor
 │
 │ Final Processing
 ▼
Retailer
 │
 │ Product Sold
 ▼
Consumer
```

At each stage, authorized participants can contribute a provenance event.

The consumer can then scan the product's QR code and view the recorded journey.

---

# Other Potential Applications

The underlying provenance architecture can also support other industries.

### Pharmaceuticals

Track:

* Manufacturing batches
* Quality checks
* Distribution checkpoints
* Cold-chain events
* Pharmacy receipt

### Fashion & Textiles

Track:

* Raw material origin
* Manufacturing
* Certifications
* Distribution
* Recycling

### Electronics

Track:

* Component origin
* Manufacturing
* Assembly
* Distribution
* Recycling

### Luxury Goods

Track:

* Product identity
* Ownership transfers
* Authentication records
* Resale history

These are potential expansion areas. The initial product focus is provenance and verification rather than trying to solve every supply-chain problem simultaneously.

---

# Security & Privacy

Supply-chain systems contain both public and sensitive information.

ChainLogistics follows a separation-of-concerns approach.

### On-chain

Only information that benefits from shared verification should be stored on-chain.

### Off-chain

Sensitive or operational information can remain within traditional application infrastructure.

### Authorization

Only authorized actors should be able to modify the provenance state associated with a product.

### Verification

Cryptographic signatures and blockchain state provide a mechanism for verifying that recorded events were submitted by the expected account.

### Formal Verification

The repository includes formal verification work for critical contract behavior.

Security remains an ongoing part of the project's development process rather than a one-time feature.

---

# Current Deployment

ChainLogistics currently has a Stellar testnet deployment for its MVP.

### Network

```text
Stellar Testnet
```

### Main Contract

```text
CDN45LYNJLEHVWLYAN34CFSBUT4RWTFWKG5I7LMDJS2QNC2L6RLLEZWR
```

### Authorization Contract

```text
CCAPPFD5PERFZ6T66XPU74NZBZXHJBSQIFST4GOVMYIDZR4D54VYRXHQ
```

The frontend/backend configuration points to the main product registry contract.

For deployment instructions, see:

```text
DEPLOYMENT.md
```

> Testnet deployments are for development and demonstration. Contract addresses may change as the protocol evolves.

---

# Project Structure

```text
ChainLogistics/
│
├── .github/
│   └── workflows/
│
├── backend/
│   ├── API
│   ├── database
│   ├── caching
│   └── webhooks
│
├── docker/
│
├── docs/
│   └── project documentation
│
├── formal_verification/
│   └── verification work
│
├── frontend/
│   ├── dashboards
│   ├── product registration
│   ├── QR verification
│   └── wallet integration
│
├── sdk/
│   └── integration libraries
│
├── smart-contract/
│   ├── contracts
│   ├── tests
│   └── deployment scripts
│
├── docker-compose.yml
├── DEPLOYMENT.md
├── Contributing.md
└── Readme.md
```

---

# Getting Started

## Prerequisites

You will need:

* Rust 1.84+
* Stellar CLI
* `wasm32v1-none` Rust target
* Node.js 18+
* npm or yarn
* PostgreSQL 14+
* Redis 6+

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install the Stellar CLI

```bash
curl --proto '=https' --tlsv1.2 -sSf https://stellar.org/install.sh | sh
```

### Add the WASM target

```bash
rustup target add wasm32v1-none
```

---

# Run the Smart Contracts

```bash
cd smart-contract

cargo build --target wasm32v1-none --release

cargo test
```

---

# Run the Frontend

```bash
cd frontend

npm install

npm run dev
```

The frontend runs locally on:

```text
http://localhost:3000
```

---

# Run the Backend

```bash
cd backend

cp .env.example .env

cargo build

cargo run
```

The backend runs locally on:

```text
http://localhost:3001
```

For the complete deployment process, see:

```text
DEPLOYMENT.md
```

---

# API

The backend exposes REST endpoints for interacting with products and provenance data.

Examples include:

```text
GET    /api/products
POST   /api/products

GET    /api/products/:id
GET    /api/products/:id/events
POST   /api/products/:id/events

GET    /api/analytics
```

The API is designed to provide a conventional integration layer while the blockchain remains responsible for the verifiable provenance state.

---

# Testing

The project contains testing across the smart-contract and application layers.

Smart-contract tests can be run with:

```bash
cd smart-contract
cargo test
```

When contributing, new functionality should include appropriate tests covering:

* Expected behavior
* Authorization
* Invalid input
* State transitions
* Edge cases
* Integration behavior where applicable

---

# Development Status

## Current

ChainLogistics is an actively developed **Stellar testnet MVP**.

The repository currently includes:

* Smart-contract implementation
* Product registration
* Provenance event tracking
* Authorization
* Frontend application
* QR-based verification
* Rust backend
* PostgreSQL integration
* Redis-backed infrastructure
* SDK work
* Automated testing
* Formal verification work
* Deployment tooling

## Next Priorities

The next stage of development is focused on turning the existing technical foundation into a more complete production-ready provenance platform.

### Product

* Improve the end-to-end producer-to-consumer experience
* Improve product verification
* Improve supply-chain participant workflows
* Improve onboarding

### Infrastructure

* Improve indexing and query performance
* Strengthen API integrations
* Improve observability
* Improve deployment reliability

### Security

* Expand contract test coverage
* Continue formal verification
* Conduct deeper security review
* Strengthen authorization boundaries

### Adoption

* Focus the initial use case on a specific supply-chain vertical
* Develop pilot workflows
* Gather feedback from real supply-chain participants
* Measure verification and provenance usage

---

# Roadmap

Rather than attaching arbitrary dates to features, development is organized around product maturity.

## Phase 1 — Provenance MVP

* [x] Product registration
* [x] Provenance events
* [x] Actor authorization
* [x] Ownership management
* [x] QR verification foundation
* [x] Stellar testnet deployment

## Phase 2 — Product Readiness

* [x] Backend API foundation
* [x] Frontend application
* [x] SDK development
* [x] Automated testing
* [x] Caching and rate limiting
* [x] Formal verification work

## Phase 3 — Pilot Readiness

* [ ] Complete end-to-end supply-chain demonstration
* [ ] Improve participant onboarding
* [ ] Improve consumer verification experience
* [ ] Expand integration documentation
* [ ] Strengthen monitoring and observability

## Phase 4 — Production

* [ ] Production security review
* [ ] Mainnet deployment
* [ ] Production infrastructure
* [ ] External integrations
* [ ] Initial supply-chain pilot
* [ ] Production monitoring

---

# What We Are Building Toward

The long-term goal is not to replace every existing supply-chain system.

It is to provide a **shared provenance and verification layer** that existing systems can integrate with.

```text
Existing Business Systems
        │
        │
        ▼
┌─────────────────────────┐
│      ChainLogistics     │
│                         │
│ Provenance + Verification│
└────────────┬────────────┘
             │
             ▼
       Stellar Soroban
```

This approach allows organizations to continue using their existing operational tools while sharing critical provenance information through a common verification layer.

---

# Open Source

ChainLogistics is open source and welcomes contributions from developers, designers, researchers, supply-chain professionals, and anyone interested in building better infrastructure for product provenance.

You can contribute through:

* Smart-contract development
* Backend development
* Frontend development
* SDK development
* Testing
* Security research
* Documentation
* UI/UX
* Supply-chain research
* Product feedback

Please read:

```text
Contributing.md
```

before submitting a contribution.

---

# Security

If you discover a security vulnerability, please avoid publicly disclosing sensitive details through a GitHub issue.

Follow the project's security reporting process where available.

Security is especially important because ChainLogistics deals with provenance records, authorization, and potentially valuable business information.

---

# License

ChainLogistics is released under the MIT License.

See:

```text
LICENSE
```

for the full license text.

---

# Vision

Supply chains should not require consumers to blindly trust a label, a document, or a company's database.

They should be able to verify the journey.

**ChainLogistics is building the infrastructure that makes that possible.**

```text
Track what happened.
Verify who recorded it.
Prove the product's journey.
```

**Built with Rust, Stellar Soroban, and open-source technology.**

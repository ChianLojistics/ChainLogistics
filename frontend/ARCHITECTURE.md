# Frontend Architecture

## Goals

- Follow Next.js App Router conventions.
- Keep UI and non-UI concerns separated.
- Provide a scalable structure for a supply chain tracking application.
- Centralize Stellar/Soroban integration.
- Keep the dev server runnable without requiring chain configuration.

## High-level structure

- `app/`: Routes (App Router). Uses route groups for separation.
- `components/`: Reusable UI and feature components.
- `lib/`: Non-UI code (SDK integration, state, hooks, utilities, shared types).

## Routing

- `app/(marketing)`: Public/landing pages.
- `app/(app)`: Application routes (registration, products, tracking, dashboard).

Route groups do not affect the URL; they only organize the filesystem.

## Components

- `components/ui`: Low-level reusable primitives (no domain logic).
- `components/<feature>`: Feature-oriented components (wallet/products/tracking/forms).
- `components/layouts`: App shell pieces.

## State management

- Global state uses Zustand in `lib/state/*`.
- Page-local state stays local or in `react-hook-form`.

## Stellar / Soroban integration

- All SDK and contract-client code lives under `lib/stellar/*`.
- Modules export functions and types only; no side effects at import time.

This keeps pages/components free of direct SDK wiring and makes later testing/mocking easier.

## TypeScript

- Shared domain types live in `lib/types/*`.
- The `@/*` path alias maps to the `frontend/` project root.

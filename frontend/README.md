
# ChainLogistics Frontend

Next.js App Router frontend scaffold for a supply chain tracking application.

## Getting Started

```bash
npm install
cp .env.example .env.local
npm run dev
```

Open http://localhost:3000

## Environment Variables

Use `.env.example` as the source of truth for frontend environment variables.

- `NEXT_PUBLIC_CONTRACT_ID` is required for real contract interactions.
- `NEXT_PUBLIC_STELLAR_NETWORK` and `NEXT_PUBLIC_RPC_URL` control network/RPC behavior.
- `NEXT_PUBLIC_E2E_MOCKS` and `NEXT_PUBLIC_USE_MOCK_DATA` help with test/local mock flows.

## Scripts

```bash
npm run dev
npm run build
npm run start
npm run lint
npm test
npm run e2e
```

## Project structure

```txt
app/
  (marketing)/          Public landing/marketing pages
  (app)/                Application routes (register/products/tracking/dashboard)
  api/                  Route handlers (API endpoints)

components/
  ui/                   Reusable primitives (no domain logic)
  layouts/              App shell pieces
  wallet/               Wallet-related components
  products/             Product-related components
  tracking/             Tracking/event components
  forms/                Form components

lib/
  stellar/              Stellar/Soroban integration (SDK + contract client)
  state/                Global state (Zustand)
  hooks/                Custom hooks
  utils/                Helpers/formatters/constants
  types/                Shared TypeScript domain types
```

## Architectural decisions

See `ARCHITECTURE.md` for the reasoning behind routing, state management, and Stellar integration placement.

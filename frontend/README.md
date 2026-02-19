
# ChainLogistics Frontend

Next.js App Router frontend scaffold for a supply chain tracking application.

## Getting Started

```bash
npm install
npm run dev
```

Open http://localhost:3000

## Scripts

```bash
npm run dev
npm run build
npm run start
npm run lint
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

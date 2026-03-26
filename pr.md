# Pull Request

## Description

This PR implements several key features from the issues backlog:

1. **Fix Placeholder Owner (#129):**
   - Updated `/products` page to handle disconnected states.
   - Removed the hardcoded placeholder address.
   - Now prompts users to connect their wallet to view products.

2. **Internationalization (i18n) Setup (#120):**
   - Integrated `i18next` and `react-i18next`.
   - Added `en.json` and `es.json` locale files with base translations.
   - Created `LanguageSelector` component for toggling between languages.
   - Enabled language persistence using `localStorage`.

3. **Frontend Unit Tests (#137):**
   - Implemented unit tests for utility functions (`format.test.ts`).
   - Implemented unit tests for Zod validation schemas (`schemas.test.ts`).
   - Implemented unit tests for the generic UI components (`Button.test.tsx`).

4. **Rust REST API Backend scaffolding (#123 & #21AH):**
   - Created a heavily optimized Rust/Axum project backend to work together with the Soroban smart contracts.
   - Setup `Cargo.toml` with `tokio`, `axum`, `sqlx`, `redis`, etc.
   - Setup `main.rs` as the API Server entry point.
   - Pre-allocated modular system for `routes`, `handlers`, `services`, `models`, and `database`.

## Checks
- [x] Tested locally.
- [x] All PR checks (lint/tests) passed.
- [x] Fixed unit test expectations and handled Stellar SDK validation mocks.
- [x] Adhered to ESLint rules and handled hydration state correctly in LanguageSelector.
- [x] Dependencies are installed.

## Related Issues
- Closes #129
- Closes #120
- Closes #137
- Closes #123
- Closes #21AH
- fixes: #77
- fixes: #55

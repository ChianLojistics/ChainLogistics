# Pull Request

## Description

- Pre-allocated modular system for `routes`, `handlers`, `services`, `models`, and `database`.

5. **Backend Security and Performance (#234, #249, #250, #239):**
   - **Distributed Rate Limiting (#234)**: Replaced in-memory rate limiting with a scalable Redis-backed implementation.
   - **Query Optimization (#249)**: Refactored expensive analytics and product lookups for better database performance.
   - **Redis Caching Strategy (#250)**: Implemented a caching layer for global stats and product details to reduce DB load.
   - **Data Encryption at Rest (#239)**: Secured sensitive user PII (emails, Stellar addresses) using AES-256-GCM.

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
- Closes #234
- Closes #249
- Closes #250
- Closes #239

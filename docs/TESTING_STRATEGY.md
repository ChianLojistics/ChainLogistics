# Testing Strategy and Quality Assurance

This document defines the end-to-end testing and QA standards for ChainLogistics.

## 1. Testing Approach

We use a test pyramid with clear responsibilities at each level:

- Unit tests: validate isolated functions and modules.
- Integration tests: validate service boundaries and cross-module behavior.
- End-to-end tests: validate user workflows and critical paths.
- Non-functional tests: validate performance, reliability, and security.

### Principles

- Shift left: add tests in the same change that introduces behavior.
- Risk-based coverage: prioritize product registration, tracking events, auth, and verification.
- Deterministic tests: avoid flaky tests by controlling data, time, and external dependencies.
- Fast feedback: lint + unit tests run first, then integration and e2e.

## 2. Testing Levels and Scope

### Unit Testing

- Frontend: Vitest for components, hooks, utilities, and state stores.
- Backend and smart contracts: Rust unit tests with `cargo test`.
- SDKs: language-native unit tests for API client behavior and serialization.

Required for:

- New business logic.
- Bug fixes that can regress.
- Validation and parsing code.

### Integration Testing

- Backend integration tests cover DB migrations, service layer, and route contracts.
- Frontend integration tests cover route-level behavior and cross-component flows.
- SDK integration tests validate API contract compatibility against staging or mocked APIs.

Required scenarios:

- Product CRUD lifecycle.
- Tracking event lifecycle and query filters.
- Authentication and authorization paths.
- Failure paths (timeouts, 4xx/5xx, malformed payloads).

### End-to-End Testing

Playwright E2E tests validate complete workflows:

- Landing and navigation.
- Wallet connect flow (mocked in CI when needed).
- Product registration and tracking views.
- Verification and user-facing status.

Minimum E2E suite should cover:

- Happy path registration.
- One negative path per critical feature.
- Cross-browser execution in CI where supported.

### Performance Testing

Performance checks are required for high-risk changes and release candidates.

- Frontend: Core Web Vitals and route load-time checks.
- Backend: API latency for read/write endpoints under representative load.
- Smart contracts: gas/cost regression tracking (see `smart-contract/GAS_OPTIMIZATION_REPORT.md` and `docs/gas_usage_benchmarks.md`).

Suggested baseline targets:

- API p95 latency < 300 ms for common reads under expected load.
- No critical route should regress by > 15% from baseline without approval.
- Contract cost regressions > 10% require documented justification.

### Security Testing

Security validation is mandatory before release:

- Dependency vulnerability scanning (OSV and ecosystem tools in CI).
- Static analysis (CodeQL and lint checks).
- Authn/authz tests for protected endpoints.
- Input validation and error-handling tests.

Security checklist:

- No secrets in source or test fixtures.
- No auth bypass paths in integration tests.
- Validation failures return expected error responses.
- Dependency scan findings triaged before merge.

## 3. Coverage Requirements

Coverage thresholds are quality gates, not finish lines.

- Frontend (Vitest): 80% global threshold for lines, statements, branches, and functions.
- Backend/service code: target >= 80% for core modules and critical paths.
- Smart contracts: target >= 90% on contract logic and authorization rules.
- SDKs: target >= 80% on API clients, models, and error handling.

Additional requirements:

- Every bug fix adds at least one regression test.
- Critical modules (auth, payments/financial flows, tracking integrity) require branch coverage on decision logic.

## 4. Test Data Management

Test data rules:

- Keep deterministic fixture data in version-controlled test fixtures.
- Use generated IDs and timestamps to avoid collisions.
- Isolate test tenants/accounts where possible.
- Never use production secrets or PII in tests.

Recommended patterns:

- Unit tests: inline builders/factories.
- Integration tests: migration-seeded DB with per-test cleanup.
- E2E tests: stable mock mode where blockchain/network calls are not required.

## 5. Automation Pipeline

Testing automation currently runs in GitHub Actions workflows under `.github/workflows/`.

Default validation order on PRs:

1. Lint and static analysis.
2. Unit tests.
3. Integration tests.
4. E2E tests.
5. Build verification.

Pipeline requirements for contributions:

- All lint jobs pass.
- All required tests pass.
- No critical/high security findings remain untriaged.
- Build artifacts complete successfully.

## 6. Quality Gates and Release Criteria

A change is release-ready only if all gates pass:

- Linting and formatting checks pass.
- Required tests pass for touched areas.
- Coverage thresholds are met or exception approved.
- Security checks completed and reviewed.
- Documentation updated for behavior or interface changes.

Release checklist:

- Changelog/release notes updated.
- Migration notes added for breaking SDK or API changes.
- Operational checks completed (monitoring, alerts, rollback plan).

## 7. Integration Test Scenarios (Minimum Set)

- Product registration -> retrieval -> update -> deletion.
- Event creation -> list by product -> list by type.
- Unauthorized request handling.
- Input validation on malformed payloads.
- SDK-to-API contract compatibility for key endpoints.

## 8. Ownership and Maintenance

- The author of a change owns adding/updating tests in the same PR.
- Reviewers enforce quality gates and test relevance.
- Outdated tests should be fixed or removed in the same PR that changes behavior.
- Flaky tests should be quarantined only with a follow-up issue and owner.

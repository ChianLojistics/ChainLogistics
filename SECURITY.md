# Security Policy

This document describes how ChainLogistics manages dependency vulnerabilities,
how to report a security issue, and the risk-assessment framework used to
prioritize remediation.

## Reporting a Vulnerability

Do **not** open public GitHub issues for suspected security vulnerabilities.

Email `support@chainlogistics.io` with:

1. Affected component (backend, frontend, smart contract, SDK).
2. Affected version or commit.
3. Reproduction steps and impact.
4. Any suggested mitigation.

We will acknowledge receipt within **2 business days** and provide a triage
update within **5 business days**.

## Supported Versions

| Component       | Supported branches |
| --------------- | ------------------ |
| Backend (Rust)  | `main`             |
| Frontend (Next) | `main`             |
| Smart contracts | latest deployed    |
| Python SDK      | `1.x`              |
| Rust SDK        | `1.x`              |

Older release branches receive critical patches only.

## Automated Scanning

| Layer                | Tool                            | Cadence              |
| -------------------- | ------------------------------- | -------------------- |
| Rust crates          | `cargo-audit`, `cargo-deny`     | PR + daily 06:00 UTC |
| Cross-ecosystem CVEs | OSV-Scanner (Google)            | PR + daily 06:00 UTC |
| Node packages        | `npm audit`                     | PR + daily 06:00 UTC |
| Python packages      | `pip-audit` (OSV)               | PR + daily 06:00 UTC |
| Container images     | Trivy (config + filesystem)     | PR + daily 06:00 UTC |
| Static analysis (JS) | CodeQL                          | Weekly + push        |
| License compliance   | `license-checker`, `cargo-deny` | Weekly + push        |
| Secret scanning      | gitleaks                        | Weekly + push        |
| Dependency updates   | Dependabot (cargo/npm/pip/      | Daily / weekly       |
|                      | github-actions/docker)          |                      |

The PR gate is `Dependency scan gate` in
`.github/workflows/dependency-scan.yml`. It must succeed before merge.

To run the full scan suite locally:

```bash
./scripts/scan-deps.sh
```

The script auto-installs missing tools into a per-repo cache and skips any it
cannot install on the current platform.

## Risk Assessment Framework

Every newly reported vulnerability is scored on three axes before remediation
is scheduled.

### 1. Severity (CVSS v3.1)

| CVSS score   | Class    | Patch SLA            |
| ------------ | -------- | -------------------- |
| 9.0 – 10.0   | Critical | 24 hours             |
| 7.0 – 8.9    | High     | 7 days               |
| 4.0 – 6.9    | Medium   | 30 days              |
| 0.1 – 3.9    | Low      | Next minor release   |

### 2. Exploitability

| Factor                                              | Weight |
| --------------------------------------------------- | ------ |
| Public PoC / weaponized exploit available           | +2     |
| Reachable from untrusted user input in this codebase| +2     |
| Requires authenticated attacker                     | -1     |
| Requires local code execution                       | -1     |
| Mitigated by existing controls (WAF, rate limit)    | -1     |

If the adjusted score raises severity by one class, treat the issue at the
higher class.

### 3. Business Impact

| Component touched                          | Class      |
| ------------------------------------------ | ---------- |
| Smart contract or signing key path         | Critical   |
| Auth, KYC, payments, or PII handling       | High       |
| Customer-facing read paths                 | Medium     |
| Internal tooling, build, docs              | Low        |

Final priority = max(Severity, Exploitability-adjusted, Business-impact).

## Update Management

### Patch updates (semver patch)

- Auto-merged by Dependabot once the `Dependency scan gate` passes and CI is
  green, **except** for the manually-pinned dependencies enumerated in
  `.github/dependabot.yml` (e.g. `react`, `next`, `axum`, `soroban-sdk`).

### Minor updates

- Reviewed by a code owner. Merged after CI is green and a smoke test is run
  against the staging environment.

### Major updates

- Require a tracking issue, a manual test plan, and approval from the relevant
  area owner. Schedule outside merge-freeze windows.

### Emergency security patches

For Critical or High vulnerabilities with a public PoC:

1. File an internal incident issue (label: `security-incident`).
2. Open a hotfix PR targeting `main` with the upgrade only.
3. Bypass the normal review window with on-call approval.
4. Deploy to staging, run smoke tests, deploy to production.
5. Post-mortem within 5 business days documenting root cause and any
   detection gap.

## Rollback

Any dependency update can be rolled back by reverting the merge commit on
`main` and redeploying. The `deploy.yml` workflow's rollback job covers
frontend rollbacks; smart-contract rollbacks are documented inline in
`.github/workflows/deploy.yml` (contracts are immutable on-chain — rollback
means redeploying the prior WASM and updating the router/proxy).

## False-Positive Handling

When a scanner flags an advisory that does not apply to this codebase:

1. Open a PR adding the advisory ID to the relevant ignore list:
   - `deny.toml` → `[advisories].ignore` (with a `reason` and `expires` date)
   - `.github/dependabot.yml` → `ignore:` block for the dependency
2. Link the PR to the scanner output and a written justification.
3. The `expires` date forces a periodic re-evaluation; do not set it more
   than 90 days out.

## Audit Trail

- Dependency upgrades: every change is a Git commit referencing the advisory
  or release notes.
- Scan results: GitHub Security tab (SARIF uploads from OSV and Trivy on push
  to `main`) and the `Actions` tab for ad-hoc runs.
- Triage decisions: tracked as PR review comments on the Dependabot PR or as
  issues with the `security` label.

## Compliance

This policy is intended to satisfy the dependency-management controls in:

- SOC 2 CC7.1 (system monitoring) and CC7.2 (vulnerability management)
- ISO 27001 A.12.6.1 (technical vulnerability management)
- OWASP ASVS V14 (configuration)

Compliance evidence (scan logs, advisory history, patch SLAs) is preserved
via GitHub Actions retention (90 days) and Security tab history.

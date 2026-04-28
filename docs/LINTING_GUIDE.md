# Linting and Code Quality

This repository uses language-specific linting plus CI enforcement.

## Frontend (TypeScript/JavaScript)

- Tool: ESLint (`frontend/eslint.config.mjs`)
- Local command:

```bash
cd frontend && npm run lint
```

## Rust Codebases

Rust quality checks currently enforce Clippy for:

- `smart-contract/`

Local commands:

```bash
cargo clippy --manifest-path smart-contract/Cargo.toml --all-targets --all-features
```

Backend and SDK rust linting remain recommended, but are temporarily not enforced in CI while existing compile and migration debt is addressed.

## Repository-Level Lint Command

Run all lint checks from the repository root:

```bash
npm run lint
```

## Pre-Commit Hooks

Pre-commit hooks are defined in `.pre-commit-config.yaml`.

Install and enable:

```bash
pip install pre-commit
pre-commit install
```

Run manually:

```bash
pre-commit run --all-files
```

## CI Linting

CI lint checks run in `.github/workflows/lint.yml` and cover:

- Frontend ESLint
- Rust Clippy for `smart-contract/`

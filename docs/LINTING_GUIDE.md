# Linting and Code Quality

This repository uses language-specific linting plus CI enforcement.

## Frontend (TypeScript/JavaScript)

- Tool: ESLint (`frontend/eslint.config.mjs`)
- Local command:

```bash
cd frontend && npm run lint
```

## Rust Codebases

Rust quality checks use Clippy for:

- `smart-contract/`
- `backend/`
- `sdk/rust/`

Local commands:

```bash
cargo clippy --manifest-path smart-contract/Cargo.toml --all-targets --all-features
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features
cargo clippy --manifest-path sdk/rust/Cargo.toml --all-targets --all-features
```

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
- Rust Clippy for all Rust codebases

# ChainLogistics Smart Contracts

The smart-contract workspace contains the Soroban contracts that back product registration, event tracking, authorization, governance, and upgrade safety for ChainLogistics.

## Key Security Modules

- `contracts/src/oracle.rs`
  manages external data feeds with source registration, freshness checks, range validation, consensus aggregation, fallback snapshots, and circuit-breaker state.
- `contracts/src/timelock.rs`
  enforces delayed execution for critical governance actions with signer approvals, queue/cancel flows, and trusted contract-to-contract execution.
- `contracts/src/upgrade.rs`
  tracks upgrade status and versioning, and now accepts trusted timelock execution for delayed upgrade actions.
- `contracts/src/product_transfer.rs`
  exposes gas policy, batch cost estimation, and resumable chunk processing for large ownership-transfer workloads.

## Run Tests

```bash
cargo test -p chainlogistics --lib
```

## Notes

- The workspace uses `soroban-sdk = 25.3.0`.
- Host-side tests compile the full contract suite, including oracle and timelock flows.
- WASM builds still focus on the main contract entrypoint to avoid export collisions from multiple `#[contract]` modules in one crate.

# Smart Contract Gas Usage Benchmarks

This document outlines the gas usage benchmarks captured for the main execution paths in the ChainLogistics smart contracts. The measurements reflect the **budget consumed** per integration test flow, measured directly from the Soroban testing environment environment budget tracker.

## Product Lifecycle Gas Costs

### 1. Product Registration
- **Function / Flow**: `register_product`
- **CPU Instruction Cost**: 474,354
- **Memory Bytes Cost**: 202,201

### 2. Event Tracking
Events appended to a product's lifecycle have slightly varying costs based on the state growth and the number of authorized actors:
- **First Event (by Owner)**: `tracking_add_event`
  - CPU Instruction Cost: 311,449
  - Memory Bytes Cost: 144,980
- **Second Event (by Actor A)**: `tracking_add_event`
  - CPU Instruction Cost: 337,196
  - Memory Bytes Cost: 145,197
- **Third Event (by Actor B)**: `tracking_add_event`
  - CPU Instruction Cost: 345,262
  - Memory Bytes Cost: 151,419

### 3. Ownership Transfer
- **Function / Flow**: `transfer_product`
  - CPU Instruction Cost: 414,386
  - Memory Bytes Cost: 149,538

---

_These metrics are continuously tracked by integration tests in `smart-contract/contracts/src/test/integration_tests.rs`._

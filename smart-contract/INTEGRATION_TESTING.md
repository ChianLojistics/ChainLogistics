# Integration Testing Guide

## Overview

Comprehensive integration tests for ChainLogistics smart contracts covering complete supply chain scenarios.

## Test Scenarios

### 1. Complete Electronics Supply Chain

**File**: `contracts/src/test/supply_chain_scenarios.rs::test_complete_electronics_supply_chain`

**Flow**:

1. Manufacturer registers laptop product
2. Authorizes warehouse operator
3. Manufacturing complete event
4. Warehouse storage event
5. Transfer to distributor
6. Distributor shipping event
7. Transfer to retailer
8. Retailer received event

**Validates**:

- Multi-party workflow
- Authorization management
- Ownership transfers
- Event tracking
- Complete audit trail

### 2. Pharmaceutical Cold Chain with Compliance

**File**: `contracts/src/test/supply_chain_scenarios.rs::test_pharmaceutical_cold_chain_with_compliance`

**Flow**:

1. Register vaccine with FDA/GMP certifications
2. Authorize cold storage operator
3. Manufacturing with temperature metadata (-70C)
4. Cold storage with temperature/humidity monitoring
5. Transfer to pharmacy
6. Pharmacy receives with temperature validation

**Validates**:

- Certification tracking
- Metadata storage (temperature, humidity)
- Compliance requirements
- Cold chain integrity
- Regulatory data preservation

### 3. Food Supply Chain with Recall

**File**: `contracts/src/test/supply_chain_scenarios.rs::test_food_supply_chain_with_recall`

**Flow**:

1. Farmer registers organic beef
2. Harvesting event
3. Transfer to processor
4. Processing with USDA inspection
5. Transfer to distributor
6. Shipping event
7. RECALL: Contamination detected
8. Product deactivation

**Validates**:

- Product recall workflow
- Deactivation prevents new events
- Audit trail preservation
- Traceability for recalls
- Safety compliance

### 4. Multi-Product Batch Operations

**File**: `contracts/src/test/supply_chain_scenarios.rs::test_multi_product_batch_operations`

**Flow**:

1. Register 3 products
2. Add manufacturing events to all
3. Batch transfer all products
4. Verify ownership changes
5. Verify statistics

**Validates**:

- Batch operations efficiency
- Atomic transfers
- Statistics accuracy
- Scalability

### 5. Authorized Actor Workflow

**File**: `contracts/src/test/supply_chain_scenarios.rs::test_authorized_actor_workflow`

**Flow**:

1. Owner registers product
2. Authorizes logistics partner
3. Authorizes warehouse
4. Both add tracking events
5. Owner removes logistics authorization
6. Verify authorization states

**Validates**:

- Authorization management
- Multi-actor permissions
- Authorization revocation
- Access control

## Running Tests

### All Integration Tests

```bash
cargo test --lib supply_chain_scenarios
```

### Specific Scenario

```bash
cargo test --lib test_complete_electronics_supply_chain
```

### With Output

```bash
cargo test --lib supply_chain_scenarios -- --nocapture
```

## Test Environment

### Setup

- All contracts initialized (ChainLogistics, Registry, Authorization, Transfer, Tracking)
- Mock authentication enabled
- Fresh environment per test

### Actors

- Dynamically generated addresses
- Unique per test
- Properly authorized

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run Integration Tests
  run: |
    cd smart-contract
    cargo test --lib supply_chain_scenarios
```

### Test Coverage

- Multi-contract interactions: ✓
- Full supply chain workflows: ✓
- Error scenarios: ✓
- Edge cases: ✓

## Gas Benchmarking

Integration tests can be used for gas benchmarking:

```bash
cargo test --lib supply_chain_scenarios -- --nocapture | grep "gas"
```

## Future Scenarios

### Planned Tests

1. Cross-border customs workflow
2. Multi-modal transportation
3. Quality inspection checkpoints
4. Insurance claim scenarios
5. Sustainability tracking
6. Carbon footprint calculation

## Best Practices

1. **Realistic Scenarios**: Model real-world supply chains
2. **Complete Workflows**: Test end-to-end processes
3. **Error Handling**: Include failure scenarios
4. **Data Validation**: Verify all state changes
5. **Performance**: Monitor gas usage

## Troubleshooting

### Test Failures

- Check contract initialization
- Verify authorization setup
- Review event ordering
- Validate data formats

### Performance Issues

- Reduce batch sizes
- Optimize storage operations
- Use pagination for large datasets

## Documentation

Each test includes:

- Scenario description
- Actor roles
- Step-by-step flow
- Validation points
- Expected outcomes

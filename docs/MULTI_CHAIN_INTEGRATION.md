# Multi-Chain Integration Guide

## Overview

ChainLogistics now supports multiple blockchain networks, enabling enterprise customers to use their preferred blockchain ecosystem. This eliminates vendor lock-in and enables integration with existing enterprise blockchain infrastructure.

## Supported Networks

### EVM-Compatible Chains
- **Ethereum**: Largest DeFi and NFT ecosystem
- **Polygon**: Major scaling solution for enterprises
- **Quorum**: JP Morgan's enterprise blockchain

### Non-EVM Chains
- **Stellar**: Original network with Soroban smart contracts
- **Hyperledger Fabric**: 70% of enterprise blockchain deployments
- **Corda**: Financial industry standard

## Architecture

### Frontend Multi-Chain Support

The frontend uses a provider factory pattern to abstract blockchain interactions:

```typescript
import { blockchainFactory } from '@/lib/blockchain/factory';
import { BlockchainNetwork } from '@/lib/blockchain/types';

// Get provider for specific network
const provider = blockchainFactory.getProvider('ethereum');

// Connect wallet
const connection = await provider.connect();

// Send transaction
const txHash = await provider.sendTransaction({
  from: connection.address,
  to: '0x...',
  value: '1000000000000000000', // 1 ETH in wei
});
```

### Backend Multi-Chain Support

The backend provides a trait-based provider system:

```rust
use crate::blockchain::{BlockchainProvider, BlockchainNetwork};

// Get provider for network
let provider = match network {
    BlockchainNetwork::Stellar => Box::new(StellarProvider::new(rpc_url)),
    BlockchainNetwork::Ethereum => Box::new(EVMProvider::new(BlockchainNetwork::Ethereum, rpc_url)),
    // ...
};

// Call contract
let result = provider.call_contract(&call).await?;
```

## Configuration

### Environment Variables

```bash
# Stellar
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_CONTRACT_ID=CBUWSKT2UGOAXK4ZREVDJV5XHSYB42PZ3CERU2ZFUTUMAZLJEHNZIECA

# Ethereum
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ETH_CONTRACT_ID=0x...

# Polygon
POLYGON_RPC_URL=https://polygon-rpc.com
POLYGON_CONTRACT_ID=0x...

# Hyperledger
HYPERLEDGER_RPC_URL=http://localhost:7051
HYPERLEDGER_CONTRACT_ID=...

# Corda
CORDA_RPC_URL=http://localhost:10003
CORDA_CONTRACT_ID=...

# Quorum
QUORUM_RPC_URL=http://localhost:8545
QUORUM_CONTRACT_ID=0x...
```

## Frontend Components

### BlockchainSelector

Allows users to select their preferred blockchain network:

```tsx
import { BlockchainSelector } from '@/components/blockchain/BlockchainSelector';

<BlockchainSelector 
  onNetworkChange={(network) => console.log(network)}
  selectedNetwork="ethereum"
/>
```

### WalletConnector

Handles wallet connection for the selected network:

```tsx
import { WalletConnector } from '@/components/blockchain/WalletConnector';

<WalletConnector 
  network="ethereum"
  onConnect={(connection) => console.log(connection)}
  onDisconnect={() => console.log('disconnected')}
/>
```

## API Endpoints

### Get Blockchain Config

```bash
GET /api/v1/blockchain/config/:network
```

### Get Supported Networks

```bash
GET /api/v1/blockchain/networks
```

### Get Network Balance

```bash
GET /api/v1/blockchain/:network/balance/:address
```

## Migration Path

### For Existing Stellar Users

1. No changes required - Stellar remains fully supported
2. Optionally add additional networks for specific use cases
3. Use BlockchainSelector to switch between networks

### For New Customers

1. Choose preferred blockchain network during onboarding
2. Connect wallet for selected network
3. All operations use selected network

## Cross-Chain Considerations

### Data Consistency

- Each blockchain maintains independent state
- Sync service periodically reconciles data across chains
- Audit trail tracks all cross-chain operations

### Transaction Finality

Different networks have different confirmation requirements:

```rust
// Confirmation blocks by network
Stellar: 1 block
Ethereum: 12 blocks
Polygon: 128 blocks
Hyperledger: 1 block
Corda: 1 block
Quorum: 1 block
```

### Gas/Fee Estimation

Each provider implements gas estimation for its network:

```typescript
const estimatedGas = await provider.estimateGas({
  from: address,
  to: contractAddress,
  value: '0',
  data: encodedFunctionCall,
});
```

## Troubleshooting

### Wallet Connection Issues

1. Ensure wallet extension is installed
2. Verify network is added to wallet
3. Check RPC URL configuration
4. Verify contract address for network

### Transaction Failures

1. Check account balance
2. Verify gas/fee estimation
3. Check contract address validity
4. Review transaction data encoding

### Network Switching

1. Disconnect current wallet
2. Select new network
3. Reconnect wallet
4. Verify network in wallet matches selection

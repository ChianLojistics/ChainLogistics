import { BlockchainConfig, BlockchainNetwork } from './types';

const BLOCKCHAIN_CONFIGS: Record<BlockchainNetwork, BlockchainConfig> = {
    stellar: {
        network: 'stellar',
        rpcUrl: process.env.NEXT_PUBLIC_STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org',
        contractAddress: process.env.NEXT_PUBLIC_STELLAR_CONTRACT_ID || '',
        nativeToken: 'XLM',
        explorerUrl: 'https://stellar.expert',
        confirmationBlocks: 1,
    },
    ethereum: {
        network: 'ethereum',
        rpcUrl: process.env.NEXT_PUBLIC_ETH_RPC_URL || 'https://eth-mainnet.g.alchemy.com/v2/',
        chainId: 1,
        contractAddress: process.env.NEXT_PUBLIC_ETH_CONTRACT_ID || '',
        nativeToken: 'ETH',
        explorerUrl: 'https://etherscan.io',
        confirmationBlocks: 12,
    },
    polygon: {
        network: 'polygon',
        rpcUrl: process.env.NEXT_PUBLIC_POLYGON_RPC_URL || 'https://polygon-rpc.com',
        chainId: 137,
        contractAddress: process.env.NEXT_PUBLIC_POLYGON_CONTRACT_ID || '',
        nativeToken: 'MATIC',
        explorerUrl: 'https://polygonscan.com',
        confirmationBlocks: 128,
    },
    hyperledger: {
        network: 'hyperledger',
        rpcUrl: process.env.NEXT_PUBLIC_HYPERLEDGER_RPC_URL || '',
        contractAddress: process.env.NEXT_PUBLIC_HYPERLEDGER_CONTRACT_ID || '',
        nativeToken: 'HLF',
        explorerUrl: '',
        confirmationBlocks: 1,
    },
    corda: {
        network: 'corda',
        rpcUrl: process.env.NEXT_PUBLIC_CORDA_RPC_URL || '',
        contractAddress: process.env.NEXT_PUBLIC_CORDA_CONTRACT_ID || '',
        nativeToken: 'CORDA',
        explorerUrl: '',
        confirmationBlocks: 1,
    },
    quorum: {
        network: 'quorum',
        rpcUrl: process.env.NEXT_PUBLIC_QUORUM_RPC_URL || '',
        chainId: 1337,
        contractAddress: process.env.NEXT_PUBLIC_QUORUM_CONTRACT_ID || '',
        nativeToken: 'ETH',
        explorerUrl: '',
        confirmationBlocks: 1,
    },
};

export function getBlockchainConfig(network: BlockchainNetwork): BlockchainConfig {
    const config = BLOCKCHAIN_CONFIGS[network];
    if (!config) {
        throw new Error(`Unsupported blockchain network: ${network}`);
    }
    return config;
}

export function getSupportedNetworks(): BlockchainNetwork[] {
    return Object.keys(BLOCKCHAIN_CONFIGS) as BlockchainNetwork[];
}

export function getNetworkName(network: BlockchainNetwork): string {
    const names: Record<BlockchainNetwork, string> = {
        stellar: 'Stellar',
        ethereum: 'Ethereum',
        polygon: 'Polygon',
        hyperledger: 'Hyperledger Fabric',
        corda: 'Corda',
        quorum: 'Quorum',
    };
    return names[network];
}

import { BlockchainProvider, BlockchainNetwork } from './types';
import { StellarProvider } from './providers/stellar';
import { EVMProvider } from './providers/evm';

class BlockchainProviderFactory {
    private providers: Map<BlockchainNetwork, BlockchainProvider> = new Map();

    getProvider(network: BlockchainNetwork): BlockchainProvider {
        if (this.providers.has(network)) {
            return this.providers.get(network)!;
        }

        let provider: BlockchainProvider;

        switch (network) {
            case 'stellar':
                provider = new StellarProvider();
                break;
            case 'ethereum':
            case 'polygon':
            case 'quorum':
                provider = new EVMProvider(network);
                break;
            case 'hyperledger':
            case 'corda':
                throw new Error(`${network} provider not yet implemented`);
            default:
                throw new Error(`Unknown blockchain network: ${network}`);
        }

        this.providers.set(network, provider);
        return provider;
    }

    clearProvider(network: BlockchainNetwork): void {
        this.providers.delete(network);
    }

    clearAll(): void {
        this.providers.clear();
    }
}

export const blockchainFactory = new BlockchainProviderFactory();

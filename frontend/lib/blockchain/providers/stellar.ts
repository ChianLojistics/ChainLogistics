import { BaseBlockchainProvider } from './base';
import { WalletConnection, Transaction, BlockchainNetwork } from '../types';
import { getBlockchainConfig } from '../config';

export class StellarProvider extends BaseBlockchainProvider {
    network: BlockchainNetwork = 'stellar';
    private wallet: any = null;

    async connect(): Promise<WalletConnection> {
        try {
            if (typeof window === 'undefined') {
                throw new Error('Stellar wallet requires browser environment');
            }

            const freighter = (window as any).freighter;
            if (!freighter) {
                throw new Error('Freighter wallet not installed');
            }

            const publicKey = await freighter.getPublicKey();
            this.wallet = freighter;

            return {
                address: publicKey,
                network: this.network,
                isConnected: true,
            };
        } catch (error) {
            throw new Error(`Failed to connect Stellar wallet: ${error}`);
        }
    }

    async disconnect(): Promise<void> {
        this.wallet = null;
    }

    async getBalance(address: string): Promise<string> {
        if (!this.validateAddress(address)) {
            throw new Error('Invalid Stellar address');
        }

        try {
            const config = getBlockchainConfig('stellar');
            const response = await fetch(`${config.rpcUrl}/accounts/${address}`);
            const data = await response.json();

            const nativeBalance = data.balances.find((b: any) => b.asset_type === 'native');
            return nativeBalance?.balance || '0';
        } catch (error) {
            throw new Error(`Failed to get Stellar balance: ${error}`);
        }
    }

    async sendTransaction(tx: Partial<Transaction>): Promise<string> {
        if (!this.wallet) {
            throw new Error('Wallet not connected');
        }

        try {
            // Stellar transaction signing via Freighter
            const result = await this.wallet.signTransaction(tx.data);
            return result.hash;
        } catch (error) {
            throw new Error(`Failed to send Stellar transaction: ${error}`);
        }
    }

    async getTransaction(hash: string): Promise<Transaction> {
        if (!this.validateHash(hash)) {
            throw new Error('Invalid transaction hash');
        }

        try {
            const config = getBlockchainConfig('stellar');
            const response = await fetch(`${config.rpcUrl}/transactions/${hash}`);
            const data = await response.json();

            return {
                hash: data.hash,
                from: data.source_account,
                to: '',
                value: '0',
                status: data.successful ? 'confirmed' : 'failed',
                confirmations: 1,
                timestamp: new Date(data.created_at).getTime(),
            };
        } catch (error) {
            throw new Error(`Failed to get Stellar transaction: ${error}`);
        }
    }

    async callContract(method: string, params: any[]): Promise<any> {
        if (!this.wallet) {
            throw new Error('Wallet not connected');
        }

        try {
            // Soroban contract invocation
            const result = await this.wallet.invokeContract({
                method,
                params,
            });
            return result;
        } catch (error) {
            throw new Error(`Failed to call Stellar contract: ${error}`);
        }
    }

    async estimateGas(tx: Partial<Transaction>): Promise<string> {
        // Stellar uses fixed fees
        return '100'; // stroops
    }
}

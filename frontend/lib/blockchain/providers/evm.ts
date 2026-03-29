import { BaseBlockchainProvider } from './base';
import { WalletConnection, Transaction, BlockchainNetwork } from '../types';
import { getBlockchainConfig } from '../config';

export class EVMProvider extends BaseBlockchainProvider {
    network: BlockchainNetwork;
    private provider: Record<string, unknown> | null = null;
    private signer: Record<string, unknown> | null = null;

    constructor(network: BlockchainNetwork) {
        super();
        if (!['ethereum', 'polygon', 'quorum'].includes(network)) {
            throw new Error(`Invalid EVM network: ${network}`);
        }
        this.network = network;
    }

    async connect(): Promise<WalletConnection> {
        try {
            if (typeof window === 'undefined') {
                throw new Error('EVM wallet requires browser environment');
            }

            const ethereum = (window as unknown as Record<string, unknown>).ethereum as Record<string, unknown> | undefined;
            if (!ethereum) {
                throw new Error('MetaMask or compatible wallet not installed');
            }

            const config = getBlockchainConfig(this.network);
            const accounts = await (ethereum.request as (args: Record<string, unknown>) => Promise<string[]>)({ method: 'eth_requestAccounts' });

            // Switch to correct chain
            try {
                await (ethereum.request as (args: Record<string, unknown>) => Promise<void>)({
                    method: 'wallet_switchEthereumChain',
                    params: [{ chainId: `0x${config.chainId?.toString(16)}` }],
                });
            } catch (switchError: unknown) {
                const error = switchError as Record<string, unknown>;
                if (error.code === 4902) {
                    // Chain not added, user needs to add it manually
                    throw new Error(`Please add ${this.network} network to your wallet`);
                }
            }

            this.provider = ethereum;
            return {
                address: accounts[0],
                network: this.network,
                isConnected: true,
                chainId: config.chainId,
            };
        } catch (error) {
            throw new Error(`Failed to connect EVM wallet: ${error}`);
        }
    }

    async disconnect(): Promise<void> {
        this.provider = null;
        this.signer = null;
    }

    async getBalance(address: string): Promise<string> {
        if (!this.validateAddress(address)) {
            throw new Error('Invalid EVM address');
        }

        try {
            const config = getBlockchainConfig(this.network);
            const response = await fetch(config.rpcUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method: 'eth_getBalance',
                    params: [address, 'latest'],
                    id: 1,
                }),
            });

            const data = await response.json();
            return (BigInt(data.result) / BigInt(10 ** 18)).toString();
        } catch (error) {
            throw new Error(`Failed to get EVM balance: ${error}`);
        }
    }

    async sendTransaction(tx: Partial<Transaction>): Promise<string> {
        if (!this.provider) {
            throw new Error('Wallet not connected');
        }

        try {
            const hash = await (this.provider.request as (args: Record<string, unknown>) => Promise<string>)({
                method: 'eth_sendTransaction',
                params: [{
                    from: tx.from,
                    to: tx.to,
                    value: tx.value,
                    data: tx.data,
                    gas: tx.gasLimit,
                    gasPrice: tx.gasPrice,
                }],
            });

            return hash;
        } catch (error) {
            throw new Error(`Failed to send EVM transaction: ${error}`);
        }
    }

    async getTransaction(hash: string): Promise<Transaction> {
        if (!this.validateHash(hash)) {
            throw new Error('Invalid transaction hash');
        }

        try {
            const config = getBlockchainConfig(this.network);
            const response = await fetch(config.rpcUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method: 'eth_getTransactionReceipt',
                    params: [hash],
                    id: 1,
                }),
            });

            const data = await response.json();
            const receipt = data.result;

            return {
                hash,
                from: receipt.from,
                to: receipt.to,
                value: receipt.value,
                status: receipt.status === '0x1' ? 'confirmed' : 'failed',
                confirmations: receipt.blockNumber ? 1 : 0,
                timestamp: Date.now(),
            };
        } catch (error) {
            throw new Error(`Failed to get EVM transaction: ${error}`);
        }
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    async callContract(method: string, _params: Record<string, unknown>[]): Promise<Record<string, unknown>> {
        if (!this.provider) {
            throw new Error('Wallet not connected');
        }

        try {
            const config = getBlockchainConfig(this.network);
            // This is a simplified call - actual implementation would use ethers.js or web3.js
            const result = await (this.provider.request as (args: Record<string, unknown>) => Promise<Record<string, unknown>>)({
                method: 'eth_call',
                params: [{
                    to: config.contractAddress,
                    data: method,
                }],
            });

            return result;
        } catch (error) {
            throw new Error(`Failed to call EVM contract: ${error}`);
        }
    }

    async estimateGas(tx: Partial<Transaction>): Promise<string> {
        if (!this.provider) {
            throw new Error('Wallet not connected');
        }

        try {
            const config = getBlockchainConfig(this.network);
            const response = await fetch(config.rpcUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method: 'eth_estimateGas',
                    params: [{
                        from: tx.from,
                        to: tx.to,
                        value: tx.value,
                        data: tx.data,
                    }],
                    id: 1,
                }),
            });

            const data = await response.json();
            return data.result;
        } catch (error) {
            throw new Error(`Failed to estimate EVM gas: ${error}`);
        }
    }
}

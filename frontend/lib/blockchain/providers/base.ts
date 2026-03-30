import { BlockchainProvider, WalletConnection, Transaction } from '../types';
import { BlockchainNetwork } from '../types';

export abstract class BaseBlockchainProvider implements BlockchainProvider {
    abstract network: BlockchainNetwork;

    abstract connect(): Promise<WalletConnection>;
    abstract disconnect(): Promise<void>;
    abstract getBalance(address: string): Promise<string>;
    abstract sendTransaction(tx: Partial<Transaction>): Promise<string>;
    abstract getTransaction(hash: string): Promise<Transaction>;
    abstract callContract(method: string, params: Record<string, unknown>[]): Promise<Record<string, unknown>>;
    abstract estimateGas(tx: Partial<Transaction>): Promise<string>;

    protected validateAddress(address: string): boolean {
        return address.length > 0;
    }

    protected validateHash(hash: string): boolean {
        return hash.length > 0;
    }
}

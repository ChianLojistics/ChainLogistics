// Multi-chain blockchain abstraction types
export type BlockchainNetwork = 'stellar' | 'ethereum' | 'polygon' | 'hyperledger' | 'corda' | 'quorum';

export interface BlockchainConfig {
    network: BlockchainNetwork;
    rpcUrl: string;
    chainId?: number; // For EVM chains
    contractAddress: string;
    nativeToken: string;
    explorerUrl: string;
    confirmationBlocks: number;
}

export interface WalletConnection {
    address: string;
    network: BlockchainNetwork;
    isConnected: boolean;
    balance?: string;
    chainId?: number;
}

export interface Transaction {
    hash: string;
    from: string;
    to: string;
    value: string;
    data?: string;
    gasPrice?: string;
    gasLimit?: string;
    nonce?: number;
    status: 'pending' | 'confirmed' | 'failed';
    confirmations: number;
    timestamp: number;
}

export interface SmartContractMethod {
    name: string;
    inputs: Array<{ name: string; type: string }>;
    outputs: Array<{ name: string; type: string }>;
    stateMutability: 'pure' | 'view' | 'nonpayable' | 'payable';
}

export interface BlockchainProvider {
    network: BlockchainNetwork;
    connect(): Promise<WalletConnection>;
    disconnect(): Promise<void>;
    getBalance(address: string): Promise<string>;
    sendTransaction(tx: Partial<Transaction>): Promise<string>;
    getTransaction(hash: string): Promise<Transaction>;
    callContract(method: string, params: Record<string, unknown>[]): Promise<Record<string, unknown>>;
    estimateGas(tx: Partial<Transaction>): Promise<string>;
}

use crate::blockchain::BlockchainNetwork;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    pub network: BlockchainNetwork,
    pub rpc_url: String,
    pub chain_id: Option<u64>,
    pub contract_address: String,
    pub native_token: String,
    pub explorer_url: String,
    pub confirmation_blocks: u32,
}

pub struct BlockchainConfigManager {
    configs: HashMap<BlockchainNetwork, BlockchainConfig>,
}

impl BlockchainConfigManager {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        configs.insert(
            BlockchainNetwork::Stellar,
            BlockchainConfig {
                network: BlockchainNetwork::Stellar,
                rpc_url: std::env::var("STELLAR_RPC_URL")
                    .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
                chain_id: None,
                contract_address: std::env::var("STELLAR_CONTRACT_ID").unwrap_or_default(),
                native_token: "XLM".to_string(),
                explorer_url: "https://stellar.expert".to_string(),
                confirmation_blocks: 1,
            },
        );

        configs.insert(
            BlockchainNetwork::Ethereum,
            BlockchainConfig {
                network: BlockchainNetwork::Ethereum,
                rpc_url: std::env::var("ETH_RPC_URL")
                    .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/".to_string()),
                chain_id: Some(1),
                contract_address: std::env::var("ETH_CONTRACT_ID").unwrap_or_default(),
                native_token: "ETH".to_string(),
                explorer_url: "https://etherscan.io".to_string(),
                confirmation_blocks: 12,
            },
        );

        configs.insert(
            BlockchainNetwork::Polygon,
            BlockchainConfig {
                network: BlockchainNetwork::Polygon,
                rpc_url: std::env::var("POLYGON_RPC_URL")
                    .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),
                chain_id: Some(137),
                contract_address: std::env::var("POLYGON_CONTRACT_ID").unwrap_or_default(),
                native_token: "MATIC".to_string(),
                explorer_url: "https://polygonscan.com".to_string(),
                confirmation_blocks: 128,
            },
        );

        configs.insert(
            BlockchainNetwork::Hyperledger,
            BlockchainConfig {
                network: BlockchainNetwork::Hyperledger,
                rpc_url: std::env::var("HYPERLEDGER_RPC_URL").unwrap_or_default(),
                chain_id: None,
                contract_address: std::env::var("HYPERLEDGER_CONTRACT_ID").unwrap_or_default(),
                native_token: "HLF".to_string(),
                explorer_url: String::new(),
                confirmation_blocks: 1,
            },
        );

        configs.insert(
            BlockchainNetwork::Corda,
            BlockchainConfig {
                network: BlockchainNetwork::Corda,
                rpc_url: std::env::var("CORDA_RPC_URL").unwrap_or_default(),
                chain_id: None,
                contract_address: std::env::var("CORDA_CONTRACT_ID").unwrap_or_default(),
                native_token: "CORDA".to_string(),
                explorer_url: String::new(),
                confirmation_blocks: 1,
            },
        );

        configs.insert(
            BlockchainNetwork::Quorum,
            BlockchainConfig {
                network: BlockchainNetwork::Quorum,
                rpc_url: std::env::var("QUORUM_RPC_URL").unwrap_or_default(),
                chain_id: Some(1337),
                contract_address: std::env::var("QUORUM_CONTRACT_ID").unwrap_or_default(),
                native_token: "ETH".to_string(),
                explorer_url: String::new(),
                confirmation_blocks: 1,
            },
        );

        BlockchainConfigManager { configs }
    }

    pub fn get_config(&self, network: BlockchainNetwork) -> Option<BlockchainConfig> {
        self.configs.get(&network).cloned()
    }

    pub fn get_all_networks(&self) -> Vec<BlockchainNetwork> {
        self.configs.keys().copied().collect()
    }
}

impl Default for BlockchainConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

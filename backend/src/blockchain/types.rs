use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockchainNetwork {
    Stellar,
    Ethereum,
    Polygon,
    Hyperledger,
    Corda,
    Quorum,
}

impl BlockchainNetwork {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockchainNetwork::Stellar => "stellar",
            BlockchainNetwork::Ethereum => "ethereum",
            BlockchainNetwork::Polygon => "polygon",
            BlockchainNetwork::Hyperledger => "hyperledger",
            BlockchainNetwork::Corda => "corda",
            BlockchainNetwork::Quorum => "quorum",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: Option<String>,
    pub gas_price: Option<String>,
    pub gas_limit: Option<String>,
    pub nonce: Option<u64>,
    pub status: TransactionStatus,
    pub confirmations: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContractCall {
    pub method: String,
    pub params: Vec<String>,
    pub contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub network: BlockchainNetwork,
    pub contract_address: String,
    pub event_name: String,
    pub indexed_params: Vec<String>,
    pub data: String,
    pub block_number: u64,
    pub transaction_hash: String,
    pub log_index: u32,
}

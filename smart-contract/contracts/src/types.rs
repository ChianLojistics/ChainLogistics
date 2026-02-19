use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub owner: Address,
    pub authorized_actors: Vec<Address>,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SupplyChainEvent {
    pub product_id: u64,
    pub description: String,
    pub timestamp: u64,
    pub actor: Address,
}

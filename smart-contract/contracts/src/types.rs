 use soroban_sdk::{contracttype, Address, BytesN, Map, String, Symbol, Vec};
 
 #[contracttype]
 #[derive(Clone, Debug, Eq, PartialEq)]
 pub struct Origin {
     pub location: String,
 }
 
 #[contracttype]
 #[derive(Clone, Debug, Eq, PartialEq)]
 pub struct Product {
     pub id: String,
     pub name: String,
     pub description: String,
     pub origin: Origin,
     pub owner: Address,
     pub created_at: u64,
     pub active: bool,
     pub category: String,
     pub tags: Vec<String>,
     pub certifications: Vec<BytesN<32>>,
     pub media_hashes: Vec<BytesN<32>>,
     pub custom: Map<Symbol, String>,
 }
 
 #[contracttype]
 #[derive(Clone, Debug, Eq, PartialEq)]
 pub struct TrackingEvent {
     pub event_id: u64,
     pub product_id: String,
     pub actor: Address,
     pub timestamp: u64,
     pub event_type: Symbol,
     pub data_hash: BytesN<32>,
     pub note: String,
 }
 
 #[contracttype]
 #[derive(Clone, Debug, Eq, PartialEq)]
 pub enum DataKey {
     Product(String),
     ProductEventIds(String),
     Event(u64),
     EventSeq,
     Auth(String, Address),
 }

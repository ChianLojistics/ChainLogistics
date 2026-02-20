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

/// Enhanced tracking event for supply chain events
/// Supports rich metadata for various industries (coffee, pharma, etc.)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEvent {
    pub event_id: u64,
    pub product_id: String,
    pub actor: Address,
    pub timestamp: u64,
    pub event_type: Symbol,
    pub location: String,
    pub data_hash: BytesN<32>,
    pub note: String,
    /// Flexible metadata as key-value pairs
    /// Examples:
    /// - temperature: "2.5" (for cold chain)
    /// - quality_score: "95"
    /// - gps_coords: "6.5244,38.4356"
    /// - batch_number: "B2024-001"
    pub metadata: Map<Symbol, String>,
}

/// Paginated result for events
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventPage {
    pub events: Vec<TrackingEvent>,
    pub total_count: u64,
    pub has_more: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Product(String),
    ProductEventIds(String),
    Event(u64),
    EventSeq,
    Auth(String, Address),
    /// Index for events by type: (ProductId, EventType, Index) -> EventId
    EventTypeIndex(String, Symbol, u64),
    /// Count of events by type: (ProductId, EventType) -> Count
    EventTypeCount(String, Symbol),
}

/// Product statistics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductStats {
    pub total_products: u64,
    pub active_products: u64,
}

/// Event filter criteria for querying events
/// Uses sentinel values to indicate "no filter":
/// - event_type: empty Symbol means any type
/// - location: empty String means any location  
/// - start_time: 0 means no lower bound
/// - end_time: u64::MAX means no upper bound
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventFilter {
    pub event_type: Symbol,
    pub start_time: u64,
    pub end_time: u64,
    pub location: String,
}

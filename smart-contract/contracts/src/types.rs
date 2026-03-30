use soroban_sdk::{contracttype, Address, BytesN, Map, String, Symbol, Val, Vec};

// ─── Sustainability Types ──────────────────────────────────────────────────────

/// Verification status of a sustainability record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SustainabilityStatus {
    /// Submitted and awaiting third-party review.
    Pending,
    /// Approved and anchored on-chain with a certificate hash.
    Verified,
    /// Rejected due to invalid or unverifiable claims.
    Rejected,
}

/// On-chain sustainability record for a supply-chain product.
///
/// Numeric environmental values use integer milli-units to avoid floating point:
/// - `carbon_footprint_g`: CO₂ in grams
/// - `water_usage_ml`: water consumed in millilitres
/// - `renewable_energy_pct`: 0–100 integer percentage
/// - `waste_recycled_pct`: 0–100 integer percentage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SustainabilityRecord {
    /// CO₂-equivalent emissions in grams (non-negative).
    pub carbon_footprint_g: i128,
    /// Water consumption in millilitres (non-negative).
    pub water_usage_ml: i128,
    /// Percentage of energy from renewable sources (0–100).
    pub renewable_energy_pct: u32,
    /// Percentage of waste that is recycled (0–100).
    pub waste_recycled_pct: u32,
    /// Hash of the off-chain labor-compliance documentation.
    pub labor_compliance_hash: BytesN<32>,
    /// Optional third-party certification document hash.
    pub certificate_hash: Option<BytesN<32>>,
    /// Current verification status.
    pub status: SustainabilityStatus,
    /// Ledger timestamp when this record was submitted.
    pub recorded_at: u64,
    /// Address that submitted this record.
    pub recorded_by: Address,
    /// Address that verified/rejected this record, if any.
    pub verified_by: Option<Address>,
    /// Ledger timestamp of verification/rejection (0 if pending).
    pub verified_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeactInfo {
    pub reason: String,
    pub deactivated_at: u64,
    pub deactivated_by: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Origin {
    pub location: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub origin_location: String,
    pub category: String,
    pub tags: Vec<String>,
    pub certifications: Vec<BytesN<32>>,
    pub media_hashes: Vec<BytesN<32>>,
    pub custom: Map<Symbol, String>,
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
    pub deactivation_info: Vec<DeactInfo>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEvent {
    pub event_id: u64,
    pub product_id: String,
    pub actor: Address,
    pub timestamp: u64,
    pub event_type: Symbol,
    pub location: String, // Added missing location field
    pub data_hash: BytesN<32>,
    pub note: String,
    pub metadata: Map<Symbol, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEventPage {
    pub events: Vec<TrackingEvent>,
    pub total_count: u64,
    pub has_more: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductStats {
    pub total_products: u64,
    pub active_products: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Product(String),
    ProductEventIds(String),
    ProductEventTimestamps(String),
    ProductEventIdsByType(String, Symbol),
    ProductEventIdsByActor(String, Address),
    Event(u64),
    EventSeq,
    EventTypeIndex(String, Symbol, u64),
    EventTypeCount(String, Symbol),
    Auth(String, Address),
    Admin,
    Paused,
    AuthContract, // Added for cross-contract delegation
    MainContract, // Added for ProductTransferContract
    TransferContract,
    MultiSigContract, // Multi-signature contract address
    TotalProducts,
    ActiveProducts,
    SearchIndex(IndexKey), // For product search functionality
    ContractVersion,       // Current contract version
    UpgradeInfo,           // Current upgrade information
    UpgradeStatus,         // Current upgrade status
    EmergencyPause,        // Emergency pause flag
    MultiSigConfig,        // Multi-signature configuration
    Proposal(u64),         // Proposal by ID
    NextProposalId,        // Next proposal ID counter
    Sustainability(String), // SustainabilityRecord keyed by product_id
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEventInput {
    pub product_id: String,
    pub event_type: Symbol,
    pub data_hash: BytesN<32>,
    pub note: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEventFilter {
    pub event_type: Symbol,
    pub start_time: u64,
    pub end_time: u64,
    pub location: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexKey {
    Keyword(String), // keyword -> Vec<product_id>
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeInfo {
    pub new_version: ContractVersion,
    pub new_contract_address: Address,
    pub upgrade_timestamp: u64,
    pub upgraded_by: Address,
    pub migration_required: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpgradeStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

// ─── Multi-Signature Types ─────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub kind: Symbol, // "transfer_admin", "initiate_upgrade", "complete_upgrade", "fail_upgrade", "pause", "unpause"
    pub args: Vec<Val>,
    pub proposer: Address,
    pub created_at: u64,
    pub executed: bool,
    pub approvals: Vec<Address>,
}

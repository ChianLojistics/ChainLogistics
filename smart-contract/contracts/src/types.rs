use soroban_sdk::{contracttype, Address, BytesN, Map, String, Symbol, Val, Vec};

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
    AllProductIds,
    Auth(String, Address),
    EventTypeIndex(String, Symbol, u64),
    EventTypeCount(String, Symbol),
    EventActorIndex(String, Address, u64),
    EventActorCount(String, Address),
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
    Admin,                 // Admin address
    Paused,                // Pause status
    AuthContract,          // Authorization contract address
    MainContract,          // Main contract address
    TransferContract,      // Transfer contract address
    MultiSigContract,      // Multisig contract address
    TimelockContract,      // Timelock contract address
    OracleFeedConfig(Symbol),
    OracleFeedSources(Symbol),
    OracleSource(Symbol, Address),
    OracleReport(Symbol, Address),
    OracleSnapshot(Symbol),
    OracleFallback(Symbol),
    OracleCircuitBreaker(Symbol),
    TimelockConfig,
    TimelockOperation(u64),
    NextTimelockOperationId,
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

// ─── Oracle Security Types ───────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OracleFeedType {
    CommodityPrice,
    FuelPrice,
    ExchangeRate,
    Temperature,
    Humidity,
    GpsLocation,
    ShipmentCondition,
    SecureTimestamp,
    ComplianceStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleFeedConfig {
    pub feed_id: Symbol,
    pub feed_type: OracleFeedType,
    pub min_value: i128,
    pub max_value: i128,
    pub max_age_seconds: u64,
    pub min_sources: u32,
    pub max_deviation_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSource {
    pub reporter: Address,
    pub stake: i128,
    pub active: bool,
    pub reward_points: u32,
    pub slash_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleReport {
    pub reporter: Address,
    pub value: i128,
    pub observed_at: u64,
    pub submitted_at: u64,
    pub proof_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSnapshot {
    pub feed_id: Symbol,
    pub value: i128,
    pub observed_at: u64,
    pub source_count: u32,
    pub using_fallback: bool,
    pub circuit_broken: bool,
}

// ─── Gas Handling Types ──────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasPolicy {
    pub max_batch_size: u32,
    pub recommended_chunk_size: u32,
    pub base_cost_units: u64,
    pub per_item_cost_units: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasEstimate {
    pub item_count: u32,
    pub estimated_cost_units: u64,
    pub recommended_chunk_size: u32,
    pub recommended_chunk_count: u32,
    pub fits_single_transaction: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchProgress {
    pub requested: u32,
    pub processed: u32,
    pub succeeded: u32,
    pub next_cursor: u32,
    pub complete: bool,
}

// ─── Timelock Types ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimelockAction {
    PauseMain,
    UnpauseMain,
    SetMainMultisig(Address),
    InitiateUpgrade(ContractVersion, Address, bool),
    CompleteUpgrade,
    FailUpgrade(Symbol),
    EmergencyPause(Symbol),
    EmergencyUnpause,
    ConfigureOracleFeed(Address, OracleFeedConfig),
    SetOracleFallback(Address, Symbol, i128, u64),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub min_delay_seconds: u64,
    pub max_delay_seconds: u64,
    pub grace_period_seconds: u64,
    pub main_contract: Address,
    pub upgrade_contract: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimelockStatus {
    PendingApprovals,
    Queued,
    Executed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockOperation {
    pub id: u64,
    pub proposer: Address,
    pub action: TimelockAction,
    pub created_at: u64,
    pub ready_at: u64,
    pub execute_by: u64,
    pub status: TimelockStatus,
    pub approvals: Vec<Address>,
}

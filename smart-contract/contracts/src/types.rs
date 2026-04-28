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

// ─── Quality Control Types ────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityCertification {
    pub certification_id: String,
    pub certification_type: String,
    pub issuer: String,
    pub certificate_id: String,
    pub valid_from: u64,
    pub valid_until: u64,
    pub status: String,
    pub metadata: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityReading {
    pub reading_id: String,
    pub product_id: String,
    pub sensor_id: String,
    pub parameter: String,
    pub value: i128,
    pub unit: String,
    pub timestamp: u64,
    pub location: String,
    pub status: String, // "normal", "warning", "critical"
    pub threshold_min: Option<i128>,
    pub threshold_max: Option<i128>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityParameter {
    pub name: String,
    pub unit: String,
    pub threshold_min: Option<i128>,
    pub threshold_max: Option<i128>,
    pub critical_threshold_min: Option<i128>,
    pub critical_threshold_max: Option<i128>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterStats {
    pub count: u32,
    pub sum: i128,
    pub min: i128,
    pub max: i128,
    pub avg: i128,
    pub last_reading: i128,
    pub last_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensorInfo {
    pub address: Address,
    pub sensor_id: String,
    pub sensor_type: String,
    pub authorized: bool,
}

// Extend DataKey for quality control
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityDataKey {
    QualityControlEnabled(String),
    QualityControlAdmin(String),
    QualityCertification(String, String), // product_id, certification_id
    QualityCertifications(String),         // product_id -> Vec<certification_id>
    QualityReading(String, String),        // product_id, reading_id
    QualityReadings(String),               // product_id -> Vec<reading_id>
    QualityParameters(String),             // product_id -> Vec<QualityParameter>
    ParameterStats(String, String),        // product_id, parameter_name
    AuthorizedSensor(String, Address),     // product_id, sensor_address
    AuthorizedSensors(String),             // product_id -> Vec<sensor_address>
}

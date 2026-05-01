/// Type definitions for the ChainLogistics smart contracts.
/// This module contains all data structures used across the contract suite.
use soroban_sdk::{contracttype, Address, BytesN, Map, String, Symbol, Val, Vec};

/// Information about product deactivation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeactInfo {
    /// The reason for deactivation
    pub reason: String,
    /// Timestamp when the product was deactivated
    pub deactivated_at: u64,
    /// Address that deactivated the product
    pub deactivated_by: Address,
}

/// Product origin information.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Origin {
    /// The location where the product originated
    pub location: String,
}

/// Configuration for product registration.
/// Used as input when creating a new product.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductConfig {
    /// Unique product identifier
    pub id: String,
    /// Product name
    pub name: String,
    /// Product description
    pub description: String,
    /// Origin location
    pub origin_location: String,
    /// Product category
    pub category: String,
    /// Product tags for classification
    pub tags: Vec<String>,
    /// Certification hashes
    pub certifications: Vec<BytesN<32>>,
    /// Media file hashes (images, videos, etc.)
    pub media_hashes: Vec<BytesN<32>>,
    /// Custom key-value metadata
    pub custom: Map<Symbol, String>,
}

/// Complete product information stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Product {
    /// Unique product identifier
    pub id: String,
    /// Product name
    pub name: String,
    /// Product description
    pub description: String,
    /// Product origin information
    pub origin: Origin,
    /// Current product owner
    pub owner: Address,
    /// Timestamp when product was created
    pub created_at: u64,
    /// Whether the product is active (can receive events)
    pub active: bool,
    /// Product category
    pub category: String,
    /// Product tags
    pub tags: Vec<String>,
    /// Certification hashes
    pub certifications: Vec<BytesN<32>>,
    /// Media file hashes
    pub media_hashes: Vec<BytesN<32>>,
    /// Custom metadata
    pub custom: Map<Symbol, String>,
    /// Deactivation history
    pub deactivation_info: Vec<DeactInfo>,
}

/// A tracking event in the product's supply chain journey.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEvent {
    /// Unique event identifier
    pub event_id: u64,
    /// ID of the product this event belongs to
    pub product_id: String,
    /// Address that added this event
    pub actor: Address,
    /// Timestamp when the event occurred
    pub timestamp: u64,
    /// Type of event (e.g., "shipped", "received", "processed")
    pub event_type: Symbol,
    /// Location where the event occurred
    pub location: String,
    /// Hash of the event data for integrity verification
    pub data_hash: BytesN<32>,
    /// Optional note about the event
    pub note: String,
    /// Additional event metadata
    pub metadata: Map<Symbol, String>,
}

/// Paginated result for tracking events.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEventPage {
    /// The events in this page
    pub events: Vec<TrackingEvent>,
    /// Total number of events available
    pub total_count: u64,
    /// Whether more events are available
    pub has_more: bool,
}

/// Global product statistics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductStats {
    /// Total number of products ever registered
    pub total_products: u64,
    /// Number of currently active products
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
    ReentrancyLock(Symbol),
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

/// Contract version information following semantic versioning.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVersion {
    /// Major version (incompatible API changes)
    pub major: u32,
    /// Minor version (backwards-compatible functionality additions)
    pub minor: u32,
    /// Patch version (backwards-compatible bug fixes)
    pub patch: u32,
}

/// Information about a contract upgrade.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeInfo {
    /// The new contract version
    pub new_version: ContractVersion,
    /// Address of the new contract
    pub new_contract_address: Address,
    /// When the upgrade was initiated
    pub upgrade_timestamp: u64,
    /// Address that initiated the upgrade
    pub upgraded_by: Address,
    /// Whether data migration is required
    pub migration_required: bool,
}

/// Status of a contract upgrade process.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpgradeStatus {
    /// Upgrade has not been started
    NotStarted,
    /// Upgrade is in progress
    InProgress,
    /// Upgrade completed successfully
    Completed,
    /// Upgrade failed
    Failed,
}

/// Status of a multi-signature proposal.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is active and awaiting approvals
    Active,
    /// Threshold reached, awaiting time-lock or execution
    Approved,
    /// Proposal has been executed
    Executed,
    /// Proposal was rejected by signers
    Rejected,
    /// Proposal has expired
    Expired,
}

// ─── Multi-Signature Types ─────────────────────────────────────────────────────

/// Multi-signature configuration for administrative actions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    /// List of authorized signer addresses
    pub signers: Vec<Address>,
    /// Default number of signatures required
    pub threshold: u32,
    /// Specific thresholds for different operation types (kind -> threshold)
    pub thresholds: Map<Symbol, u32>,
    /// Time locks for different operation types (kind -> delay in seconds)
    pub time_locks: Map<Symbol, u64>,
}

/// A multi-signature proposal for administrative actions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    /// Unique proposal identifier
    pub id: u64,
    /// Type of proposal (e.g., "transfer_admin", "pause", "unpause")
    pub kind: Symbol,
    /// Target contract for the proposal
    pub target: Address,
    /// Arguments for the proposal
    pub args: Vec<Val>,
    /// Address that created the proposal
    pub proposer: Address,
    /// When the proposal was created
    pub created_at: u64,
    /// Timestamp when threshold was reached (0 if not reached)
    pub approved_at: u64,
    /// Current status of the proposal
    pub status: ProposalStatus,
    /// Addresses that have approved the proposal
    pub approvals: Vec<Address>,
    /// Addresses that have rejected the proposal
    pub rejections: Vec<Address>,
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
    QualityCertifications(String),        // product_id -> Vec<certification_id>
    QualityReading(String, String),       // product_id, reading_id
    QualityReadings(String),              // product_id -> Vec<reading_id>
    QualityParameters(String),            // product_id -> Vec<QualityParameter>
    ParameterStats(String, String),       // product_id, parameter_name
    AuthorizedSensor(String, Address),    // product_id, sensor_address
    AuthorizedSensors(String),            // product_id -> Vec<sensor_address>
}

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
    // ── Circuit Breaker ──────────────────────────────────────────────────
    CircuitBreakerContract,         // Address of the circuit breaker contract
    CircuitBreakerState,            // Global CircuitBreakerState struct
    CircuitBreakerGuardians,        // Vec<Address> of authorised guardians
    CircuitBreakerPauseRecord(u64), // PauseRecord by sequential ID
    CircuitBreakerNextRecordId,     // u64 counter for pause records
    CircuitBreakerPendingApproval(u64), // PauseApproval struct for multi-auth pauses
    CircuitBreakerNextApprovalId,   // u64 counter for pending approvals
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

// ─── Circuit Breaker Types ────────────────────────────────────────────────────

/// Severity level of a pause — controls which functions are blocked.
///
/// | Level      | Blocked operations                                      |
/// |------------|---------------------------------------------------------|
/// | Advisory   | None — informational only, no functions blocked         |
/// | Partial    | Tracking event writes; product registration             |
/// | Full       | All state-mutating operations                           |
/// | Emergency  | All operations including reads (except status queries)  |
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PauseLevel {
    /// Informational — no functions blocked.
    Advisory,
    /// Block write operations (add_tracking_event, register_product).
    Partial,
    /// Block all state-mutating operations.
    Full,
    /// Block everything including reads (maximum lockdown).
    Emergency,
}

/// Reason category for a pause — used for off-chain monitoring and audit.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PauseReason {
    SecurityBreach,
    OracleFailure,
    RegulatoryAction,
    ContractBug,
    MarketVolatility,
    Maintenance,
    Other,
}

/// Immutable record written to storage when a pause is activated or lifted.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseRecord {
    pub record_id: u64,
    pub activated_by: Address,
    pub level: PauseLevel,
    pub reason: PauseReason,
    pub description: String,
    pub activated_at: u64,
    /// 0 means no expiry.
    pub expires_at: u64,
    pub lifted_at: u64,
    pub lifted_by: Vec<Address>,
}

/// Live state of the circuit breaker stored in persistent storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreakerState {
    pub is_paused: bool,
    pub level: PauseLevel,
    pub current_record_id: u64,
    pub paused_at: u64,
    pub expires_at: u64,
}

/// A pending multi-authority pause approval request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseApproval {
    pub approval_id: u64,
    pub proposed_level: PauseLevel,
    pub proposed_reason: PauseReason,
    pub description: String,
    pub expires_at: u64,
    pub proposer: Address,
    pub approvals: Vec<Address>,
    pub required_approvals: u32,
    pub executed: bool,
}

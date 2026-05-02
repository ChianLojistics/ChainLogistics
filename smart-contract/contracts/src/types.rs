use soroban_sdk::{contracttype, i64, u64, Address, BytesN, Map, String, Symbol, Vec, I256, U256};

// ─── Core Product Types ───────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Product {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub origin: String,
    pub category: String,
    pub description: String,
    pub tags: Vec<String>,
    pub certifications: Vec<String>,
    pub media_hashes: Vec<BytesN<32>>,
    pub custom_fields: Map<String, String>,
    pub custom: Map<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ProductStatus,
    pub supply_chain_events: Vec<SupplyChainEvent>,
    pub owner: Address,
    pub active: bool,
    pub deactivation_info: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductStatus {
    Active,
    Deactivated,
    Recalled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainEvent {
    pub event_id: u64,
    pub product_id: String,
    pub event_type: Symbol,
    pub location: String,
    pub timestamp: u64,
    pub participant: Address,
    pub data_hash: BytesN<32>,
    pub metadata: Map<String, String>,
}

// ─── Batch Operations ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchProductTransfer {
    pub batch_id: u64,
    pub transfers: Vec<ProductTransfer>,
    pub total_amount: U256,
    pub from_address: Address,
    pub to_address: Address,
    pub timestamp: u64,
    pub metadata: Map<String, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTransfer {
    pub product_id: String,
    pub quantity: U256,
    pub unit_price: Option<U256>,
}

// ─── Search and Query Types ───────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSearchQuery {
    pub keywords: Vec<String>,
    pub category: Option<String>,
    pub origin: Option<String>,
    pub certification: Option<String>,
    pub status: Option<ProductStatus>,
    pub limit: u32,
    pub offset: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSearchResult {
    pub products: Vec<Product>,
    pub total_count: u32,
    pub has_more: bool,
}

// ─── Validation Types ───────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub error_code: u32,
    pub message: String,
}

// ─── Storage Keys ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Product(String),
    ProductCount,
    Batch(u64),
    BatchCount,
    SearchIndex(String),
    ValidationCache(String),
    Auth(String, Address),
    Admin,
    Paused,
    AuthContract,
    MainContract,
    TransferContract,
    MultiSigContract,
    TotalProducts,
    ActiveProducts,
    SearchIndex(IndexKey),
    ContractVersion,
    UpgradeInfo,
    UpgradeStatus,
    EmergencyPause,
    MultiSigConfig,
    Proposal(u64),
    NextProposalId,
    // ── Circuit Breaker ──────────────────────────────────────────────────
    CircuitBreakerContract,
    CircuitBreakerState,
    CircuitBreakerGuardians,
    CircuitBreakerPauseRecord(u64),
    CircuitBreakerNextRecordId,
    CircuitBreakerPendingApproval(u64),
    CircuitBreakerNextApprovalId,
    EventSeq,
}

// ─── Event Types ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEventInput {
    pub product_id: String,
    pub event_type: Symbol,
    pub location: String,
    pub participant: Address,
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

// ─── Index Types ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexKey {
    Keyword(String),
}

// ─── Upgrade Types ─────────────────────────────────────────────────────────

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

// ─── Multi-Signature Types ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub proposal_expiry: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub proposal_id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub target_address: Address,
    pub call_data: BytesN<32>,
    pub approvals: Vec<Address>,
    pub executed: bool,
    pub created_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalType {
    Upgrade,
    Transfer,
    Pause,
    Unpause,
    Custom,
}

// ─── Circuit Breaker Types ─────────────────────────────────────────────────

/// Pause severity levels.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PauseLevel {
    /// Informational — no functions blocked.
    Advisory,
    /// Block write operations (add_tracking_event, register_product).
    Partial,
    /// Block writes and mutations (transfer_product, batch_transfer).
    Full,
    /// Block everything including reads (except status queries).
    Emergency,
}

/// Reason category for a pause — used for off-chain monitoring and audit.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PauseReason {
    SecurityBreach,
    OracleFailure,
    RegulatoryAction,
    Maintenance,
    MarketVolatility,
    SystemUpgrade,
    ContractBug,
    Other,
}

/// Historical pause record stored for audit and analytics.
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
    /// Addresses that lifted the pause (empty if still active).
    pub lifted_by: Vec<Address>,
    /// When the pause was lifted (0 if still active).
    pub lifted_at: u64,
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

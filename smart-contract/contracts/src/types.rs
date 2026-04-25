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

// ─── Multi-Signature Types ─────────────────────────────────────────────────────

/// Multi-signature configuration for administrative actions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    /// List of authorized signer addresses
    pub signers: Vec<Address>,
    /// Number of signatures required to execute a proposal
    pub threshold: u32,
}

/// A multi-signature proposal for administrative actions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    /// Unique proposal identifier
    pub id: u64,
    /// Type of proposal (e.g., "transfer_admin", "pause", "unpause")
    pub kind: Symbol,
    /// Arguments for the proposal
    pub args: Vec<Val>,
    /// Address that created the proposal
    pub proposer: Address,
    /// When the proposal was created
    pub created_at: u64,
    /// Whether the proposal has been executed
    pub executed: bool,
    /// Addresses that have approved the proposal
    pub approvals: Vec<Address>,
}

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Core (1-10) ---
    ProductAlreadyExists = 1,
    ProductNotFound = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    EventNotFound = 5,
    NotInitialized = 6,
    AlreadyInitialized = 7,
    ContractPaused = 8,
    ContractNotPaused = 9,

    // --- Validation (10-30) ---
    InvalidProductId = 10,
    InvalidProductName = 11,
    InvalidOrigin = 12,
    InvalidCategory = 13,
    ProductIdTooLong = 14,
    ProductNameTooLong = 15,
    OriginTooLong = 16,
    CategoryTooLong = 17,
    DescriptionTooLong = 18,
    TooManyTags = 19,
    TagTooLong = 20,
    TooManyCertifications = 21,
    TooManyMediaHashes = 22,
    TooManyCustomFields = 23,
    CustomFieldValueTooLong = 24,

    // --- Batch (30-40) ---
    EmptyBatch = 30,
    BatchTooLarge = 31,
    DuplicateProductIdInBatch = 32,

    // --- Lifecycle (40-50) ---
    ProductDeactivated = 40,
    DeactivationReasonRequired = 41,
    ProductAlreadyActive = 42,

    // --- Upgrade (50-60) ---
    InvalidUpgrade = 50,
    UpgradeInProgress = 51,
    NoUpgradeInProgress = 52,
    EmergencyPaused = 53,
    NotEmergencyPaused = 54,

    // --- Multi-Signature (60-70) ---
    MultiSigNotConfigured = 60,
    NotSigner = 61,
    ProposalNotFound = 62,
    AlreadyApproved = 63,
    ProposalAlreadyExecuted = 64,
    ThresholdNotReached = 65,
    InvalidThreshold = 66,
    TooManySigners = 67,
    DuplicateSigner = 68,

    // --- Sustainability (70-80) ---
    /// No sustainability record exists for this product.
    SustainabilityNotFound = 70,
    /// Carbon footprint value is negative.
    InvalidCarbonData = 71,
    /// Water usage value is negative.
    InvalidWaterData = 72,
    /// Renewable energy percentage is out of range (must be 0–100).
    InvalidRenewableEnergyData = 73,
    /// Waste-recycled percentage is out of range (must be 0–100).
    InvalidWasteData = 74,
    /// Record has already been verified and cannot be updated.
    SustainabilityAlreadyVerified = 75,
    /// Operation requires a verified sustainability record.
    SustainabilityClaimUnverified = 76,
}

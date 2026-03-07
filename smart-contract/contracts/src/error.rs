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
}
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Core ---
    ProductAlreadyExists = 1,
    ProductNotFound = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    EventNotFound = 5,

    // --- Validation ---
    InvalidProductId = 6,
    InvalidProductName = 7,
    InvalidOrigin = 8,
    InvalidCategory = 9,

    ProductIdTooLong = 10,
    ProductNameTooLong = 11,
    OriginTooLong = 12,
    CategoryTooLong = 13,
    DescriptionTooLong = 14,

    TooManyTags = 15,
    TagTooLong = 16,
    TooManyCertifications = 17,
    TooManyMediaHashes = 18,

    TooManyCustomFields = 19,
    CustomFieldValueTooLong = 20,

    EmptyBatch = 21,
    BatchTooLarge = 22,
    DuplicateProductIdInBatch = 23,

    NotInitialized = 24,
    AlreadyInitialized = 25,
    ContractPaused = 26,
    // --- Lifecycle ---
    /// Attempted to add a tracking event to a deactivated product.
    ProductDeactivated = 21,
    /// Deactivation reason string is empty.
    DeactivationReasonRequired = 22,
    /// Attempted to reactivate a product that is already active.
    ProductAlreadyActive = 23,
}
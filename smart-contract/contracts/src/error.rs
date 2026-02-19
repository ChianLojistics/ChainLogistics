use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ProductAlreadyExists = 1,
    ProductNotFound = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    EventNotFound = 5,

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
}
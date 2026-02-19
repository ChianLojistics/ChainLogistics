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
    NotAuthorized = 6,
    AlreadyAuthorized = 7,
    CannotRemoveSelf = 8,
    ActorNotFound = 9,
}
use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    // Global counters
    TotalProducts,
    ActiveProducts,

    // Product storage: ProductId -> Product
    Product(u64),

    // Global Index: Index -> ProductId
    AllProductsIndex(u64),

    // Owner Index: (Owner, Index) -> ProductId
    OwnerProductIndex(Address, u64),
    OwnerProductCount(Address),

    // Origin Index: (Origin, Index) -> ProductId
    OriginProductIndex(String, u64),
    OriginProductCount(String),

    // Authorization: (ProductId, Actor) -> bool
    Auth(u64, Address),
}


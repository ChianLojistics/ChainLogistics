// Typed Contract Events
//
// This module defines strongly-typed contract events using the `#[contractevent]` macro.
// This approach replaces the deprecated `env.events().publish()` method with several
// advantages:
//
// 1. **Type Safety**: Events have defined fields with specific types, preventing
//    runtime errors from incorrect event data.
//
// 2. **Better Tooling**: IDEs can provide autocomplete and type checking for events.
//
// 3. **Easier Testing**: Events can be easily constructed and compared in tests.
//
// 4. **Documentation**: Each event struct serves as self-documenting API.
//
// Note: Some events include `_unused: u32` fields because the soroban-sdk 25.3.0
// `#[contractevent]` macro requires at least one field and doesn't support unit structs.

use soroban_sdk::{contractevent, Address, Symbol, Val, Vec};

use crate::types::{ContractVersion, Product, TrackingEvent};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackingEventPublished {
    pub product_id: soroban_sdk::String,
    pub event_id: u64,
    pub event: TrackingEvent,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRegistered {
    pub product_id: soroban_sdk::String,
    pub product: Product,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductDeactivated {
    pub product_id: soroban_sdk::String,
    pub owner: Address,
    pub reason: soroban_sdk::String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductReactivated {
    pub product_id: soroban_sdk::String,
    pub owner: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTransferred {
    pub product_id: soroban_sdk::String,
    pub from: Address,
    pub to: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminInitialized {
    pub admin: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPaused {
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractUnpaused {
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    pub current_admin: Address,
    pub new_admin: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeInitiated {
    pub current_version: ContractVersion,
    pub new_version: ContractVersion,
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeCompleted {
    pub new_version: ContractVersion,
    pub new_contract_address: Address,
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeFailed {
    pub caller: Address,
    pub reason: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyPause {
    pub caller: Address,
    pub reason: Symbol,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyUnpause {
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeReset {
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigInitialized {
    pub signers: Vec<Address>,
    pub threshold: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalSubmitted {
    pub proposal_id: u64,
    pub proposer: Address,
    pub kind: Symbol,
    pub args: Vec<Val>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalApproved {
    pub proposal_id: u64,
    pub approver: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferExecuted {
    pub args: Vec<Val>,
    pub executor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeInitiateExecuted {
    pub args: Vec<Val>,
    pub executor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeCompleteExecuted {
    pub _unused: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeFailExecuted {
    pub args: Vec<Val>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseExecuted {
    pub _unused: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpauseExecuted {
    pub _unused: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecuted {
    pub proposal_id: u64,
    pub executor: Address,
    pub kind: Symbol,
    pub args: Vec<Val>,
}

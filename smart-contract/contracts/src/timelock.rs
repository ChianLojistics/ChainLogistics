use soroban_sdk::{contracttype, Address, Env, Vec, u64, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockOperation {
    pub target: Address,
    pub value: u64,
    pub data: BytesN<32>,
    pub operation_type: u32,
    pub timelock_duration: u64,
    pub created_at: u64,
    pub executable_at: u64,
    pub executed: bool,
    pub required_approvals: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockConfig {
    pub admin: Address,
    pub min_delay: u64,
    pub max_delay: u64,
    pub grace_period: u64,
}

pub fn create_timelock_operation(
    env: &Env,
    target: Address,
    value: u64,
    data: BytesN<32>,
    operation_type: u32,
    timelock_duration: u64,
) -> TimelockOperation {
    let now = env.ledger().timestamp();
    TimelockOperation {
        target,
        value,
        data,
        operation_type,
        timelock_duration,
        created_at: now,
        executable_at: now + timelock_duration,
        executed: false,
        required_approvals: 1,
    }
}

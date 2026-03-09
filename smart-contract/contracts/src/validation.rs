 #![allow(dead_code)]
use soroban_sdk::String;

use crate::validation_contract::ValidationContract;

pub fn non_empty(s: &String) -> bool {
    ValidationContract::non_empty(s).is_ok()
}

pub fn max_len(s: &String, max: u32) -> bool {
    ValidationContract::max_len(s, max).is_ok()
}

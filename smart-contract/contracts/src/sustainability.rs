/// Sustainability contract for managing product sustainability claims.
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec};

use crate::error::Error;
use crate::types::{DataKey, SustainabilityClaim, SustainabilityMetric};
use crate::ChainLogisticsContractClient;

// ─── Storage helpers for SustainabilityContract ────────────────────────────────

fn get_main_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::MainContract)
}

fn set_main_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::MainContract, address);
}

fn require_init(env: &Env) -> Result<(), Error> {
    if get_main_contract(env).is_none() {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

fn next_claim_id(env: &Env) -> u64 {
    let id: u64 = env.storage().instance().get(&DataKey::NextSustainabilityClaimId).unwrap_or(0);
    env.storage().instance().set(&DataKey::NextSustainabilityClaimId, &(id + 1));
    id + 1
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct SustainabilityContract;

#[contractimpl]
impl SustainabilityContract {
    pub fn init(env: Env, main_contract: Address) -> Result<(), Error> {
        if get_main_contract(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        set_main_contract(&env, &main_contract);
        env.storage().instance().set(&DataKey::NextSustainabilityClaimId, &0u64);
        Ok(())
    }

    pub fn add_claim(
        env: Env,
        actor: Address,
        product_id: String,
        metric: SustainabilityMetric,
        value: String,
        certificate_hash: BytesN<32>,
        metadata: Map<Symbol, String>,
    ) -> Result<u64, Error> {
        require_init(&env)?;
        actor.require_auth();

        let claim_id = next_claim_id(&env);

        let claim = SustainabilityClaim {
            claim_id,
            product_id: product_id.clone(),
            metric,
            value,
            verifier: actor,
            timestamp: env.ledger().timestamp(),
            certificate_hash,
            metadata,
        };

        env.storage().persistent().set(&DataKey::SustainabilityClaim(product_id.clone(), claim_id), &claim);

        let mut claims: Vec<u64> = env.storage().persistent().get(&DataKey::ProductSustainabilityClaims(product_id.clone())).unwrap_or(Vec::new(&env));
        claims.push_back(claim_id);
        env.storage().persistent().set(&DataKey::ProductSustainabilityClaims(product_id.clone()), &claims);

        env.events().publish(
            (Symbol::new(&env, "sustainability_claim"), product_id, claim_id),
            claim,
        );

        Ok(claim_id)
    }

    pub fn get_claims(env: Env, product_id: String) -> Vec<SustainabilityClaim> {
        let ids: Vec<u64> = env.storage().persistent().get(&DataKey::ProductSustainabilityClaims(product_id.clone())).unwrap_or(Vec::new(&env));
        let mut claims = Vec::new(&env);
        for id in ids.iter() {
            if let Some(claim) = env.storage().persistent().get(&DataKey::SustainabilityClaim(product_id.clone(), id)) {
                claims.push_back(claim);
            }
        }
        claims
    }
}

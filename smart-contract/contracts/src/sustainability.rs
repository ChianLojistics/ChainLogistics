use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol};

use crate::error::Error;
use crate::storage;
use crate::types::{SustainabilityRecord, SustainabilityStatus};

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = storage::get_admin(env).ok_or(Error::NotInitialized)?;
    if &admin != caller {
        return Err(Error::Unauthorized);
    }
    caller.require_auth();
    Ok(())
}

fn validate_record(
    carbon_footprint_g: i128,
    water_usage_ml: i128,
    renewable_energy_pct: u32,
    waste_recycled_pct: u32,
) -> Result<(), Error> {
    if carbon_footprint_g < 0 {
        return Err(Error::InvalidCarbonData);
    }
    if water_usage_ml < 0 {
        return Err(Error::InvalidWaterData);
    }
    if renewable_energy_pct > 100 {
        return Err(Error::InvalidRenewableEnergyData);
    }
    if waste_recycled_pct > 100 {
        return Err(Error::InvalidWasteData);
    }
    Ok(())
}

/// Contract for recording and verifying supply-chain sustainability claims.
///
/// Flow:
/// 1. A product owner calls `record_sustainability` to submit environmental data.
/// 2. An admin calls `verify_sustainability` (anchoring a certificate hash) or
///    `reject_sustainability` to update the record status.
/// 3. Anyone can call `get_sustainability` to read the current record.
#[contract]
pub struct SustainabilityContract;

#[contractimpl]
impl SustainabilityContract {
    /// Submit a sustainability record for a product.
    ///
    /// Fails if a `Verified` record already exists for this product.
    /// Overwrites a `Pending` or `Rejected` record.
    pub fn record_sustainability(
        env: Env,
        caller: Address,
        product_id: String,
        carbon_footprint_g: i128,
        water_usage_ml: i128,
        renewable_energy_pct: u32,
        waste_recycled_pct: u32,
        labor_compliance_hash: BytesN<32>,
        certificate_hash: Option<BytesN<32>>,
    ) -> Result<(), Error> {
        caller.require_auth();

        validate_record(
            carbon_footprint_g,
            water_usage_ml,
            renewable_energy_pct,
            waste_recycled_pct,
        )?;

        // Prevent overwriting an already-verified record.
        if let Some(existing) = storage::get_sustainability(&env, &product_id) {
            if existing.status == SustainabilityStatus::Verified {
                return Err(Error::SustainabilityAlreadyVerified);
            }
        }

        let record = SustainabilityRecord {
            carbon_footprint_g,
            water_usage_ml,
            renewable_energy_pct,
            waste_recycled_pct,
            labor_compliance_hash,
            certificate_hash,
            status: SustainabilityStatus::Pending,
            recorded_at: env.ledger().timestamp(),
            recorded_by: caller.clone(),
            verified_by: None,
            verified_at: 0,
        };

        storage::put_sustainability(&env, &product_id, &record);

        env.events().publish(
            (Symbol::new(&env, "sustainability"), Symbol::new(&env, "recorded")),
            (product_id, caller),
        );

        Ok(())
    }

    /// Mark a pending sustainability record as verified (admin only).
    ///
    /// `certificate_hash` is the hash of the third-party verification document
    /// that is stored off-chain and anchored here for auditability.
    pub fn verify_sustainability(
        env: Env,
        admin: Address,
        product_id: String,
        certificate_hash: BytesN<32>,
    ) -> Result<(), Error> {
        require_admin(&env, &admin)?;

        let mut record = storage::get_sustainability(&env, &product_id)
            .ok_or(Error::SustainabilityNotFound)?;

        if record.status == SustainabilityStatus::Verified {
            return Err(Error::SustainabilityAlreadyVerified);
        }

        record.status = SustainabilityStatus::Verified;
        record.certificate_hash = Some(certificate_hash);
        record.verified_by = Some(admin.clone());
        record.verified_at = env.ledger().timestamp();

        storage::put_sustainability(&env, &product_id, &record);

        env.events().publish(
            (Symbol::new(&env, "sustainability"), Symbol::new(&env, "verified")),
            (product_id, admin),
        );

        Ok(())
    }

    /// Mark a pending sustainability record as rejected (admin only).
    pub fn reject_sustainability(
        env: Env,
        admin: Address,
        product_id: String,
    ) -> Result<(), Error> {
        require_admin(&env, &admin)?;

        let mut record = storage::get_sustainability(&env, &product_id)
            .ok_or(Error::SustainabilityNotFound)?;

        if record.status == SustainabilityStatus::Verified {
            return Err(Error::SustainabilityAlreadyVerified);
        }

        record.status = SustainabilityStatus::Rejected;
        record.verified_by = Some(admin.clone());
        record.verified_at = env.ledger().timestamp();

        storage::put_sustainability(&env, &product_id, &record);

        env.events().publish(
            (Symbol::new(&env, "sustainability"), Symbol::new(&env, "rejected")),
            (product_id, admin),
        );

        Ok(())
    }

    /// Retrieve the sustainability record for a product.
    pub fn get_sustainability(
        env: Env,
        product_id: String,
    ) -> Result<SustainabilityRecord, Error> {
        storage::get_sustainability(&env, &product_id)
            .ok_or(Error::SustainabilityNotFound)
    }
}

#[cfg(test)]
mod sustainability_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, BytesN, Env};

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, SustainabilityContract);
        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &admin);
        });
        let owner = Address::generate(&env);
        (env, admin, owner)
    }

    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    fn one_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[1u8; 32])
    }

    #[test]
    fn test_record_and_get() {
        let (env, _admin, owner) = setup();
        let contract_id = env.register_contract(None, SustainabilityContract);
        let client = SustainabilityContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &_admin);
        });

        let product_id = soroban_sdk::String::from_str(&env, "P001");

        client
            .record_sustainability(
                &owner,
                &product_id,
                &5000_i128,
                &10000_i128,
                &80_u32,
                &60_u32,
                &zero_hash(&env),
                &None,
            )
            .unwrap();

        let record = client.get_sustainability(&product_id).unwrap();
        assert_eq!(record.status, SustainabilityStatus::Pending);
        assert_eq!(record.carbon_footprint_g, 5000);
        assert_eq!(record.renewable_energy_pct, 80);
    }

    #[test]
    fn test_verify_sustainability() {
        let (env, admin, owner) = setup();
        let contract_id = env.register_contract(None, SustainabilityContract);
        let client = SustainabilityContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &admin);
        });

        let product_id = soroban_sdk::String::from_str(&env, "P002");

        client
            .record_sustainability(
                &owner,
                &product_id,
                &1000_i128,
                &500_i128,
                &100_u32,
                &100_u32,
                &zero_hash(&env),
                &None,
            )
            .unwrap();

        client
            .verify_sustainability(&admin, &product_id, &one_hash(&env))
            .unwrap();

        let record = client.get_sustainability(&product_id).unwrap();
        assert_eq!(record.status, SustainabilityStatus::Verified);
        assert_eq!(record.certificate_hash, Some(one_hash(&env)));
    }

    #[test]
    fn test_reject_sustainability() {
        let (env, admin, owner) = setup();
        let contract_id = env.register_contract(None, SustainabilityContract);
        let client = SustainabilityContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &admin);
        });

        let product_id = soroban_sdk::String::from_str(&env, "P003");

        client
            .record_sustainability(
                &owner,
                &product_id,
                &0_i128,
                &0_i128,
                &0_u32,
                &0_u32,
                &zero_hash(&env),
                &None,
            )
            .unwrap();

        client.reject_sustainability(&admin, &product_id).unwrap();

        let record = client.get_sustainability(&product_id).unwrap();
        assert_eq!(record.status, SustainabilityStatus::Rejected);
    }

    #[test]
    fn test_invalid_carbon_data() {
        let (env, _admin, owner) = setup();
        let contract_id = env.register_contract(None, SustainabilityContract);
        let client = SustainabilityContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &_admin);
        });

        let product_id = soroban_sdk::String::from_str(&env, "P004");

        let result = client.try_record_sustainability(
            &owner,
            &product_id,
            &(-1_i128),
            &0_i128,
            &0_u32,
            &0_u32,
            &zero_hash(&env),
            &None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_percentage() {
        let (env, _admin, owner) = setup();
        let contract_id = env.register_contract(None, SustainabilityContract);
        let client = SustainabilityContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &_admin);
        });

        let product_id = soroban_sdk::String::from_str(&env, "P005");

        let result = client.try_record_sustainability(
            &owner,
            &product_id,
            &0_i128,
            &0_i128,
            &101_u32,
            &0_u32,
            &zero_hash(&env),
            &None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_overwrite_verified() {
        let (env, admin, owner) = setup();
        let contract_id = env.register_contract(None, SustainabilityContract);
        let client = SustainabilityContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            storage::set_admin(&env, &admin);
        });

        let product_id = soroban_sdk::String::from_str(&env, "P006");

        client
            .record_sustainability(
                &owner,
                &product_id,
                &0_i128,
                &0_i128,
                &0_u32,
                &0_u32,
                &zero_hash(&env),
                &None,
            )
            .unwrap();

        client
            .verify_sustainability(&admin, &product_id, &one_hash(&env))
            .unwrap();

        // Attempting to re-record a verified entry must fail.
        let result = client.try_record_sustainability(
            &owner,
            &product_id,
            &0_i128,
            &0_i128,
            &0_u32,
            &0_u32,
            &zero_hash(&env),
            &None,
        );
        assert!(result.is_err());
    }
}

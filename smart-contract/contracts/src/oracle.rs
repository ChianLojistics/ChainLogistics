use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Vec};

use crate::error::Error;
use crate::types::{DataKey, OracleFeedConfig, OracleReport, OracleSnapshot, OracleSource};

const SOURCE_SLASH_PENALTY: i128 = 1;

fn has_admin(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Admin)
}

fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

fn load_timelock_contract(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::TimelockContract)
}

fn store_timelock_contract(env: &Env, timelock_contract: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TimelockContract, timelock_contract);
}

fn load_feed_config(env: &Env, feed_id: &Symbol) -> Option<OracleFeedConfig> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleFeedConfig(feed_id.clone()))
}

fn store_feed_config(env: &Env, config: &OracleFeedConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleFeedConfig(config.feed_id.clone()), config);
}

fn load_feed_sources(env: &Env, feed_id: &Symbol) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleFeedSources(feed_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

fn store_feed_sources(env: &Env, feed_id: &Symbol, reporters: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleFeedSources(feed_id.clone()), reporters);
}

fn load_source(env: &Env, feed_id: &Symbol, reporter: &Address) -> Option<OracleSource> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleSource(feed_id.clone(), reporter.clone()))
}

fn store_source(env: &Env, feed_id: &Symbol, source: &OracleSource) {
    env.storage().persistent().set(
        &DataKey::OracleSource(feed_id.clone(), source.reporter.clone()),
        source,
    );
}

fn load_report(env: &Env, feed_id: &Symbol, reporter: &Address) -> Option<OracleReport> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleReport(feed_id.clone(), reporter.clone()))
}

fn store_report(env: &Env, feed_id: &Symbol, report: &OracleReport) {
    env.storage().persistent().set(
        &DataKey::OracleReport(feed_id.clone(), report.reporter.clone()),
        report,
    );
}

fn load_snapshot(env: &Env, feed_id: &Symbol) -> Option<OracleSnapshot> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleSnapshot(feed_id.clone()))
}

fn store_snapshot(env: &Env, snapshot: &OracleSnapshot) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleSnapshot(snapshot.feed_id.clone()), snapshot);
}

fn load_fallback(env: &Env, feed_id: &Symbol) -> Option<OracleSnapshot> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleFallback(feed_id.clone()))
}

fn store_fallback(env: &Env, snapshot: &OracleSnapshot) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleFallback(snapshot.feed_id.clone()), snapshot);
}

fn load_circuit_broken(env: &Env, feed_id: &Symbol) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OracleCircuitBreaker(feed_id.clone()))
        .unwrap_or(false)
}

fn store_circuit_broken(env: &Env, feed_id: &Symbol, broken: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleCircuitBreaker(feed_id.clone()), &broken);
}

fn require_controller(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin = get_admin(env).ok_or(Error::NotInitialized)?;
    if &admin == caller {
        caller.require_auth();
        return Ok(());
    }

    if let Some(timelock) = load_timelock_contract(env) {
        if &timelock == caller {
            if env.current_contract_address() != timelock {
                caller.require_auth();
            }
            return Ok(());
        }
    }

    Err(Error::Unauthorized)
}

fn validate_feed_config(config: &OracleFeedConfig) -> Result<(), Error> {
    if config.min_value > config.max_value
        || config.max_age_seconds == 0
        || config.min_sources == 0
        || config.max_deviation_bps == 0
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

fn validate_report(
    env: &Env,
    config: &OracleFeedConfig,
    value: i128,
    observed_at: u64,
    proof_hash: &BytesN<32>,
) -> Result<(), Error> {
    if value < config.min_value || value > config.max_value {
        return Err(Error::OracleValueOutOfRange);
    }

    let now = env.ledger().timestamp();
    if observed_at > now || now.saturating_sub(observed_at) > config.max_age_seconds {
        return Err(Error::OracleReportStale);
    }

    if *proof_hash == BytesN::from_array(env, &[0; 32]) {
        return Err(Error::InvalidInput);
    }

    Ok(())
}

fn is_report_fresh(env: &Env, config: &OracleFeedConfig, report: &OracleReport) -> bool {
    let now = env.ledger().timestamp();
    report.observed_at <= now
        && now.saturating_sub(report.observed_at) <= config.max_age_seconds
        && report.value >= config.min_value
        && report.value <= config.max_value
}

fn insert_sorted(env: &Env, values: &mut Vec<i128>, value: i128) {
    let mut inserted = false;
    let mut sorted = Vec::new(env);
    for i in 0..values.len() {
        let existing = values.get_unchecked(i);
        if !inserted && value < existing {
            sorted.push_back(value);
            inserted = true;
        }
        sorted.push_back(existing);
    }

    if !inserted {
        sorted.push_back(value);
    }

    *values = sorted;
}

fn median(values: &Vec<i128>) -> i128 {
    let len = values.len();
    let mid = len / 2;
    if len % 2 == 1 {
        values.get_unchecked(mid)
    } else {
        let left = values.get_unchecked(mid - 1);
        let right = values.get_unchecked(mid);
        (left + right) / 2
    }
}

fn abs_i128(value: i128) -> i128 {
    if value < 0 {
        -value
    } else {
        value
    }
}

fn deviation_bps(value: i128, reference: i128) -> u32 {
    if reference == 0 {
        return if value == 0 { 0 } else { u32::MAX };
    }

    let numerator = abs_i128(value - reference).saturating_mul(10_000);
    let denominator = abs_i128(reference);
    (numerator / denominator) as u32
}

fn reward_source(env: &Env, feed_id: &Symbol, reporter: &Address) {
    if let Some(mut source) = load_source(env, feed_id, reporter) {
        source.reward_points += 1;
        store_source(env, feed_id, &source);
    }
}

fn slash_source(env: &Env, feed_id: &Symbol, reporter: &Address) {
    if let Some(mut source) = load_source(env, feed_id, reporter) {
        source.slash_count += 1;
        source.stake = if source.stake > SOURCE_SLASH_PENALTY {
            source.stake - SOURCE_SLASH_PENALTY
        } else {
            0
        };
        store_source(env, feed_id, &source);
    }
}

fn fallback_snapshot(
    env: &Env,
    feed_id: &Symbol,
    source_count: u32,
) -> Result<OracleSnapshot, Error> {
    store_circuit_broken(env, feed_id, true);

    if let Some(last_good) = load_snapshot(env, feed_id) {
        return Ok(OracleSnapshot {
            feed_id: feed_id.clone(),
            value: last_good.value,
            observed_at: last_good.observed_at,
            source_count,
            using_fallback: true,
            circuit_broken: true,
        });
    }

    if let Some(fallback) = load_fallback(env, feed_id) {
        return Ok(OracleSnapshot {
            feed_id: feed_id.clone(),
            value: fallback.value,
            observed_at: fallback.observed_at,
            source_count,
            using_fallback: true,
            circuit_broken: true,
        });
    }

    Err(Error::OracleFallbackUnavailable)
}

fn recompute_snapshot(env: &Env, feed_id: &Symbol) -> Result<OracleSnapshot, Error> {
    let config = load_feed_config(env, feed_id).ok_or(Error::OracleFeedNotConfigured)?;
    let reporters = load_feed_sources(env, feed_id);

    let mut sorted_values = Vec::new(env);
    let mut fresh_reports = Vec::new(env);

    for i in 0..reporters.len() {
        let reporter = reporters.get_unchecked(i);
        if let Some(report) = load_report(env, feed_id, &reporter) {
            if is_report_fresh(env, &config, &report) {
                insert_sorted(env, &mut sorted_values, report.value);
                fresh_reports.push_back(report);
            }
        }
    }

    if sorted_values.len() < config.min_sources {
        return fallback_snapshot(env, feed_id, sorted_values.len());
    }

    let reference = median(&sorted_values);
    let mut accepted_values = Vec::new(env);
    let mut accepted_count = 0u32;
    let mut observed_at = 0u64;

    for i in 0..fresh_reports.len() {
        let report = fresh_reports.get_unchecked(i);
        if deviation_bps(report.value, reference) <= config.max_deviation_bps {
            insert_sorted(env, &mut accepted_values, report.value);
            accepted_count += 1;
            observed_at = observed_at.max(report.observed_at);
            reward_source(env, feed_id, &report.reporter);
        } else {
            slash_source(env, feed_id, &report.reporter);
        }
    }

    if accepted_count < config.min_sources {
        return fallback_snapshot(env, feed_id, accepted_count);
    }

    let snapshot = OracleSnapshot {
        feed_id: feed_id.clone(),
        value: median(&accepted_values),
        observed_at,
        source_count: accepted_count,
        using_fallback: false,
        circuit_broken: false,
    };
    store_snapshot(env, &snapshot);
    store_circuit_broken(env, feed_id, false);
    Ok(snapshot)
}

#[contract]
pub struct OracleSecurityContract;

#[contractimpl]
impl OracleSecurityContract {
    pub fn oracle_init(env: Env, admin: Address) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        set_admin(&env, &admin);
        Ok(())
    }

    pub fn set_timelock_contract(
        env: Env,
        caller: Address,
        timelock_contract: Address,
    ) -> Result<(), Error> {
        require_controller(&env, &caller)?;
        store_timelock_contract(&env, &timelock_contract);
        Ok(())
    }

    pub fn configure_feed(
        env: Env,
        caller: Address,
        config: OracleFeedConfig,
    ) -> Result<(), Error> {
        require_controller(&env, &caller)?;
        validate_feed_config(&config)?;
        store_feed_config(&env, &config);
        env.events().publish(
            (
                Symbol::new(&env, "oracle_feed_configured"),
                config.feed_id.clone(),
            ),
            config,
        );
        Ok(())
    }

    pub fn register_source(
        env: Env,
        caller: Address,
        feed_id: Symbol,
        reporter: Address,
        stake: i128,
    ) -> Result<(), Error> {
        require_controller(&env, &caller)?;
        let _ = load_feed_config(&env, &feed_id).ok_or(Error::OracleFeedNotConfigured)?;
        if stake <= 0 {
            return Err(Error::OracleInvalidStake);
        }
        if load_source(&env, &feed_id, &reporter).is_some() {
            return Err(Error::DuplicateOracleSource);
        }

        let source = OracleSource {
            reporter: reporter.clone(),
            stake,
            active: true,
            reward_points: 0,
            slash_count: 0,
        };
        store_source(&env, &feed_id, &source);

        let mut reporters = load_feed_sources(&env, &feed_id);
        reporters.push_back(reporter.clone());
        store_feed_sources(&env, &feed_id, &reporters);

        env.events().publish(
            (
                Symbol::new(&env, "oracle_source_registered"),
                feed_id,
                reporter,
            ),
            stake,
        );
        Ok(())
    }

    pub fn set_fallback_value(
        env: Env,
        caller: Address,
        feed_id: Symbol,
        value: i128,
        observed_at: u64,
    ) -> Result<(), Error> {
        require_controller(&env, &caller)?;
        let config = load_feed_config(&env, &feed_id).ok_or(Error::OracleFeedNotConfigured)?;
        if value < config.min_value || value > config.max_value {
            return Err(Error::OracleValueOutOfRange);
        }

        let snapshot = OracleSnapshot {
            feed_id: feed_id.clone(),
            value,
            observed_at,
            source_count: 0,
            using_fallback: true,
            circuit_broken: false,
        };
        store_fallback(&env, &snapshot);
        Ok(())
    }

    pub fn submit_report(
        env: Env,
        reporter: Address,
        feed_id: Symbol,
        value: i128,
        observed_at: u64,
        proof_hash: BytesN<32>,
    ) -> Result<OracleSnapshot, Error> {
        reporter.require_auth();

        let config = load_feed_config(&env, &feed_id).ok_or(Error::OracleFeedNotConfigured)?;
        let source = load_source(&env, &feed_id, &reporter).ok_or(Error::OracleSourceNotFound)?;
        if !source.active {
            return Err(Error::OracleSourceNotFound);
        }

        validate_report(&env, &config, value, observed_at, &proof_hash)?;

        let report = OracleReport {
            reporter: reporter.clone(),
            value,
            observed_at,
            submitted_at: env.ledger().timestamp(),
            proof_hash,
        };
        store_report(&env, &feed_id, &report);

        let snapshot = recompute_snapshot(&env, &feed_id)?;
        env.events().publish(
            (
                Symbol::new(&env, "oracle_report_accepted"),
                feed_id,
                reporter,
            ),
            snapshot.clone(),
        );
        Ok(snapshot)
    }

    pub fn get_feed_config(env: Env, feed_id: Symbol) -> Result<OracleFeedConfig, Error> {
        load_feed_config(&env, &feed_id).ok_or(Error::OracleFeedNotConfigured)
    }

    pub fn get_feed_value(env: Env, feed_id: Symbol) -> Result<OracleSnapshot, Error> {
        let config = load_feed_config(&env, &feed_id).ok_or(Error::OracleFeedNotConfigured)?;

        if let Some(snapshot) = load_snapshot(&env, &feed_id) {
            let now = env.ledger().timestamp();
            if !snapshot.circuit_broken
                && now.saturating_sub(snapshot.observed_at) <= config.max_age_seconds
            {
                return Ok(snapshot);
            }
        }

        fallback_snapshot(&env, &feed_id, 0)
    }

    pub fn get_source(env: Env, feed_id: Symbol, reporter: Address) -> Result<OracleSource, Error> {
        load_source(&env, &feed_id, &reporter).ok_or(Error::OracleSourceNotFound)
    }

    pub fn is_circuit_broken(env: Env, feed_id: Symbol) -> bool {
        load_circuit_broken(&env, &feed_id)
    }

    pub fn clear_circuit_breaker(env: Env, caller: Address, feed_id: Symbol) -> Result<(), Error> {
        require_controller(&env, &caller)?;
        store_circuit_broken(&env, &feed_id, false);
        Ok(())
    }
}

#[cfg(test)]
mod test_oracle_security {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, Env};

    fn setup(env: &Env) -> (OracleSecurityContractClient, Address) {
        let contract_id = env.register_contract(None, OracleSecurityContract);
        let client = OracleSecurityContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.oracle_init(&admin);
        (client, admin)
    }

    fn temperature_config(env: &Env, feed_id: &Symbol) -> OracleFeedConfig {
        OracleFeedConfig {
            feed_id: feed_id.clone(),
            feed_type: crate::types::OracleFeedType::Temperature,
            min_value: -500,
            max_value: 5_000,
            max_age_seconds: 60,
            min_sources: 2,
            max_deviation_bps: 1_000,
        }
    }

    #[test]
    fn test_oracle_median_consensus() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(100);

        let (client, admin) = setup(&env);
        let feed_id = Symbol::new(&env, "temp");
        client.configure_feed(&admin, &temperature_config(&env, &feed_id));

        let source_a = Address::generate(&env);
        let source_b = Address::generate(&env);
        let source_c = Address::generate(&env);

        client.register_source(&admin, &feed_id, &source_a, &10);
        client.register_source(&admin, &feed_id, &source_b, &10);
        client.register_source(&admin, &feed_id, &source_c, &10);
        client.set_fallback_value(&admin, &feed_id, &400, &90);

        client.submit_report(
            &source_a,
            &feed_id,
            &390,
            &100,
            &BytesN::from_array(&env, &[1; 32]),
        );
        let snapshot = client.submit_report(
            &source_b,
            &feed_id,
            &400,
            &100,
            &BytesN::from_array(&env, &[2; 32]),
        );
        client.submit_report(
            &source_c,
            &feed_id,
            &410,
            &100,
            &BytesN::from_array(&env, &[3; 32]),
        );

        assert_eq!(snapshot.value, 395);
        assert_eq!(client.get_feed_value(&feed_id).value, 400);
        assert!(!client.is_circuit_broken(&feed_id));
        assert_eq!(client.get_source(&feed_id, &source_a).reward_points, 2);
    }

    #[test]
    fn test_oracle_slashes_outlier_source() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(200);

        let (client, admin) = setup(&env);
        let feed_id = Symbol::new(&env, "price");
        let mut config = temperature_config(&env, &feed_id);
        config.feed_type = crate::types::OracleFeedType::CommodityPrice;
        config.min_value = 0;
        config.max_value = 10_000;
        client.configure_feed(&admin, &config);
        client.set_fallback_value(&admin, &feed_id, &1_000, &190);

        let source_a = Address::generate(&env);
        let source_b = Address::generate(&env);
        let source_c = Address::generate(&env);
        client.register_source(&admin, &feed_id, &source_a, &10);
        client.register_source(&admin, &feed_id, &source_b, &10);
        client.register_source(&admin, &feed_id, &source_c, &10);

        client.submit_report(
            &source_a,
            &feed_id,
            &1_000,
            &200,
            &BytesN::from_array(&env, &[4; 32]),
        );
        let snapshot = client.submit_report(
            &source_b,
            &feed_id,
            &1_010,
            &200,
            &BytesN::from_array(&env, &[5; 32]),
        );
        client.submit_report(
            &source_c,
            &feed_id,
            &5_000,
            &200,
            &BytesN::from_array(&env, &[6; 32]),
        );

        assert_eq!(snapshot.value, 1_005);
        let outlier = client.get_source(&feed_id, &source_c);
        assert_eq!(outlier.slash_count, 1);
        assert_eq!(outlier.stake, 9);
        assert_eq!(client.get_feed_value(&feed_id).source_count, 2);
    }

    #[test]
    fn test_oracle_fallback_when_consensus_missing() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(300);

        let (client, admin) = setup(&env);
        let feed_id = Symbol::new(&env, "humid");
        client.configure_feed(&admin, &temperature_config(&env, &feed_id));
        client.set_fallback_value(&admin, &feed_id, &450, &250);

        let source_a = Address::generate(&env);
        client.register_source(&admin, &feed_id, &source_a, &10);

        let snapshot = client.submit_report(
            &source_a,
            &feed_id,
            &460,
            &300,
            &BytesN::from_array(&env, &[7; 32]),
        );

        assert!(snapshot.using_fallback);
        assert!(snapshot.circuit_broken);
        assert_eq!(snapshot.value, 450);
    }

    #[test]
    fn test_oracle_rejects_stale_reports() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(500);

        let (client, admin) = setup(&env);
        let feed_id = Symbol::new(&env, "stamp");
        client.configure_feed(&admin, &temperature_config(&env, &feed_id));

        let source = Address::generate(&env);
        client.register_source(&admin, &feed_id, &source, &10);

        let res = client.try_submit_report(
            &source,
            &feed_id,
            &400,
            &430,
            &BytesN::from_array(&env, &[8; 32]),
        );
        assert_eq!(res, Err(Ok(Error::OracleReportStale)));
    }
}

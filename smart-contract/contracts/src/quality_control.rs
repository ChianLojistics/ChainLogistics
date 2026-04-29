#![allow(unexpected_cfgs)]
#![allow(dead_code)]

use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Symbol, Vec};

use crate::error::Error;
use crate::storage;
use crate::types::{QualityCertification, QualityParameter, QualityReading};

/// Quality control contract for managing IoT sensor data,
/// quality certifications, and compliance tracking.
#[contract]
pub struct QualityControlContract;

#[contractimpl]
impl QualityControlContract {
    /// Initialize the quality control module for a product
    pub fn init_quality_control(env: Env, admin: Address, product_id: String) -> Result<(), Error> {
        admin.require_auth();

        // Verify product exists
        if storage::get_product(&env, &product_id).is_none() {
            return Err(Error::ProductNotFound);
        }

        // Initialize quality control storage
        storage::set_quality_control_enabled(&env, &product_id, true);
        storage::set_quality_control_admin(&env, &product_id, &admin);

        // Emit initialization event
        env.events().publish(
            (
                Symbol::new(&env, "quality_control_initialized"),
                product_id.clone(),
            ),
            admin,
        );

        Ok(())
    }

    /// Add a quality certification to a product (ISO, HACCP, etc.)
    pub fn add_quality_certification(
        env: Env,
        caller: Address,
        product_id: String,
        certification_type: String,
        issuer: String,
        certificate_id: String,
        valid_from: u64,
        valid_until: u64,
        metadata: String,
    ) -> Result<QualityCertification, Error> {
        caller.require_auth();

        // Verify caller is authorized
        Self::require_quality_control_auth(&env, &product_id, &caller)?;

        // Create certification record
        let certification = QualityCertification {
            certification_id: certificate_id.clone(),
            certification_type: certification_type.clone(),
            issuer: issuer.clone(),
            certificate_id: certificate_id.clone(),
            valid_from,
            valid_until,
            status: String::from_str(&env, "active"),
            metadata: metadata.clone(),
        };

        // Store certification
        storage::add_quality_certification(&env, &product_id, &certification);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "certification_added"), product_id.clone()),
            (certification_type, issuer, certificate_id),
        );

        Ok(certification)
    }

    /// Record IoT sensor reading for quality monitoring
    pub fn record_quality_reading(
        env: Env,
        caller: Address,
        product_id: String,
        sensor_id: String,
        parameter: String,
        value: i128,
        unit: String,
        location: String,
        threshold_min: Option<i128>,
        threshold_max: Option<i128>,
    ) -> Result<QualityReading, Error> {
        caller.require_auth();

        // Verify caller is authorized sensor or admin
        if !Self::is_authorized_sensor(&env, &product_id, &caller)
            && !Self::is_quality_control_admin(&env, &product_id, &caller)
        {
            return Err(Error::Unauthorized);
        }

        let timestamp = env.ledger().timestamp();

        // Determine status based on thresholds
        let status = Self::determine_reading_status(&env, value, threshold_min, threshold_max);

        // Create reading record
        let reading = QualityReading {
            reading_id: Self::generate_reading_id(&env, &product_id, &sensor_id, timestamp),
            product_id: product_id.clone(),
            sensor_id: sensor_id.clone(),
            parameter: parameter.clone(),
            value,
            unit: unit.clone(),
            timestamp,
            location: location.clone(),
            status: status.clone(),
            threshold_min,
            threshold_max,
        };

        // Store reading
        storage::add_quality_reading(&env, &product_id, &reading);

        // Update parameter statistics
        Self::update_parameter_stats(&env, &product_id, &parameter, value);

        // Emit event
        env.events().publish(
            (
                Symbol::new(&env, "quality_reading_recorded"),
                product_id.clone(),
            ),
            (sensor_id, parameter.clone(), value, unit, status.clone()),
        );

        // Trigger alert if out of range
        if status == String::from_str(&env, "critical") {
            env.events().publish(
                (
                    Symbol::new(&env, "quality_alert_triggered"),
                    product_id.clone(),
                ),
                (
                    parameter,
                    value,
                    String::from_str(&env, "threshold_exceeded"),
                ),
            );
        }

        Ok(reading)
    }

    /// Get quality readings for a product with optional filtering
    pub fn get_quality_readings(
        env: Env,
        product_id: String,
        parameter: Option<String>,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
        limit: u32,
    ) -> Result<Vec<QualityReading>, Error> {
        if limit > 100 {
            return Err(Error::InvalidLimit);
        }

        let readings = storage::get_quality_readings(&env, &product_id);
        let mut filtered = Vec::new(&env);

        for reading in readings.iter() {
            if let Some(ref param) = parameter {
                if reading.parameter != *param {
                    continue;
                }
            }

            if let Some(from) = from_timestamp {
                if reading.timestamp < from {
                    continue;
                }
            }

            if let Some(to) = to_timestamp {
                if reading.timestamp > to {
                    continue;
                }
            }

            filtered.push_back(reading);

            if filtered.len() >= limit {
                break;
            }
        }

        Ok(filtered)
    }

    /// Get active quality certifications for a product
    pub fn get_quality_certifications(
        env: Env,
        product_id: String,
    ) -> Result<Vec<QualityCertification>, Error> {
        let certifications = storage::get_quality_certifications(&env, &product_id);
        let now = env.ledger().timestamp();
        let mut active = Vec::new(&env);

        for cert in certifications.iter() {
            if cert.valid_from <= now && cert.valid_until >= now {
                active.push_back(cert);
            }
        }

        Ok(active)
    }

    /// Define quality parameters to monitor for a product
    pub fn define_quality_parameters(
        env: Env,
        caller: Address,
        product_id: String,
        parameters: Vec<QualityParameter>,
    ) -> Result<(), Error> {
        caller.require_auth();

        Self::require_quality_control_auth(&env, &product_id, &caller)?;

        storage::set_quality_parameters(&env, &product_id, &parameters);

        env.events().publish(
            (
                Symbol::new(&env, "quality_parameters_defined"),
                product_id.clone(),
            ),
            parameters.len(),
        );

        Ok(())
    }

    /// Get defined quality parameters for a product
    pub fn get_quality_parameters(
        env: Env,
        product_id: String,
    ) -> Result<Vec<QualityParameter>, Error> {
        Ok(storage::get_quality_parameters(&env, &product_id))
    }

    /// Authorize a sensor to submit readings for a product
    pub fn authorize_sensor(
        env: Env,
        admin: Address,
        product_id: String,
        sensor_address: Address,
        sensor_id: String,
        sensor_type: String,
    ) -> Result<(), Error> {
        admin.require_auth();

        if !Self::is_quality_control_admin(&env, &product_id, &admin) {
            return Err(Error::Unauthorized);
        }

        storage::authorize_sensor(&env, &product_id, &sensor_address, &sensor_id, &sensor_type);

        env.events().publish(
            (Symbol::new(&env, "sensor_authorized"), product_id.clone()),
            (sensor_address, sensor_id, sensor_type),
        );

        Ok(())
    }

    /// Revoke sensor authorization
    pub fn revoke_sensor_authorization(
        env: Env,
        admin: Address,
        product_id: String,
        sensor_address: Address,
    ) -> Result<(), Error> {
        admin.require_auth();

        if !Self::is_quality_control_admin(&env, &product_id, &admin) {
            return Err(Error::Unauthorized);
        }

        storage::revoke_sensor_authorization(&env, &product_id, &sensor_address);

        env.events().publish(
            (Symbol::new(&env, "sensor_revoked"), product_id.clone()),
            sensor_address,
        );

        Ok(())
    }

    /// Get quality summary statistics for a product
    pub fn get_quality_summary(env: Env, product_id: String) -> Result<Map<String, String>, Error> {
        let mut summary = Map::new(&env);
        let readings = storage::get_quality_readings(&env, &product_id);

        let total_readings = readings.len();
        let mut critical_count = 0u32;
        let mut warning_count = 0u32;
        let mut normal_count = 0u32;

        for reading in readings.iter() {
            if reading.status == String::from_str(&env, "critical") {
                critical_count += 1;
            } else if reading.status == String::from_str(&env, "warning") {
                warning_count += 1;
            } else {
                normal_count += 1;
            }
        }

        summary.set(
            String::from_str(&env, "total_readings"),
            format_count(&env, total_readings),
        );
        summary.set(
            String::from_str(&env, "critical_count"),
            format_count(&env, critical_count),
        );
        summary.set(
            String::from_str(&env, "warning_count"),
            format_count(&env, warning_count),
        );
        summary.set(
            String::from_str(&env, "normal_count"),
            format_count(&env, normal_count),
        );

        let certifications = storage::get_quality_certifications(&env, &product_id);
        summary.set(
            String::from_str(&env, "certification_count"),
            format_count(&env, certifications.len()),
        );

        Ok(summary)
    }

    /// Internal helper: Check if caller is quality control admin
    fn is_quality_control_admin(env: &Env, product_id: &String, caller: &Address) -> bool {
        if let Some(admin) = storage::get_quality_control_admin(env, product_id) {
            &admin == caller
        } else {
            false
        }
    }

    /// Internal helper: Check if address is authorized sensor
    fn is_authorized_sensor(env: &Env, product_id: &String, address: &Address) -> bool {
        storage::is_authorized_sensor(env, product_id, address)
    }

    /// Internal helper: Require quality control authorization
    fn require_quality_control_auth(
        env: &Env,
        product_id: &String,
        caller: &Address,
    ) -> Result<(), Error> {
        if Self::is_quality_control_admin(env, product_id, caller) {
            return Ok(());
        }

        // Also allow product owner
        if let Some(product) = storage::get_product(env, product_id) {
            if &product.owner == caller {
                return Ok(());
            }
        }

        Err(Error::Unauthorized)
    }

    /// Internal helper: Generate unique reading ID
    fn generate_reading_id(
        env: &Env,
        product_id: &String,
        sensor_id: &String,
        timestamp: u64,
    ) -> String {
        let ts_str = Self::u64_to_string(env, timestamp);
        let pid_len = product_id.len() as usize;
        let sid_len = sensor_id.len() as usize;
        let ts_len = ts_str.len() as usize;
        // product_id + ":" + sensor_id + ":" + timestamp digits
        let total = pid_len + 1 + sid_len + 1 + ts_len;
        let mut buf = [0u8; 512];
        let mut pos = 0usize;
        product_id.copy_into_slice(&mut buf[pos..pos + pid_len]);
        pos += pid_len;
        buf[pos] = b':';
        pos += 1;
        sensor_id.copy_into_slice(&mut buf[pos..pos + sid_len]);
        pos += sid_len;
        buf[pos] = b':';
        pos += 1;
        ts_str.copy_into_slice(&mut buf[pos..pos + ts_len]);
        pos += ts_len;
        let _ = total; // pos == total
        String::from_bytes(env, &buf[..pos])
    }

    /// Convert u64 to String
    fn u64_to_string(env: &Env, n: u64) -> String {
        if n == 0 {
            return String::from_str(env, "0");
        }
        let mut buf = [0u8; 20];
        let mut pos = 20usize;
        let mut num = n;
        while num > 0 {
            pos -= 1;
            buf[pos] = b'0' + (num % 10) as u8;
            num /= 10;
        }
        String::from_bytes(env, &buf[pos..])
    }

    /// Internal helper: Determine reading status based on thresholds
    fn determine_reading_status(
        env: &Env,
        value: i128,
        threshold_min: Option<i128>,
        threshold_max: Option<i128>,
    ) -> String {
        let mut is_warning = false;
        let mut is_critical = false;

        if let Some(min) = threshold_min {
            if value < min {
                // Check if critically low (20% below threshold)
                if value < min - (min / 5) {
                    is_critical = true;
                } else {
                    is_warning = true;
                }
            }
        }

        if let Some(max) = threshold_max {
            if value > max {
                // Check if critically high (20% above threshold)
                if value > max + (max / 5) {
                    is_critical = true;
                } else {
                    is_warning = true;
                }
            }
        }

        if is_critical {
            String::from_str(env, "critical")
        } else if is_warning {
            String::from_str(env, "warning")
        } else {
            String::from_str(env, "normal")
        }
    }

    /// Internal helper: Update parameter statistics
    fn update_parameter_stats(env: &Env, product_id: &String, parameter: &String, value: i128) {
        let mut stats = storage::get_parameter_stats(env, product_id, parameter);

        stats.count += 1;
        stats.sum += value;

        if stats.count == 1 || value < stats.min {
            stats.min = value;
        }

        if stats.count == 1 || value > stats.max {
            stats.max = value;
        }

        stats.avg = stats.sum / stats.count as i128;
        stats.last_reading = value;
        stats.last_timestamp = env.ledger().timestamp();

        storage::set_parameter_stats(env, product_id, parameter, &stats);
    }
}

/// Format a count as a string
fn format_count(env: &Env, count: u32) -> String {
    if count == 0 {
        return String::from_str(env, "0");
    }
    let mut buf = [0u8; 10];
    let mut pos = 10usize;
    let mut n = count;
    while n > 0 {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    String::from_bytes(env, &buf[pos..])
}

// Quality control module for IoT sensor integration and quality certification tracking

use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::storage_contract::StorageContract;
use crate::types::{
    ParameterStats, Product, QualityCertification, QualityDataKey, QualityParameter,
    QualityReading, SensorInfo, TrackingEvent,
};

pub fn get_auth_contract(env: &Env) -> Option<Address> {
    StorageContract::get_auth_contract(env)
}

pub fn set_auth_contract(env: &Env, address: &Address) {
    StorageContract::set_auth_contract(env, address)
}

pub fn get_multisig_contract(env: &Env) -> Option<Address> {
    StorageContract::get_multisig_contract(env)
}

pub fn set_multisig_contract(env: &Env, address: &Address) {
    StorageContract::set_multisig_contract(env, address)
}

pub fn get_timelock_contract(env: &Env) -> Option<Address> {
    StorageContract::get_timelock_contract(env)
}

pub fn set_timelock_contract(env: &Env, address: &Address) {
    StorageContract::set_timelock_contract(env, address)
}

// ─── Product ────────────────────────────────────────────────────────────────

pub fn has_product(env: &Env, product_id: &String) -> bool {
    StorageContract::has_product(env, product_id)
}

pub fn put_product(env: &Env, product: &Product) {
    StorageContract::put_product(env, product)
}

pub fn get_product(env: &Env, product_id: &String) -> Option<Product> {
    StorageContract::get_product(env, product_id)
}

// ─── Event IDs per product ───────────────────────────────────────────────────

pub fn put_product_event_ids(env: &Env, product_id: &String, ids: &Vec<u64>) {
    StorageContract::put_product_event_ids(env, product_id, ids)
}

pub fn get_product_event_ids(env: &Env, product_id: &String) -> Vec<u64> {
    StorageContract::get_product_event_ids(env, product_id)
}

pub fn get_product_event_ids_paginated(
    env: &Env,
    product_id: &String,
    offset: u64,
    limit: u64,
) -> Vec<u64> {
    StorageContract::get_product_event_ids_paginated(env, product_id, offset, limit)
}

// ─── Events ─────────────────────────────────────────────────────────────────

pub fn put_event(env: &Env, event: &TrackingEvent) {
    StorageContract::put_event(env, event)
}

pub fn get_event(env: &Env, event_id: u64) -> Option<TrackingEvent> {
    StorageContract::get_event(env, event_id)
}

pub fn next_event_id(env: &Env) -> Result<u64, Error> {
    StorageContract::next_event_id(env)
}

// ─── Event type index ────────────────────────────────────────────────────────

pub fn index_event_by_type(
    env: &Env,
    product_id: &String,
    event_type: &Symbol,
    event_id: u64,
) -> Result<(), Error> {
    StorageContract::index_event_by_type(env, product_id, event_type, event_id)
}

pub fn acquire_reentrancy_lock(env: &Env, scope: &Symbol) -> Result<(), Error> {
    StorageContract::acquire_reentrancy_lock(env, scope)
}

pub fn release_reentrancy_lock(env: &Env, scope: &Symbol) {
    StorageContract::release_reentrancy_lock(env, scope)
}

pub fn get_event_ids_by_type(
    env: &Env,
    product_id: &String,
    event_type: &Symbol,
    offset: u64,
    limit: u64,
) -> Vec<u64> {
    StorageContract::get_event_ids_by_type(env, product_id, event_type, offset, limit)
}

pub fn get_event_count_by_type(env: &Env, product_id: &String, event_type: &Symbol) -> u64 {
    StorageContract::get_event_count_by_type(env, product_id, event_type)
}

// ─── Authorization ───────────────────────────────────────────────────────────

pub fn set_auth(env: &Env, product_id: &String, actor: &Address, value: bool) {
    StorageContract::set_auth(env, product_id, actor, value)
}

pub fn is_authorized(env: &Env, product_id: &String, actor: &Address) -> bool {
    StorageContract::is_authorized(env, product_id, actor)
}

// ─── Global Management ───────────────────────────────────────────────────────

pub fn has_admin(env: &Env) -> bool {
    StorageContract::has_admin(env)
}

pub fn get_admin(env: &Env) -> Option<Address> {
    StorageContract::get_admin(env)
}

pub fn set_admin(env: &Env, admin: &Address) {
    StorageContract::set_admin(env, admin)
}

pub fn is_paused(env: &Env) -> bool {
    StorageContract::is_paused(env)
}

pub fn set_paused(env: &Env, paused: bool) {
    StorageContract::set_paused(env, paused)
}

// ─── Global counters ─────────────────────────────────────────────────────────

pub fn get_total_products(env: &Env) -> u64 {
    StorageContract::get_total_products(env)
}

pub fn set_total_products(env: &Env, count: u64) {
    StorageContract::set_total_products(env, count)
}

pub fn get_active_products(env: &Env) -> u64 {
    StorageContract::get_active_products(env)
}

pub fn set_active_products(env: &Env, count: u64) {
    StorageContract::set_active_products(env, count)
}

// ─── Search Index ───────────────────────────────────────────────────────────

pub fn put_search_index(env: &Env, keyword: &String, product_ids: &Vec<String>) {
    env.storage().persistent().set(
        &crate::types::DataKey::SearchIndex(crate::types::IndexKey::Keyword(keyword.clone())),
        product_ids,
    );
}

pub fn get_search_index(env: &Env, keyword: &String) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&crate::types::DataKey::SearchIndex(
            crate::types::IndexKey::Keyword(keyword.clone()),
        ))
        .unwrap_or_else(|| Vec::new(env))
}

// Gas-optimized: Check existence before loading full vector
pub fn add_to_search_index(env: &Env, keyword: String, product_id: &String) {
    let mut ids = get_search_index(env, &keyword);
    // Early exit if already indexed - saves gas
    if ids.contains(product_id) {
        return;
    }
    ids.push_back(product_id.clone());
    put_search_index(env, &keyword, &ids);
}

// Gas-optimized: Use iterator pattern and early exit
pub fn remove_from_search_index(env: &Env, keyword: String, product_id: &String) {
    let mut ids = get_search_index(env, &keyword);
    // Find and remove in single pass
    for i in 0..ids.len() {
        if ids.get(i).unwrap() == product_id.clone() {
            ids.remove(i);
            put_search_index(env, &keyword, &ids);
            return; // Early exit saves gas
        }
    }
    // No write if not found - saves gas
}

// ─── Quality Control ────────────────────────────────────────────────────────

pub fn set_quality_control_enabled(env: &Env, product_id: &String, enabled: bool) {
    env.storage().persistent().set(
        &QualityDataKey::QualityControlEnabled(product_id.clone()),
        &enabled,
    );
}

pub fn is_quality_control_enabled(env: &Env, product_id: &String) -> bool {
    env.storage()
        .persistent()
        .get(&QualityDataKey::QualityControlEnabled(product_id.clone()))
        .unwrap_or(false)
}

pub fn set_quality_control_admin(env: &Env, product_id: &String, admin: &Address) {
    env.storage().persistent().set(
        &QualityDataKey::QualityControlAdmin(product_id.clone()),
        admin,
    );
}

pub fn get_quality_control_admin(env: &Env, product_id: &String) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&QualityDataKey::QualityControlAdmin(product_id.clone()))
}

pub fn add_quality_certification(
    env: &Env,
    product_id: &String,
    certification: &QualityCertification,
) {
    let key = QualityDataKey::QualityCertification(
        product_id.clone(),
        certification.certification_id.clone(),
    );
    env.storage().persistent().set(&key, certification);

    // Add to certifications list
    let list_key = QualityDataKey::QualityCertifications(product_id.clone());
    let mut certs: Vec<QualityCertification> = env
        .storage()
        .persistent()
        .get(&list_key)
        .unwrap_or_else(|| Vec::new(env));
    certs.push_back(certification.clone());
    env.storage().persistent().set(&list_key, &certs);
}

pub fn get_quality_certifications(env: &Env, product_id: &String) -> Vec<QualityCertification> {
    env.storage()
        .persistent()
        .get(&QualityDataKey::QualityCertifications(product_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_quality_reading(env: &Env, product_id: &String, reading: &QualityReading) {
    let key = QualityDataKey::QualityReading(product_id.clone(), reading.reading_id.clone());
    env.storage().persistent().set(&key, reading);

    // Add to readings list
    let list_key = QualityDataKey::QualityReadings(product_id.clone());
    let mut readings: Vec<QualityReading> = env
        .storage()
        .persistent()
        .get(&list_key)
        .unwrap_or_else(|| Vec::new(env));
    readings.push_back(reading.clone());
    env.storage().persistent().set(&list_key, &readings);
}

pub fn get_quality_readings(env: &Env, product_id: &String) -> Vec<QualityReading> {
    env.storage()
        .persistent()
        .get(&QualityDataKey::QualityReadings(product_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_quality_parameters(env: &Env, product_id: &String, parameters: &Vec<QualityParameter>) {
    env.storage().persistent().set(
        &QualityDataKey::QualityParameters(product_id.clone()),
        parameters,
    );
}

pub fn get_quality_parameters(env: &Env, product_id: &String) -> Vec<QualityParameter> {
    env.storage()
        .persistent()
        .get(&QualityDataKey::QualityParameters(product_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_parameter_stats(
    env: &Env,
    product_id: &String,
    parameter: &String,
    stats: &ParameterStats,
) {
    env.storage().persistent().set(
        &QualityDataKey::ParameterStats(product_id.clone(), parameter.clone()),
        stats,
    );
}

pub fn get_parameter_stats(env: &Env, product_id: &String, parameter: &String) -> ParameterStats {
    env.storage()
        .persistent()
        .get(&QualityDataKey::ParameterStats(
            product_id.clone(),
            parameter.clone(),
        ))
        .unwrap_or(ParameterStats {
            count: 0,
            sum: 0,
            min: 0,
            max: 0,
            avg: 0,
            last_reading: 0,
            last_timestamp: 0,
        })
}

pub fn authorize_sensor(
    env: &Env,
    product_id: &String,
    sensor_address: &Address,
    sensor_id: &String,
    sensor_type: &String,
) {
    let key = QualityDataKey::AuthorizedSensor(product_id.clone(), sensor_address.clone());
    let info = SensorInfo {
        address: sensor_address.clone(),
        sensor_id: sensor_id.clone(),
        sensor_type: sensor_type.clone(),
        authorized: true,
    };
    env.storage().persistent().set(&key, &info);

    // Add to authorized sensors list
    let list_key = QualityDataKey::AuthorizedSensors(product_id.clone());
    let mut sensors: Vec<Address> = env
        .storage()
        .persistent()
        .get(&list_key)
        .unwrap_or_else(|| Vec::new(env));
    if !sensors.contains(sensor_address) {
        sensors.push_back(sensor_address.clone());
        env.storage().persistent().set(&list_key, &sensors);
    }
}

pub fn revoke_sensor_authorization(env: &Env, product_id: &String, sensor_address: &Address) {
    let key = QualityDataKey::AuthorizedSensor(product_id.clone(), sensor_address.clone());
    if let Some(mut info) = env
        .storage()
        .persistent()
        .get::<QualityDataKey, SensorInfo>(&key)
    {
        info.authorized = false;
        env.storage().persistent().set(&key, &info);
    }
}

pub fn is_authorized_sensor(env: &Env, product_id: &String, sensor_address: &Address) -> bool {
    let key = QualityDataKey::AuthorizedSensor(product_id.clone(), sensor_address.clone());
    env.storage()
        .persistent()
        .get::<QualityDataKey, SensorInfo>(&key)
        .map(|info| info.authorized)
        .unwrap_or(false)
}

use crate::compliance::ComplianceProver;
use crate::error::AppError;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ProofRequest {
    pub temperature: u32,
    pub speed: u32,
    pub temp_threshold: u32,
    pub speed_threshold: u32,
}

#[derive(Serialize)]
pub struct ProofResponse {
    pub proof_a: String, // Hex encoded
    pub proof_b: String,
    pub proof_c: String,
    pub public_inputs: Vec<String>,
}

pub async fn generate_compliance_proof(
    // State(state): State<AppState>, // We'll need a prover in AppState
    Json(payload): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, AppError> {
    // For this demonstration, we'll initialize a prover on the fly
    // In production, the proving key should be loaded once at startup
    let (pk, _vk) = ComplianceProver::setup();
    let prover = ComplianceProver { proving_key: pk };

    let (proof_bytes, public_inputs_bytes) = prover.generate_proof(
        payload.temperature,
        payload.speed,
        payload.temp_threshold,
        payload.speed_threshold,
    );

    let (a, b, c) = crate::compliance::serialize_to_soroban_format(&proof_bytes);

    // We expect 2 public inputs of 32 bytes each
    let mut inputs = Vec::new();
    inputs.push(hex::encode(&public_inputs_bytes[0..32]));
    inputs.push(hex::encode(&public_inputs_bytes[32..64]));

    Ok(Json(ProofResponse {
        proof_a: hex::encode(a),
        proof_b: hex::encode(b),
        proof_c: hex::encode(c),
        public_inputs: inputs,
    }))
}

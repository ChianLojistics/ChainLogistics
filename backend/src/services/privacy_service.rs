use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockZkProof {
    pub proof_hash: String,
    pub public_inputs_hash: String,
}

pub struct PrivacyService {
    encryption_key: [u8; 32],
}

impl PrivacyService {
    pub fn new(key_hex: &str) -> Result<Self, anyhow::Error> {
        let key_bytes = hex::decode(key_hex).unwrap_or_else(|_| vec![0; 32]);
        let mut key = [0u8; 32];
        if key_bytes.len() == 32 {
            key.copy_from_slice(&key_bytes);
        } else {
            // fallback key for development if invalid hex provided
            let default_key = b"0123456789abcdef0123456789abcdef";
            key.copy_from_slice(default_key);
        }
        Ok(Self { encryption_key: key })
    }

    /// Encrypts sensitive data using AES-256-GCM
    pub fn encrypt_data(&self, plaintext: &str) -> Result<String, anyhow::Error> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes().as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption error: {}", e))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);
        
        Ok(general_purpose::STANDARD.encode(combined))
    }

    /// Decrypts sensitive data using AES-256-GCM
    pub fn decrypt_data(&self, encrypted_base64: &str) -> Result<String, anyhow::Error> {
        let combined = general_purpose::STANDARD.decode(encrypted_base64)
            .map_err(|e| anyhow::anyhow!("Base64 decode error: {}", e))?;

        if combined.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }

        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];

        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let plaintext_bytes = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption error: {}", e))?;

        Ok(String::from_utf8(plaintext_bytes)?)
    }

    /// Generates a mock ZK proof by hashing the data with a salt.
    /// In a real system, this would be a Circom/Groth16 proof generation.
    pub fn generate_mock_proof(&self, sensitive_data: &str, public_data: &str) -> MockZkProof {
        let mut hasher = Sha256::new();
        hasher.update(sensitive_data.as_bytes());
        let secret_hash = hex::encode(hasher.finalize());

        let mut pub_hasher = Sha256::new();
        pub_hasher.update(public_data.as_bytes());
        let pub_hash = hex::encode(pub_hasher.finalize());

        // The "proof" binds the public inputs to the knowledge of the secret
        let mut proof_hasher = Sha256::new();
        proof_hasher.update(secret_hash.as_bytes());
        proof_hasher.update(pub_hash.as_bytes());
        let proof_hash = hex::encode(proof_hasher.finalize());

        MockZkProof {
            proof_hash,
            public_inputs_hash: pub_hash,
        }
    }

    /// Verifies a mock ZK proof.
    /// In a real system, this would be on-chain or off-chain SNARK verification.
    pub fn verify_mock_proof(&self, proof: &MockZkProof, public_data: &str) -> bool {
        let mut pub_hasher = Sha256::new();
        pub_hasher.update(public_data.as_bytes());
        let expected_pub_hash = hex::encode(pub_hasher.finalize());

        proof.public_inputs_hash == expected_pub_hash
    }

    /// Checks if a user has access based on selective disclosure rules
    pub fn check_selective_disclosure(
        disclosure_rules: &serde_json::Value,
        requester_address: &str,
        requester_role: &str,
    ) -> bool {
        if disclosure_rules.is_null() {
            return true; // No rules, default to public
        }

        if let Some(allowed_addresses) = disclosure_rules.get("allowed_addresses").and_then(|v| v.as_array()) {
            if allowed_addresses.iter().any(|v| v.as_str() == Some(requester_address)) {
                return true;
            }
        }

        if let Some(allowed_roles) = disclosure_rules.get("allowed_roles").and_then(|v| v.as_array()) {
            if allowed_roles.iter().any(|v| v.as_str() == Some(requester_role)) {
                return true;
            }
        }

        false
    }
}

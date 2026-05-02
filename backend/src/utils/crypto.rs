use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key
};
use base64::{engine::general_purpose, Engine as _};
use crate::error::AppError;

pub fn encrypt(data: &str, key_str: &str) -> Result<String, AppError> {
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce 12"); // In production, use a random nonce and prepend it to the ciphertext

    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| AppError::Internal(format!("Encryption error: {}", e)))?;

    Ok(general_purpose::STANDARD.encode(ciphertext))
}

pub fn decrypt(encrypted_data: &str, key_str: &str) -> Result<String, AppError> {
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce 12"); // Must match the nonce used for encryption

    let ciphertext = general_purpose::STANDARD
        .decode(encrypted_data)
        .map_err(|e| AppError::Internal(format!("Base64 decode error: {}", e)))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|e| AppError::Internal(format!("Decryption error: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("UTF-8 decode error: {}", e)))
}

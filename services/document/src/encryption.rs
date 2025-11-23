use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
};
use common::error::AppError;
use rand::Rng;

// Use a constant key for demo purposes. In production, this should be managed via KMS or env vars.
const MASTER_KEY: &[u8; 32] = b"01234567890123456789012345678901";

pub fn encrypt(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let key = Key::<Aes256Gcm>::from_slice(MASTER_KEY);
    let cipher = Aes256Gcm::new(key);

    let mut rng = rand::thread_rng();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| AppError::Encryption(e.to_string()))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt(ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    let key = Key::<Aes256Gcm>::from_slice(MASTER_KEY);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Encryption(e.to_string()))?;

    Ok(plaintext)
}

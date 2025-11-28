use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use common::error::AppError;
use rand::Rng;
use std::sync::OnceLock;

/// Encryption key manager - loads key from environment on first use
static ENCRYPTION_KEY: OnceLock<[u8; 32]> = OnceLock::new();

fn get_master_key() -> &'static [u8; 32] {
    ENCRYPTION_KEY.get_or_init(|| {
        let key_hex = std::env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| {
                tracing::warn!("ENCRYPTION_KEY not set, using default key (NOT FOR PRODUCTION)");
                // Default key for development only - 32 bytes = 64 hex chars
                "3031323334353637383930313233343536373839303132333435363738393031".to_string()
            });
        
        let key_bytes = hex::decode(&key_hex)
            .expect("ENCRYPTION_KEY must be valid hex (64 characters for 32 bytes)");
        
        if key_bytes.len() != 32 {
            panic!("ENCRYPTION_KEY must be exactly 32 bytes (64 hex characters)");
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        key_array
    })
}

pub fn encrypt(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let master_key = get_master_key();
    let key = Key::<Aes256Gcm>::from_slice(master_key);
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
    let master_key = get_master_key();
    let key = Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Encryption(e.to_string()))?;

    Ok(plaintext)
}

/// Validate encryption key is properly configured (call at startup)
pub fn validate_encryption_config() -> Result<(), AppError> {
    let _ = get_master_key();
    Ok(())
}

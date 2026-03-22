//! AES-256-GCM encryption with Argon2id key derivation.
//! Provides encrypt/decrypt for vault seed storage.

/// Derive an encryption key from a passphrase using Argon2id.
pub fn derive_key(_passphrase: &str) -> Result<[u8; 32], EncryptionError> {
    // TODO: Implement Argon2id KDF
    Err(EncryptionError::NotImplemented)
}

/// Encrypt plaintext using AES-256-GCM.
pub fn encrypt(_key: &[u8; 32], _plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    // TODO: Implement AES-256-GCM encryption
    Err(EncryptionError::NotImplemented)
}

/// Decrypt ciphertext using AES-256-GCM.
pub fn decrypt(_key: &[u8; 32], _ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    // TODO: Implement AES-256-GCM decryption
    Err(EncryptionError::NotImplemented)
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("encryption operation not yet implemented")]
    NotImplemented,
    #[error("decryption failed: invalid key or corrupted data")]
    DecryptionFailed,
    #[error("key derivation failed")]
    KeyDerivationFailed,
}

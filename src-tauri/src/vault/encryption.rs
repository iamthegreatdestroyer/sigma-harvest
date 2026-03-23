//! AES-256-GCM encryption with Argon2id key derivation.
//! Provides encrypt/decrypt for vault seed storage.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore, Nonce,
};
use argon2::{Argon2, Algorithm, Params, Version};
use zeroize::Zeroize;

/// Argon2id parameters tuned for security on desktop hardware.
/// These values balance security against ~500ms derivation time.
const ARGON2_M_COST: u32 = 65536; // 64 MiB memory
const ARGON2_T_COST: u32 = 3; // 3 iterations
const ARGON2_P_COST: u32 = 1; // 1 lane (single-threaded)

/// Salt length for Argon2id key derivation.
const SALT_LEN: usize = 16;

/// Nonce length for AES-256-GCM (96 bits).
const NONCE_LEN: usize = 12;

/// Derive a 256-bit encryption key from a passphrase using Argon2id.
/// Returns a 32-byte key suitable for AES-256-GCM.
pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], EncryptionError> {
    if salt.len() < SALT_LEN {
        return Err(EncryptionError::InvalidSalt);
    }

    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|_| EncryptionError::KeyDerivationFailed)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| EncryptionError::KeyDerivationFailed)?;

    Ok(key)
}

/// Generate a cryptographically secure random salt.
pub fn generate_salt() -> [u8; SALT_LEN] {
    use rand::RngCore;
    let mut salt = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypt plaintext using AES-256-GCM.
/// Returns: salt (16) || nonce (12) || ciphertext (variable) || tag (16).
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    // Prepend nonce to ciphertext for storage
    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(nonce.as_slice());
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt ciphertext produced by `encrypt`.
/// Expects: nonce (12) || ciphertext+tag (variable).
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    if data.len() < NONCE_LEN + 16 {
        return Err(EncryptionError::DecryptionFailed);
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| EncryptionError::DecryptionFailed)?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| EncryptionError::DecryptionFailed)
}

/// Convenience: encrypt plaintext with a passphrase (generates salt internally).
/// Returns: salt (16) || nonce (12) || ciphertext+tag.
pub fn encrypt_with_passphrase(
    passphrase: &str,
    plaintext: &[u8],
) -> Result<Vec<u8>, EncryptionError> {
    let salt = generate_salt();
    let mut key = derive_key(passphrase, &salt)?;

    let encrypted = encrypt(&key, plaintext)?;
    key.zeroize();

    let mut output = Vec::with_capacity(SALT_LEN + encrypted.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&encrypted);
    Ok(output)
}

/// Convenience: decrypt data produced by `encrypt_with_passphrase`.
/// Expects: salt (16) || nonce (12) || ciphertext+tag.
pub fn decrypt_with_passphrase(
    passphrase: &str,
    data: &[u8],
) -> Result<Vec<u8>, EncryptionError> {
    if data.len() < SALT_LEN + NONCE_LEN + 16 {
        return Err(EncryptionError::DecryptionFailed);
    }

    let (salt, encrypted) = data.split_at(SALT_LEN);
    let mut key = derive_key(passphrase, salt)?;

    let plaintext = decrypt(&key, encrypted)?;
    key.zeroize();

    Ok(plaintext)
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("decryption failed: invalid key or corrupted data")]
    DecryptionFailed,
    #[error("encryption failed")]
    EncryptionFailed,
    #[error("key derivation failed")]
    KeyDerivationFailed,
    #[error("invalid salt (must be at least 16 bytes)")]
    InvalidSalt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_derivation_deterministic() {
        let salt = generate_salt();
        let key1 = derive_key("test passphrase", &salt).unwrap();
        let key2 = derive_key("test passphrase", &salt).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn different_passphrase_different_key() {
        let salt = generate_salt();
        let key1 = derive_key("passphrase A", &salt).unwrap();
        let key2 = derive_key("passphrase B", &salt).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn different_salt_different_key() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let key1 = derive_key("same passphrase", &salt1).unwrap();
        let key2 = derive_key("same passphrase", &salt2).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let salt = generate_salt();
        let key = derive_key("my secure passphrase", &salt).unwrap();
        let plaintext = b"BIP-39 mnemonic seed material here";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_decrypt() {
        let salt = generate_salt();
        let key_good = derive_key("correct", &salt).unwrap();
        let key_bad = derive_key("wrong", &salt).unwrap();

        let encrypted = encrypt(&key_good, b"secret data").unwrap();
        let result = decrypt(&key_bad, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn passphrase_convenience_roundtrip() {
        let plaintext = b"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let encrypted = encrypt_with_passphrase("hunter2", plaintext).unwrap();
        let decrypted = decrypt_with_passphrase("hunter2", &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let encrypted = encrypt_with_passphrase("correct", b"secret").unwrap();
        let result = decrypt_with_passphrase("wrong", &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_data_fails() {
        let mut encrypted = encrypt_with_passphrase("passphrase", b"data").unwrap();
        // Corrupt a byte in the ciphertext
        if let Some(last) = encrypted.last_mut() {
            *last ^= 0xFF;
        }
        let result = decrypt_with_passphrase("passphrase", &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn too_short_data_fails() {
        let result = decrypt_with_passphrase("pass", &[0u8; 10]);
        assert!(result.is_err());
    }

    // ── Extended encryption tests ─────────────────────────────────

    #[test]
    fn salt_is_16_bytes() {
        let salt = generate_salt();
        assert_eq!(salt.len(), 16);
    }

    #[test]
    fn two_salts_are_different() {
        let s1 = generate_salt();
        let s2 = generate_salt();
        assert_ne!(s1, s2);
    }

    #[test]
    fn key_is_32_bytes() {
        let salt = generate_salt();
        let key = derive_key("test", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn short_salt_rejected() {
        let result = derive_key("test", &[0u8; 8]);
        assert!(result.is_err());
    }

    #[test]
    fn empty_passphrase_still_works() {
        let salt = generate_salt();
        let key = derive_key("", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn encrypt_empty_plaintext() {
        let salt = generate_salt();
        let key = derive_key("pass", &salt).unwrap();
        let encrypted = encrypt(&key, b"").unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn encrypt_large_data() {
        let salt = generate_salt();
        let key = derive_key("pass", &salt).unwrap();
        let data = vec![0xABu8; 1_000_000]; // 1 MB
        let encrypted = encrypt(&key, &data).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn ciphertext_differs_each_time() {
        // AES-GCM with random nonce should produce different ciphertext for same plaintext
        let salt = generate_salt();
        let key = derive_key("pass", &salt).unwrap();
        let ct1 = encrypt(&key, b"same data").unwrap();
        let ct2 = encrypt(&key, b"same data").unwrap();
        assert_ne!(ct1, ct2); // Random nonce ensures different ciphertext
    }

    #[test]
    fn ciphertext_is_larger_than_plaintext() {
        let salt = generate_salt();
        let key = derive_key("pass", &salt).unwrap();
        let plaintext = b"hello";
        let encrypted = encrypt(&key, plaintext).unwrap();
        // Should include nonce (12) + ciphertext + tag (16)
        assert!(encrypted.len() > plaintext.len());
        assert_eq!(encrypted.len(), 12 + plaintext.len() + 16);
    }

    #[test]
    fn passphrase_output_includes_salt() {
        let encrypted = encrypt_with_passphrase("test", b"data").unwrap();
        // salt (16) + nonce (12) + plaintext (4) + tag (16) = 48
        assert_eq!(encrypted.len(), 16 + 12 + 4 + 16);
    }

    #[test]
    fn truncated_ciphertext_fails() {
        let encrypted = encrypt_with_passphrase("pass", b"hello world").unwrap();
        // Truncate to remove tag
        let truncated = &encrypted[..encrypted.len() - 4];
        let result = decrypt_with_passphrase("pass", truncated);
        assert!(result.is_err());
    }

    #[test]
    fn bit_flip_in_nonce_fails() {
        let mut encrypted = encrypt_with_passphrase("pass", b"data").unwrap();
        // Flip a bit in the nonce area (bytes 16..28)
        encrypted[16] ^= 0x01;
        let result = decrypt_with_passphrase("pass", &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn bit_flip_in_salt_fails() {
        let mut encrypted = encrypt_with_passphrase("pass", b"data").unwrap();
        // Flip a bit in the salt area (bytes 0..16)
        encrypted[0] ^= 0x01;
        let result = decrypt_with_passphrase("pass", &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn very_long_passphrase_works() {
        let long_pass: String = "A".repeat(1000);
        let encrypted = encrypt_with_passphrase(&long_pass, b"data").unwrap();
        let decrypted = decrypt_with_passphrase(&long_pass, &encrypted).unwrap();
        assert_eq!(decrypted, b"data");
    }

    #[test]
    fn unicode_passphrase_works() {
        let encrypted = encrypt_with_passphrase("密码🔒", b"secret").unwrap();
        let decrypted = decrypt_with_passphrase("密码🔒", &encrypted).unwrap();
        assert_eq!(decrypted, b"secret");
    }
}

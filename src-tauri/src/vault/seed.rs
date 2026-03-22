//! BIP-39 mnemonic generation and seed derivation.
//! All seed material is zeroized on drop.

use bip39::{Language, Mnemonic};
use rand::RngCore;
use zeroize::Zeroize;

/// A securely-held mnemonic phrase that zeroizes on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecureMnemonic {
    phrase: String,
}

impl SecureMnemonic {
    /// Generate a new 12-word BIP-39 mnemonic.
    pub fn generate_12() -> Result<Self, SeedError> {
        let mut entropy = [0u8; 16]; // 128 bits → 12 words
        rand::rngs::OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| SeedError::Generation(e.to_string()))?;
        entropy.zeroize();
        Ok(Self {
            phrase: mnemonic.to_string(),
        })
    }

    /// Generate a new 24-word BIP-39 mnemonic.
    pub fn generate_24() -> Result<Self, SeedError> {
        let mut entropy = [0u8; 32]; // 256 bits → 24 words
        rand::rngs::OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| SeedError::Generation(e.to_string()))?;
        entropy.zeroize();
        Ok(Self {
            phrase: mnemonic.to_string(),
        })
    }

    /// Validate and construct from an existing mnemonic phrase.
    pub fn from_phrase(phrase: &str) -> Result<Self, SeedError> {
        let _mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase)
            .map_err(|_| SeedError::InvalidMnemonic)?;
        Ok(Self {
            phrase: phrase.to_string(),
        })
    }

    /// Derive a 64-byte seed from the mnemonic using an optional passphrase.
    /// The passphrase adds an extra layer of protection (BIP-39 standard).
    pub fn to_seed(&self, passphrase: &str) -> Result<SeedBytes, SeedError> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, &self.phrase)
            .map_err(|_| SeedError::InvalidMnemonic)?;
        let seed = mnemonic.to_seed(passphrase);
        Ok(SeedBytes(seed))
    }

    /// Get the mnemonic phrase (for display during creation only).
    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    /// Word count of this mnemonic.
    pub fn word_count(&self) -> usize {
        self.phrase.split_whitespace().count()
    }
}

/// A 64-byte seed derived from a BIP-39 mnemonic. Zeroized on drop.
pub struct SeedBytes([u8; 64]);

impl SeedBytes {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Construct from a raw 64-byte array (used during vault unlock).
    pub fn from_array(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

impl Drop for SeedBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("mnemonic generation failed: {0}")]
    Generation(String),
    #[error("invalid mnemonic phrase")]
    InvalidMnemonic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_12_word_mnemonic() {
        let mnemonic = SecureMnemonic::generate_12().unwrap();
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn generate_24_word_mnemonic() {
        let mnemonic = SecureMnemonic::generate_24().unwrap();
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn roundtrip_from_phrase() {
        let original = SecureMnemonic::generate_12().unwrap();
        let phrase = original.phrase().to_string();
        let restored = SecureMnemonic::from_phrase(&phrase).unwrap();
        assert_eq!(restored.phrase(), phrase.to_lowercase());
    }

    #[test]
    fn invalid_phrase_rejected() {
        let result = SecureMnemonic::from_phrase("not a valid mnemonic phrase at all");
        assert!(result.is_err());
    }

    #[test]
    fn seed_derivation_deterministic() {
        let mnemonic = SecureMnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        let seed1 = mnemonic.to_seed("").unwrap();
        let seed2 = mnemonic.to_seed("").unwrap();
        assert_eq!(seed1.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn different_passphrase_different_seed() {
        let mnemonic = SecureMnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        let seed_a = mnemonic.to_seed("").unwrap();
        let seed_b = mnemonic.to_seed("my passphrase").unwrap();
        assert_ne!(seed_a.as_bytes(), seed_b.as_bytes());
    }
}

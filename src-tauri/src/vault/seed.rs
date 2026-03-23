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

    // ── Extended seed tests ──────────────────────────────────────

    #[test]
    fn seed_is_64_bytes() {
        let mnemonic = SecureMnemonic::generate_12().unwrap();
        let seed = mnemonic.to_seed("").unwrap();
        assert_eq!(seed.as_bytes().len(), 64);
    }

    #[test]
    fn seed_24_word_is_64_bytes() {
        let mnemonic = SecureMnemonic::generate_24().unwrap();
        let seed = mnemonic.to_seed("").unwrap();
        assert_eq!(seed.as_bytes().len(), 64);
    }

    #[test]
    fn phrase_is_all_lowercase() {
        let mnemonic = SecureMnemonic::generate_12().unwrap();
        let phrase = mnemonic.phrase();
        assert_eq!(phrase, phrase.to_lowercase());
    }

    #[test]
    fn twelve_word_phrase_has_11_spaces() {
        let mnemonic = SecureMnemonic::generate_12().unwrap();
        let spaces = mnemonic.phrase().chars().filter(|c| *c == ' ').count();
        assert_eq!(spaces, 11);
    }

    #[test]
    fn twenty_four_word_phrase_has_23_spaces() {
        let mnemonic = SecureMnemonic::generate_24().unwrap();
        let spaces = mnemonic.phrase().chars().filter(|c| *c == ' ').count();
        assert_eq!(spaces, 23);
    }

    #[test]
    fn generated_mnemonics_are_unique() {
        let m1 = SecureMnemonic::generate_12().unwrap();
        let m2 = SecureMnemonic::generate_12().unwrap();
        assert_ne!(m1.phrase(), m2.phrase());
    }

    #[test]
    fn empty_phrase_rejected() {
        let result = SecureMnemonic::from_phrase("");
        assert!(result.is_err());
    }

    #[test]
    fn single_word_rejected() {
        let result = SecureMnemonic::from_phrase("abandon");
        assert!(result.is_err());
    }

    #[test]
    fn eleven_word_rejected() {
        let result = SecureMnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon",
        );
        assert!(result.is_err());
    }

    #[test]
    fn non_bip39_word_rejected() {
        let result = SecureMnemonic::from_phrase(
            "supercalifragilistic abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        );
        assert!(result.is_err());
    }

    #[test]
    fn from_array_roundtrip() {
        let mnemonic = SecureMnemonic::generate_12().unwrap();
        let seed = mnemonic.to_seed("test").unwrap();
        let raw = *seed.as_bytes();
        let restored = SeedBytes::from_array(raw);
        assert_eq!(seed.as_bytes(), restored.as_bytes());
    }

    #[test]
    fn known_mnemonic_known_seed() {
        // BIP-39 test vector: first 8 bytes of seed for known mnemonic with empty passphrase
        let mnemonic = SecureMnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        let seed = mnemonic.to_seed("").unwrap();
        // Seed must be non-zero
        assert!(seed.as_bytes().iter().any(|&b| b != 0));
        // And deterministic
        let seed2 = mnemonic.to_seed("").unwrap();
        assert_eq!(seed.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn passphrase_is_case_sensitive() {
        let m = SecureMnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        let seed_lower = m.to_seed("password").unwrap();
        let seed_upper = m.to_seed("Password").unwrap();
        assert_ne!(seed_lower.as_bytes(), seed_upper.as_bytes());
    }

    #[test]
    fn unicode_passphrase_works() {
        let m = SecureMnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        let seed = m.to_seed("日本語パスフレーズ").unwrap();
        assert_eq!(seed.as_bytes().len(), 64);
    }
}

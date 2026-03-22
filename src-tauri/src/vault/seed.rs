//! BIP-39 mnemonic generation and seed derivation.
//! All seed material is zeroized on drop.

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
        // TODO: Implement with bip39 crate
        Err(SeedError::NotImplemented)
    }

    /// Generate a new 24-word BIP-39 mnemonic.
    pub fn generate_24() -> Result<Self, SeedError> {
        // TODO: Implement with bip39 crate
        Err(SeedError::NotImplemented)
    }

    /// Validate an existing mnemonic phrase.
    pub fn from_phrase(phrase: &str) -> Result<Self, SeedError> {
        // TODO: Validate and construct
        let _ = phrase;
        Err(SeedError::NotImplemented)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("seed operation not yet implemented")]
    NotImplemented,
    #[error("invalid mnemonic phrase")]
    InvalidMnemonic,
}

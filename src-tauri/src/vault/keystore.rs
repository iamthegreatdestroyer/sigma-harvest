//! Encrypted keystore: save/load/lock/unlock the vault seed.
//! The keystore is the central coordinator for all vault operations.
//! It manages encrypted seed persistence and wallet derivation.

use super::derivation::{Chain, DerivedWallet};
use super::encryption;
use super::seed::{SecureMnemonic, SeedBytes};
use super::VaultStatus;
use std::sync::Mutex;

/// The keystore manages encrypted seed persistence and in-memory vault state.
/// While the vault is unlocked, the decrypted seed is held in memory.
/// On lock, all sensitive data is zeroized.
pub struct Keystore {
    status: VaultStatus,
    /// Decrypted seed, only present while vault is unlocked.
    seed: Option<SeedBytes>,
    /// Derived wallets cached while vault is open.
    wallets: Vec<DerivedWallet>,
    /// Next derivation index per chain.
    next_index: u32,
}

impl Keystore {
    pub fn new() -> Self {
        Self {
            status: VaultStatus::default(),
            seed: None,
            wallets: Vec::new(),
            next_index: 0,
        }
    }

    pub fn status(&self) -> &VaultStatus {
        &self.status
    }

    pub fn is_locked(&self) -> bool {
        self.status.locked
    }

    pub fn wallets(&self) -> &[DerivedWallet] {
        &self.wallets
    }

    /// Create a new vault with a fresh mnemonic.
    /// Returns the mnemonic phrase (must be shown to user once, then never stored in plaintext).
    /// The seed is encrypted with the passphrase and returned as bytes for DB storage.
    pub fn create(
        &mut self,
        passphrase: &str,
        word_count: usize,
    ) -> Result<(String, Vec<u8>), KeystoreError> {
        let mnemonic = match word_count {
            12 => SecureMnemonic::generate_12().map_err(|e| KeystoreError::SeedError(e.to_string()))?,
            24 => SecureMnemonic::generate_24().map_err(|e| KeystoreError::SeedError(e.to_string()))?,
            _ => return Err(KeystoreError::InvalidWordCount),
        };

        let phrase = mnemonic.phrase().to_string();

        // Derive seed from mnemonic (no additional BIP-39 passphrase for simplicity)
        let seed = mnemonic
            .to_seed("")
            .map_err(|e| KeystoreError::SeedError(e.to_string()))?;

        // Encrypt the seed with the user's passphrase
        let encrypted = encryption::encrypt_with_passphrase(passphrase, seed.as_bytes())
            .map_err(|e| KeystoreError::EncryptionError(e.to_string()))?;

        // Derive initial wallet (index 0)
        let wallet = super::derivation::derive_wallet(seed.as_bytes(), &Chain::Ethereum, 0)
            .map_err(|e| KeystoreError::DerivationError(e.to_string()))?;

        // Unlock the vault in memory
        self.seed = Some(seed);
        self.wallets = vec![wallet];
        self.next_index = 1;
        self.status.locked = false;
        self.status.wallet_count = 1;
        self.status.last_unlock = Some(chrono::Utc::now().to_rfc3339());

        Ok((phrase, encrypted))
    }

    /// Unlock the vault by decrypting the stored seed data.
    pub fn unlock(
        &mut self,
        passphrase: &str,
        encrypted_seed: &[u8],
    ) -> Result<(), KeystoreError> {
        let decrypted = encryption::decrypt_with_passphrase(passphrase, encrypted_seed)
            .map_err(|_| KeystoreError::InvalidPassphrase)?;

        if decrypted.len() != 64 {
            return Err(KeystoreError::CorruptedSeed);
        }

        let mut seed_array = [0u8; 64];
        seed_array.copy_from_slice(&decrypted);

        // Re-derive wallets up to stored index
        let mut wallets = Vec::new();
        for i in 0..self.next_index.max(1) {
            let wallet = super::derivation::derive_wallet(&seed_array, &Chain::Ethereum, i)
                .map_err(|e| KeystoreError::DerivationError(e.to_string()))?;
            wallets.push(wallet);
        }

        self.seed = Some(SeedBytes::from_array(seed_array));
        self.wallets = wallets;
        self.status.locked = false;
        self.status.wallet_count = self.wallets.len() as u32;
        self.status.last_unlock = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Lock the vault and zeroize all sensitive data in memory.
    pub fn lock(&mut self) {
        self.seed = None; // SeedBytes::Drop will zeroize
        self.wallets.clear();
        self.status.locked = true;
        tracing::info!("Vault locked, sensitive data zeroized");
    }

    /// Derive the next wallet for a given chain.
    pub fn derive_next(
        &mut self,
        chain: &Chain,
    ) -> Result<DerivedWallet, KeystoreError> {
        let seed = self.seed.as_ref().ok_or(KeystoreError::VaultLocked)?;

        let wallet = super::derivation::derive_wallet(seed.as_bytes(), chain, self.next_index)
            .map_err(|e| KeystoreError::DerivationError(e.to_string()))?;

        self.next_index += 1;
        self.wallets.push(wallet.clone());
        self.status.wallet_count = self.wallets.len() as u32;

        Ok(wallet)
    }

    /// Set the next derivation index (used when restoring from DB).
    pub fn set_next_index(&mut self, index: u32) {
        self.next_index = index;
    }
}

/// Thread-safe global vault state wrapped in a Mutex.
/// Used by Tauri commands to access the vault from any thread.
pub struct VaultState(pub Mutex<Keystore>);

impl VaultState {
    pub fn new() -> Self {
        Self(Mutex::new(Keystore::new()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("vault is locked")]
    VaultLocked,
    #[error("invalid passphrase")]
    InvalidPassphrase,
    #[error("seed error: {0}")]
    SeedError(String),
    #[error("encryption error: {0}")]
    EncryptionError(String),
    #[error("derivation error: {0}")]
    DerivationError(String),
    #[error("corrupted seed data")]
    CorruptedSeed,
    #[error("invalid word count (must be 12 or 24)")]
    InvalidWordCount,
}

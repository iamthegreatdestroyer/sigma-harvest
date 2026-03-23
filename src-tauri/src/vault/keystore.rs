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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vault() -> (Keystore, String, Vec<u8>) {
        let mut ks = Keystore::new();
        let (phrase, encrypted) = ks.create("test1234", 12).unwrap();
        (ks, phrase, encrypted)
    }

    // ── Creation tests ────────────────────────────────────────════

    #[test]
    fn new_keystore_is_locked() {
        let ks = Keystore::new();
        assert!(ks.is_locked());
        assert!(ks.wallets().is_empty());
    }

    #[test]
    fn create_12_word_vault() {
        let mut ks = Keystore::new();
        let (phrase, encrypted) = ks.create("test1234", 12).unwrap();
        assert_eq!(phrase.split_whitespace().count(), 12);
        assert!(!encrypted.is_empty());
        assert!(!ks.is_locked());
    }

    #[test]
    fn create_24_word_vault() {
        let mut ks = Keystore::new();
        let (phrase, encrypted) = ks.create("test1234", 24).unwrap();
        assert_eq!(phrase.split_whitespace().count(), 24);
        assert!(!encrypted.is_empty());
    }

    #[test]
    fn create_invalid_word_count() {
        let mut ks = Keystore::new();
        let err = ks.create("test1234", 15).unwrap_err();
        assert!(matches!(err, KeystoreError::InvalidWordCount));
    }

    #[test]
    fn create_generates_first_wallet() {
        let (ks, _, _) = create_test_vault();
        assert_eq!(ks.wallets().len(), 1);
        assert!(ks.wallets()[0].address.starts_with("0x"));
    }

    #[test]
    fn create_sets_wallet_count() {
        let (ks, _, _) = create_test_vault();
        assert_eq!(ks.status().wallet_count, 1);
    }

    #[test]
    fn create_sets_last_unlock() {
        let (ks, _, _) = create_test_vault();
        assert!(ks.status().last_unlock.is_some());
    }

    // ── Lock / Unlock lifecycle ───────────────────────────────────

    #[test]
    fn lock_clears_wallets() {
        let (mut ks, _, _) = create_test_vault();
        assert!(!ks.wallets().is_empty());
        ks.lock();
        assert!(ks.is_locked());
        assert!(ks.wallets().is_empty());
    }

    #[test]
    fn unlock_with_correct_passphrase() {
        let (mut ks, _, encrypted) = create_test_vault();
        let addr_before = ks.wallets()[0].address.clone();
        ks.lock();
        ks.unlock("test1234", &encrypted).unwrap();
        assert!(!ks.is_locked());
        assert_eq!(ks.wallets()[0].address, addr_before);
    }

    #[test]
    fn unlock_with_wrong_passphrase() {
        let (mut ks, _, encrypted) = create_test_vault();
        ks.lock();
        let err = ks.unlock("WRONG", &encrypted).unwrap_err();
        assert!(matches!(err, KeystoreError::InvalidPassphrase));
        assert!(ks.is_locked());
    }

    #[test]
    fn unlock_with_corrupted_data() {
        let (mut ks, _, mut encrypted) = create_test_vault();
        ks.lock();
        // Corrupt the ciphertext
        if let Some(b) = encrypted.last_mut() {
            *b ^= 0xFF;
        }
        assert!(ks.unlock("test1234", &encrypted).is_err());
    }

    #[test]
    fn unlock_with_truncated_data() {
        let (mut ks, _, encrypted) = create_test_vault();
        ks.lock();
        let truncated = &encrypted[..10];
        assert!(ks.unlock("test1234", truncated).is_err());
    }

    #[test]
    fn unlock_with_empty_data() {
        let mut ks = Keystore::new();
        assert!(ks.unlock("test1234", &[]).is_err());
    }

    // ── Derivation through keystore ───────────────────────────────

    #[test]
    fn derive_next_wallet_while_unlocked() {
        let (mut ks, _, _) = create_test_vault();
        let wallet = ks.derive_next(&Chain::Ethereum).unwrap();
        assert_eq!(wallet.index, 1);
        assert_eq!(ks.wallets().len(), 2);
    }

    #[test]
    fn derive_multiple_wallets_increments_index() {
        let (mut ks, _, _) = create_test_vault();
        let w1 = ks.derive_next(&Chain::Ethereum).unwrap();
        let w2 = ks.derive_next(&Chain::Ethereum).unwrap();
        let w3 = ks.derive_next(&Chain::Ethereum).unwrap();
        assert_eq!(w1.index, 1);
        assert_eq!(w2.index, 2);
        assert_eq!(w3.index, 3);
        assert_eq!(ks.wallets().len(), 4); // 1 from create + 3 derived
    }

    #[test]
    fn derive_while_locked_fails() {
        let mut ks = Keystore::new();
        let err = ks.derive_next(&Chain::Ethereum).unwrap_err();
        assert!(matches!(err, KeystoreError::VaultLocked));
    }

    #[test]
    fn derive_on_different_chains() {
        let (mut ks, _, _) = create_test_vault();
        let arb = ks.derive_next(&Chain::Arbitrum).unwrap();
        let op = ks.derive_next(&Chain::Optimism).unwrap();
        assert_eq!(arb.chain, Chain::Arbitrum);
        assert_eq!(op.chain, Chain::Optimism);
    }

    #[test]
    fn set_next_index_affects_derivation() {
        let (mut ks, _, _) = create_test_vault();
        ks.set_next_index(42);
        let wallet = ks.derive_next(&Chain::Ethereum).unwrap();
        assert_eq!(wallet.index, 42);
    }

    // ── Lock/unlock/derive roundtrip ──────────────────────────────

    #[test]
    fn full_lifecycle_roundtrip() {
        let mut ks = Keystore::new();

        // Create
        let (phrase, encrypted) = ks.create("myP@ssw0rd", 12).unwrap();
        assert!(!ks.is_locked());
        assert_eq!(phrase.split_whitespace().count(), 12);
        let first_addr = ks.wallets()[0].address.clone();

        // Derive
        let _w1 = ks.derive_next(&Chain::Ethereum).unwrap();
        assert_eq!(ks.wallets().len(), 2);

        // Lock
        ks.lock();
        assert!(ks.is_locked());
        assert!(ks.wallets().is_empty());

        // Unlock (should re-derive wallets)
        ks.set_next_index(2);
        ks.unlock("myP@ssw0rd", &encrypted).unwrap();
        assert!(!ks.is_locked());
        assert_eq!(ks.wallets()[0].address, first_addr);
        assert_eq!(ks.wallets().len(), 2);
    }

    // ── Thread-safety ─────────────────────────────────────────────

    #[test]
    fn vault_state_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<VaultState>();
    }

    #[test]
    fn vault_state_mutex_works() {
        let state = VaultState::new();
        let ks = state.0.lock().unwrap();
        assert!(ks.is_locked());
    }
}

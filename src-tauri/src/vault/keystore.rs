//! Encrypted keystore: save/load/lock/unlock the vault seed.

use super::VaultStatus;

/// The keystore manages encrypted seed persistence.
pub struct Keystore {
    status: VaultStatus,
}

impl Keystore {
    pub fn new() -> Self {
        Self {
            status: VaultStatus::default(),
        }
    }

    pub fn status(&self) -> &VaultStatus {
        &self.status
    }

    /// Unlock the vault with a passphrase.
    pub fn unlock(&mut self, _passphrase: &str) -> Result<(), KeystoreError> {
        // TODO: Decrypt seed, derive wallets
        Err(KeystoreError::NotImplemented)
    }

    /// Lock the vault and zeroize sensitive data.
    pub fn lock(&mut self) {
        self.status.locked = true;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("keystore operation not yet implemented")]
    NotImplemented,
    #[error("vault is locked")]
    VaultLocked,
    #[error("invalid passphrase")]
    InvalidPassphrase,
}

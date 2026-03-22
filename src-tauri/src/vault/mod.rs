pub mod encryption;
pub mod derivation;
pub mod keystore;
pub mod seed;

use serde::{Deserialize, Serialize};

/// Status of the encrypted vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub locked: bool,
    pub wallet_count: u32,
    pub last_unlock: Option<String>,
}

impl Default for VaultStatus {
    fn default() -> Self {
        Self {
            locked: true,
            wallet_count: 0,
            last_unlock: None,
        }
    }
}

//! BIP-44 HD wallet derivation.
//! Derives child wallets from master seed for multiple chains.

use serde::{Deserialize, Serialize};

/// Supported derivation paths per chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    Ethereum,
    Arbitrum,
    Optimism,
    Base,
    Polygon,
    ZkSync,
}

impl Chain {
    /// BIP-44 derivation path prefix for this chain.
    pub fn derivation_path(&self, index: u32) -> String {
        // All EVM chains use ETH coin type (60')
        format!("m/44'/60'/0'/0/{}", index)
    }
}

/// A derived wallet with public address only (private key stays in vault).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedWallet {
    pub address: String,
    pub chain: Chain,
    pub path: String,
    pub index: u32,
    pub label: Option<String>,
}

/// Derive a wallet at the given index.
pub fn derive_wallet(_seed: &[u8], chain: &Chain, index: u32) -> Result<DerivedWallet, DerivationError> {
    let _ = (chain, index);
    // TODO: Implement BIP-32/44 derivation
    Err(DerivationError::NotImplemented)
}

#[derive(Debug, thiserror::Error)]
pub enum DerivationError {
    #[error("derivation not yet implemented")]
    NotImplemented,
    #[error("invalid seed")]
    InvalidSeed,
    #[error("derivation path error")]
    PathError,
}

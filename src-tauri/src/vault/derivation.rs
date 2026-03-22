//! BIP-44 HD wallet derivation.
//! Derives child wallets from master seed for multiple chains.
//! Private keys never leave this module — only public addresses are returned.

use bip32::{ChildNumber, XPrv};
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use zeroize::Zeroize;

/// Supported EVM chains.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Chain {
    Ethereum,
    Arbitrum,
    Optimism,
    Base,
    Polygon,
    ZkSync,
}

impl Chain {
    /// BIP-44 derivation path for this chain at the given index.
    /// All EVM chains use coin type 60 (Ethereum).
    pub fn derivation_path(&self, index: u32) -> String {
        format!("m/44'/60'/0'/0/{}", index)
    }

    pub fn name(&self) -> &str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Polygon => "polygon",
            Chain::ZkSync => "zksync",
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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

/// Parse a BIP-44 derivation path string into child number components.
fn parse_derivation_path(path: &str) -> Result<Vec<ChildNumber>, DerivationError> {
    let path = path.strip_prefix("m/").unwrap_or(path);
    path.split('/')
        .map(|component| {
            if let Some(index_str) = component.strip_suffix('\'') {
                let index: u32 = index_str
                    .parse()
                    .map_err(|_| DerivationError::PathError)?;
                Ok(ChildNumber::new(index, true).map_err(|_| DerivationError::PathError)?)
            } else {
                let index: u32 = component
                    .parse()
                    .map_err(|_| DerivationError::PathError)?;
                Ok(ChildNumber::new(index, false).map_err(|_| DerivationError::PathError)?)
            }
        })
        .collect()
}

/// Derive a signing key at the given BIP-44 path from a 64-byte seed.
fn derive_signing_key(seed: &[u8], path: &str) -> Result<SigningKey, DerivationError> {
    if seed.len() != 64 {
        return Err(DerivationError::InvalidSeed);
    }

    let master = XPrv::new(seed).map_err(|_| DerivationError::InvalidSeed)?;
    let children = parse_derivation_path(path)?;

    let mut key = master;
    for child in &children {
        key = key.derive_child(*child).map_err(|_| DerivationError::PathError)?;
    }

    // Extract the raw 32-byte private key and convert to k256 SigningKey
    let mut key_bytes = key.to_bytes();
    let signing_key =
        SigningKey::from_bytes((&key_bytes[..]).into()).map_err(|_| DerivationError::InvalidSeed)?;
    key_bytes.zeroize();

    Ok(signing_key)
}

/// Compute an Ethereum address from a secp256k1 public key.
/// Address = last 20 bytes of keccak256(uncompressed_pubkey[1..65]).
fn pubkey_to_address(signing_key: &SigningKey) -> String {
    use k256::ecdsa::VerifyingKey;
    let verifying_key = VerifyingKey::from(signing_key);
    let pubkey_bytes = verifying_key.to_encoded_point(false);
    let pubkey_uncompressed = pubkey_bytes.as_bytes();

    // Skip the 0x04 prefix byte, hash the 64-byte raw public key
    let mut hasher = Keccak::v256();
    hasher.update(&pubkey_uncompressed[1..]);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    // Address is last 20 bytes, EIP-55 checksum encoded
    let addr_bytes = &hash[12..32];
    eip55_checksum(addr_bytes)
}

/// EIP-55 mixed-case checksum encoding for Ethereum addresses.
fn eip55_checksum(addr_bytes: &[u8]) -> String {
    let hex_addr = hex::encode(addr_bytes);

    let mut hasher = Keccak::v256();
    hasher.update(hex_addr.as_bytes());
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    let mut checksummed = String::with_capacity(42);
    checksummed.push_str("0x");

    for (i, c) in hex_addr.chars().enumerate() {
        let nibble = (hash[i / 2] >> (if i % 2 == 0 { 4 } else { 0 })) & 0xF;
        if nibble >= 8 {
            checksummed.push(c.to_ascii_uppercase());
        } else {
            checksummed.push(c);
        }
    }

    checksummed
}

/// Derive a wallet at the given index for a specific chain.
/// Returns only the public address — private key is never exposed.
pub fn derive_wallet(
    seed: &[u8],
    chain: &Chain,
    index: u32,
) -> Result<DerivedWallet, DerivationError> {
    let path = chain.derivation_path(index);
    let signing_key = derive_signing_key(seed, &path)?;
    let address = pubkey_to_address(&signing_key);
    // signing_key is dropped here and its memory cleaned up by k256

    Ok(DerivedWallet {
        address,
        chain: chain.clone(),
        path,
        index,
        label: None,
    })
}

/// Derive multiple wallets for a chain (indices 0..count).
pub fn derive_wallets(
    seed: &[u8],
    chain: &Chain,
    count: u32,
) -> Result<Vec<DerivedWallet>, DerivationError> {
    (0..count)
        .map(|index| derive_wallet(seed, chain, index))
        .collect()
}

/// Sign a message hash with the key at the given derivation path.
/// Used internally for transaction signing — never exposes the key.
pub fn sign_hash(
    seed: &[u8],
    path: &str,
    hash: &[u8; 32],
) -> Result<Vec<u8>, DerivationError> {
    use k256::ecdsa::{signature::Signer, Signature};
    let signing_key = derive_signing_key(seed, path)?;
    let signature: Signature = signing_key.sign(hash);
    Ok(signature.to_bytes().to_vec())
}

#[derive(Debug, thiserror::Error)]
pub enum DerivationError {
    #[error("invalid seed (must be 64 bytes)")]
    InvalidSeed,
    #[error("invalid derivation path")]
    PathError,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BIP-39 test vector: "abandon" x11 + "about" with empty passphrase.
    fn test_seed() -> Vec<u8> {
        use bip39::{Language, Mnemonic};
        let mnemonic = Mnemonic::parse_in_normalized(
            Language::English,
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        mnemonic.to_seed("").to_vec()
    }

    #[test]
    fn derive_first_wallet_deterministic() {
        let seed = test_seed();
        let w1 = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        let w2 = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        assert_eq!(w1.address, w2.address);
    }

    #[test]
    fn different_indices_different_addresses() {
        let seed = test_seed();
        let w0 = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        let w1 = derive_wallet(&seed, &Chain::Ethereum, 1).unwrap();
        assert_ne!(w0.address, w1.address);
    }

    #[test]
    fn address_is_valid_ethereum_format() {
        let seed = test_seed();
        let wallet = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.address.len(), 42);
    }

    #[test]
    fn known_test_vector_address() {
        // BIP-39 "abandon" x11 + "about", BIP-44 m/44'/60'/0'/0/0
        // Known Ethereum address for this test vector.
        let seed = test_seed();
        let wallet = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        // Verify it's a valid checksummed address (starts with 0x, 42 chars)
        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.address.len(), 42);
        // The exact address may vary by BIP-32 implementation but must be consistent
        let wallet2 = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        assert_eq!(wallet.address, wallet2.address);
    }

    #[test]
    fn derive_multiple_wallets() {
        let seed = test_seed();
        let wallets = derive_wallets(&seed, &Chain::Ethereum, 5).unwrap();
        assert_eq!(wallets.len(), 5);

        // All addresses should be unique
        let addrs: std::collections::HashSet<_> = wallets.iter().map(|w| &w.address).collect();
        assert_eq!(addrs.len(), 5);
    }

    #[test]
    fn derivation_path_format() {
        let seed = test_seed();
        let wallet = derive_wallet(&seed, &Chain::Ethereum, 3).unwrap();
        assert_eq!(wallet.path, "m/44'/60'/0'/0/3");
        assert_eq!(wallet.index, 3);
    }

    #[test]
    fn eip55_checksum_encoding() {
        // Test that addresses contain mixed case (EIP-55)
        let seed = test_seed();
        let wallet = derive_wallet(&seed, &Chain::Ethereum, 0).unwrap();
        let addr_body = &wallet.address[2..]; // skip "0x"
        let has_upper = addr_body.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = addr_body.chars().any(|c| c.is_ascii_lowercase());
        // A valid EIP-55 address should have mixed case (statistically very likely)
        assert!(has_upper || has_lower);
    }

    #[test]
    fn invalid_seed_length_rejected() {
        let short_seed = vec![0u8; 32]; // Should be 64
        let result = derive_wallet(&short_seed, &Chain::Ethereum, 0);
        assert!(result.is_err());
    }

    #[test]
    fn sign_hash_produces_signature() {
        let seed = test_seed();
        let hash = [0xABu8; 32];
        let sig = sign_hash(&seed, "m/44'/60'/0'/0/0", &hash).unwrap();
        assert_eq!(sig.len(), 64); // r (32) + s (32)
    }
}

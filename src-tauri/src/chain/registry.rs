//! Chain configuration registry.
//! Defines all supported chains with RPC endpoints, block explorers, and gas ceilings.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Configuration for a single EVM chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub symbol: String,
    pub rpc_urls: Vec<String>,
    pub block_explorer: String,
    pub default_gas_ceiling_gwei: f64,
    pub is_l2: bool,
}

/// Registry helper for looking up chain configs.
pub struct ChainRegistry;

impl ChainRegistry {
    /// Look up a chain config by name (case-insensitive).
    pub fn get(name: &str) -> Option<&'static ChainConfig> {
        SUPPORTED_CHAINS
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// Look up a chain config by chain ID.
    pub fn get_by_id(chain_id: u64) -> Option<&'static ChainConfig> {
        SUPPORTED_CHAINS.iter().find(|c| c.chain_id == chain_id)
    }

    /// Get all supported chain names.
    pub fn chain_names() -> Vec<&'static str> {
        SUPPORTED_CHAINS.iter().map(|c| c.name.as_str()).collect()
    }
}

/// All Phase-1 supported chains with primary + fallback RPC endpoints.
pub static SUPPORTED_CHAINS: LazyLock<Vec<ChainConfig>> = LazyLock::new(|| {
    vec![
        ChainConfig {
            chain_id: 1,
            name: "ethereum".to_string(),
            symbol: "ETH".to_string(),
            rpc_urls: vec![
                "https://eth.llamarpc.com".to_string(),
                "https://rpc.ankr.com/eth".to_string(),
            ],
            block_explorer: "https://etherscan.io".to_string(),
            default_gas_ceiling_gwei: 30.0,
            is_l2: false,
        },
        ChainConfig {
            chain_id: 42161,
            name: "arbitrum".to_string(),
            symbol: "ARB".to_string(),
            rpc_urls: vec![
                "https://arb1.arbitrum.io/rpc".to_string(),
                "https://rpc.ankr.com/arbitrum".to_string(),
            ],
            block_explorer: "https://arbiscan.io".to_string(),
            default_gas_ceiling_gwei: 0.5,
            is_l2: true,
        },
        ChainConfig {
            chain_id: 10,
            name: "optimism".to_string(),
            symbol: "OP".to_string(),
            rpc_urls: vec![
                "https://mainnet.optimism.io".to_string(),
                "https://rpc.ankr.com/optimism".to_string(),
            ],
            block_explorer: "https://optimistic.etherscan.io".to_string(),
            default_gas_ceiling_gwei: 0.1,
            is_l2: true,
        },
        ChainConfig {
            chain_id: 8453,
            name: "base".to_string(),
            symbol: "BASE".to_string(),
            rpc_urls: vec![
                "https://mainnet.base.org".to_string(),
                "https://rpc.ankr.com/base".to_string(),
            ],
            block_explorer: "https://basescan.org".to_string(),
            default_gas_ceiling_gwei: 0.1,
            is_l2: true,
        },
        ChainConfig {
            chain_id: 137,
            name: "polygon".to_string(),
            symbol: "MATIC".to_string(),
            rpc_urls: vec![
                "https://polygon-rpc.com".to_string(),
                "https://rpc.ankr.com/polygon".to_string(),
            ],
            block_explorer: "https://polygonscan.com".to_string(),
            default_gas_ceiling_gwei: 100.0,
            is_l2: true,
        },
        ChainConfig {
            chain_id: 324,
            name: "zksync".to_string(),
            symbol: "ZK".to_string(),
            rpc_urls: vec![
                "https://mainnet.era.zksync.io".to_string(),
                "https://rpc.ankr.com/zksync_era".to_string(),
            ],
            block_explorer: "https://explorer.zksync.io".to_string(),
            default_gas_ceiling_gwei: 0.5,
            is_l2: true,
        },
    ]
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_chains_present() {
        assert_eq!(SUPPORTED_CHAINS.len(), 6);
    }

    #[test]
    fn chain_ids_are_unique() {
        let ids: Vec<u64> = SUPPORTED_CHAINS.iter().map(|c| c.chain_id).collect();
        let unique: std::collections::HashSet<u64> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn chain_names_are_unique() {
        let names: Vec<&str> = SUPPORTED_CHAINS.iter().map(|c| c.name.as_str()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn all_chains_have_rpc_urls() {
        for chain in SUPPORTED_CHAINS.iter() {
            assert!(!chain.rpc_urls.is_empty(), "{} has no RPC URLs", chain.name);
        }
    }

    #[test]
    fn all_chains_have_fallback_rpc() {
        for chain in SUPPORTED_CHAINS.iter() {
            assert!(
                chain.rpc_urls.len() >= 2,
                "{} needs at least 2 RPC URLs for failover",
                chain.name
            );
        }
    }

    #[test]
    fn ethereum_is_not_l2() {
        let eth = SUPPORTED_CHAINS.iter().find(|c| c.name == "ethereum").unwrap();
        assert!(!eth.is_l2);
    }

    #[test]
    fn all_l2s_are_marked() {
        for chain in SUPPORTED_CHAINS.iter() {
            if chain.name != "ethereum" {
                assert!(chain.is_l2, "{} should be marked as L2", chain.name);
            }
        }
    }

    #[test]
    fn l2_gas_ceilings_lower_than_l1() {
        let eth = SUPPORTED_CHAINS.iter().find(|c| c.name == "ethereum").unwrap();
        for chain in SUPPORTED_CHAINS.iter().filter(|c| c.is_l2) {
            // polygon can be higher
            if chain.name != "polygon" {
                assert!(
                    chain.default_gas_ceiling_gwei < eth.default_gas_ceiling_gwei,
                    "{} ceiling should be < ethereum",
                    chain.name
                );
            }
        }
    }

    #[test]
    fn ethereum_chain_id_is_1() {
        let eth = SUPPORTED_CHAINS.iter().find(|c| c.name == "ethereum").unwrap();
        assert_eq!(eth.chain_id, 1);
    }

    #[test]
    fn block_explorer_urls_are_https() {
        for chain in SUPPORTED_CHAINS.iter() {
            assert!(
                chain.block_explorer.starts_with("https://"),
                "{} explorer should be HTTPS",
                chain.name
            );
        }
    }

    #[test]
    fn rpc_urls_are_https() {
        for chain in SUPPORTED_CHAINS.iter() {
            for url in &chain.rpc_urls {
                assert!(
                    url.starts_with("https://"),
                    "{} RPC URL should be HTTPS: {}",
                    chain.name,
                    url
                );
            }
        }
    }

    #[test]
    fn chain_config_serializable() {
        let eth = SUPPORTED_CHAINS.iter().find(|c| c.name == "ethereum").unwrap();
        let json = serde_json::to_string(eth).unwrap();
        let deser: ChainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.chain_id, 1);
        assert_eq!(deser.name, "ethereum");
    }

    #[test]
    fn chain_registry_lookup_by_name() {
        let names = ChainRegistry::chain_names();
        assert_eq!(names.len(), 6);
        assert!(names.contains(&"ethereum"));
        assert!(names.contains(&"zksync"));
    }
}

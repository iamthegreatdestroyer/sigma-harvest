//! Gas price monitoring across supported chains.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    pub chain: String,
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub timestamp: String,
}

/// Fetch current gas prices for a chain.
pub async fn fetch_gas_price(_chain: &str, _rpc_url: &str) -> Result<GasPrice, GasOracleError> {
    // TODO: Query EIP-1559 gas parameters
    Err(GasOracleError::NotImplemented)
}

/// Check if current gas is below the configured ceiling.
pub fn gas_below_ceiling(current: &GasPrice, ceiling_gwei: f64) -> bool {
    current.base_fee_gwei + current.priority_fee_gwei <= ceiling_gwei
}

#[derive(Debug, thiserror::Error)]
pub enum GasOracleError {
    #[error("gas oracle not yet implemented")]
    NotImplemented,
    #[error("RPC error: {0}")]
    RpcError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_below_ceiling_true() {
        let price = GasPrice {
            chain: "ethereum".to_string(),
            base_fee_gwei: 15.0,
            priority_fee_gwei: 2.0,
            timestamp: "2026-01-01".to_string(),
        };
        assert!(gas_below_ceiling(&price, 30.0));
    }

    #[test]
    fn gas_below_ceiling_false() {
        let price = GasPrice {
            chain: "ethereum".to_string(),
            base_fee_gwei: 25.0,
            priority_fee_gwei: 10.0,
            timestamp: "2026-01-01".to_string(),
        };
        assert!(!gas_below_ceiling(&price, 30.0));
    }

    #[test]
    fn gas_exactly_at_ceiling() {
        let price = GasPrice {
            chain: "ethereum".to_string(),
            base_fee_gwei: 28.0,
            priority_fee_gwei: 2.0,
            timestamp: "2026-01-01".to_string(),
        };
        assert!(gas_below_ceiling(&price, 30.0)); // equal is below
    }

    #[test]
    fn gas_price_serializable() {
        let price = GasPrice {
            chain: "arbitrum".to_string(),
            base_fee_gwei: 0.1,
            priority_fee_gwei: 0.01,
            timestamp: "2026-01-01".to_string(),
        };
        let json = serde_json::to_string(&price).unwrap();
        let roundtrip: GasPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.chain, "arbitrum");
    }

    #[tokio::test]
    async fn fetch_gas_price_returns_not_implemented() {
        let result = fetch_gas_price("ethereum", "https://eth.llamarpc.com").await;
        assert!(result.is_err());
    }

    #[test]
    fn gas_oracle_error_display() {
        let err = GasOracleError::NotImplemented;
        assert!(format!("{}", err).contains("not yet implemented"));
        let err = GasOracleError::RpcError("timeout".to_string());
        assert!(format!("{}", err).contains("timeout"));
    }
}

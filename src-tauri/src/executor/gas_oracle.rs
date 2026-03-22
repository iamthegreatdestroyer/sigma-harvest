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

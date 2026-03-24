//! RPC provider with automatic failover, rate limiting, and balance/gas fetching.
//! Uses raw JSON-RPC over reqwest to avoid heavy alloy dependency.

use crate::chain::registry::{ChainConfig, SUPPORTED_CHAINS};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Balance result for a single address on a single chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBalance {
    pub address: String,
    pub chain: String,
    pub chain_id: u64,
    pub balance_wei: String,
    pub balance_eth: f64,
}

/// Gas price result (EIP-1559 aware).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPriceResult {
    pub chain: String,
    pub chain_id: u64,
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub total_gwei: f64,
}

/// JSON-RPC request body.
#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// JSON-RPC response body.
#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChainClientError {
    #[error("all RPC endpoints failed for {chain}: {last_error}")]
    AllEndpointsFailed { chain: String, last_error: String },

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    ParseError(String),

    #[error("unknown chain: {0}")]
    UnknownChain(String),
}

/// Thread-safe RPC client with failover and rate limiting.
pub struct ChainClient {
    http: Client,
    /// Per-chain semaphore to limit concurrent requests (free tier friendly).
    rate_limiter: Arc<Semaphore>,
}

impl ChainClient {
    /// Create a new chain client.
    /// `max_concurrent` controls how many simultaneous RPC calls are allowed.
    pub fn new(max_concurrent: usize) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            rate_limiter: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Public RPC call for use by other modules (e.g., simulation).
    pub async fn rpc_call_public(
        &self,
        chain_config: &ChainConfig,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ChainClientError> {
        self.rpc_call(chain_config, method, params).await
    }

    /// Make a JSON-RPC call with automatic failover across a chain's RPC URLs.
    async fn rpc_call(
        &self,
        chain_config: &ChainConfig,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ChainClientError> {
        let _permit = self
            .rate_limiter
            .acquire()
            .await
            .map_err(|e| ChainClientError::RpcError(format!("semaphore: {e}")))?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: 1,
        };

        let mut last_error = String::new();

        for rpc_url in &chain_config.rpc_urls {
            match self.http.post(rpc_url).json(&request).send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        last_error = format!("HTTP {}", resp.status());
                        continue;
                    }
                    match resp.json::<JsonRpcResponse>().await {
                        Ok(rpc_resp) => {
                            if let Some(err) = rpc_resp.error {
                                last_error = format!("RPC error: {}", err);
                                continue;
                            }
                            if let Some(result) = rpc_resp.result {
                                return Ok(result);
                            }
                            last_error = "null result from RPC".to_string();
                            continue;
                        }
                        Err(e) => {
                            last_error = format!("JSON parse: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    last_error = format!("connection: {}", e);
                    continue;
                }
            }
        }

        Err(ChainClientError::AllEndpointsFailed {
            chain: chain_config.name.clone(),
            last_error,
        })
    }

    /// Get the native token balance (ETH, MATIC, etc.) for an address.
    pub async fn get_balance(
        &self,
        chain_name: &str,
        address: &str,
    ) -> Result<AddressBalance, ChainClientError> {
        let chain = find_chain(chain_name)?;
        let params = serde_json::json!([address, "latest"]);
        let result = self.rpc_call(chain, "eth_getBalance", params).await?;

        let hex_balance = result
            .as_str()
            .ok_or_else(|| ChainClientError::ParseError("expected hex string".to_string()))?;

        let balance_wei = parse_hex_u128(hex_balance);
        let balance_eth = balance_wei as f64 / 1e18;

        Ok(AddressBalance {
            address: address.to_string(),
            chain: chain.name.clone(),
            chain_id: chain.chain_id,
            balance_wei: balance_wei.to_string(),
            balance_eth,
        })
    }

    /// Get balances for an address across all supported chains.
    pub async fn get_all_balances(
        &self,
        address: &str,
    ) -> Vec<Result<AddressBalance, ChainClientError>> {
        let mut results = Vec::new();
        for chain in SUPPORTED_CHAINS.iter() {
            results.push(self.get_balance(&chain.name, address).await);
        }
        results
    }

    /// Fetch current gas prices (EIP-1559) for a chain.
    pub async fn get_gas_price(
        &self,
        chain_name: &str,
    ) -> Result<GasPriceResult, ChainClientError> {
        let chain = find_chain(chain_name)?;

        // Try EIP-1559 fee history first
        let fee_history = self
            .rpc_call(
                chain,
                "eth_feeHistory",
                serde_json::json!(["0x1", "latest", [50]]),
            )
            .await;

        match fee_history {
            Ok(history) => {
                let base_fee = history
                    .get("baseFeePerGas")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.last())
                    .and_then(|v| v.as_str())
                    .map(parse_hex_u128)
                    .unwrap_or(0);

                let priority_fee = history
                    .get("reward")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .map(parse_hex_u128)
                    .unwrap_or(1_000_000_000); // 1 gwei fallback

                let base_gwei = base_fee as f64 / 1e9;
                let priority_gwei = priority_fee as f64 / 1e9;

                Ok(GasPriceResult {
                    chain: chain.name.clone(),
                    chain_id: chain.chain_id,
                    base_fee_gwei: base_gwei,
                    priority_fee_gwei: priority_gwei,
                    total_gwei: base_gwei + priority_gwei,
                })
            }
            Err(_) => {
                // Fallback to legacy eth_gasPrice
                let result = self
                    .rpc_call(chain, "eth_gasPrice", serde_json::json!([]))
                    .await?;

                let gas_wei = result
                    .as_str()
                    .map(parse_hex_u128)
                    .unwrap_or(20_000_000_000);

                let total_gwei = gas_wei as f64 / 1e9;

                Ok(GasPriceResult {
                    chain: chain.name.clone(),
                    chain_id: chain.chain_id,
                    base_fee_gwei: total_gwei,
                    priority_fee_gwei: 0.0,
                    total_gwei,
                })
            }
        }
    }

    /// Fetch gas prices for all supported chains.
    pub async fn get_all_gas_prices(&self) -> Vec<Result<GasPriceResult, ChainClientError>> {
        let mut results = Vec::new();
        for chain in SUPPORTED_CHAINS.iter() {
            results.push(self.get_gas_price(&chain.name).await);
        }
        results
    }
}

/// Find a chain config by name.
fn find_chain(name: &str) -> Result<&'static ChainConfig, ChainClientError> {
    SUPPORTED_CHAINS
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| ChainClientError::UnknownChain(name.to_string()))
}

/// Parse a "0x"-prefixed hex string to u128.
fn parse_hex_u128(hex: &str) -> u128 {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u128::from_str_radix(hex, 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_u128_basic() {
        assert_eq!(parse_hex_u128("0x0"), 0);
        assert_eq!(parse_hex_u128("0x1"), 1);
        assert_eq!(parse_hex_u128("0xff"), 255);
        assert_eq!(
            parse_hex_u128("0xde0b6b3a7640000"),
            1_000_000_000_000_000_000
        ); // 1 ETH
    }

    #[test]
    fn parse_hex_u128_no_prefix() {
        assert_eq!(parse_hex_u128("ff"), 255);
        assert_eq!(parse_hex_u128("de0b6b3a7640000"), 1_000_000_000_000_000_000);
    }

    #[test]
    fn parse_hex_u128_invalid_returns_zero() {
        assert_eq!(parse_hex_u128("0xZZZ"), 0);
        assert_eq!(parse_hex_u128("not_hex"), 0);
    }

    #[test]
    fn find_chain_case_insensitive() {
        assert!(find_chain("ethereum").is_ok());
        assert!(find_chain("Ethereum").is_ok());
        assert!(find_chain("ETHEREUM").is_ok());
    }

    #[test]
    fn find_chain_unknown() {
        assert!(find_chain("solana").is_err());
    }

    #[test]
    fn chain_client_new() {
        let client = ChainClient::new(4);
        assert_eq!(client.rate_limiter.available_permits(), 4);
    }

    #[test]
    fn address_balance_serializable() {
        let balance = AddressBalance {
            address: "0xABC".to_string(),
            chain: "ethereum".to_string(),
            chain_id: 1,
            balance_wei: "1000000000000000000".to_string(),
            balance_eth: 1.0,
        };
        let json = serde_json::to_string(&balance).unwrap();
        let deser: AddressBalance = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.balance_eth, 1.0);
    }

    #[test]
    fn gas_price_result_serializable() {
        let gas = GasPriceResult {
            chain: "arbitrum".to_string(),
            chain_id: 42161,
            base_fee_gwei: 0.1,
            priority_fee_gwei: 0.01,
            total_gwei: 0.11,
        };
        let json = serde_json::to_string(&gas).unwrap();
        let deser: GasPriceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.chain, "arbitrum");
    }

    #[test]
    fn chain_client_error_display() {
        let err = ChainClientError::UnknownChain("solana".to_string());
        assert!(format!("{err}").contains("solana"));

        let err = ChainClientError::AllEndpointsFailed {
            chain: "ethereum".to_string(),
            last_error: "timeout".to_string(),
        };
        assert!(format!("{err}").contains("ethereum"));
        assert!(format!("{err}").contains("timeout"));
    }
}

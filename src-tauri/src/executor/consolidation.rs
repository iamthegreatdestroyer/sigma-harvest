//! Token consolidation — sweep native + ERC-20 tokens from HD-derived wallets
//! to a designated cold/destination wallet.
//!
//! SECURITY: Only public addresses and balances cross the IPC boundary.
//! Actual signing happens inside the vault module.

use crate::chain::provider::ChainClient;
use crate::chain::registry::SUPPORTED_CHAINS;
use serde::{Deserialize, Serialize};

/// ERC-20 balanceOf(address) function selector.
const BALANCE_OF_SELECTOR: &str = "0x70a08231";
/// ERC-20 transfer(address,uint256) function selector.
const TRANSFER_SELECTOR: &str = "0xa9059cbb";
/// Standard gas limit for a native ETH transfer.
const NATIVE_TRANSFER_GAS: u64 = 21_000;
/// Standard gas limit for an ERC-20 transfer.
const ERC20_TRANSFER_GAS: u64 = 65_000;

/// Configuration for a consolidation sweep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Destination wallet address (cold storage).
    pub destination: String,
    /// Chain to sweep on.
    pub chain: String,
    /// Minimum native token balance (in wei) to bother sweeping.
    pub min_native_wei: u128,
    /// Minimum ERC-20 token balance (in smallest unit) to bother sweeping.
    pub min_erc20_units: u128,
    /// Maximum gas price (in gwei) — skip sweep if gas is too expensive.
    pub max_gas_gwei: f64,
    /// ERC-20 token contract addresses to check.
    pub erc20_tokens: Vec<String>,
}

/// A single wallet's sweep-eligible balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepCandidate {
    pub wallet_address: String,
    /// Native token balance in wei.
    pub native_balance_wei: u128,
    /// ERC-20 balances: (token_address, balance_units).
    pub erc20_balances: Vec<(String, u128)>,
    /// Whether the native balance is worth sweeping after gas.
    pub native_sweepable: bool,
    /// Estimated gas cost for sweeping this wallet's native tokens (in wei).
    pub native_gas_cost_wei: u128,
}

/// Result of a consolidation plan (before execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationPlan {
    pub chain: String,
    pub destination: String,
    pub candidates: Vec<SweepCandidate>,
    pub total_native_wei: u128,
    pub total_erc20_sweeps: usize,
    pub total_gas_cost_wei: u128,
    pub skipped_dust: usize,
}

/// Result of executing a consolidation sweep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub chain: String,
    pub destination: String,
    pub native_sweeps: usize,
    pub erc20_sweeps: usize,
    pub total_value_wei: u128,
    pub total_gas_wei: u128,
    pub errors: Vec<String>,
}

/// Scan wallets and build a consolidation plan.
///
/// This queries native balances and ERC-20 balances for each wallet,
/// then determines which are worth sweeping given gas costs.
pub async fn plan_consolidation(
    config: &ConsolidationConfig,
    wallet_addresses: &[String],
    client: &ChainClient,
    gas_price_gwei: f64,
) -> Result<ConsolidationPlan, ConsolidationError> {
    if !config.destination.starts_with("0x") || config.destination.len() != 42 {
        return Err(ConsolidationError::InvalidDestination(
            config.destination.clone(),
        ));
    }

    let chain = SUPPORTED_CHAINS
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(&config.chain))
        .ok_or_else(|| ConsolidationError::UnknownChain(config.chain.clone()))?;

    if gas_price_gwei > config.max_gas_gwei {
        return Err(ConsolidationError::GasTooExpensive {
            current: gas_price_gwei,
            max: config.max_gas_gwei,
        });
    }

    let native_gas_cost_wei =
        (NATIVE_TRANSFER_GAS as f64 * gas_price_gwei * 1e9) as u128;

    let mut candidates = Vec::new();
    let mut total_native = 0u128;
    let mut total_erc20_sweeps = 0usize;
    let mut total_gas = 0u128;
    let mut skipped = 0usize;

    for addr in wallet_addresses {
        // Skip the destination — don't sweep from yourself
        if addr.eq_ignore_ascii_case(&config.destination) {
            continue;
        }

        // Fetch native balance
        let native_balance = match client.get_balance(&config.chain, addr).await {
            Ok(bal) => parse_balance_wei(&bal.balance_wei),
            Err(e) => {
                tracing::warn!("Failed to get balance for {} on {}: {}", addr, config.chain, e);
                0
            }
        };

        // Determine if native is worth sweeping (balance - gas > min threshold)
        let native_sweepable = native_balance > native_gas_cost_wei + config.min_native_wei;

        // Fetch ERC-20 balances
        let mut erc20_balances = Vec::new();
        for token in &config.erc20_tokens {
            match fetch_erc20_balance(client, chain, addr, token).await {
                Ok(balance) if balance >= config.min_erc20_units => {
                    erc20_balances.push((token.clone(), balance));
                    total_erc20_sweeps += 1;
                }
                Ok(_) => {
                    skipped += 1;
                }
                Err(e) => {
                    tracing::warn!("ERC-20 balance check failed for {} token {}: {}", addr, token, e);
                }
            }
        }

        if native_sweepable || !erc20_balances.is_empty() {
            if native_sweepable {
                total_native += native_balance - native_gas_cost_wei;
                total_gas += native_gas_cost_wei;
            }
            total_gas += erc20_balances.len() as u128
                * (ERC20_TRANSFER_GAS as f64 * gas_price_gwei * 1e9) as u128;

            candidates.push(SweepCandidate {
                wallet_address: addr.clone(),
                native_balance_wei: native_balance,
                erc20_balances,
                native_sweepable,
                native_gas_cost_wei,
            });
        } else if native_balance > 0 {
            skipped += 1;
        }
    }

    Ok(ConsolidationPlan {
        chain: config.chain.clone(),
        destination: config.destination.clone(),
        candidates,
        total_native_wei: total_native,
        total_erc20_sweeps,
        total_gas_cost_wei: total_gas,
        skipped_dust: skipped,
    })
}

/// Build the calldata for an ERC-20 transfer(address, uint256).
pub fn build_erc20_transfer_calldata(destination: &str, amount: u128) -> String {
    let dest = destination.strip_prefix("0x").unwrap_or(destination);
    let dest_padded = format!("{:0>64}", dest);
    let amount_hex = format!("{:064x}", amount);
    format!("{}{}{}", TRANSFER_SELECTOR, dest_padded, amount_hex)
}

/// Fetch an ERC-20 token balance for an address.
async fn fetch_erc20_balance(
    client: &ChainClient,
    chain: &crate::chain::registry::ChainConfig,
    wallet: &str,
    token_address: &str,
) -> Result<u128, ConsolidationError> {
    let addr = wallet.strip_prefix("0x").unwrap_or(wallet);
    let addr_padded = format!("{:0>64}", addr);
    let calldata = format!("{}{}", BALANCE_OF_SELECTOR, addr_padded);

    let params = serde_json::json!([
        {
            "to": token_address,
            "data": calldata,
        },
        "latest"
    ]);

    let result = client
        .rpc_call_public(chain, "eth_call", params)
        .await
        .map_err(|e| ConsolidationError::RpcError(e.to_string()))?;

    let hex = result.as_str().unwrap_or("0x0");
    Ok(parse_hex_u128(hex))
}

/// Parse a decimal or hex string to u128 for balance values.
fn parse_balance_wei(s: &str) -> u128 {
    if s.starts_with("0x") || s.starts_with("0X") {
        parse_hex_u128(s)
    } else {
        s.parse::<u128>().unwrap_or(0)
    }
}

/// Parse hex to u128.
fn parse_hex_u128(hex: &str) -> u128 {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u128::from_str_radix(hex, 16).unwrap_or(0)
}

#[derive(Debug, thiserror::Error)]
pub enum ConsolidationError {
    #[error("invalid destination address: {0}")]
    InvalidDestination(String),
    #[error("unknown chain: {0}")]
    UnknownChain(String),
    #[error("gas too expensive: current {current:.1} gwei > max {max:.1} gwei")]
    GasTooExpensive { current: f64, max: f64 },
    #[error("RPC error: {0}")]
    RpcError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Config validation ─────────────────────────────────────

    #[test]
    fn config_serializable() {
        let config = ConsolidationConfig {
            destination: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            chain: "ethereum".to_string(),
            min_native_wei: 1_000_000_000_000_000, // 0.001 ETH
            min_erc20_units: 1_000_000, // 1 USDC (6 decimals)
            max_gas_gwei: 30.0,
            erc20_tokens: vec!["0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let roundtrip: ConsolidationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.chain, "ethereum");
        assert_eq!(roundtrip.min_native_wei, 1_000_000_000_000_000);
    }

    // ── ERC-20 calldata building ──────────────────────────────

    #[test]
    fn build_erc20_transfer_calldata_format() {
        let dest = "0x1234567890abcdef1234567890abcdef12345678";
        let amount = 1_000_000u128; // 1 USDC
        let calldata = build_erc20_transfer_calldata(dest, amount);

        // Starts with transfer selector
        assert!(calldata.starts_with(TRANSFER_SELECTOR));
        // Contains padded destination
        assert!(calldata.contains("1234567890abcdef1234567890abcdef12345678"));
        // Total length: 10 (selector) + 64 (address) + 64 (amount) = 138
        assert_eq!(calldata.len(), 138);
    }

    #[test]
    fn build_erc20_transfer_zero_amount() {
        let dest = "0xABC";
        let calldata = build_erc20_transfer_calldata(dest, 0);
        assert!(calldata.starts_with(TRANSFER_SELECTOR));
        assert!(calldata.ends_with(&"0".repeat(64)));
    }

    #[test]
    fn build_erc20_transfer_large_amount() {
        let dest = "0xDEF";
        let amount = u128::MAX;
        let calldata = build_erc20_transfer_calldata(dest, amount);
        assert!(calldata.starts_with(TRANSFER_SELECTOR));
        // u128::MAX should produce "ffffffffffffffffffffffffffffffff" (32 hex chars padded to 64)
        assert!(calldata.contains("ffffffffffffffffffffffffffffffff"));
    }

    // ── Hex parsing ───────────────────────────────────────────

    #[test]
    fn parse_hex_u128_basic() {
        assert_eq!(parse_hex_u128("0x0"), 0);
        assert_eq!(parse_hex_u128("0x1"), 1);
        assert_eq!(parse_hex_u128("0xff"), 255);
        assert_eq!(parse_hex_u128("0xDE0B6B3A7640000"), 1_000_000_000_000_000_000); // 1 ETH
    }

    #[test]
    fn parse_hex_u128_no_prefix() {
        assert_eq!(parse_hex_u128("ff"), 255);
    }

    #[test]
    fn parse_hex_u128_invalid() {
        assert_eq!(parse_hex_u128("not_hex"), 0);
    }

    #[test]
    fn parse_balance_wei_decimal() {
        assert_eq!(parse_balance_wei("1000000000000000000"), 1_000_000_000_000_000_000);
    }

    #[test]
    fn parse_balance_wei_hex() {
        assert_eq!(parse_balance_wei("0xDE0B6B3A7640000"), 1_000_000_000_000_000_000);
    }

    // ── Plan validation ───────────────────────────────────────

    #[tokio::test]
    async fn plan_rejects_invalid_destination() {
        let config = ConsolidationConfig {
            destination: "not-an-address".to_string(),
            chain: "ethereum".to_string(),
            min_native_wei: 0,
            min_erc20_units: 0,
            max_gas_gwei: 100.0,
            erc20_tokens: vec![],
        };
        let client = ChainClient::new(1);
        let result = plan_consolidation(&config, &[], &client, 10.0).await;
        assert!(matches!(result, Err(ConsolidationError::InvalidDestination(_))));
    }

    #[tokio::test]
    async fn plan_rejects_unknown_chain() {
        let config = ConsolidationConfig {
            destination: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            chain: "solana".to_string(),
            min_native_wei: 0,
            min_erc20_units: 0,
            max_gas_gwei: 100.0,
            erc20_tokens: vec![],
        };
        let client = ChainClient::new(1);
        let result = plan_consolidation(&config, &[], &client, 10.0).await;
        assert!(matches!(result, Err(ConsolidationError::UnknownChain(_))));
    }

    #[tokio::test]
    async fn plan_rejects_high_gas() {
        let config = ConsolidationConfig {
            destination: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            chain: "ethereum".to_string(),
            min_native_wei: 0,
            min_erc20_units: 0,
            max_gas_gwei: 10.0,
            erc20_tokens: vec![],
        };
        let client = ChainClient::new(1);
        let result = plan_consolidation(&config, &[], &client, 50.0).await;
        assert!(matches!(result, Err(ConsolidationError::GasTooExpensive { .. })));
    }

    #[tokio::test]
    async fn plan_empty_wallets_succeeds() {
        let config = ConsolidationConfig {
            destination: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            chain: "ethereum".to_string(),
            min_native_wei: 0,
            min_erc20_units: 0,
            max_gas_gwei: 100.0,
            erc20_tokens: vec![],
        };
        let client = ChainClient::new(1);
        let plan = plan_consolidation(&config, &[], &client, 10.0).await.unwrap();
        assert_eq!(plan.candidates.len(), 0);
        assert_eq!(plan.total_native_wei, 0);
    }

    #[tokio::test]
    async fn plan_skips_destination_wallet() {
        let dest = "0x1234567890abcdef1234567890abcdef12345678".to_string();
        let config = ConsolidationConfig {
            destination: dest.clone(),
            chain: "ethereum".to_string(),
            min_native_wei: 0,
            min_erc20_units: 0,
            max_gas_gwei: 100.0,
            erc20_tokens: vec![],
        };
        let client = ChainClient::new(1);
        // Wallet list contains only the destination
        let plan = plan_consolidation(&config, &[dest], &client, 10.0).await.unwrap();
        assert_eq!(plan.candidates.len(), 0);
    }

    // ── SweepCandidate serializable ───────────────────────────

    #[test]
    fn sweep_candidate_serializable() {
        let candidate = SweepCandidate {
            wallet_address: "0xABC".to_string(),
            native_balance_wei: 1_000_000_000_000_000_000,
            erc20_balances: vec![("0xUSDC".to_string(), 1_000_000)],
            native_sweepable: true,
            native_gas_cost_wei: 630_000_000_000_000,
        };
        let json = serde_json::to_string(&candidate).unwrap();
        let roundtrip: SweepCandidate = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.wallet_address, "0xABC");
        assert!(roundtrip.native_sweepable);
    }

    // ── ConsolidationPlan serializable ────────────────────────

    #[test]
    fn consolidation_plan_serializable() {
        let plan = ConsolidationPlan {
            chain: "ethereum".to_string(),
            destination: "0xDEST".to_string(),
            candidates: vec![],
            total_native_wei: 0,
            total_erc20_sweeps: 0,
            total_gas_cost_wei: 0,
            skipped_dust: 3,
        };
        let json = serde_json::to_string(&plan).unwrap();
        let roundtrip: ConsolidationPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.skipped_dust, 3);
    }

    // ── ConsolidationResult serializable ──────────────────────

    #[test]
    fn consolidation_result_serializable() {
        let result = ConsolidationResult {
            chain: "arbitrum".to_string(),
            destination: "0xDEST".to_string(),
            native_sweeps: 5,
            erc20_sweeps: 2,
            total_value_wei: 5_000_000_000_000_000_000,
            total_gas_wei: 100_000_000_000_000,
            errors: vec!["one wallet failed".to_string()],
        };
        let json = serde_json::to_string(&result).unwrap();
        let roundtrip: ConsolidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.native_sweeps, 5);
        assert_eq!(roundtrip.erc20_sweeps, 2);
        assert_eq!(roundtrip.errors.len(), 1);
    }

    // ── Error display ─────────────────────────────────────────

    #[test]
    fn error_invalid_destination_display() {
        let err = ConsolidationError::InvalidDestination("bad".to_string());
        assert!(format!("{}", err).contains("bad"));
    }

    #[test]
    fn error_gas_too_expensive_display() {
        let err = ConsolidationError::GasTooExpensive {
            current: 50.0,
            max: 10.0,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("50.0"));
        assert!(msg.contains("10.0"));
    }
}

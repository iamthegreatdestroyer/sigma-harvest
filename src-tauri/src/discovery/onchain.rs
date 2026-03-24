//! On-chain event monitoring for airdrop contracts and token claims.
//! Uses eth_getLogs to detect ERC-20 Transfer events and new claim patterns.

use super::{DiscoveryError, DiscoverySource, RawOpportunity};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Keccak256 of Transfer(address,address,uint256) — the ERC-20 Transfer topic.
const TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// Keccak256 of Claim(address,uint256) — common airdrop claim topic.
const CLAIM_TOPIC: &str =
    "0x47cee97cb7acd717b3c0aa1435d004cd5b3c8c57d70dbceb4e4458bbd60e39d4";

pub struct OnChainSource {
    pub rpc_url: String,
    pub chain: String,
    /// Contracts known to be airdrop distributors.
    pub watchlist: Vec<String>,
    /// How many blocks to look back (default: ~100 blocks ≈ 20 min on Ethereum).
    pub lookback_blocks: u64,
}

impl OnChainSource {
    pub fn new(rpc_url: &str, chain: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            chain: chain.to_string(),
            watchlist: Vec::new(),
            lookback_blocks: 100,
        }
    }

    pub fn with_watchlist(mut self, contracts: Vec<String>) -> Self {
        self.watchlist = contracts;
        self
    }
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

impl DiscoverySource for OnChainSource {
    fn name(&self) -> &str {
        "onchain"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        if self.watchlist.is_empty() {
            return Ok(Vec::new());
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| DiscoveryError::Parse(e.to_string()))?;

        // Get latest block number
        let block_num = get_block_number(&client, &self.rpc_url).await?;
        let from_block = block_num.saturating_sub(self.lookback_blocks);

        let mut opportunities = Vec::new();

        // Query Transfer events on each watched contract
        for contract in &self.watchlist {
            let logs = get_logs(
                &client,
                &self.rpc_url,
                contract,
                &[TRANSFER_TOPIC, CLAIM_TOPIC],
                from_block,
                block_num,
            )
            .await;

            match logs {
                Ok(entries) => {
                    for log in entries {
                        if let Some(opp) = parse_log_to_opportunity(&log, &self.chain, contract) {
                            opportunities.push(opp);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("on-chain log fetch failed for {}: {}", contract, e);
                }
            }
        }

        Ok(opportunities)
    }
}

async fn get_block_number(client: &Client, rpc_url: &str) -> Result<u64, DiscoveryError> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let resp: JsonRpcResponse = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(DiscoveryError::Network)?
        .json::<JsonRpcResponse>()
        .await
        .map_err(|e: reqwest::Error| DiscoveryError::Parse(e.to_string()))?;

    let hex = resp
        .result
        .as_ref()
        .and_then(|v: &serde_json::Value| v.as_str())
        .ok_or_else(|| DiscoveryError::Parse("no block number".to_string()))?;

    parse_hex_u64(hex).ok_or_else(|| DiscoveryError::Parse("invalid hex".to_string()))
}

async fn get_logs(
    client: &Client,
    rpc_url: &str,
    contract: &str,
    topics: &[&str],
    from_block: u64,
    to_block: u64,
) -> Result<Vec<serde_json::Value>, DiscoveryError> {
    let topic_filter: Vec<serde_json::Value> =
        topics.iter().map(|t| serde_json::json!(t)).collect();

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getLogs",
        "params": [{
            "address": contract,
            "topics": [topic_filter],
            "fromBlock": format!("0x{:x}", from_block),
            "toBlock": format!("0x{:x}", to_block),
        }],
        "id": 1
    });

    let resp: JsonRpcResponse = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(DiscoveryError::Network)?
        .json::<JsonRpcResponse>()
        .await
        .map_err(|e: reqwest::Error| DiscoveryError::Parse(e.to_string()))?;

    if let Some(err) = resp.error {
        return Err(DiscoveryError::Parse(format!("RPC error: {}", err)));
    }

    let logs = resp
        .result
        .and_then(|v: serde_json::Value| v.as_array().cloned())
        .unwrap_or_default();

    Ok(logs)
}

fn parse_log_to_opportunity(
    log: &serde_json::Value,
    chain: &str,
    contract: &str,
) -> Option<RawOpportunity> {
    let topics = log.get("topics")?.as_array()?;
    if topics.is_empty() {
        return None;
    }

    let topic0 = topics[0].as_str()?;

    let (opportunity_type, type_label) = if topic0 == TRANSFER_TOPIC {
        (super::OpportunityType::Airdrop, "Airdrop")
    } else if topic0 == CLAIM_TOPIC {
        (super::OpportunityType::Faucet, "Claim")
    } else {
        return None;
    };

    let tx_hash = log.get("transactionHash").and_then(|v| v.as_str()).unwrap_or("unknown");
    let block = log
        .get("blockNumber")
        .and_then(|v| v.as_str())
        .and_then(parse_hex_u64)
        .unwrap_or(0);

    Some(RawOpportunity {
        source: "onchain".to_string(),
        chain: chain.to_string(),
        opportunity_type,
        title: format!("{} event at block {}", type_label, block),
        description: format!("On-chain {} detected on contract {} (tx: {})", type_label, contract, tx_hash),
        url: None,
        contract_address: Some(contract.to_string()),
        estimated_value_usd: None,
        gas_cost_estimate: None,
        deadline: None,
        discovered_at: Utc::now().to_rfc3339(),
    })
}

fn parse_hex_u64(hex: &str) -> Option<u64> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u64::from_str_radix(hex, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onchain_source_name() {
        let src = OnChainSource::new("https://rpc.example.com", "ethereum");
        assert_eq!(src.name(), "onchain");
    }

    #[test]
    fn with_watchlist_sets_contracts() {
        let src = OnChainSource::new("https://rpc.example.com", "ethereum")
            .with_watchlist(vec!["0xAbC".to_string()]);
        assert_eq!(src.watchlist.len(), 1);
    }

    #[tokio::test]
    async fn empty_watchlist_returns_empty() {
        let src = OnChainSource::new("https://rpc.example.com", "ethereum");
        let result = src.discover().await.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_hex_u64_valid() {
        assert_eq!(parse_hex_u64("0x1"), Some(1));
        assert_eq!(parse_hex_u64("0xff"), Some(255));
        assert_eq!(parse_hex_u64("0x100"), Some(256));
        assert_eq!(parse_hex_u64("ff"), Some(255));
    }

    #[test]
    fn parse_hex_u64_invalid() {
        assert_eq!(parse_hex_u64("not_hex"), None);
    }

    #[test]
    fn parse_log_transfer_event() {
        let log = serde_json::json!({
            "topics": [TRANSFER_TOPIC, "0x000000000000000000000000sender", "0x000000000000000000000000receiver"],
            "transactionHash": "0xdeadbeef",
            "blockNumber": "0x100",
        });
        let opp = parse_log_to_opportunity(&log, "ethereum", "0xContract").unwrap();
        assert!(matches!(opp.opportunity_type, crate::discovery::OpportunityType::Airdrop));
        assert_eq!(opp.chain, "ethereum");
        assert!(opp.title.contains("256"));
        assert!(opp.description.contains("0xContract"));
    }

    #[test]
    fn parse_log_claim_event() {
        let log = serde_json::json!({
            "topics": [CLAIM_TOPIC],
            "transactionHash": "0xabc",
            "blockNumber": "0x10",
        });
        let opp = parse_log_to_opportunity(&log, "arbitrum", "0xAirdrop").unwrap();
        assert!(matches!(opp.opportunity_type, crate::discovery::OpportunityType::Faucet));
        assert_eq!(opp.chain, "arbitrum");
    }

    #[test]
    fn parse_log_unknown_topic_returns_none() {
        let log = serde_json::json!({
            "topics": ["0x0000000000000000000000000000000000000000000000000000000000000000"],
            "transactionHash": "0xabc",
            "blockNumber": "0x1",
        });
        assert!(parse_log_to_opportunity(&log, "ethereum", "0xC").is_none());
    }

    #[test]
    fn parse_log_empty_topics_returns_none() {
        let log = serde_json::json!({
            "topics": [],
        });
        assert!(parse_log_to_opportunity(&log, "ethereum", "0xC").is_none());
    }

    #[test]
    fn parse_log_no_topics_returns_none() {
        let log = serde_json::json!({
            "data": "0x"
        });
        assert!(parse_log_to_opportunity(&log, "ethereum", "0xC").is_none());
    }
}

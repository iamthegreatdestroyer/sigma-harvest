//! DappRadar API integration for airdrop discovery.
//! Fetches active airdrops from the DappRadar public rankings endpoint.

use super::{DiscoveryError, DiscoverySource, OpportunityType, RawOpportunity};
use chrono::Utc;
use serde::Deserialize;

pub struct DappRadarSource {
    pub api_key: Option<String>,
}

impl DappRadarSource {
    /// Create a new DappRadarSource, reading `DAPPRADAR_API_KEY` from environment.
    /// If the env var is absent, logs a warning and continues with no API key.
    pub fn from_env() -> Self {
        let api_key = std::env::var("DAPPRADAR_API_KEY").ok();
        if api_key.is_none() {
            tracing::warn!("DAPPRADAR_API_KEY not set; DappRadar requests will use unauthenticated access");
        }
        Self { api_key }
    }
}

/// DappRadar airdrop listing response shape.
#[derive(Debug, Deserialize)]
struct DappRadarResponse {
    #[serde(default)]
    results: Vec<DappRadarItem>,
}

#[derive(Debug, Deserialize)]
struct DappRadarItem {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    link: String,
    #[serde(default)]
    chains: Vec<String>,
    #[serde(default, rename = "rewardValue")]
    reward_value: Option<f64>,
    #[serde(default)]
    category: String,
}

impl DiscoverySource for DappRadarSource {
    fn name(&self) -> &str {
        "dappradar"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        let client = reqwest::Client::builder()
            .user_agent("SigmaHarvest/1.0")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(DiscoveryError::Network)?;

        let mut req = client.get("https://apis.dappradar.com/v2/airdrops");
        if let Some(ref key) = self.api_key {
            req = req.header("X-API-KEY", key);
        }

        let response = req.send().await.map_err(DiscoveryError::Network)?;

        if !response.status().is_success() {
            return Err(DiscoveryError::Parse(format!(
                "DappRadar API returned {}",
                response.status()
            )));
        }

        let data: DappRadarResponse = response
            .json()
            .await
            .map_err(|e| DiscoveryError::Parse(format!("DappRadar JSON: {e}")))?;

        let now = Utc::now().to_rfc3339();
        let opportunities = data
            .results
            .into_iter()
            .map(|item| {
                let chain = item
                    .chains
                    .first()
                    .map(|c| normalize_chain(c))
                    .unwrap_or_else(|| "unknown".to_string());

                let opportunity_type = match item.category.to_lowercase().as_str() {
                    "quest" | "quests" => OpportunityType::Quest,
                    "faucet" => OpportunityType::Faucet,
                    "free mint" | "nft" => OpportunityType::FreeMint,
                    _ => OpportunityType::Airdrop,
                };

                RawOpportunity {
                    source: "dappradar".to_string(),
                    chain,
                    opportunity_type,
                    title: item.title,
                    description: item.description,
                    url: if item.link.is_empty() { None } else { Some(item.link) },
                    contract_address: None,
                    estimated_value_usd: item.reward_value,
                    gas_cost_estimate: None,
                    deadline: None,
                    discovered_at: now.clone(),
                }
            })
            .collect();

        Ok(opportunities)
    }
}

fn normalize_chain(raw: &str) -> String {
    match raw.to_lowercase().as_str() {
        "ethereum" | "eth" => "ethereum".to_string(),
        "arbitrum" | "arb" => "arbitrum".to_string(),
        "optimism" | "op" => "optimism".to_string(),
        "base" => "base".to_string(),
        "polygon" | "matic" => "polygon".to_string(),
        "zksync" | "zksync era" => "zksync".to_string(),
        other => other.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dappradar_source_name() {
        let source = DappRadarSource { api_key: None };
        assert_eq!(source.name(), "dappradar");
    }

    #[test]
    fn normalize_chain_ethereum() {
        assert_eq!(normalize_chain("ETH"), "ethereum");
        assert_eq!(normalize_chain("ethereum"), "ethereum");
    }

    #[test]
    fn normalize_chain_arbitrum() {
        assert_eq!(normalize_chain("ARB"), "arbitrum");
        assert_eq!(normalize_chain("Arbitrum"), "arbitrum");
    }

    #[test]
    fn normalize_chain_unknown() {
        assert_eq!(normalize_chain("fantom"), "fantom");
    }

    #[test]
    fn normalize_chain_polygon() {
        assert_eq!(normalize_chain("MATIC"), "polygon");
    }

    #[test]
    fn deserialize_empty_response() {
        let json = r#"{"results": []}"#;
        let resp: DappRadarResponse = serde_json::from_str(json).unwrap();
        assert!(resp.results.is_empty());
    }

    #[test]
    fn deserialize_item_with_defaults() {
        let json = r#"{"title": "Test", "description": "", "link": "", "chains": [], "category": ""}"#;
        let item: DappRadarItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.title, "Test");
        assert!(item.chains.is_empty());
        assert!(item.reward_value.is_none());
    }

    #[test]
    fn deserialize_item_with_reward() {
        let json = r#"{"title": "Big Drop", "description": "Free!", "link": "https://x.com", "chains": ["ethereum"], "rewardValue": 100.0, "category": "airdrop"}"#;
        let item: DappRadarItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.reward_value, Some(100.0));
        assert_eq!(item.chains[0], "ethereum");
    }
}

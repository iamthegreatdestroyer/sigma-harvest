pub mod dappradar;
pub mod galxe;
pub mod onchain;
pub mod rss;
pub mod social;

use serde::{Deserialize, Serialize};

/// A raw opportunity discovered from any source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawOpportunity {
    pub source: String,
    pub chain: String,
    pub opportunity_type: OpportunityType,
    pub title: String,
    pub description: String,
    pub url: Option<String>,
    pub contract_address: Option<String>,
    pub estimated_value_usd: Option<f64>,
    pub gas_cost_estimate: Option<f64>,
    pub deadline: Option<String>,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    Airdrop,
    Faucet,
    FreeMint,
    Quest,
    LiquidityBonus,
    BridgeIncentive,
    Retroactive,
}

/// Trait all discovery sources must implement.
#[allow(async_fn_in_trait)]
pub trait DiscoverySource {
    fn name(&self) -> &str;
    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("discovery not yet implemented for {name}")]
    NotImplemented { name: String },
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw_opportunity() -> RawOpportunity {
        RawOpportunity {
            source: "rss".to_string(),
            chain: "ethereum".to_string(),
            opportunity_type: OpportunityType::Airdrop,
            title: "Test Airdrop".to_string(),
            description: "Free tokens for early users".to_string(),
            url: Some("https://example.com".to_string()),
            contract_address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            estimated_value_usd: Some(10.0),
            gas_cost_estimate: Some(0.50),
            deadline: Some("2026-12-31T23:59:59Z".to_string()),
            discovered_at: "2026-03-22T12:00:00Z".to_string(),
        }
    }

    // ── Type construction & serialization ─────────────────────════

    #[test]
    fn raw_opportunity_serializable() {
        let opp = make_raw_opportunity();
        let json = serde_json::to_string(&opp).unwrap();
        let deserialized: RawOpportunity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "Test Airdrop");
        assert_eq!(deserialized.source, "rss");
    }

    #[test]
    fn raw_opportunity_all_fields() {
        let opp = make_raw_opportunity();
        assert_eq!(opp.chain, "ethereum");
        assert!(opp.url.is_some());
        assert!(opp.contract_address.is_some());
        assert!(opp.estimated_value_usd.is_some());
        assert!(opp.gas_cost_estimate.is_some());
        assert!(opp.deadline.is_some());
    }

    #[test]
    fn raw_opportunity_optional_fields_can_be_none() {
        let opp = RawOpportunity {
            source: "social".to_string(),
            chain: "base".to_string(),
            opportunity_type: OpportunityType::Quest,
            title: "Minimal".to_string(),
            description: "".to_string(),
            url: None,
            contract_address: None,
            estimated_value_usd: None,
            gas_cost_estimate: None,
            deadline: None,
            discovered_at: "2026-01-01".to_string(),
        };
        let json = serde_json::to_string(&opp).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn opportunity_type_all_variants_serializable() {
        let variants = vec![
            OpportunityType::Airdrop,
            OpportunityType::Faucet,
            OpportunityType::FreeMint,
            OpportunityType::Quest,
            OpportunityType::LiquidityBonus,
            OpportunityType::BridgeIncentive,
            OpportunityType::Retroactive,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            assert!(!json.is_empty());
        }
    }

    // ── Discovery source stubs ────────────────────────────────════

    #[tokio::test]
    async fn rss_source_with_no_feeds_returns_empty() {
        let source = rss::RssSource {
            feed_urls: vec![],
        };
        assert_eq!(source.name(), "rss");
        let result = source.discover().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn dappradar_source_returns_not_implemented() {
        let source = dappradar::DappRadarSource { api_key: None };
        assert_eq!(source.name(), "dappradar");
        let result = source.discover().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn galxe_source_name_is_correct() {
        let source = galxe::GalxeSource;
        assert_eq!(source.name(), "galxe");
    }

    #[tokio::test]
    async fn onchain_source_empty_watchlist_returns_empty() {
        let source = onchain::OnChainSource {
            rpc_url: "https://eth.llamarpc.com".into(),
            chain: "ethereum".to_string(),
            watchlist: vec![],
            lookback_blocks: 100,
        };
        assert_eq!(source.name(), "onchain");
        let result = source.discover().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn social_source_no_token_returns_empty() {
        let source = social::SocialSource {
            bearer_token: None,
        };
        assert_eq!(source.name(), "social");
        let result = source.discover().await.unwrap();
        assert!(result.is_empty());
    }

    // ── Error formatting ──────────────────────────────────────════

    #[test]
    fn discovery_error_not_implemented_display() {
        let err = DiscoveryError::NotImplemented {
            name: "test_source".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("test_source"));
    }

    #[test]
    fn discovery_error_parse_display() {
        let err = DiscoveryError::Parse("bad JSON".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("bad JSON"));
    }

    // ── Clone / Debug ─────────────────────────────────────════════

    #[test]
    fn raw_opportunity_is_clone() {
        let opp = make_raw_opportunity();
        let clone = opp.clone();
        assert_eq!(clone.title, opp.title);
    }

    #[test]
    fn raw_opportunity_is_debug() {
        let opp = make_raw_opportunity();
        let debug = format!("{:?}", opp);
        assert!(debug.contains("Test Airdrop"));
    }
}

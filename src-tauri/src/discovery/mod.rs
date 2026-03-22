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

//! On-chain event monitoring for airdrop contracts and token claims.

use super::{DiscoveryError, DiscoverySource, RawOpportunity};

pub struct OnChainSource {
    pub rpc_url: String,
}

impl DiscoverySource for OnChainSource {
    fn name(&self) -> &str {
        "onchain"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        // TODO: Subscribe to ERC-20 Transfer events, detect claim contracts
        Err(DiscoveryError::NotImplemented {
            name: self.name().to_string(),
        })
    }
}

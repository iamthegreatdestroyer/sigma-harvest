//! DappRadar API integration for airdrop discovery.

use super::{DiscoveryError, DiscoverySource, RawOpportunity};

pub struct DappRadarSource {
    pub api_key: Option<String>,
}

impl DiscoverySource for DappRadarSource {
    fn name(&self) -> &str {
        "dappradar"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        // TODO: Implement DappRadar airdrop API
        Err(DiscoveryError::NotImplemented {
            name: self.name().to_string(),
        })
    }
}

//! Social media signal detection (Twitter/X keyword monitoring).

use super::{DiscoveryError, DiscoverySource, RawOpportunity};

pub struct SocialSource {
    pub bearer_token: Option<String>,
}

impl DiscoverySource for SocialSource {
    fn name(&self) -> &str {
        "social"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        // TODO: Twitter/X API search for airdrop signals
        Err(DiscoveryError::NotImplemented {
            name: self.name().to_string(),
        })
    }
}

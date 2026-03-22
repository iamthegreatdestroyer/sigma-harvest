//! RSS/Atom feed scraping for airdrop announcements.

use super::{DiscoveryError, DiscoverySource, RawOpportunity};

pub struct RssSource {
    pub feed_urls: Vec<String>,
}

impl DiscoverySource for RssSource {
    fn name(&self) -> &str {
        "rss"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        // TODO: Fetch and parse RSS feeds using feed-rs
        Err(DiscoveryError::NotImplemented {
            name: self.name().to_string(),
        })
    }
}

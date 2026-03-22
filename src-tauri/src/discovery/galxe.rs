//! Galxe GraphQL campaign discovery.

use super::{DiscoveryError, DiscoverySource, RawOpportunity};

pub struct GalxeSource;

impl DiscoverySource for GalxeSource {
    fn name(&self) -> &str {
        "galxe"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        // TODO: Implement Galxe GraphQL campaign queries
        Err(DiscoveryError::NotImplemented {
            name: self.name().to_string(),
        })
    }
}

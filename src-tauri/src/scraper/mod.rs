pub mod parsers;
pub mod pipeline;

/// Configuration for the scraper pipeline.
pub struct ScraperConfig {
    pub user_agent: String,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_concurrent: usize,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            min_delay_ms: 1000,
            max_delay_ms: 3000,
            max_concurrent: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = ScraperConfig::default();
        assert_eq!(config.min_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 3000);
        assert_eq!(config.max_concurrent, 3);
        assert!(config.user_agent.contains("Mozilla"));
    }

    #[test]
    fn custom_config() {
        let config = ScraperConfig {
            user_agent: "SigmaHarvest/1.0".to_string(),
            min_delay_ms: 500,
            max_delay_ms: 1500,
            max_concurrent: 5,
        };
        assert_eq!(config.min_delay_ms, 500);
        assert_eq!(config.max_concurrent, 5);
    }

    #[test]
    fn min_delay_less_than_max() {
        let config = ScraperConfig::default();
        assert!(config.min_delay_ms < config.max_delay_ms);
    }
}

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

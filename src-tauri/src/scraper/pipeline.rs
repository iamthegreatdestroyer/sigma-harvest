//! The scraper pipeline orchestrates discovery source execution.

use super::ScraperConfig;

pub struct ScraperPipeline {
    config: ScraperConfig,
    running: bool,
}

impl ScraperPipeline {
    pub fn new(config: ScraperConfig) -> Self {
        Self {
            config,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        tracing::info!("Scraper pipeline started (delay: {}–{}ms)", 
            self.config.min_delay_ms, self.config.max_delay_ms);
    }

    pub fn stop(&mut self) {
        self.running = false;
        tracing::info!("Scraper pipeline stopped");
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

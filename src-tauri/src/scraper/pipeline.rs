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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pipeline_not_running() {
        let p = ScraperPipeline::new(ScraperConfig::default());
        assert!(!p.is_running());
    }

    #[test]
    fn start_sets_running() {
        let mut p = ScraperPipeline::new(ScraperConfig::default());
        p.start();
        assert!(p.is_running());
    }

    #[test]
    fn stop_clears_running() {
        let mut p = ScraperPipeline::new(ScraperConfig::default());
        p.start();
        assert!(p.is_running());
        p.stop();
        assert!(!p.is_running());
    }

    #[test]
    fn start_stop_cycle() {
        let mut p = ScraperPipeline::new(ScraperConfig::default());
        for _ in 0..10 {
            p.start();
            assert!(p.is_running());
            p.stop();
            assert!(!p.is_running());
        }
    }

    #[test]
    fn custom_config_pipeline() {
        let config = ScraperConfig {
            user_agent: "TestBot/1.0".to_string(),
            min_delay_ms: 100,
            max_delay_ms: 200,
            max_concurrent: 10,
        };
        let p = ScraperPipeline::new(config);
        assert!(!p.is_running());
    }
}

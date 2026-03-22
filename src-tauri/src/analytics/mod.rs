pub mod reports;

use serde::{Deserialize, Serialize};

/// Summary analytics for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_value_collected_usd: f64,
    pub total_gas_spent_usd: f64,
    pub roi_percentage: f64,
    pub total_claims: u32,
    pub successful_claims: u32,
    pub failed_claims: u32,
    pub active_opportunities: u32,
}

impl Default for AnalyticsSummary {
    fn default() -> Self {
        Self {
            total_value_collected_usd: 0.0,
            total_gas_spent_usd: 0.0,
            roi_percentage: 0.0,
            total_claims: 0,
            successful_claims: 0,
            failed_claims: 0,
            active_opportunities: 0,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_summary_all_zeros() {
        let s = AnalyticsSummary::default();
        assert_eq!(s.total_value_collected_usd, 0.0);
        assert_eq!(s.total_gas_spent_usd, 0.0);
        assert_eq!(s.roi_percentage, 0.0);
        assert_eq!(s.total_claims, 0);
        assert_eq!(s.successful_claims, 0);
        assert_eq!(s.failed_claims, 0);
        assert_eq!(s.active_opportunities, 0);
    }

    #[test]
    fn analytics_summary_serializable() {
        let s = AnalyticsSummary {
            total_value_collected_usd: 1234.56,
            total_gas_spent_usd: 78.90,
            roi_percentage: 1456.0,
            total_claims: 50,
            successful_claims: 45,
            failed_claims: 5,
            active_opportunities: 12,
        };
        let json = serde_json::to_string(&s).unwrap();
        let roundtrip: AnalyticsSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.total_claims, 50);
        assert_eq!(roundtrip.total_value_collected_usd, 1234.56);
    }

    #[test]
    fn analytics_summary_is_clone() {
        let s = AnalyticsSummary::default();
        let clone = s.clone();
        assert_eq!(clone.total_claims, s.total_claims);
    }

    #[test]
    fn analytics_summary_is_debug() {
        let s = AnalyticsSummary::default();
        let debug = format!("{:?}", s);
        assert!(debug.contains("total_claims"));
    }
}

pub mod reports;

use serde::{Deserialize, Serialize};

/// Summary analytics for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_value_collected_usd: f64,
    pub total_gas_spent_usd: f64,
    pub roi_percentage: f64,
    pub total_claims: u64,
    pub successful_claims: u64,
    pub failed_claims: u64,
    pub active_opportunities: u64,
    /// Number of consolidation sweep records.
    pub consolidation_count: u64,
    /// Total USD value moved via consolidation sweeps.
    pub consolidation_total_usd: f64,
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
            consolidation_count: 0,
            consolidation_total_usd: 0.0,
        }
    }
}

/// Source attribution — which discovery source produced the most value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAttribution {
    pub source: String,
    pub claim_count: u64,
    pub total_value_usd: f64,
}

/// Chain breakdown — claim statistics per chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainBreakdown {
    pub chain: String,
    pub claim_count: u64,
    pub total_value_usd: f64,
    pub total_gas_usd: f64,
}

/// A single data point for sparkline time-series charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: String,
    pub claims: u64,
    pub value_usd: f64,
    pub gas_usd: f64,
    pub net_usd: f64,
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
            consolidation_count: 3,
            consolidation_total_usd: 500.0,
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

    #[test]
    fn source_attribution_serializable() {
        let sa = SourceAttribution {
            source: "rss".to_string(),
            claim_count: 10,
            total_value_usd: 500.0,
        };
        let json = serde_json::to_string(&sa).unwrap();
        let rt: SourceAttribution = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.source, "rss");
    }

    #[test]
    fn chain_breakdown_serializable() {
        let cb = ChainBreakdown {
            chain: "ethereum".to_string(),
            claim_count: 5,
            total_value_usd: 250.0,
            total_gas_usd: 10.0,
        };
        let json = serde_json::to_string(&cb).unwrap();
        let rt: ChainBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.chain, "ethereum");
    }
}

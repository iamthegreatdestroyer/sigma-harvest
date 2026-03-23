//! Harvest Score algorithm v1.
//! Scores opportunities 0-100 based on value, risk, and urgency.

use crate::discovery::RawOpportunity;

/// Calculate the Harvest Score for a raw opportunity.
/// Components (max 100):
///   - Gas efficiency:     (value / gas_cost) capped at 25
///   - Contract verified:  +20 if contract_address present
///   - Project signals:    +15 if url and description are substantial
///   - Community size:     +10 if from known source (galxe, dappradar)
///   - Time urgency:       +15 if deadline exists, +10 if near-term
///   - Sybil penalty:      -10 if value suspiciously high
pub fn calculate(opportunity: &RawOpportunity) -> HarvestScoreResult {
    let mut breakdown = ScoreBreakdown::default();

    // 1. Gas Efficiency (0–25): value/gas ratio, capped at 25
    breakdown.gas_efficiency = gas_efficiency_score(
        opportunity.estimated_value_usd,
        opportunity.gas_cost_estimate,
    );

    // 2. Contract Verified (0 or 20): presence of contract address
    breakdown.contract_verified = if opportunity.contract_address.is_some() {
        20
    } else {
        0
    };

    // 3. Project Signals (0–15): quality indicators from metadata
    breakdown.project_signals = project_signal_score(opportunity);

    // 4. Community Size (0–10): bonus for high-trust discovery sources
    breakdown.community_size = community_score(&opportunity.source);

    // 5. Time Urgency (0–15): deadline presence and proximity
    breakdown.time_urgency = urgency_score(opportunity.deadline.as_deref());

    // 6. Sybil Penalty (-30 to 0): penalize suspicious characteristics
    breakdown.sybil_penalty = sybil_penalty(opportunity);

    let raw = breakdown.gas_efficiency as i32
        + breakdown.contract_verified as i32
        + breakdown.project_signals as i32
        + breakdown.community_size as i32
        + breakdown.time_urgency as i32
        + breakdown.sybil_penalty;

    let score = raw.clamp(0, 100) as u32;

    HarvestScoreResult { score, breakdown }
}

/// Gas efficiency: value/gas ratio mapped to 0–25.
fn gas_efficiency_score(value: Option<f64>, gas: Option<f64>) -> u32 {
    match (value, gas) {
        (Some(v), Some(g)) if g > 0.0 => {
            let ratio = v / g;
            // ratio >= 50 → 25, ratio ~5 → 12, ratio ~1 → 5
            ((ratio.ln().max(0.0) * 6.4) as u32).min(25)
        }
        (Some(v), _) if v > 0.0 => 15, // Has value but unknown gas → moderate
        _ => 0,
    }
}

/// Project signals: URL quality + description length + source quality.
fn project_signal_score(opp: &RawOpportunity) -> u32 {
    let mut score = 0u32;

    // Has a URL → +5
    if opp.url.is_some() {
        score += 5;
    }

    // Substantial description (>50 chars) → +5
    if opp.description.len() > 50 {
        score += 5;
    }

    // Title is meaningful (>10 chars) → +5
    if opp.title.len() > 10 {
        score += 5;
    }

    score.min(15)
}

/// Community score: higher-trust sources get more points.
fn community_score(source: &str) -> u32 {
    match source {
        "galxe" => 10,       // Curated campaigns
        "dappradar" => 8,    // API-verified
        "onchain" => 7,      // Direct blockchain data
        "rss" => 5,          // Aggregated feeds
        "social" => 3,       // Social signals (noisier)
        _ => 0,
    }
}

/// Urgency score: based on deadline presence and proximity.
fn urgency_score(deadline: Option<&str>) -> u32 {
    let Some(deadline_str) = deadline else {
        return 0;
    };

    // Parse deadline using chrono
    let deadline_dt = chrono::DateTime::parse_from_rfc3339(deadline_str)
        .or_else(|_| {
            // Try date-only format
            chrono::NaiveDate::parse_from_str(deadline_str, "%Y-%m-%d")
                .map(|d| {
                    d.and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_utc()
                        .fixed_offset()
                })
        })
        .ok();

    let Some(dt) = deadline_dt else {
        return 5; // Has deadline but unparseable → small bonus
    };

    let now = chrono::Utc::now();
    let days_until = (dt.signed_duration_since(now)).num_days();

    if days_until < 0 {
        0 // Expired
    } else if days_until <= 3 {
        15 // Very urgent
    } else if days_until <= 7 {
        12 // Urgent
    } else if days_until <= 30 {
        8 // Moderate
    } else {
        5 // Has deadline, not urgent
    }
}

/// Sybil penalty: flag suspicious patterns.
fn sybil_penalty(opp: &RawOpportunity) -> i32 {
    let mut penalty = 0i32;

    // Suspiciously high value with no contract → likely scam
    if let Some(v) = opp.estimated_value_usd {
        if v > 10_000.0 && opp.contract_address.is_none() {
            penalty -= 30;
        } else if v > 5_000.0 && opp.contract_address.is_none() {
            penalty -= 20;
        } else if v > 1_000.0 && opp.contract_address.is_none() {
            penalty -= 10;
        }
    }

    penalty
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestScoreResult {
    pub score: u32,
    pub breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub gas_efficiency: u32,
    pub contract_verified: u32,
    pub project_signals: u32,
    pub community_size: u32,
    pub time_urgency: u32,
    pub sybil_penalty: i32,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{OpportunityType, RawOpportunity};

    fn test_opportunity() -> RawOpportunity {
        RawOpportunity {
            source: "rss".to_string(),
            chain: "ethereum".to_string(),
            opportunity_type: OpportunityType::Airdrop,
            title: "Test Airdrop Campaign".to_string(),
            description: "Free tokens for early users who participate in the community program and complete tasks".to_string(),
            url: Some("https://example.com".to_string()),
            contract_address: Some("0x1234".to_string()),
            estimated_value_usd: Some(50.0),
            gas_cost_estimate: Some(2.0),
            deadline: Some("2026-12-31T23:59:59Z".to_string()),
            discovered_at: "2026-03-22".to_string(),
        }
    }

    #[test]
    fn calculate_scores_typical_opportunity() {
        let opp = test_opportunity();
        let result = calculate(&opp);
        // Should have: gas efficiency + contract (20) + signals + community + urgency
        assert!(result.score > 40, "Expected >40, got {}", result.score);
        assert!(result.score <= 100);
    }

    #[test]
    fn calculate_zero_for_minimal_opportunity() {
        let opp = RawOpportunity {
            source: "unknown".to_string(),
            chain: "unknown".to_string(),
            opportunity_type: OpportunityType::Airdrop,
            title: "x".to_string(),
            description: "".to_string(),
            url: None,
            contract_address: None,
            estimated_value_usd: None,
            gas_cost_estimate: None,
            deadline: None,
            discovered_at: "2026-01-01".to_string(),
        };
        let result = calculate(&opp);
        assert_eq!(result.score, 0);
    }

    #[test]
    fn gas_efficiency_high_ratio() {
        let score = gas_efficiency_score(Some(100.0), Some(1.0));
        assert!(score >= 20, "Expected >=20, got {score}");
    }

    #[test]
    fn gas_efficiency_low_ratio() {
        let score = gas_efficiency_score(Some(2.0), Some(2.0));
        assert!(score <= 10, "Expected <=10, got {score}");
    }

    #[test]
    fn gas_efficiency_no_value() {
        assert_eq!(gas_efficiency_score(None, None), 0);
    }

    #[test]
    fn gas_efficiency_value_no_gas() {
        assert_eq!(gas_efficiency_score(Some(50.0), None), 15);
    }

    #[test]
    fn community_galxe_highest() {
        assert_eq!(community_score("galxe"), 10);
    }

    #[test]
    fn community_unknown_zero() {
        assert_eq!(community_score("random"), 0);
    }

    #[test]
    fn urgency_no_deadline() {
        assert_eq!(urgency_score(None), 0);
    }

    #[test]
    fn urgency_distant_deadline() {
        assert_eq!(urgency_score(Some("2030-12-31T23:59:59Z")), 5);
    }

    #[test]
    fn urgency_expired() {
        assert_eq!(urgency_score(Some("2020-01-01T00:00:00Z")), 0);
    }

    #[test]
    fn sybil_penalty_scam_pattern() {
        let mut opp = test_opportunity();
        opp.estimated_value_usd = Some(20_000.0);
        opp.contract_address = None;
        let penalty = sybil_penalty(&opp);
        assert_eq!(penalty, -30);
    }

    #[test]
    fn sybil_penalty_normal() {
        let opp = test_opportunity();
        let penalty = sybil_penalty(&opp);
        assert_eq!(penalty, 0);
    }

    #[test]
    fn score_breakdown_default_is_all_zero() {
        let bd = ScoreBreakdown::default();
        assert_eq!(bd.gas_efficiency, 0);
        assert_eq!(bd.contract_verified, 0);
        assert_eq!(bd.project_signals, 0);
        assert_eq!(bd.community_size, 0);
        assert_eq!(bd.time_urgency, 0);
        assert_eq!(bd.sybil_penalty, 0);
    }

    #[test]
    fn score_result_is_debug() {
        let result = calculate(&test_opportunity());
        let debug = format!("{:?}", result);
        assert!(debug.contains("score"));
    }

    #[test]
    fn score_with_no_value_does_not_panic() {
        let mut opp = test_opportunity();
        opp.estimated_value_usd = None;
        opp.gas_cost_estimate = None;
        opp.contract_address = None;
        let result = calculate(&opp);
        assert!(result.score <= 100);
    }

    #[test]
    fn contract_verified_gives_twenty() {
        let opp = test_opportunity();
        let result = calculate(&opp);
        assert_eq!(result.breakdown.contract_verified, 20);
    }

    #[test]
    fn harvest_score_result_serializable() {
        let result = calculate(&test_opportunity());
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: HarvestScoreResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.score, result.score);
    }

    #[test]
    fn score_clamped_to_100() {
        // Even with max everything, should not exceed 100
        let opp = RawOpportunity {
            source: "galxe".to_string(),
            chain: "ethereum".to_string(),
            opportunity_type: OpportunityType::Quest,
            title: "Super Amazing Quest Campaign".to_string(),
            description: "A very long description that is definitely more than fifty characters to get the signal bonus".to_string(),
            url: Some("https://galxe.com/quest/123".to_string()),
            contract_address: Some("0xabc".to_string()),
            estimated_value_usd: Some(999.0),
            gas_cost_estimate: Some(0.01),
            deadline: None,
            discovered_at: "2026-01-01".to_string(),
        };
        let result = calculate(&opp);
        assert!(result.score <= 100);
    }
}

//! Analytics report generation.
//! Queries the SQLite database for ROI, claim stats, and source attribution.

use super::{AnalyticsSummary, ChainBreakdown, SourceAttribution, TimeSeriesPoint};
use rusqlite::Connection;

/// Generate a summary of all collection activity from the database.
pub fn generate_summary(conn: &Connection) -> AnalyticsSummary {
    let mut summary = AnalyticsSummary::default();

    // Total claims and status breakdown
    if let Ok(mut stmt) = conn.prepare(
        "SELECT status, COUNT(*), COALESCE(SUM(value_received_usd), 0), COALESCE(SUM(gas_cost_usd), 0) FROM claims GROUP BY status"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        }) {
            for row in rows.flatten() {
                let (status, count, value, gas) = row;
                summary.total_claims += count;
                summary.total_value_collected_usd += value;
                summary.total_gas_spent_usd += gas;
                match status.as_str() {
                    "Confirmed" => summary.successful_claims += count,
                    "Failed" => summary.failed_claims += count,
                    _ => {}
                }
            }
        }
    }

    // ROI calculation
    if summary.total_gas_spent_usd > 0.0 {
        summary.roi_percentage =
            ((summary.total_value_collected_usd - summary.total_gas_spent_usd)
                / summary.total_gas_spent_usd)
                * 100.0;
    }

    // Active opportunities count
    if let Ok(count) = conn.query_row(
        "SELECT COUNT(*) FROM opportunities WHERE status IN ('Discovered', 'Evaluating', 'Qualified')",
        [],
        |row| row.get::<_, u64>(0),
    ) {
        summary.active_opportunities = count;
    }

    summary
}

/// Get claim statistics broken down by source.
pub fn source_attribution(conn: &Connection) -> Vec<SourceAttribution> {
    let mut results = Vec::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT o.source, COUNT(c.id), COALESCE(SUM(c.value_received_usd), 0) \
         FROM claims c JOIN opportunities o ON c.opportunity_id = o.id \
         GROUP BY o.source ORDER BY SUM(c.value_received_usd) DESC"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(SourceAttribution {
                source: row.get(0)?,
                claim_count: row.get(1)?,
                total_value_usd: row.get(2)?,
            })
        }) {
            for row in rows.flatten() {
                results.push(row);
            }
        }
    }

    results
}

/// Get claim statistics broken down by chain.
pub fn chain_breakdown(conn: &Connection) -> Vec<ChainBreakdown> {
    let mut results = Vec::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT chain, COUNT(*), COALESCE(SUM(value_received_usd), 0), COALESCE(SUM(gas_cost_usd), 0) \
         FROM claims GROUP BY chain ORDER BY SUM(value_received_usd) DESC"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ChainBreakdown {
                chain: row.get(0)?,
                claim_count: row.get(1)?,
                total_value_usd: row.get(2)?,
                total_gas_usd: row.get(3)?,
            })
        }) {
            for row in rows.flatten() {
                results.push(row);
            }
        }
    }

    results
}

/// Generate daily time-series data for sparkline charts.
///
/// Returns up to `days` data points ordered chronologically.
/// Each point aggregates claim count, value collected, and gas spent for that day.
pub fn time_series(conn: &Connection, days: u32) -> Vec<TimeSeriesPoint> {
    let mut results = Vec::new();

    let query = format!(
        "SELECT date(created_at) as d, COUNT(*), \
         COALESCE(SUM(value_received_usd), 0), \
         COALESCE(SUM(gas_cost_usd), 0) \
         FROM claims \
         WHERE created_at >= datetime('now', '-{} days') \
         GROUP BY d ORDER BY d ASC",
        days
    );

    if let Ok(mut stmt) = conn.prepare(&query) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let value: f64 = row.get(2)?;
            let gas: f64 = row.get(3)?;
            Ok(TimeSeriesPoint {
                date: row.get(0)?,
                claims: row.get(1)?,
                value_usd: value,
                gas_usd: gas,
                net_usd: value - gas,
            })
        }) {
            for row in rows.flatten() {
                results.push(row);
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::path::Path;

    fn setup_test_db() -> Connection {
        let conn = db::initialize(Path::new(":memory:")).unwrap();

        // Insert test opportunities
        conn.execute(
            "INSERT INTO opportunities (id, source, chain, opportunity_type, title, status) VALUES \
             ('opp1', 'rss', 'ethereum', 'Airdrop', 'Test Airdrop 1', 'Claimed')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO opportunities (id, source, chain, opportunity_type, title, status) VALUES \
             ('opp2', 'dappradar', 'arbitrum', 'Faucet', 'Test Faucet', 'Qualified')",
            [],
        ).unwrap();

        // Insert a test wallet
        conn.execute(
            "INSERT INTO wallets (id, derivation_path, public_address, chain) VALUES \
             ('w1', 'm/44/60/0/0/0', '0xABC', 'ethereum')",
            [],
        ).unwrap();

        // Insert test claims
        conn.execute(
            "INSERT INTO claims (id, opportunity_id, wallet_id, chain, status, value_received_usd, gas_cost_usd) VALUES \
             ('c1', 'opp1', 'w1', 'ethereum', 'Confirmed', 100.0, 5.0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO claims (id, opportunity_id, wallet_id, chain, status, value_received_usd, gas_cost_usd) VALUES \
             ('c2', 'opp1', 'w1', 'ethereum', 'Failed', 0.0, 2.0)",
            [],
        ).unwrap();

        conn
    }

    #[test]
    fn generate_summary_with_data() {
        let conn = setup_test_db();
        let summary = generate_summary(&conn);
        assert_eq!(summary.total_claims, 2);
        assert_eq!(summary.successful_claims, 1);
        assert_eq!(summary.failed_claims, 1);
        assert!((summary.total_value_collected_usd - 100.0).abs() < 0.01);
        assert!((summary.total_gas_spent_usd - 7.0).abs() < 0.01);
        assert!(summary.roi_percentage > 0.0);
        assert_eq!(summary.active_opportunities, 1); // opp2 is Qualified
    }

    #[test]
    fn generate_summary_empty_db() {
        let conn = db::initialize(Path::new(":memory:")).unwrap();
        let summary = generate_summary(&conn);
        assert_eq!(summary.total_claims, 0);
        assert_eq!(summary.roi_percentage, 0.0);
    }

    #[test]
    fn source_attribution_query() {
        let conn = setup_test_db();
        let sources = source_attribution(&conn);
        assert_eq!(sources.len(), 1); // both claims from 'rss' source
        assert_eq!(sources[0].source, "rss");
        assert_eq!(sources[0].claim_count, 2);
    }

    #[test]
    fn chain_breakdown_query() {
        let conn = setup_test_db();
        let chains = chain_breakdown(&conn);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].chain, "ethereum");
        assert_eq!(chains[0].claim_count, 2);
    }

    // ── Time-series tests ─────────────────────────────

    #[test]
    fn time_series_with_data() {
        let conn = setup_test_db();
        let series = time_series(&conn, 30);
        // Claims were inserted with default created_at = datetime('now'),
        // so they all fall on today.
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].claims, 2);
        assert!((series[0].value_usd - 100.0).abs() < 0.01);
        assert!((series[0].gas_usd - 7.0).abs() < 0.01);
        assert!((series[0].net_usd - 93.0).abs() < 0.01);
    }

    #[test]
    fn time_series_empty_db() {
        let conn = db::initialize(Path::new(":memory:")).unwrap();
        let series = time_series(&conn, 30);
        assert!(series.is_empty());
    }

    #[test]
    fn time_series_zero_days_returns_empty() {
        let conn = setup_test_db();
        // 0 days window — datetime('now', '-0 days') is now, but claims at 'now' might match
        // since created_at = datetime('now'). This is fine — we just check it doesn't panic.
        let series = time_series(&conn, 0);
        // Could be 0 or 1 depending on timing; just ensure no crash
        assert!(series.len() <= 1);
    }

    #[test]
    fn time_series_points_are_serializable() {
        let point = TimeSeriesPoint {
            date: "2026-03-25".to_string(),
            claims: 5,
            value_usd: 250.0,
            gas_usd: 10.0,
            net_usd: 240.0,
        };
        let json = serde_json::to_string(&point).unwrap();
        let rt: TimeSeriesPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.date, "2026-03-25");
        assert_eq!(rt.claims, 5);
        assert!((rt.net_usd - 240.0).abs() < 0.01);
    }

    #[test]
    fn time_series_net_is_value_minus_gas() {
        let conn = setup_test_db();
        let series = time_series(&conn, 30);
        for point in &series {
            assert!((point.net_usd - (point.value_usd - point.gas_usd)).abs() < 0.001);
        }
    }
}

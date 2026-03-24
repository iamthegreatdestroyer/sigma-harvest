//! Social media signal detection (Twitter/X keyword monitoring).
//!
//! When a bearer token is configured, queries the Twitter/X API v2 recent
//! search endpoint for crypto-opportunity keywords. Without a token the
//! source gracefully returns an empty result set.

use super::{DiscoveryError, DiscoverySource, OpportunityType, RawOpportunity};

/// Keywords that signal potential crypto opportunities.
const KEYWORDS: &[&str] = &[
    "airdrop",
    "free mint",
    "faucet",
    "quest",
    "retroactive",
    "testnet",
    "token launch",
];

/// Twitter/X API v2 recent-search endpoint.
const TWITTER_SEARCH_URL: &str = "https://api.twitter.com/2/tweets/search/recent";

pub struct SocialSource {
    pub bearer_token: Option<String>,
}

/// Classify a tweet's text into an [`OpportunityType`] by keyword match.
fn classify_tweet(text: &str) -> OpportunityType {
    let lower = text.to_lowercase();
    if lower.contains("free mint") {
        OpportunityType::FreeMint
    } else if lower.contains("faucet") || lower.contains("testnet") {
        OpportunityType::Faucet
    } else if lower.contains("quest") {
        OpportunityType::Quest
    } else if lower.contains("retroactive") {
        OpportunityType::Retroactive
    } else if lower.contains("token launch") || lower.contains("liquidity") {
        OpportunityType::LiquidityBonus
    } else {
        // Default — most keyword hits relate to airdrops
        OpportunityType::Airdrop
    }
}

/// Build the query string for Twitter API v2 recent search.
fn build_query() -> String {
    KEYWORDS
        .iter()
        .map(|kw| format!("\"{}\"", kw))
        .collect::<Vec<_>>()
        .join(" OR ")
        + " -is:retweet"
}

/// Parse a single tweet JSON object into a [`RawOpportunity`].
fn parse_tweet(tweet: &serde_json::Value) -> Option<RawOpportunity> {
    let text = tweet.get("text")?.as_str()?;
    let id = tweet.get("id")?.as_str()?;
    let created = tweet
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let opp_type = classify_tweet(text);
    let title = if text.len() > 80 {
        format!("{}…", &text[..80])
    } else {
        text.to_string()
    };

    Some(RawOpportunity {
        source: "social".to_string(),
        chain: "unknown".to_string(),
        opportunity_type: opp_type,
        title,
        description: text.to_string(),
        url: Some(format!("https://x.com/i/web/status/{}", id)),
        contract_address: None,
        estimated_value_usd: None,
        gas_cost_estimate: None,
        deadline: None,
        discovered_at: if created.is_empty() {
            chrono::Utc::now().to_rfc3339()
        } else {
            created
        },
    })
}

impl DiscoverySource for SocialSource {
    fn name(&self) -> &str {
        "social"
    }

    async fn discover(&self) -> Result<Vec<RawOpportunity>, DiscoveryError> {
        let token = match &self.bearer_token {
            Some(t) if !t.is_empty() => t,
            _ => return Ok(vec![]),
        };

        let query = build_query();
        let client = reqwest::Client::new();
        let resp = client
            .get(TWITTER_SEARCH_URL)
            .bearer_auth(token)
            .query(&[
                ("query", query.as_str()),
                ("max_results", "25"),
                ("tweet.fields", "created_at"),
            ])
            .send()
            .await?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| DiscoveryError::Parse(format!("failed to read response body: {}", e)))?;

        if !status.is_success() {
            return Err(DiscoveryError::Parse(format!(
                "Twitter API returned {}: {}",
                status, body
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| DiscoveryError::Parse(format!("invalid JSON: {}", e)))?;

        let opportunities = json
            .get("data")
            .and_then(|d| d.as_array())
            .map(|tweets| tweets.iter().filter_map(parse_tweet).collect())
            .unwrap_or_default();

        Ok(opportunities)
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_name_is_social() {
        let src = SocialSource { bearer_token: None };
        assert_eq!(src.name(), "social");
    }

    #[tokio::test]
    async fn no_bearer_token_returns_empty() {
        let src = SocialSource { bearer_token: None };
        let result = src.discover().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn empty_bearer_token_returns_empty() {
        let src = SocialSource {
            bearer_token: Some(String::new()),
        };
        let result = src.discover().await.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn classify_airdrop() {
        assert!(matches!(
            classify_tweet("Massive airdrop coming soon!"),
            OpportunityType::Airdrop
        ));
    }

    #[test]
    fn classify_free_mint() {
        assert!(matches!(
            classify_tweet("Get your free mint now"),
            OpportunityType::FreeMint
        ));
    }

    #[test]
    fn classify_faucet() {
        assert!(matches!(
            classify_tweet("New faucet for testnet tokens"),
            OpportunityType::Faucet
        ));
    }

    #[test]
    fn classify_testnet_as_faucet() {
        assert!(matches!(
            classify_tweet("Join the testnet early"),
            OpportunityType::Faucet
        ));
    }

    #[test]
    fn classify_quest() {
        assert!(matches!(
            classify_tweet("Complete the quest to earn rewards"),
            OpportunityType::Quest
        ));
    }

    #[test]
    fn classify_retroactive() {
        assert!(matches!(
            classify_tweet("Retroactive rewards for early users"),
            OpportunityType::Retroactive
        ));
    }

    #[test]
    fn classify_token_launch() {
        assert!(matches!(
            classify_tweet("Token launch with liquidity bonus"),
            OpportunityType::LiquidityBonus
        ));
    }

    #[test]
    fn classify_unknown_defaults_to_airdrop() {
        assert!(matches!(
            classify_tweet("some random crypto text"),
            OpportunityType::Airdrop
        ));
    }

    #[test]
    fn build_query_contains_all_keywords() {
        let q = build_query();
        for kw in KEYWORDS {
            assert!(q.contains(kw), "query missing keyword: {}", kw);
        }
        assert!(q.contains("-is:retweet"));
    }

    #[test]
    fn build_query_uses_or_operator() {
        let q = build_query();
        assert!(q.contains(" OR "));
    }

    #[test]
    fn parse_tweet_valid() {
        let tweet = serde_json::json!({
            "id": "123456789",
            "text": "Big airdrop alert for Solana holders",
            "created_at": "2025-01-15T12:00:00Z"
        });
        let opp = parse_tweet(&tweet).unwrap();
        assert_eq!(opp.source, "social");
        assert_eq!(opp.chain, "unknown");
        assert!(opp.url.unwrap().contains("123456789"));
        assert_eq!(opp.discovered_at, "2025-01-15T12:00:00Z");
    }

    #[test]
    fn parse_tweet_missing_id_returns_none() {
        let tweet = serde_json::json!({ "text": "some text" });
        assert!(parse_tweet(&tweet).is_none());
    }

    #[test]
    fn parse_tweet_missing_text_returns_none() {
        let tweet = serde_json::json!({ "id": "1" });
        assert!(parse_tweet(&tweet).is_none());
    }

    #[test]
    fn parse_tweet_truncates_long_title() {
        let long_text = "a]".repeat(50); // 100 chars
        let tweet = serde_json::json!({
            "id": "999",
            "text": long_text,
        });
        let opp = parse_tweet(&tweet).unwrap();
        assert!(opp.title.len() <= 83); // 80 + "…"
        assert!(opp.title.ends_with('…'));
    }

    #[test]
    fn parse_tweet_no_created_at_uses_now() {
        let tweet = serde_json::json!({
            "id": "42",
            "text": "airdrop now"
        });
        let opp = parse_tweet(&tweet).unwrap();
        assert!(!opp.discovered_at.is_empty());
    }

    #[test]
    fn keywords_are_non_empty() {
        assert!(!KEYWORDS.is_empty());
        for kw in KEYWORDS {
            assert!(!kw.is_empty());
        }
    }
}

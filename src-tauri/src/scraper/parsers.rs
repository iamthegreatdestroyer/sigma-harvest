//! HTML and data parsers for scraped content.
//! Uses the `scraper` crate for CSS-selector based extraction.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Extracted opportunity data from an HTML page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOpportunity {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub contract_address: Option<String>,
    pub value_hint: Option<String>,
    pub deadline_hint: Option<String>,
}

/// Extract opportunity data from an HTML page.
/// Looks for common airdrop/claim page patterns.
pub fn parse_html_opportunity(html: &str) -> Option<ParsedOpportunity> {
    if html.is_empty() {
        return None;
    }

    let document = Html::parse_document(html);

    // Try to extract a title
    let title = extract_first_text(&document, &["h1", "h2", ".title", "[class*=\"title\"]"]);

    // Try to extract a description
    let description = extract_first_text(&document, &[
        ".description", "p", ".summary", "[class*=\"desc\"]",
    ]);

    // Look for claim/airdrop links
    let url = extract_first_href(&document, &[
        "a[href*=\"claim\"]", "a[href*=\"airdrop\"]", "a[href*=\"mint\"]",
        ".claim-btn", "button[data-url]",
    ]);

    // Look for Ethereum addresses (0x followed by 40 hex chars)
    let contract_address = extract_eth_address(html);

    // Look for value hints
    let value_hint = extract_first_text(&document, &[
        "[class*=\"value\"]", "[class*=\"reward\"]", "[class*=\"amount\"]",
    ]);

    // Look for deadline hints
    let deadline_hint = extract_first_text(&document, &[
        "[class*=\"deadline\"]", "[class*=\"expir\"]", "time",
        "[class*=\"countdown\"]",
    ]);

    // Only return if we found at least a title or URL
    if title.is_some() || url.is_some() {
        Some(ParsedOpportunity {
            title,
            description,
            url,
            contract_address,
            value_hint,
            deadline_hint,
        })
    } else {
        None
    }
}

fn extract_first_text(document: &Html, selectors: &[&str]) -> Option<String> {
    for sel_str in selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            if let Some(element) = document.select(&selector).next() {
                let text: String = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn extract_first_href(document: &Html, selectors: &[&str]) -> Option<String> {
    for sel_str in selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(href) = element.value().attr("href") {
                    return Some(href.to_string());
                }
            }
        }
    }
    None
}

fn extract_eth_address(text: &str) -> Option<String> {
    // Match 0x followed by exactly 40 hex characters
    let mut i = 0;
    let bytes = text.as_bytes();
    while i + 42 <= bytes.len() {
        if bytes[i] == b'0' && bytes[i + 1] == b'x' {
            let candidate = &text[i..i + 42];
            if candidate[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(candidate.to_string());
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_html_returns_none() {
        assert!(parse_html_opportunity("").is_none());
    }

    #[test]
    fn parse_random_html_returns_none() {
        let html = "<html><body></body></html>";
        assert!(parse_html_opportunity(html).is_none());
    }

    #[test]
    fn parse_airdrop_page_extracts_title() {
        let html = r#"<html><body>
            <h1>Free Token Airdrop</h1>
            <p>Claim 500 tokens now!</p>
            <a href="https://example.com/claim">Claim Now</a>
        </body></html>"#;
        let result = parse_html_opportunity(html).unwrap();
        assert_eq!(result.title.unwrap(), "Free Token Airdrop");
        assert!(result.url.unwrap().contains("claim"));
    }

    #[test]
    fn parse_extracts_description() {
        let html = r#"<html><body>
            <h2>Airdrop</h2>
            <p class="description">Get free governance tokens</p>
        </body></html>"#;
        let result = parse_html_opportunity(html).unwrap();
        assert!(result.description.unwrap().contains("governance"));
    }

    #[test]
    fn parse_extracts_eth_address() {
        let html = r#"<html><body>
            <h1>Claim</h1>
            <p>Contract: 0x1234567890abcdef1234567890abcdef12345678</p>
        </body></html>"#;
        let result = parse_html_opportunity(html).unwrap();
        assert_eq!(
            result.contract_address.unwrap(),
            "0x1234567890abcdef1234567890abcdef12345678"
        );
    }

    #[test]
    fn extract_eth_address_from_text() {
        let text = "Send to 0xAbCdEf0123456789AbCdEf0123456789AbCdEf01 please";
        let addr = extract_eth_address(text).unwrap();
        assert_eq!(addr, "0xAbCdEf0123456789AbCdEf0123456789AbCdEf01");
    }

    #[test]
    fn extract_eth_address_none_for_short() {
        assert!(extract_eth_address("0x1234").is_none());
    }

    #[test]
    fn parsed_opportunity_serializable() {
        let p = ParsedOpportunity {
            title: Some("Test".to_string()),
            description: None,
            url: Some("https://example.com".to_string()),
            contract_address: None,
            value_hint: None,
            deadline_hint: None,
        };
        let json = serde_json::to_string(&p).unwrap();
        let rt: ParsedOpportunity = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.title.unwrap(), "Test");
    }

    #[test]
    fn parse_page_with_value_and_deadline() {
        let html = r#"<html><body>
            <h1>Big Airdrop</h1>
            <span class="value">$500 USDC</span>
            <time class="deadline">2026-04-01</time>
        </body></html>"#;
        let result = parse_html_opportunity(html).unwrap();
        assert!(result.value_hint.unwrap().contains("500"));
        assert!(result.deadline_hint.unwrap().contains("2026"));
    }
}

//! HTML and data parsers for scraped content.

/// Extract opportunity data from an HTML page.
pub fn parse_html_opportunity(_html: &str) -> Option<String> {
    // TODO: Use scraper crate to extract opportunity data
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
        let html = "<html><body><h1>Hello</h1></body></html>";
        assert!(parse_html_opportunity(html).is_none());
    }

    #[test]
    fn parse_complex_html_returns_none() {
        let html = r#"<html>
            <body>
                <div class="airdrop">
                    <h2>Free Token Airdrop</h2>
                    <p>Claim 500 tokens</p>
                    <a href="https://example.com/claim">Claim Now</a>
                </div>
            </body>
        </html>"#;
        // Stub always returns None
        assert!(parse_html_opportunity(html).is_none());
    }
}

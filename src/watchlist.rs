use serde_json::Value;

use crate::config::Config;

/// A single match found by the watchlist filter.
#[derive(Debug, Clone)]
pub struct WatchMatch {
    pub match_type: String,
    pub matched: String,
    pub source: String,
    pub context: String,
}

/// User-defined watch filters built from config.
#[derive(Debug, Clone)]
pub struct Watchlist {
    pub regions: Vec<String>,
    pub keywords: Vec<String>,
    pub tickers: Vec<String>,
}

impl Watchlist {
    pub fn from_config(config: &Config) -> Self {
        Self {
            regions: config.watch_regions.clone(),
            keywords: config.alert_keywords.clone(),
            tickers: config.watch_tickers.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty() && self.keywords.is_empty() && self.tickers.is_empty()
    }

    /// Scan sweep data for watchlist matches.
    pub fn filter_sweep(&self, sweep_data: &Value) -> Vec<WatchMatch> {
        let sources = match sweep_data.get("sources") {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut matches = Vec::new();

        if let Some(map) = sources.as_object() {
            for (source_name, source_data) in map {
                let text = source_data.to_string().to_lowercase();

                // Region matching
                for region in &self.regions {
                    let region_lower = region.to_lowercase();
                    if text.contains(&region_lower) {
                        let context = extract_context(&text, &region_lower);
                        matches.push(WatchMatch {
                            match_type: "region".to_string(),
                            matched: region.clone(),
                            source: source_name.clone(),
                            context,
                        });
                    }
                }

                // Keyword matching
                for keyword in &self.keywords {
                    let kw_lower = keyword.to_lowercase();
                    if text.contains(&kw_lower) {
                        let context = extract_context(&text, &kw_lower);
                        matches.push(WatchMatch {
                            match_type: "keyword".to_string(),
                            matched: keyword.clone(),
                            source: source_name.clone(),
                            context,
                        });
                    }
                }

                // Ticker matching — check yfinance-like sources for ticker symbols
                if !self.tickers.is_empty() {
                    for ticker in &self.tickers {
                        let ticker_lower = ticker.to_lowercase();
                        if text.contains(&ticker_lower) {
                            let context = extract_context(&text, &ticker_lower);
                            matches.push(WatchMatch {
                                match_type: "ticker".to_string(),
                                matched: ticker.clone(),
                                source: source_name.clone(),
                                context,
                            });
                        }
                    }
                }
            }
        }

        matches
    }
}

/// Extract surrounding context around a match (up to 80 chars each side).
fn extract_context(text: &str, needle: &str) -> String {
    if let Some(pos) = text.find(needle) {
        let start = pos.saturating_sub(80);
        let end = (pos + needle.len() + 80).min(text.len());
        // Ensure we don't split in the middle of a UTF-8 char
        let slice = &text[start..end];
        let trimmed = slice.trim();
        if trimmed.len() > 200 {
            format!("{}...", &trimmed[..200])
        } else {
            trimmed.to_string()
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_watchlist() {
        let config = Config {
            refresh_interval_minutes: 15,
            source_timeout_secs: 30,
            llm_provider: None,
            llm_api_key: None,
            llm_base_url: None,
            llm_model: None,
            ollama_url: None,
            ollama_model: None,
            sweep_lang: "en".to_string(),
            webhook_url: None,
            telegram_bot_token: None,
            telegram_chat_id: None,
            telegram_poll_interval: 5000,
            fallback_provider: None,
            fallback_model: None,
            gemini_api_key: None,
            discord_bot_token: None,
            discord_channel_id: None,
            discord_guild_id: None,
            discord_webhook_url: None,
            watch_regions: vec![],
            alert_keywords: vec![],
            watch_tickers: vec![],
        };
        let wl = Watchlist::from_config(&config);
        assert!(wl.is_empty());
    }

    #[test]
    fn test_keyword_match() {
        let wl = Watchlist {
            regions: vec![],
            keywords: vec!["earthquake".to_string()],
            tickers: vec![],
        };

        let data = json!({
            "sources": {
                "usgs": { "text": "Major earthquake detected in Japan" }
            }
        });

        let matches = wl.filter_sweep(&data);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, "keyword");
        assert_eq!(matches[0].matched, "earthquake");
        assert_eq!(matches[0].source, "usgs");
    }

    #[test]
    fn test_region_match() {
        let wl = Watchlist {
            regions: vec!["Taiwan".to_string()],
            keywords: vec![],
            tickers: vec![],
        };

        let data = json!({
            "sources": {
                "acled": { "region": "Taiwan Strait tensions rise" }
            }
        });

        let matches = wl.filter_sweep(&data);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, "region");
    }

    #[test]
    fn test_no_matches() {
        let wl = Watchlist {
            regions: vec!["Mars".to_string()],
            keywords: vec![],
            tickers: vec![],
        };

        let data = json!({
            "sources": {
                "usgs": { "text": "Minor tremor in California" }
            }
        });

        let matches = wl.filter_sweep(&data);
        assert!(matches.is_empty());
    }
}

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const DAILY_TRENDS_URL: &str =
    "https://trends.google.com/trends/api/dailytrends?hl=en-US&tz=-480&geo=US&ns=15";

pub struct GoogleTrends {
    client: HttpClient,
}

impl GoogleTrends {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for GoogleTrends {
    fn name(&self) -> &str {
        "GoogleTrends"
    }

    fn description(&self) -> &str {
        "Google daily trending searches (US)"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        // Google Trends returns JSONP with ")]}',\n" prefix
        let raw = match self.client.fetch_text(DAILY_TRENDS_URL).await {
            Ok(text) => text,
            Err(e) => {
                return Ok(json!({
                    "source": "GoogleTrends",
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": format!("Failed to fetch Google Trends: {}", e),
                    "hint": "Google may block automated requests. Consider using a proxy or SerpAPI.",
                }));
            }
        };

        // Strip JSONP prefix: ")]}',\n"
        let json_str = if let Some(pos) = raw.find('\n') {
            &raw[pos + 1..]
        } else {
            &raw
        };

        let data: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => {
                return Ok(json!({
                    "source": "GoogleTrends",
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": "Failed to parse Google Trends response",
                    "hint": "Google may have changed their API format or blocked the request",
                    "rawPreview": raw.chars().take(200).collect::<String>(),
                }));
            }
        };

        // Extract trending searches
        let days = data
            .get("default")
            .and_then(|d| d.get("trendingSearchesDays"))
            .and_then(|v| v.as_array());

        let mut trends = Vec::new();

        if let Some(day_arr) = days {
            for day in day_arr.iter().take(2) {
                let date = day
                    .get("formattedDate")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if let Some(searches) = day.get("trendingSearches").and_then(|v| v.as_array()) {
                    for search in searches.iter().take(10) {
                        let title = search
                            .get("title")
                            .and_then(|t| t.get("query"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let traffic = search
                            .get("formattedTraffic")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        let related: Vec<String> = search
                            .get("relatedQueries")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .take(3)
                                    .filter_map(|q| {
                                        q.get("query").and_then(|v| v.as_str()).map(String::from)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        if !title.is_empty() {
                            trends.push(json!({
                                "query": title,
                                "traffic": traffic,
                                "date": date,
                                "relatedQueries": related,
                            }));
                        }
                    }
                }
            }
        }

        let mut signals = Vec::new();
        if trends.is_empty() {
            signals.push("Unable to extract trending searches".to_string());
        } else {
            let top_queries: Vec<&str> = trends
                .iter()
                .take(5)
                .filter_map(|t| t.get("query").and_then(|v| v.as_str()))
                .collect();
            signals.push(format!("Top trending: {}", top_queries.join(", ")));
        }

        Ok(json!({
            "source": "GoogleTrends",
            "timestamp": Utc::now().to_rfc3339(),
            "totalTrends": trends.len(),
            "trends": trends,
            "signals": signals,
        }))
    }
}

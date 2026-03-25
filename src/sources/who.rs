use anyhow::Result;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate, Utc};
use serde_json::{json, Value};

use crate::http::HttpClient;
use crate::util::strip_html;
use super::IntelSource;

const WHO_URL: &str = "https://www.who.int/api/news/diseaseoutbreaknews";

pub struct Who {
    client: HttpClient,
}

impl Who {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn truncate_to(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{}...", truncated.trim_end())
}

#[async_trait]
impl IntelSource for Who {
    fn name(&self) -> &str {
        "WHO"
    }

    fn description(&self) -> &str {
        "WHO disease outbreak news"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(WHO_URL).await?;

        let items = data
            .get("value")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let cutoff = Utc::now().naive_utc().date() - Duration::days(30);

        // Collect items with parsed dates for sorting
        let mut dated_items: Vec<(NaiveDate, &Value)> = items
            .iter()
            .filter_map(|item| {
                let date_str = item.get("PublicationDate").and_then(|v| v.as_str())?;
                // Dates are like "2025-03-15T00:00:00Z" or "2025-03-15"
                let date = NaiveDate::parse_from_str(
                    &date_str[..10.min(date_str.len())],
                    "%Y-%m-%d",
                )
                .ok()?;
                if date >= cutoff {
                    Some((date, item))
                } else {
                    None
                }
            })
            .collect();

        // Sort by date descending
        dated_items.sort_by(|a, b| b.0.cmp(&a.0));

        let mut news = Vec::new();
        for (_, item) in &dated_items {
            let title = item
                .get("Title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let pub_date = item
                .get("PublicationDate")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let don_id = item
                .get("DonId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let summary_raw = item
                .get("Summary")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let summary = truncate_to(&strip_html(summary_raw).trim().replace('\n', " "), 300);

            let url = if don_id.is_empty() {
                String::new()
            } else {
                format!(
                    "https://www.who.int/emergencies/disease-outbreak-news/{}",
                    don_id
                )
            };

            news.push(json!({
                "title": title,
                "date": pub_date,
                "url": url,
                "summary": summary,
            }));
        }

        Ok(json!({
            "source": "WHO",
            "timestamp": Utc::now().to_rfc3339(),
            "alerts": news,
        }))
    }
}

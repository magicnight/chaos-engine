use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const SUBREDDITS: &[&str] = &["worldnews", "geopolitics", "economics", "technology"];

pub struct Reddit {
    client: HttpClient,
}

impl Reddit {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn compact_post(child: &Value) -> Option<Value> {
    let d = child.get("data")?;
    let title = d.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let score = d.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
    let comments = d
        .get("num_comments")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let url = d.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let created = d
        .get("created_utc")
        .and_then(|v| v.as_f64())
        .map(|ts| {
            chrono::DateTime::from_timestamp(ts as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    Some(json!({
        "title": title,
        "score": score,
        "comments": comments,
        "url": url,
        "created": created,
    }))
}

#[async_trait]
impl IntelSource for Reddit {
    fn name(&self) -> &str {
        "Reddit"
    }

    fn description(&self) -> &str {
        "Reddit social sentiment monitoring"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let mut subreddit_results = serde_json::Map::new();

        for sub in SUBREDDITS {
            let url = format!(
                "https://www.reddit.com/r/{}/hot.json?limit=10&raw_json=1",
                sub
            );

            let resp = self
                .client
                .raw_client()
                .get(&url)
                .header("User-Agent", "CHAOS/3.0 intel-engine")
                .send()
                .await;

            let posts: Vec<Value> = match resp {
                Ok(r) => {
                    let text = r.text().await.unwrap_or_default();
                    let data: Value =
                        serde_json::from_str(&text).unwrap_or(json!({}));
                    data.get("data")
                        .and_then(|d| d.get("children"))
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(compact_post).collect())
                        .unwrap_or_default()
                }
                Err(_) => Vec::new(),
            };

            subreddit_results.insert(sub.to_string(), json!(posts));

            // Rate limiting: small delay between requests
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(json!({
            "source": "Reddit",
            "timestamp": Utc::now().to_rfc3339(),
            "subreddits": subreddit_results,
        }))
    }
}

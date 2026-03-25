// World News API — Global news search with sentiment analysis
// API: https://worldnewsapi.com/
// Auth: API key required (WORLDNEWS_API_KEY)

use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use serde_json::{json, Value};
use std::time::Duration;
use crate::http::HttpClient;
use super::IntelSource;

const BASE_URL: &str = "https://api.worldnewsapi.com";
const PER_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WorldNews {
    client: HttpClient,
}

impl WorldNews {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for WorldNews {
    fn name(&self) -> &str { "WorldNews" }
    fn description(&self) -> &str { "World News API — global news with sentiment" }
    fn tier(&self) -> u8 { 2 }

    async fn sweep(&self) -> Result<Value> {
        let api_key = match std::env::var("WORLDNEWS_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => return Ok(json!({
                "source": "WorldNews",
                "error": "WORLDNEWS_API_KEY required. Get one at https://worldnewsapi.com/",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        };

        let timestamp = chrono::Utc::now().to_rfc3339();
        let queries: Vec<(&str, &str)> = vec![
            ("geopolitics conflict military", "conflict"),
            ("economy markets sanctions trade", "economy"),
            ("technology AI semiconductor", "tech"),
            ("climate disaster earthquake hurricane", "climate"),
        ];

        // Run all 4 queries in parallel with individual 10s timeouts
        let futures: Vec<_> = queries.iter().map(|(query, category)| {
            let url = format!(
                "{}/search-news?text={}&language=en&number=10&api-key={}",
                BASE_URL,
                urlencoding::encode(query),
                api_key
            );
            let client = self.client.clone();
            let cat = category.to_string();
            async move {
                let result = tokio::time::timeout(PER_QUERY_TIMEOUT, client.fetch_json(&url)).await;
                (cat, result)
            }
        }).collect();

        let results = join_all(futures).await;

        let mut all_articles = Vec::new();
        let mut categories = serde_json::Map::new();

        for (category, result) in results {
            match result {
                Ok(Ok(data)) => {
                    if let Some(news) = data.get("news").and_then(|n| n.as_array()) {
                        let articles: Vec<Value> = news.iter().map(|a| {
                            json!({
                                "title": a.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                                "url": a.get("url").and_then(|u| u.as_str()).unwrap_or(""),
                                "author": a.get("author").and_then(|au| au.as_str()),
                                "publishDate": a.get("publish_date").and_then(|d| d.as_str()),
                                "sentiment": a.get("sentiment").and_then(|s| s.as_f64()),
                                "sourceCountry": a.get("source_country").and_then(|c| c.as_str()),
                                "image": a.get("image").and_then(|i| i.as_str()),
                                "category": &category,
                            })
                        }).collect();

                        categories.insert(category, json!(articles.len()));
                        all_articles.extend(articles);
                    }
                }
                Ok(Err(e)) => {
                    categories.insert(category, json!(format!("error: {}", e)));
                }
                Err(_) => {
                    categories.insert(category, json!("error: query timed out (10s)"));
                }
            }
        }

        // Sort by sentiment (most negative first = most alarming)
        all_articles.sort_by(|a, b| {
            let sa = a.get("sentiment").and_then(|s| s.as_f64()).unwrap_or(0.0);
            let sb = b.get("sentiment").and_then(|s| s.as_f64()).unwrap_or(0.0);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(json!({
            "source": "WorldNews",
            "timestamp": timestamp,
            "totalArticles": all_articles.len(),
            "categories": categories,
            "articles": all_articles,
            "topNegative": all_articles.iter().take(5).cloned().collect::<Vec<_>>(),
        }))
    }
}

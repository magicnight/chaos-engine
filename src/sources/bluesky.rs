use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BSKY_SEARCH_URL: &str = "https://public.api.bsky.app/xrpc/app.bsky.feed.searchPosts";

const SEARCH_QUERIES: &[(&str, &str)] = &[
    ("geopolitics", "geopolitics"),
    ("markets", "market crash"),
    ("breaking", "breaking news"),
    ("conflict", "conflict"),
];

pub struct Bluesky {
    client: HttpClient,
}

impl Bluesky {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn compact_post(post: &Value) -> Value {
    let record = post.get("record").unwrap_or(post);
    let author = post.get("author");
    let text = record
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let handle = author
        .and_then(|a| a.get("handle"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let likes = post
        .get("likeCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let reposts = post
        .get("repostCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let created = record
        .get("createdAt")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let truncated = if text.len() > 200 {
        format!("{}...", &text[..text.char_indices().nth(200).map(|(i, _)| i).unwrap_or(text.len())])
    } else {
        text.to_string()
    };

    json!({
        "text": truncated,
        "author": handle,
        "likes": likes,
        "reposts": reposts,
        "createdAt": created,
    })
}

#[async_trait]
impl IntelSource for Bluesky {
    fn name(&self) -> &str {
        "Bluesky"
    }

    fn description(&self) -> &str {
        "Bluesky social sentiment intelligence"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let mut topics = serde_json::Map::new();

        for (label, query) in SEARCH_QUERIES {
            let url = format!("{}?q={}&limit=20", BSKY_SEARCH_URL, query);

            let result = self.client.fetch_json(&url).await;

            let posts: Vec<Value> = match result {
                Ok(data) => data
                    .get("posts")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(compact_post).collect())
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            topics.insert(label.to_string(), json!(posts));

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(json!({
            "source": "Bluesky",
            "timestamp": Utc::now().to_rfc3339(),
            "topics": topics,
        }))
    }
}

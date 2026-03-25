// ============================================================================
// CHAOS Engine — Data Source Template
// ============================================================================
//
// This file is NOT compiled (no `pub mod` in mod.rs). Copy it as a starting
// point when adding a new intelligence source.
//
// Steps:
//   1. Copy this file to src/sources/mysource.rs
//   2. Rename the struct and update name()/description()/tier()
//   3. Implement the sweep() logic
//   4. Add `pub mod mysource;` to src/sources/mod.rs
//   5. Add `Box::new(mysource::MySource::new(client.clone()))` to build_sources()
//   6. Test: `cargo run -- source MySource`
//
// Rules:
//   - MUST include "source": self.name() in the returned JSON
//   - MUST handle missing API keys gracefully (return error JSON, never panic)
//   - MUST use self.client for HTTP (it has retry + timeout built in)
//   - SHOULD use self.client.raw_client() when custom headers are needed
//   - SHOULD include "timestamp" and "signals" in output
//   - SHOULD run parallel fetches with futures::future::join_all when possible
// ============================================================================

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const API_URL: &str = "https://api.example.com/v1/data";

// ---------------------------------------------------------------------------
// Struct
// ---------------------------------------------------------------------------

pub struct MySource {
    client: HttpClient,
}

impl MySource {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

// ---------------------------------------------------------------------------
// IntelSource implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl IntelSource for MySource {
    fn name(&self) -> &str {
        "MySource"
    }

    fn description(&self) -> &str {
        "One-line description of what this source provides"
    }

    fn tier(&self) -> u8 {
        3 // 1=core OSINT, 2=economic, 3=supplementary, 4=specialized, 5=experimental
    }

    async fn sweep(&self) -> Result<Value> {
        // ── 1. API key check (skip if source is free) ───────────────────
        let api_key = match std::env::var("MYSOURCE_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                return Ok(json!({
                    "source": self.name(),
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": "MYSOURCE_API_KEY required",
                    "hint": "Get one at https://example.com/register",
                }));
            }
        };

        // ── 2. Fetch data ───────────────────────────────────────────────
        // Option A: Simple JSON GET (uses built-in retry)
        let url = format!("{}?key={}", API_URL, api_key);
        let data = self.client.fetch_json(&url).await?;

        // Option B: Custom headers (no automatic retry)
        // let resp = self.client.raw_client()
        //     .get(API_URL)
        //     .header("Authorization", format!("Bearer {}", api_key))
        //     .send()
        //     .await?;
        // let data: Value = resp.json().await?;

        // Option C: Fetch XML/text and parse manually (like gdacs.rs)
        // let text = self.client.fetch_text(API_URL).await?;

        // ── 3. Parse response ───────────────────────────────────────────
        let items = data
            .get("results")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .take(50) // limit to avoid huge payloads
                    .map(|item| {
                        json!({
                            "title": item.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                            "value": item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // ── 4. Generate intelligence signals ────────────────────────────
        let mut signals = Vec::new();
        if items.len() > 20 {
            signals.push(format!("HIGH ACTIVITY: {} items detected", items.len()));
        }
        if signals.is_empty() {
            signals.push("Activity within normal parameters".to_string());
        }

        // ── 5. Return structured JSON ───────────────────────────────────
        Ok(json!({
            "source": self.name(),
            "timestamp": Utc::now().to_rfc3339(),
            "totalItems": items.len(),
            "items": items,
            "signals": signals,
        }))
    }
}

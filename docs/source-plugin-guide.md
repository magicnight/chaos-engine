# CHAOS Engine — Data Source Plugin Guide

## Overview

Every intelligence source in CHAOS implements the `IntelSource` trait. Adding a new source requires:
1. Create a `.rs` file in `src/sources/`
2. Implement the `IntelSource` trait
3. Register in `src/sources/mod.rs`

## The IntelSource Trait

```rust
#[async_trait]
pub trait IntelSource: Send + Sync {
    /// Unique identifier (e.g. "MySource")
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Intelligence tier 1-5 (1=highest priority)
    fn tier(&self) -> u8;

    /// Execute one data collection sweep
    async fn sweep(&self) -> Result<Value>;
}
```

## Template: Minimal Source

```rust
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::http::HttpClient;
use super::IntelSource;

pub struct MySource {
    client: HttpClient,
}

impl MySource {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for MySource {
    fn name(&self) -> &str { "MySource" }
    fn description(&self) -> &str { "Description of what this source provides" }
    fn tier(&self) -> u8 { 3 }

    async fn sweep(&self) -> Result<Value> {
        // 1. Check for API key if needed
        let api_key = match std::env::var("MYSOURCE_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => return Ok(json!({
                "source": self.name(),
                "error": "MYSOURCE_API_KEY required",
                "hint": "Get one at https://example.com/api"
            })),
        };

        // 2. Fetch data
        let url = format!("https://api.example.com/data?key={}", api_key);
        let data = self.client.fetch_json(&url).await?;

        // 3. Parse and structure
        let items = data.get("results")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().map(|item| {
                json!({
                    "title": item.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                    "value": item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
            }).collect::<Vec<_>>())
            .unwrap_or_default();

        // 4. Return structured JSON (MUST include "source" field)
        Ok(json!({
            "source": self.name(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "totalItems": items.len(),
            "items": items,
        }))
    }
}
```

## Rules
1. **MUST** include `"source": self.name()` in output JSON
2. **MUST** handle missing API keys gracefully (return error JSON, don't panic)
3. **MUST** use `self.client` (HttpClient) for HTTP requests — it has retry and timeout
4. **SHOULD** use `self.client.raw_client()` for custom headers when needed
5. **SHOULD** run multiple API calls in parallel with `futures::future::join_all`
6. **SHOULD** include `"timestamp"` in output
7. **SHOULD** include meaningful signals/alerts when anomalies detected

## Tier Assignment
| Tier | Description | Examples |
|------|------------|---------|
| 1 | Core OSINT — critical for situational awareness | ACLED, GDELT, USGS, WHO |
| 2 | Economic/Financial — market and macro data | FRED, EIA, YFinance |
| 3 | Supplementary — environment, social, cyber | NOAA, Reddit, CVE |
| 4 | Specialized — niche but valuable | CelesTrak, KiwiSDR |
| 5 | Experimental — new or unreliable | (testing sources) |

## Registration
Add to `src/sources/mod.rs`:
```rust
pub mod mysource;
// ...in build_sources():
Box::new(mysource::MySource::new(client.clone())),
```

## Testing
```bash
# Test your source independently
chaos source mysource

# Run full sweep to see it in context
chaos sweep --json | jq '.sources.MySource'
```

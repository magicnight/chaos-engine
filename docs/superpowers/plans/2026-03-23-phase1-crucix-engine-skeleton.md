# Phase 1: Crucix Engine Skeleton — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a working Rust CLI that can parallel-sweep 5 OSINT sources and output structured JSON.

**Architecture:** A Cargo binary project in `rust/` with clap-driven CLI, a shared async HTTP client with exponential backoff, an `IntelSource` trait that each source implements, and a `briefing` orchestrator that runs all sources in parallel via `tokio::join_all`. Phase 1 has no web server, no SQLite, no LLM — pure CLI data collection.

**Tech Stack:** Rust 2021, tokio, reqwest (rustls), clap (derive), serde/serde_json, thiserror, anyhow, chrono, async-trait

**Spec:** `docs/superpowers/specs/2026-03-23-crucix-newspredict-design.md` (sections 3.1–3.6)

---

## File Map

| File | Responsibility |
|------|---------------|
| `rust/Cargo.toml` | Dependencies, binary config |
| `rust/.env.example` | Env var template |
| `rust/src/main.rs` | CLI entry, clap command dispatch |
| `rust/src/config.rs` | Self-implemented .env loader + Config struct |
| `rust/src/error.rs` | `CrucixError` enum via thiserror |
| `rust/src/http.rs` | `HttpClient` with timeout, retries, exponential backoff |
| `rust/src/sources/mod.rs` | `IntelSource` trait, `SourceResult`, `run_source()`, `build_sources()` |
| `rust/src/sources/yfinance.rs` | Yahoo Finance market quotes (no auth) |
| `rust/src/sources/usgs.rs` | USGS earthquake feed (GeoJSON, no auth) |
| `rust/src/sources/noaa.rs` | NWS severe weather alerts (no auth) |
| `rust/src/sources/gdelt.rs` | GDELT global news events (no auth) |
| `rust/src/sources/who.rs` | WHO disease outbreak news (no auth) |
| `rust/src/briefing.rs` | Parallel sweep orchestrator |
| `rust/tests/integration_test.rs` | End-to-end sweep tests |

---

## Task 1: Project Initialization

**Files:**
- Create: `rust/Cargo.toml`
- Create: `rust/src/main.rs`
- Create: `rust/src/error.rs`
- Create: `rust/.env.example`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "crucix"
version = "3.0.0"
edition = "2021"
description = "Local OSINT intelligence engine — 34 sources, single binary, zero cloud"
license = "AGPL-3.0-only"

[[bin]]
name = "crucix"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
thiserror = "2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
futures = "0.3"
```

- [ ] **Step 2: Create error.rs**

```rust
// rust/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrucixError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Source '{name}' failed: {message}")]
    Source { name: String, message: String },

    #[error("Source '{name}' timed out after {timeout_secs}s")]
    Timeout { name: String, timeout_secs: u64 },

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}
```

- [ ] **Step 3: Create minimal main.rs with clap**

```rust
// rust/src/main.rs
mod config;
mod error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "crucix", version, about = "Local OSINT intelligence engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show system status
    Status,
    /// Run intelligence sweep
    Sweep {
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Test a single source
    Source {
        /// Source name (e.g. yfinance, usgs)
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => {
            println!("Crucix Intelligence Engine v{}", env!("CARGO_PKG_VERSION"));
            println!("Status: OK");
        }
        Commands::Sweep { json: _ } => {
            println!("Sweep not yet implemented");
        }
        Commands::Source { name } => {
            println!("Source test for '{}' not yet implemented", name);
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Create .env.example**

```env
# === Crucix Engine Configuration ===

# Scan interval (minutes)
REFRESH_INTERVAL_MINUTES=15
# Per-source timeout (seconds)
SOURCE_TIMEOUT_SECS=30

# === Optional API Keys (progressive enhancement) ===
FRED_API_KEY=
FIRMS_MAP_KEY=
EIA_API_KEY=DEMO_KEY
```

- [ ] **Step 5: Verify it compiles and runs**

Run: `cd rust && cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors

Run: `cd rust && cargo run -- status`
Expected: `Crucix Intelligence Engine v3.0.0` + `Status: OK`

Run: `cd rust && cargo run -- --help`
Expected: Shows help with `status`, `sweep`, `source` subcommands

- [ ] **Step 6: Commit**

```bash
cd rust && git add -A && git commit -m "feat(rust): Phase 1.1 — project skeleton with clap CLI

- Cargo.toml with core dependencies (tokio, reqwest, serde, clap, etc.)
- main.rs with Status/Sweep/Source subcommands
- error.rs with CrucixError enum (thiserror)
- .env.example template

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Config Loader (self-implemented .env)

**Files:**
- Create: `rust/src/config.rs`
- Modify: `rust/src/main.rs` (add `mod config`, use in status)

- [ ] **Step 1: Implement config.rs with .env parser and Config struct**

```rust
// rust/src/config.rs
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// App configuration, loaded from environment variables (with .env fallback).
#[derive(Debug, Clone)]
pub struct Config {
    pub refresh_interval_minutes: u64,
    pub source_timeout_secs: u64,
    // API keys (all optional)
    pub fred_api_key: Option<String>,
    pub firms_map_key: Option<String>,
    pub eia_api_key: Option<String>,
}

impl Config {
    /// Load config: parse .env file (if present), then read env vars.
    /// .env values do NOT override existing env vars.
    pub fn load() -> Self {
        load_dotenv(".env");
        Self {
            refresh_interval_minutes: env_u64("REFRESH_INTERVAL_MINUTES", 15),
            source_timeout_secs: env_u64("SOURCE_TIMEOUT_SECS", 30),
            fred_api_key: env_opt("FRED_API_KEY"),
            firms_map_key: env_opt("FIRMS_MAP_KEY"),
            eia_api_key: env_opt("EIA_API_KEY"),
        }
    }
}

/// Parse a .env file and set env vars (skip if var already set).
fn load_dotenv(path: &str) {
    let Ok(content) = fs::read_to_string(Path::new(path)) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // Do not override existing env vars
            if std::env::var(key).is_err() && !value.is_empty() {
                std::env::set_var(key, value);
            }
        }
    }
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_dotenv_skips_comments_and_empty() {
        // Write a temp .env
        let tmp = std::env::temp_dir().join("crucix_test.env");
        fs::write(&tmp, "# comment\n\nTEST_KEY_123=hello\nTEST_EMPTY=\n").unwrap();
        load_dotenv(tmp.to_str().unwrap());
        assert_eq!(std::env::var("TEST_KEY_123").unwrap(), "hello");
        // Empty values are not set
        assert!(std::env::var("TEST_EMPTY").is_err());
        fs::remove_file(tmp).ok();
    }

    #[test]
    fn test_env_u64_default() {
        assert_eq!(env_u64("NONEXISTENT_KEY_XYZ", 42), 42);
    }

    #[test]
    fn test_config_loads_defaults() {
        let config = Config::load();
        assert_eq!(config.refresh_interval_minutes, 15);
        assert_eq!(config.source_timeout_secs, 30);
    }
}
```

- [ ] **Step 2: Update main.rs to use config in status command**

Update the `Commands::Status` arm in `main.rs`:

```rust
// In main.rs, update the Status handler:
Commands::Status => {
    let config = config::Config::load();
    println!("╔══════════════════════════════════════╗");
    println!("║   CRUCIX INTELLIGENCE ENGINE         ║");
    println!("║   v{}                          ║", env!("CARGO_PKG_VERSION"));
    println!("╠══════════════════════════════════════╣");
    println!("║  Refresh:  Every {} min              ║", config.refresh_interval_minutes);
    println!("║  Timeout:  {}s per source             ║", config.source_timeout_secs);
    println!("║  FRED:     {}  ║", if config.fred_api_key.is_some() { "configured" } else { "not set   " });
    println!("║  FIRMS:    {}  ║", if config.firms_map_key.is_some() { "configured" } else { "not set   " });
    println!("║  EIA:      {}  ║", if config.eia_api_key.is_some() { "configured" } else { "not set   " });
    println!("╚══════════════════════════════════════╝");
}
```

- [ ] **Step 3: Run tests and verify**

Run: `cd rust && cargo test -- --nocapture 2>&1 | tail -10`
Expected: 3 tests pass

Run: `cd rust && cargo run -- status`
Expected: Formatted status box showing defaults

- [ ] **Step 4: Commit**

```bash
cd rust && git add -A && git commit -m "feat(rust): Phase 1.1 — config loader with self-implemented .env parser

- Config struct with typed fields and defaults
- .env parser (comments, empty lines, no-override semantics)
- Unit tests for parser and defaults
- Status command shows live config

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: HTTP Client with Exponential Backoff

**Files:**
- Create: `rust/src/http.rs`
- Modify: `rust/src/main.rs` (add `mod http`)

- [ ] **Step 1: Implement HttpClient**

```rust
// rust/src/http.rs
use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// Shared HTTP client with connection pooling, timeout, and retry.
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    max_retries: u32,
}

impl HttpClient {
    pub fn new(timeout_secs: u64, max_retries: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(10)
            .user_agent("Crucix/3.0 IntelligenceEngine")
            .build()
            .expect("Failed to build HTTP client");
        Self { client, max_retries }
    }

    /// Fetch URL and parse as JSON. Retries with exponential backoff.
    pub async fn fetch_json(&self, url: &str) -> Result<Value> {
        self.fetch_with_retry(url, |resp| async {
            let text = resp.text().await?;
            let val: Value = serde_json::from_str(&text)?;
            Ok(val)
        })
        .await
    }

    /// Fetch URL and return raw text. Retries with exponential backoff.
    pub async fn fetch_text(&self, url: &str) -> Result<String> {
        self.fetch_with_retry(url, |resp| async {
            Ok(resp.text().await?)
        })
        .await
    }

    async fn fetch_with_retry<F, Fut, T>(&self, url: &str, parse: F) -> Result<T>
    where
        F: Fn(reqwest::Response) -> Fut + Copy,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_err = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }

            match self.do_fetch(url, parse).await {
                Ok(val) => return Ok(val),
                Err(e) => last_err = Some(e),
            }
        }

        Err(last_err.unwrap())
    }

    async fn do_fetch<F, Fut, T>(&self, url: &str, parse: F) -> Result<T>
    where
        F: Fn(reqwest::Response) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let resp = self.client.get(url).send().await?;

        if !resp.status().is_success() {
            bail!("HTTP {} for {}", resp.status(), url);
        }

        parse(resp).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpClient::new(15, 1);
        assert_eq!(client.max_retries, 1);
    }
}
```

- [ ] **Step 2: Add mod http to main.rs**

Add `mod http;` to the top of `main.rs` alongside the other mod declarations.

- [ ] **Step 3: Verify compilation**

Run: `cd rust && cargo build 2>&1 | tail -3`
Expected: `Finished` with no errors

Run: `cd rust && cargo test 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
cd rust && git add -A && git commit -m "feat(rust): Phase 1.2 — HTTP client with exponential backoff

- HttpClient with reqwest connection pool (10/host)
- fetch_json() and fetch_text() with configurable retries
- Exponential backoff: 500ms, 1s, 2s between retries

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: IntelSource Trait and Source Runner

**Files:**
- Create: `rust/src/sources/mod.rs`
- Modify: `rust/src/main.rs` (add `mod sources`)

- [ ] **Step 1: Define IntelSource trait, SourceResult, and runner**

```rust
// rust/src/sources/mod.rs
pub mod gdelt;
pub mod noaa;
pub mod usgs;
pub mod who;
pub mod yfinance;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::time::timeout;

use crate::http::HttpClient;

/// Every intelligence source implements this trait.
#[async_trait]
pub trait IntelSource: Send + Sync {
    /// Unique identifier (e.g. "gdelt", "yfinance")
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Intelligence tier 1-5 (1 = highest priority)
    fn tier(&self) -> u8;

    /// Execute one data collection sweep, returning structured JSON.
    /// The returned Value MUST contain a "source" field with self.name().
    async fn sweep(&self) -> Result<Value>;
}

/// Result of running a single source.
#[derive(Debug, Clone)]
pub struct SourceResult {
    pub name: String,
    pub status: SourceStatus,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tier: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceStatus {
    Ok,
    Error,
    Timeout,
}

impl std::fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceStatus::Ok => write!(f, "ok"),
            SourceStatus::Error => write!(f, "error"),
            SourceStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Run a single source with an independent timeout.
pub async fn run_source(source: &dyn IntelSource, max_timeout: Duration) -> SourceResult {
    let start = Instant::now();
    let result = timeout(max_timeout, source.sweep()).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(data)) => SourceResult {
            name: source.name().to_string(),
            status: SourceStatus::Ok,
            data: Some(data),
            error: None,
            duration_ms,
            tier: source.tier(),
        },
        Ok(Err(e)) => SourceResult {
            name: source.name().to_string(),
            status: SourceStatus::Error,
            data: None,
            error: Some(e.to_string()),
            duration_ms,
            tier: source.tier(),
        },
        Err(_) => SourceResult {
            name: source.name().to_string(),
            status: SourceStatus::Timeout,
            data: None,
            error: Some(format!("Timeout after {}s", max_timeout.as_secs())),
            duration_ms,
            tier: source.tier(),
        },
    }
}

/// Build all enabled sources for a sweep.
pub fn build_sources(client: &HttpClient) -> Vec<Box<dyn IntelSource>> {
    vec![
        Box::new(yfinance::YFinance::new(client.clone())),
        Box::new(usgs::Usgs::new(client.clone())),
        Box::new(noaa::Noaa::new(client.clone())),
        Box::new(gdelt::Gdelt::new(client.clone())),
        Box::new(who::Who::new(client.clone())),
    ]
}
```

- [ ] **Step 2: Create placeholder source files**

Create 5 placeholder files so the module compiles. Each will be a minimal stub that returns a dummy JSON:

```rust
// rust/src/sources/yfinance.rs
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::http::HttpClient;
use super::IntelSource;

pub struct YFinance {
    client: HttpClient,
}

impl YFinance {
    pub fn new(client: HttpClient) -> Self { Self { client } }
}

#[async_trait]
impl IntelSource for YFinance {
    fn name(&self) -> &str { "YFinance" }
    fn description(&self) -> &str { "Yahoo Finance live market quotes" }
    fn tier(&self) -> u8 { 5 }
    async fn sweep(&self) -> Result<Value> {
        Ok(json!({ "source": "YFinance", "status": "stub" }))
    }
}
```

Create the same pattern for `usgs.rs`, `noaa.rs`, `gdelt.rs`, `who.rs` (changing name/description/tier).

Tier assignments: GDELT=1, NOAA=3, USGS=1, WHO=1, YFinance=5.

- [ ] **Step 3: Add mod sources to main.rs**

Add `mod sources;` and `mod briefing;` (placeholder) to `main.rs`.

- [ ] **Step 4: Verify compilation and test**

Run: `cd rust && cargo build 2>&1 | tail -3`
Expected: `Finished` with no errors

- [ ] **Step 5: Commit**

```bash
cd rust && git add -A && git commit -m "feat(rust): Phase 1.3 — IntelSource trait, SourceResult, runner, 5 stub sources

- IntelSource trait with name/description/tier/sweep
- run_source() with independent timeout per source
- build_sources() registry returning 5 sources
- Stub implementations for yfinance, usgs, noaa, gdelt, who

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Implement 5 MVP Sources

**Files:**
- Modify: `rust/src/sources/yfinance.rs`
- Modify: `rust/src/sources/usgs.rs`
- Modify: `rust/src/sources/noaa.rs`
- Modify: `rust/src/sources/gdelt.rs`
- Modify: `rust/src/sources/who.rs`

Each source follows the same pattern: use `self.client.fetch_json()` to hit a public API, parse the response, return structured JSON with `"source"` field.

- [ ] **Step 1: Implement yfinance.rs**

Replace the stub with the full implementation. Key details from Node.js `yfinance.mjs`:
- URL: `https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?range=5d&interval=1d`
- Symbols: SPY, QQQ, DIA, BTC-USD, ETH-USD, GC=F, CL=F, ^VIX, etc.
- Parse `chart.result[0]`: `meta.regularMarketPrice`, `meta.chartPreviousClose`
- Calculate change and changePct
- Group by category: indexes, crypto, commodities, volatility
- User-Agent header needed (Yahoo blocks default)
- Each symbol fetched concurrently with `futures::future::join_all`

Output JSON shape:
```json
{
  "source": "YFinance",
  "timestamp": "...",
  "summary": { "totalSymbols": 15, "ok": 13, "failed": 2 },
  "quotes": { "SPY": { "symbol": "SPY", "name": "S&P 500", "price": 523.4, ... } }
}
```

- [ ] **Step 2: Verify yfinance compiles**

Run: `cd rust && cargo build 2>&1 | tail -3`

- [ ] **Step 3: Implement usgs.rs**

Key details (new source, not in Node.js):
- URL: `https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson`
- GeoJSON response: iterate `features` array
- Extract: magnitude, place, tsunami flag, coordinates
- Filter significant quakes (M5.0+)
- Track max magnitude, tsunami count

Output JSON shape:
```json
{
  "source": "USGS",
  "timestamp": "...",
  "totalQuakes": 45,
  "maxMagnitude": 6.2,
  "tsunamiWarnings": 0,
  "significantQuakes": [ { "magnitude": 6.2, "place": "...", "lat": ..., "lon": ... } ]
}
```

- [ ] **Step 4: Implement noaa.rs**

Key details from Node.js `noaa.mjs`:
- URL: `https://api.weather.gov/alerts/active?status=actual&severity=Extreme,Severe&limit=50`
- Header: `Accept: application/geo+json`
- Parse `features` array, categorize by event type (hurricane, tornado, flood, winter, fire)
- Extract centroid from GeoJSON geometry (Polygon/MultiPolygon/Point)

Output JSON shape:
```json
{
  "source": "NOAA/NWS",
  "timestamp": "...",
  "totalSevereAlerts": 12,
  "summary": { "hurricanes": 0, "tornadoes": 2, "floods": 5, ... },
  "topAlerts": [ { "event": "Tornado Warning", "severity": "Extreme", ... } ]
}
```

- [ ] **Step 5: Implement gdelt.rs**

Key details from Node.js `gdelt.mjs`:
- URL: `https://api.gdeltproject.org/api/v2/doc/doc?query=...&mode=ArtList&maxrecords=50&timespan=24h&format=json&sort=DateDesc`
- Query: `conflict OR military OR economy OR crisis OR war OR sanctions`
- Parse `articles` array, extract title/url/date/domain/language/sourcecountry
- Categorize by keywords in titles: conflicts, economy, health, crisis
- Rate limit: 5s between requests (skip GEO API in Phase 1)

Output JSON shape:
```json
{
  "source": "GDELT",
  "timestamp": "...",
  "totalArticles": 50,
  "conflicts": [...],
  "economy": [...],
  "health": [...],
  "crisis": [...]
}
```

- [ ] **Step 6: Implement who.rs**

Key details from Node.js `who.mjs`:
- URL: `https://www.who.int/api/news/diseaseoutbreaknews`
- JSON response with `value` array
- Sort by PublicationDate descending (server ignores OData orderby)
- Filter to last 30 days
- Strip HTML tags from summary (self-implemented, no library)
- Extract: title, date, URL, summary

Output JSON shape:
```json
{
  "source": "WHO",
  "timestamp": "...",
  "diseaseOutbreakNews": [ { "title": "...", "date": "...", "url": "...", "summary": "..." } ]
}
```

- [ ] **Step 7: Verify all sources compile**

Run: `cd rust && cargo build 2>&1 | tail -3`
Expected: `Finished` with no errors

- [ ] **Step 8: Commit**

```bash
cd rust && git add -A && git commit -m "feat(rust): Phase 1.4 — implement 5 MVP sources (YFinance, USGS, NOAA, GDELT, WHO)

- YFinance: 15 symbols (indexes, crypto, commodities, VIX)
- USGS: M2.5+ earthquakes from GeoJSON feed
- NOAA/NWS: severe weather alerts with geo centroid extraction
- GDELT: global news events categorized by conflict/economy/health/crisis
- WHO: disease outbreak news with HTML tag stripping
All sources: zero auth required, public APIs only

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Briefing Orchestrator (Parallel Sweep)

**Files:**
- Create: `rust/src/briefing.rs`
- Modify: `rust/src/main.rs` (wire sweep + source commands)

- [ ] **Step 1: Implement briefing.rs**

```rust
// rust/src/briefing.rs
use crate::http::HttpClient;
use crate::sources::{build_sources, run_source, SourceResult, SourceStatus};
use chrono::Utc;
use futures::future::join_all;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// Run a full parallel sweep of all sources.
pub async fn full_sweep(client: &HttpClient, timeout_secs: u64) -> Value {
    let sources = build_sources(client);
    let timeout = Duration::from_secs(timeout_secs);
    let start = Instant::now();

    eprintln!(
        "[Crucix] Starting intelligence sweep — {} sources...",
        sources.len()
    );

    // Run ALL sources in parallel (join_all = allSettled, not try_join_all)
    let futures: Vec<_> = sources
        .iter()
        .map(|src| run_source(src.as_ref(), timeout))
        .collect();
    let results = join_all(futures).await;

    let total_ms = start.elapsed().as_millis() as u64;
    build_output(results, total_ms)
}

/// Run a single named source.
pub async fn single_source(client: &HttpClient, name: &str, timeout_secs: u64) -> Option<Value> {
    let sources = build_sources(client);
    let timeout = Duration::from_secs(timeout_secs);

    let source = sources.iter().find(|s| s.name().eq_ignore_ascii_case(name))?;

    eprintln!("[Crucix] Testing source: {}", source.name());
    let result = run_source(source.as_ref(), timeout).await;

    let status_str = result.status.to_string();
    Some(json!({
        "source": result.name,
        "status": status_str,
        "tier": result.tier,
        "duration_ms": result.duration_ms,
        "data": result.data,
        "error": result.error,
    }))
}

/// List all available sources.
pub fn list_sources(client: &HttpClient) -> Vec<Value> {
    build_sources(client)
        .iter()
        .map(|s| {
            json!({
                "name": s.name(),
                "description": s.description(),
                "tier": s.tier(),
            })
        })
        .collect()
}

fn build_output(results: Vec<SourceResult>, total_ms: u64) -> Value {
    let sources_ok = results.iter().filter(|r| r.status == SourceStatus::Ok).count();
    let sources_err = results.iter().filter(|r| r.status != SourceStatus::Ok).count();

    let sources_data: serde_json::Map<String, Value> = results
        .iter()
        .filter(|r| r.status == SourceStatus::Ok)
        .filter_map(|r| r.data.clone().map(|d| (r.name.clone(), d)))
        .collect();

    let errors: Vec<Value> = results
        .iter()
        .filter(|r| r.status != SourceStatus::Ok)
        .map(|r| {
            json!({
                "name": r.name,
                "error": r.error,
                "status": r.status.to_string(),
            })
        })
        .collect();

    let timing: serde_json::Map<String, Value> = results
        .iter()
        .map(|r| {
            (
                r.name.clone(),
                json!({ "status": r.status.to_string(), "ms": r.duration_ms }),
            )
        })
        .collect();

    json!({
        "crucix": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": Utc::now().to_rfc3339(),
            "totalDurationMs": total_ms,
            "sourcesQueried": results.len(),
            "sourcesOk": sources_ok,
            "sourcesFailed": sources_err,
        },
        "sources": sources_data,
        "errors": errors,
        "timing": timing,
    })
}
```

- [ ] **Step 2: Wire CLI commands to briefing**

Update `main.rs` to use `briefing::full_sweep`, `briefing::single_source`, and `briefing::list_sources` in the `Sweep`, `Source`, and `Status` commands:

```rust
// main.rs — updated match arms

mod briefing;
mod config;
mod error;
mod http;
mod sources;

use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::http::HttpClient;

// ... Cli and Commands structs unchanged ...

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load();
    let client = HttpClient::new(config.source_timeout_secs, 1);

    match cli.command {
        Commands::Status => {
            println!("Crucix Intelligence Engine v{}", env!("CARGO_PKG_VERSION"));
            println!("Refresh: every {} min | Timeout: {}s/source",
                config.refresh_interval_minutes, config.source_timeout_secs);
            println!("\nAvailable sources:");
            for s in briefing::list_sources(&client) {
                println!("  [T{}] {} — {}",
                    s["tier"], s["name"], s["description"]);
            }
        }
        Commands::Sweep { json } => {
            let data = briefing::full_sweep(&client, config.source_timeout_secs).await;
            if json {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                // Human-readable summary
                let c = &data["crucix"];
                println!("\n[Crucix] Sweep complete in {}ms", c["totalDurationMs"]);
                println!("[Crucix] {}/{} sources OK",
                    c["sourcesOk"], c["sourcesQueried"]);
                if let Some(errors) = data["errors"].as_array() {
                    for e in errors {
                        eprintln!("  [FAIL] {}: {}", e["name"], e["error"]);
                    }
                }
                // Print timing
                if let Some(timing) = data["timing"].as_object() {
                    println!("\nTiming:");
                    for (name, t) in timing {
                        println!("  {} — {} ({}ms)",
                            name, t["status"], t["ms"]);
                    }
                }
            }
        }
        Commands::Source { name } => {
            match briefing::single_source(&client, &name, config.source_timeout_secs).await {
                Some(data) => println!("{}", serde_json::to_string_pretty(&data)?),
                None => {
                    eprintln!("Unknown source: '{}'. Run 'crucix status' to see available sources.", name);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Verify compilation and test CLI**

Run: `cd rust && cargo build 2>&1 | tail -3`
Expected: `Finished`

Run: `cd rust && cargo run -- status`
Expected: Shows version + 5 sources with tier/description

Run: `cd rust && cargo run -- sweep --json 2>/dev/null | head -20`
Expected: JSON output with `crucix.sourcesQueried: 5`

Run: `cd rust && cargo run -- source yfinance 2>/dev/null | head -10`
Expected: JSON for YFinance source

Run: `cd rust && cargo run -- source nonexistent 2>/dev/null`
Expected: Error message + exit code 1

- [ ] **Step 4: Commit**

```bash
cd rust && git add -A && git commit -m "feat(rust): Phase 1.5+1.6 — briefing orchestrator + CLI wiring

- full_sweep(): parallel execution via join_all (allSettled semantics)
- single_source(): test individual source by name
- list_sources(): enumerate available sources
- CLI: crucix status (list sources), sweep [--json], source <name>
- Output format matches Node.js version (crucix/sources/errors/timing)

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Integration Tests

**Files:**
- Create: `rust/tests/sweep_test.rs`

- [ ] **Step 1: Write integration tests**

```rust
// rust/tests/sweep_test.rs
//
// Integration tests for the Crucix sweep pipeline.
// These hit real APIs — run with: cargo test --test sweep_test
// Skip in CI with: cargo test --test sweep_test -- --ignored

use std::process::Command;

/// Test that `crucix status` runs and shows sources.
#[test]
fn test_status_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "status"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run crucix status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "status command failed: {}", stdout);
    assert!(stdout.contains("Crucix Intelligence Engine"));
    assert!(stdout.contains("YFinance"));
    assert!(stdout.contains("USGS"));
    assert!(stdout.contains("NOAA"));
    assert!(stdout.contains("GDELT"));
    assert!(stdout.contains("WHO"));
}

/// Test that `crucix sweep --json` produces valid JSON with expected structure.
#[test]
#[ignore] // Hits real APIs — run explicitly with: cargo test -- --ignored
fn test_sweep_json_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "sweep", "--json"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run crucix sweep");

    assert!(output.status.success(), "sweep failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&stdout)
        .expect("sweep output is not valid JSON");

    // Verify structure
    assert!(data["crucix"]["version"].is_string());
    assert!(data["crucix"]["timestamp"].is_string());
    assert_eq!(data["crucix"]["sourcesQueried"].as_u64().unwrap(), 5);
    assert!(data["crucix"]["sourcesOk"].as_u64().unwrap() >= 1,
        "At least 1 source should succeed");
    assert!(data["sources"].is_object());
    assert!(data["errors"].is_array());
    assert!(data["timing"].is_object());
}

/// Test that `crucix source <name>` works for a known source.
#[test]
#[ignore] // Hits real APIs
fn test_single_source() {
    // USGS is the most reliable free API
    let output = Command::new("cargo")
        .args(["run", "--", "source", "usgs"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run crucix source usgs");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&stdout)
        .expect("source output is not valid JSON");

    assert_eq!(data["source"].as_str().unwrap(), "USGS");
    assert_eq!(data["status"].as_str().unwrap(), "ok");
    assert!(data["data"]["totalQuakes"].is_number());
}

/// Test that unknown source name produces error exit.
#[test]
fn test_unknown_source_exits_with_error() {
    let output = Command::new("cargo")
        .args(["run", "--", "source", "nonexistent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run crucix source");

    assert!(!output.status.success(), "Should fail for unknown source");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown source"));
}

/// Test that parallel execution respects timeout isolation.
/// If one source hangs, others should still complete.
#[test]
#[ignore] // Hits real APIs
fn test_parallel_isolation() {
    let output = Command::new("cargo")
        .args(["run", "--", "sweep", "--json"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run sweep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Not valid JSON");

    // Even if some sources fail, others must succeed
    let queried = data["crucix"]["sourcesQueried"].as_u64().unwrap();
    let ok = data["crucix"]["sourcesOk"].as_u64().unwrap();
    assert_eq!(queried, 5, "All 5 sources should be queried");
    assert!(ok >= 1, "At least 1 should succeed even if others fail");

    // Every source must have timing data
    let timing = data["timing"].as_object().unwrap();
    assert_eq!(timing.len(), 5);
    for (_, t) in timing {
        assert!(t["ms"].is_number());
        assert!(t["status"].is_string());
    }
}
```

- [ ] **Step 2: Run non-API tests**

Run: `cd rust && cargo test --test sweep_test -- --skip ignored 2>&1 | tail -10`
Expected: `test_status_command` and `test_unknown_source_exits_with_error` pass

- [ ] **Step 3: Run full API integration tests (optional)**

Run: `cd rust && cargo test --test sweep_test -- --ignored --nocapture 2>&1 | tail -20`
Expected: Most tests pass (some sources may intermittently fail due to API availability)

- [ ] **Step 4: Run all unit tests too**

Run: `cd rust && cargo test 2>&1 | tail -10`
Expected: All unit tests + non-ignored integration tests pass

- [ ] **Step 5: Commit**

```bash
cd rust && git add -A && git commit -m "test(rust): Phase 1.7 — integration tests for sweep pipeline

- test_status_command: verifies CLI output lists all 5 sources
- test_sweep_json_output: validates JSON structure from full sweep
- test_single_source: tests individual source execution (USGS)
- test_unknown_source_exits_with_error: verifies error handling
- test_parallel_isolation: confirms timeout isolation across sources
- API-hitting tests marked #[ignore], run explicitly with --ignored

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Verification Checklist

After all tasks are complete, verify:

- [ ] `cd rust && cargo build --release` succeeds with no warnings
- [ ] `cd rust && cargo test` — all non-ignored tests pass
- [ ] `cd rust && cargo run -- status` — shows 5 sources
- [ ] `cd rust && cargo run -- sweep --json` — valid JSON with `sourcesQueried: 5`
- [ ] `cd rust && cargo run -- source usgs` — valid USGS data
- [ ] `cd rust && cargo run -- source nonexistent` — exits with error
- [ ] Each source's JSON output contains `"source": "SourceName"` field
- [ ] Git log shows 6 clean commits (Tasks 1-7, with Task 5+6 merged where noted)

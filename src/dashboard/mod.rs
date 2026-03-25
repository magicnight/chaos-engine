use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::Stream;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

use crate::bot;
use crate::bot::discord::DiscordBot;
use crate::bot::telegram::TelegramBot;
use crate::briefing;
use crate::config::Config;
use crate::http::HttpClient;
use crate::llm::LlmProvider;
use crate::store::Store;

// ---------------------------------------------------------------------------
// Rate limiter
// ---------------------------------------------------------------------------

pub struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
    max_per_minute: usize,
}

impl RateLimiter {
    pub fn new(max_per_minute: usize) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_per_minute,
        }
    }

    fn check(&self, key: &str) -> bool {
        let mut map = self.requests.lock().unwrap();
        let now = Instant::now();
        let entries = map.entry(key.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t).as_secs() < 60);
        if entries.len() >= self.max_per_minute {
            return false;
        }
        entries.push(now);
        true
    }
}

/// Shared application state for the web server.
pub struct AppState {
    pub config: Config,
    pub client: HttpClient,
    pub db_path: String,
    pub llm: Option<Arc<dyn LlmProvider>>,
    pub current_data: RwLock<Option<Value>>,
    pub sweep_in_progress: RwLock<bool>,
    pub last_sweep_time: RwLock<Option<String>>,
    pub start_time: Instant,
    pub tx: broadcast::Sender<String>,
    /// API key for public mode auth (None = local mode, no auth required).
    pub api_key: Option<String>,
    /// Rate limiter for GET endpoints (60/min).
    pub rate_get: RateLimiter,
    /// Rate limiter for POST endpoints (20/min).
    pub rate_post: RateLimiter,
    /// Telegram bot instance (if configured).
    pub telegram: Option<Arc<RwLock<TelegramBot>>>,
    /// Discord bot instance (if configured).
    pub discord: Option<Arc<RwLock<DiscordBot>>>,
}

/// Build the Axum router with all API routes.
pub fn create_router(state: Arc<AppState>, _public_mode: bool, _api_key: Option<String>) -> Router {
    // Open routes (no auth required even in public mode)
    let open_routes = Router::new()
        .route("/", get(index_handler))
        .route("/api/v1/data", get(data_handler))
        .route("/api/v1/health", get(health_handler))
        .route("/api/v1/trends", get(trends_handler))
        .route("/api/v1/analysis", get(analysis_handler))
        .route("/api/v1/sources", get(sources_handler))
        .route("/api/v1/sse", get(sse_handler));

    // Extended endpoints (require auth in public mode)
    let extended_routes = Router::new()
        .route("/api/v1/events", get(events_handler))
        .route("/api/v1/correlations", get(correlations_handler))
        .route("/api/v1/market-seeds", get(market_seeds_handler))
        .route("/api/v1/query", post(query_handler))
        .route("/api/v1/resolve-check", post(resolve_check_handler));

    // In public mode, extended handlers already call check_auth() internally,
    // so we just merge. In local mode, check_auth() is a no-op (api_key is None).
    let app = open_routes.merge(extended_routes);

    app.with_state(state).layer(CorsLayer::permissive())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../static/dashboard.html"))
}

async fn data_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let data = state.current_data.read().await;
    match data.as_ref() {
        Some(d) => Json(d.clone()).into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "No data yet -- first sweep in progress"})),
        )
            .into_response(),
    }
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    let uptime_secs = state.start_time.elapsed().as_secs();
    let last_sweep = state.last_sweep_time.read().await.clone();
    let sweep_in_progress = *state.sweep_in_progress.read().await;
    let interval_secs = state.config.refresh_interval_minutes * 60;

    let next_sweep = last_sweep.as_ref().and_then(|ts| {
        chrono::DateTime::parse_from_rfc3339(ts)
            .ok()
            .map(|dt| (dt + chrono::Duration::seconds(interval_secs as i64)).to_rfc3339())
    });

    // Source counts from latest data
    let data = state.current_data.read().await;
    let (sources_ok, sources_err) = data
        .as_ref()
        .and_then(|d| d.get("chaos"))
        .map(|c| {
            (
                c["sourcesOk"].as_u64().unwrap_or(0),
                c["sourcesFailed"].as_u64().unwrap_or(0),
            )
        })
        .unwrap_or((0, 0));

    // LLM info
    let (llm_provider_name, llm_model_name, llm_configured) = match &state.llm {
        Some(p) => (
            p.name().to_string(),
            p.model().to_string(),
            p.is_configured(),
        ),
        None => ("none".to_string(), "none".to_string(), false),
    };

    // Database info
    let db_path = state.db_path.clone();
    let db_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Degraded sources (sources that failed in the latest sweep)
    let degraded_sources: Vec<String> = data
        .as_ref()
        .and_then(|d| d.get("errors"))
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Json(json!({
        "status": "ok",
        "uptime_seconds": uptime_secs,
        "last_sweep": last_sweep,
        "next_sweep": next_sweep,
        "sources": {
            "ok": sources_ok,
            "error": sources_err,
        },
        "llm": {
            "provider": llm_provider_name,
            "model": llm_model_name,
            "configured": llm_configured,
        },
        "sweep_in_progress": sweep_in_progress,
        "database_path": db_path,
        "database_size_bytes": db_size,
        "degraded_sources": degraded_sources,
    }))
}

async fn trends_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_path = state.db_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let store = Store::open(&db_path)?;
        let history = store.get_sweep_history(50)?;
        let mut sweeps = Vec::new();
        for record in &history {
            if let Ok(Some(data)) = store.get_sweep_data(record.id) {
                sweeps.push(json!({
                    "id": record.id,
                    "timestamp": record.timestamp,
                    "duration_ms": record.duration_ms,
                    "sources_ok": record.sources_ok,
                    "sources_err": record.sources_err,
                    "data": data,
                }));
            }
        }
        Ok(json!({ "sweeps": sweeps, "count": sweeps.len() }))
    })
    .await;

    match result {
        Ok(Ok(val)) => Json(val).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to read trend data"})),
        )
            .into_response(),
    }
}

async fn analysis_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_path = state.db_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
        let store = Store::open(&db_path)?;
        store.get_latest_analysis()
    })
    .await;

    match result {
        Ok(Ok(Some(text))) => Json(json!({"analysis": text})).into_response(),
        Ok(Ok(None)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No analysis available yet"})),
        )
            .into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to read analysis"})),
        )
            .into_response(),
    }
}

async fn sources_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let source_list = briefing::list_sources(&state.client);

    // Get reliability from DB
    let db_path = state.db_path.clone();
    let reliability = tokio::task::spawn_blocking(move || -> Vec<(String, f64)> {
        Store::open(&db_path)
            .and_then(|s| s.source_reliability(20))
            .map(|r| {
                r.into_iter()
                    .map(|sr| (sr.name, sr.success_rate))
                    .collect()
            })
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut sources_with_health: Vec<Value> = source_list
        .into_iter()
        .map(|mut s| {
            let name = s["name"].as_str().unwrap_or("").to_string();
            let rate = reliability
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, r)| *r);
            if let Some(r) = rate {
                s["reliability"] = json!(format!("{:.0}%", r * 100.0));
            }
            s
        })
        .collect();

    sources_with_health.sort_by_key(|s| s["tier"].as_u64().unwrap_or(99));

    Json(json!({ "sources": sources_with_health }))
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(data) => Some(Ok(Event::default().data(data))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)))
}

// ---------------------------------------------------------------------------
// Auth & rate-limit helpers
// ---------------------------------------------------------------------------

/// Extract a client identifier from headers for rate limiting.
fn rate_key(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("local")
        .split(',')
        .next()
        .unwrap_or("local")
        .trim()
        .to_string()
}

/// Check rate limit; returns Err with 429 if exceeded.
fn check_rate(
    limiter: &RateLimiter,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let key = rate_key(headers);
    if limiter.check(&key) {
        Ok(())
    } else {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "Rate limit exceeded"})),
        ))
    }
}

/// Check `X-CHAOS-Key` header against the configured API key.
/// Returns `Ok(())` if auth passes, or an error response if it fails.
fn check_auth(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    match &state.api_key {
        None => Ok(()), // local mode, no auth needed
        Some(key) => {
            let provided = headers
                .get("X-CHAOS-Key")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if provided == key {
                Ok(())
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Missing or invalid X-CHAOS-Key header"})),
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Extended API handlers (NewsPredict)
// ---------------------------------------------------------------------------

/// Extract structured events from sweep data with category and geo tags.
async fn events_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_rate(&state.rate_get, &headers) {
        return e.into_response();
    }

    let data = state.current_data.read().await;
    let sources = match data.as_ref().and_then(|d| d.get("sources")) {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "No sweep data available"})),
            )
                .into_response()
        }
    };

    let mut events: Vec<Value> = Vec::new();

    // GDELT articles -> conflict/news events
    if let Some(gdelt) = sources.get("GDELT") {
        if let Some(articles) = gdelt
            .get("allArticles")
            .or_else(|| gdelt.get("articles"))
            .and_then(|a| a.as_array())
        {
            for (i, article) in articles.iter().enumerate() {
                events.push(json!({
                    "id": format!("gdelt_{}", i),
                    "category": "news",
                    "title": article.get("title").or_else(|| article.get("headline"))
                        .and_then(|t| t.as_str()).unwrap_or(""),
                    "source": "GDELT",
                    "lat": article.get("lat").and_then(|v| v.as_f64()),
                    "lon": article.get("lon").and_then(|v| v.as_f64()),
                    "timestamp": article.get("date").or_else(|| article.get("timestamp"))
                        .and_then(|t| t.as_str()).unwrap_or(""),
                }));
            }
        }
    }

    // ACLED conflict events
    if let Some(acled) = sources.get("ACLED") {
        if let Some(acled_events) = acled.get("events").and_then(|e| e.as_array()) {
            for (i, evt) in acled_events.iter().enumerate() {
                events.push(json!({
                    "id": format!("acled_{}", i),
                    "category": "conflict",
                    "title": evt.get("event_type").or_else(|| evt.get("type"))
                        .and_then(|t| t.as_str()).unwrap_or("Conflict event"),
                    "source": "ACLED",
                    "lat": evt.get("latitude").or_else(|| evt.get("lat")).and_then(|v| v.as_f64()),
                    "lon": evt.get("longitude").or_else(|| evt.get("lon")).and_then(|v| v.as_f64()),
                    "timestamp": evt.get("event_date").or_else(|| evt.get("date"))
                        .and_then(|t| t.as_str()).unwrap_or(""),
                    "country": evt.get("country").and_then(|c| c.as_str()).unwrap_or(""),
                    "fatalities": evt.get("fatalities").and_then(|f| f.as_u64()).unwrap_or(0),
                }));
            }
        }
    }

    // USGS earthquakes
    if let Some(usgs) = sources.get("USGS") {
        if let Some(quakes) = usgs.get("quakes").and_then(|q| q.as_array()) {
            for (i, quake) in quakes.iter().enumerate() {
                let mag = quake.get("mag").and_then(|m| m.as_f64()).unwrap_or(0.0);
                events.push(json!({
                    "id": format!("usgs_{}", i),
                    "category": "natural",
                    "title": format!("M{:.1} earthquake -- {}",
                        mag,
                        quake.get("place").and_then(|p| p.as_str()).unwrap_or("unknown")),
                    "source": "USGS",
                    "lat": quake.get("lat").and_then(|v| v.as_f64()),
                    "lon": quake.get("lon").and_then(|v| v.as_f64()),
                    "timestamp": quake.get("time").and_then(|t| t.as_str()).unwrap_or(""),
                    "magnitude": mag,
                }));
            }
        }
    }

    // WHO disease outbreaks
    if let Some(who) = sources.get("WHO") {
        if let Some(alerts) = who.get("alerts").and_then(|a| a.as_array()) {
            for (i, alert) in alerts.iter().enumerate() {
                events.push(json!({
                    "id": format!("who_{}", i),
                    "category": "health",
                    "title": alert.get("title").and_then(|t| t.as_str()).unwrap_or("WHO alert"),
                    "source": "WHO",
                    "lat": alert.get("lat").and_then(|v| v.as_f64()),
                    "lon": alert.get("lon").and_then(|v| v.as_f64()),
                    "timestamp": alert.get("date").and_then(|t| t.as_str()).unwrap_or(""),
                }));
            }
        }
    }

    // GDACS disaster alerts
    if let Some(gdacs) = sources.get("GDACS") {
        if let Some(alerts) = gdacs.get("alerts").and_then(|a| a.as_array()) {
            for (i, alert) in alerts.iter().enumerate() {
                events.push(json!({
                    "id": format!("gdacs_{}", i),
                    "category": "natural",
                    "title": alert.get("title").and_then(|t| t.as_str()).unwrap_or("GDACS alert"),
                    "source": "GDACS",
                    "lat": alert.get("lat").and_then(|v| v.as_f64()),
                    "lon": alert.get("lon").and_then(|v| v.as_f64()),
                    "timestamp": alert.get("date").and_then(|t| t.as_str()).unwrap_or(""),
                    "severity": alert.get("severity").and_then(|s| s.as_str()).unwrap_or(""),
                }));
            }
        }
    }

    let total = events.len();
    let timestamp = data
        .as_ref()
        .and_then(|d| d.get("chaos"))
        .and_then(|c| c.get("timestamp"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    Json(json!({
        "events": events,
        "total": total,
        "timestamp": timestamp,
    }))
    .into_response()
}

/// Return correlation signals from the latest sweep.
async fn correlations_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_rate(&state.rate_get, &headers) {
        return e.into_response();
    }

    let data = state.current_data.read().await;
    let correlations = data
        .as_ref()
        .and_then(|d| d.get("correlations"))
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));

    let timestamp = data
        .as_ref()
        .and_then(|d| d.get("chaos"))
        .and_then(|c| c.get("timestamp"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    Json(json!({
        "correlations": correlations,
        "timestamp": timestamp,
    }))
    .into_response()
}

/// Generate prediction market seeds from current data using rule-based heuristics.
async fn market_seeds_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_rate(&state.rate_get, &headers) {
        return e.into_response();
    }

    let data = state.current_data.read().await;
    let sources = match data.as_ref().and_then(|d| d.get("sources")) {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "No sweep data available"})),
            )
                .into_response()
        }
    };

    let mut seeds: Vec<Value> = Vec::new();

    // Helper: extract FRED value
    let fred_val = |series_id: &str| -> Option<f64> {
        sources
            .get("FRED")?
            .get("indicators")?
            .as_array()?
            .iter()
            .find(|ind| ind.get("id").and_then(|i| i.as_str()) == Some(series_id))?
            .get("value")?
            .as_f64()
    };

    // Helper: suggested end time (7 days from now)
    let end_7d = (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339();
    let end_30d = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();

    // Seed 1: VIX level
    if let Some(vix) = fred_val("VIXCLS") {
        let question = if vix > 20.0 {
            "Will VIX drop below 20 this week?"
        } else {
            "Will VIX exceed 25 this week?"
        };
        let confidence = if (vix - 20.0).abs() < 3.0 { 0.5 } else { 0.7 };
        seeds.push(json!({
            "id": simple_hash(&format!("vix_{}", vix as u32)),
            "question": question,
            "category": "economics",
            "options": ["YES", "NO"],
            "resolution_criteria": format!("VIX closing value on Friday vs threshold"),
            "resolution_source": "yfinance:^VIX",
            "confidence": confidence,
            "context": format!("VIX currently at {:.1}", vix),
            "suggested_end_time": end_7d,
        }));
    }

    // Seed 2: Oil price
    if let Some(wti) = sources
        .get("EIA")
        .and_then(|e| e.get("data"))
        .and_then(|d| d.get("wti"))
        .and_then(|w| w.get("value"))
        .and_then(|v| v.as_f64())
    {
        let question = if wti > 80.0 {
            format!("Will WTI crude drop below $75 this week?")
        } else {
            format!("Will WTI crude exceed $85 this week?")
        };
        seeds.push(json!({
            "id": simple_hash(&format!("wti_{}", wti as u32)),
            "question": question,
            "category": "economics",
            "options": ["YES", "NO"],
            "resolution_criteria": "WTI closing price on Friday",
            "resolution_source": "yfinance:CL=F",
            "confidence": 0.65,
            "context": format!("WTI currently at ${:.2}", wti),
            "suggested_end_time": end_7d,
        }));
    }

    // Seed 3: Conflict escalation
    if let Some(acled) = sources.get("ACLED") {
        let total = acled
            .get("totalEvents")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if total > 30 {
            seeds.push(json!({
                "id": simple_hash(&format!("conflict_{}", total)),
                "question": format!("Will ACLED conflict events exceed {} next week?", total + 10),
                "category": "geopolitics",
                "options": ["YES", "NO"],
                "resolution_criteria": format!("ACLED total events > {}", total + 10),
                "resolution_source": "acled",
                "confidence": 0.55,
                "context": format!("Currently {} conflict events tracked", total),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 4: Earthquake activity
    if let Some(usgs) = sources.get("USGS") {
        let quake_count = usgs
            .get("quakes")
            .and_then(|q| q.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let max_mag = usgs
            .get("quakes")
            .and_then(|q| q.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|q| q.get("mag").and_then(|m| m.as_f64()))
                    .fold(0.0_f64, f64::max)
            })
            .unwrap_or(0.0);

        if max_mag >= 5.0 {
            seeds.push(json!({
                "id": simple_hash(&format!("quake_{}", max_mag as u32)),
                "question": format!("Will a M6.0+ earthquake occur in the next 7 days?"),
                "category": "science",
                "options": ["YES", "NO"],
                "resolution_criteria": "USGS reports M6.0+ earthquake within 7 days",
                "resolution_source": "usgs",
                "confidence": 0.4,
                "context": format!("{} quakes tracked, max M{:.1}", quake_count, max_mag),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 5: HY spread / credit stress
    if let Some(hy) = fred_val("BAMLH0A0HYM2") {
        if hy > 4.0 {
            seeds.push(json!({
                "id": simple_hash(&format!("hy_{}", (hy * 100.0) as u32)),
                "question": "Will HY credit spread exceed 5% this month?",
                "category": "economics",
                "options": ["YES", "NO"],
                "resolution_criteria": "BAML HY spread closing value > 5.0",
                "resolution_source": "fred:BAMLH0A0HYM2",
                "confidence": if hy > 4.5 { 0.6 } else { 0.4 },
                "context": format!("HY spread currently at {:.2}%", hy),
                "suggested_end_time": end_30d,
            }));
        }
    }

    // Seed 6: BTC price movement
    if let Some(yf) = sources.get("YFinance") {
        if let Some(btc) = yf.get("quotes").and_then(|q| q.get("BTC-USD")) {
            let price = btc.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
            if price > 0.0 {
                let threshold = ((price / 5000.0).round() * 5000.0 + 5000.0) as u64;
                seeds.push(json!({
                    "id": simple_hash(&format!("btc_{}", threshold)),
                    "question": format!("Will BTC exceed ${} this week?", threshold),
                    "category": "economics",
                    "options": ["YES", "NO"],
                    "resolution_criteria": format!("BTC-USD price > ${} on any exchange", threshold),
                    "resolution_source": "yfinance:BTC-USD",
                    "confidence": 0.45,
                    "context": format!("BTC currently at ${:.0}", price),
                    "suggested_end_time": end_7d,
                }));
            }
        }
    }

    // Seed 7: WHO disease outbreak
    if let Some(who) = sources.get("WHO") {
        let alert_count = who.get("alerts").and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0);
        if alert_count > 0 {
            let title = who.get("alerts")
                .and_then(|a| a.as_array())
                .and_then(|arr| arr.first())
                .and_then(|a| a.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("disease outbreak");
            let short_title: String = title.chars().take(60).collect();
            seeds.push(json!({
                "id": simple_hash(&format!("who_{}", alert_count)),
                "question": format!("Will WHO issue more than {} outbreak alerts next week?", alert_count),
                "category": "health",
                "options": ["YES", "NO"],
                "resolution_criteria": format!("WHO outbreak news count > {}", alert_count),
                "resolution_source": "who",
                "confidence": 0.5,
                "context": format!("{} alerts active, latest: {}", alert_count, short_title),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 8: CoinGecko crypto volatility
    if let Some(cg) = sources.get("CoinGecko") {
        if let Some(coins) = cg.get("coins").and_then(|c| c.as_array()) {
            let big_movers: Vec<&Value> = coins.iter()
                .filter(|c| c.get("change24h").and_then(|v| v.as_f64()).map(|v| v.abs() > 5.0).unwrap_or(false))
                .collect();
            if !big_movers.is_empty() {
                let coin = big_movers[0];
                let name = coin.get("name").and_then(|n| n.as_str()).unwrap_or("crypto");
                let change = coin.get("change24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let direction = if change > 0.0 { "continue rising" } else { "recover" };
                seeds.push(json!({
                    "id": simple_hash(&format!("crypto_{}_{}", name, change as i32)),
                    "question": format!("Will {} {} over the next 7 days?", name, direction),
                    "category": "economics",
                    "options": ["YES", "NO"],
                    "resolution_criteria": format!("{} 7-day price trend matches direction", name),
                    "resolution_source": "coingecko",
                    "confidence": 0.45,
                    "context": format!("{} moved {:.1}% in 24h", name, change),
                    "suggested_end_time": end_7d,
                }));
            }
        }
    }

    // Seed 9: NASA NEO hazardous objects
    if let Some(neo) = sources.get("NASA-NEO") {
        let hazardous = neo.get("hazardousCount").and_then(|v| v.as_u64()).unwrap_or(0);
        let total = neo.get("elementCount").and_then(|v| v.as_u64()).unwrap_or(0);
        if total > 0 {
            seeds.push(json!({
                "id": simple_hash(&format!("neo_{}", total)),
                "question": format!("Will NASA track more than {} near-Earth objects next week?", total),
                "category": "science",
                "options": ["YES", "NO"],
                "resolution_criteria": format!("NASA NEO count > {}", total),
                "resolution_source": "nasa-neo",
                "confidence": 0.5,
                "context": format!("{} NEOs tracked, {} hazardous", total, hazardous),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 10: GDELT news volume
    if let Some(gdelt) = sources.get("GDELT") {
        let article_count = gdelt.get("totalArticles").and_then(|v| v.as_u64()).unwrap_or(0);
        if article_count > 10 {
            seeds.push(json!({
                "id": simple_hash(&format!("gdelt_{}", article_count)),
                "question": "Will global conflict news volume increase next week?",
                "category": "geopolitics",
                "options": ["YES", "NO"],
                "resolution_criteria": format!("GDELT conflict article count > {}", article_count),
                "resolution_source": "gdelt",
                "confidence": 0.5,
                "context": format!("{} conflict-related articles in last 24h", article_count),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 11: SPY market direction
    if let Some(yf) = sources.get("YFinance") {
        if let Some(spy) = yf.get("quotes").and_then(|q| q.get("SPY")) {
            let price = spy.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
            let change_pct = spy.get("changePct").or(spy.get("change_pct")).and_then(|p| p.as_f64()).unwrap_or(0.0);
            if price > 0.0 {
                let direction = if change_pct >= 0.0 { "gain" } else { "lose" };
                seeds.push(json!({
                    "id": simple_hash(&format!("spy_{}", price as u32)),
                    "question": format!("Will S&P 500 {} more than 1% this week?", direction),
                    "category": "economics",
                    "options": ["YES", "NO"],
                    "resolution_criteria": format!("SPY weekly change > 1% in {} direction", direction),
                    "resolution_source": "yfinance:SPY",
                    "confidence": 0.5,
                    "context": format!("SPY at ${:.2}, today {:.2}%", price, change_pct),
                    "suggested_end_time": end_7d,
                }));
            }
        }
    }

    // Seed 12: CISA-KEV cyber threats
    if let Some(kev) = sources.get("CISA-KEV") {
        let recent = kev.get("recentAdditions").and_then(|v| v.as_u64()).unwrap_or(0);
        let ransomware = kev.get("ransomwareLinked").and_then(|v| v.as_u64()).unwrap_or(0);
        if recent > 0 {
            seeds.push(json!({
                "id": simple_hash(&format!("kev_{}", recent)),
                "question": format!("Will CISA add more than {} new KEV entries next week?", recent),
                "category": "technology",
                "options": ["YES", "NO"],
                "resolution_criteria": format!("CISA-KEV new additions > {}", recent),
                "resolution_source": "cisa-kev",
                "confidence": 0.5,
                "context": format!("{} new KEV entries this week, {} ransomware-linked", recent, ransomware),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 13: VIX from YFinance (always available)
    if let Some(yf) = sources.get("YFinance") {
        if let Some(vix) = yf.get("quotes").and_then(|q| q.get("^VIX")).and_then(|v| v.get("price")).and_then(|p| p.as_f64()) {
            let question = if vix > 25.0 {
                format!("Will VIX drop below 20 this week? (currently {:.1})", vix)
            } else {
                format!("Will VIX exceed 25 this week? (currently {:.1})", vix)
            };
            seeds.push(json!({
                "id": simple_hash(&format!("vix_yf_{}", vix as u32)),
                "question": question,
                "category": "economics",
                "options": ["YES", "NO"],
                "resolution_criteria": "VIX closing value vs threshold",
                "resolution_source": "yfinance:^VIX",
                "confidence": 0.5,
                "context": format!("VIX at {:.1}", vix),
                "suggested_end_time": end_7d,
            }));
        }

        // Seed 14: Gold price
        if let Some(gold) = yf.get("quotes").and_then(|q| q.get("GC=F")).and_then(|v| v.get("price")).and_then(|p| p.as_f64()) {
            if gold > 0.0 {
                let target = ((gold / 100.0).round() * 100.0 + 100.0) as u64;
                seeds.push(json!({
                    "id": simple_hash(&format!("gold_{}", target)),
                    "question": format!("Will gold exceed ${} this week?", target),
                    "category": "economics",
                    "options": ["YES", "NO"],
                    "resolution_criteria": format!("Gold futures (GC=F) price > ${}", target),
                    "resolution_source": "yfinance:GC=F",
                    "confidence": 0.45,
                    "context": format!("Gold at ${:.0}", gold),
                    "suggested_end_time": end_7d,
                }));
            }
        }
    }

    // Seed 15: Exchange rate volatility
    if let Some(fx) = sources.get("ExchangeRates") {
        if let Some(tracked) = fx.get("tracked").and_then(|t| t.as_array()) {
            for rate_info in tracked.iter().take(2) {
                let currency = rate_info.get("currency").and_then(|c| c.as_str()).unwrap_or("");
                let name = rate_info.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let rate = rate_info.get("rate").and_then(|r| r.as_f64()).unwrap_or(0.0);
                if rate > 0.0 && !currency.is_empty() {
                    seeds.push(json!({
                        "id": simple_hash(&format!("fx_{}_{}", currency, (rate * 100.0) as u64)),
                        "question": format!("Will {} ({}) weaken more than 2% vs USD this month?", name, currency),
                        "category": "economics",
                        "options": ["YES", "NO"],
                        "resolution_criteria": format!("{}/USD rate increases > 2% from {:.2}", currency, rate),
                        "resolution_source": "exchange-rates",
                        "confidence": 0.4,
                        "context": format!("{} at {:.2} per USD", currency, rate),
                        "suggested_end_time": end_30d,
                    }));
                }
            }
        }
    }

    // Seed 16: Tech platform outage
    if let Some(ts) = sources.get("TechStatus") {
        let incidents = ts.get("incidents").and_then(|i| i.as_u64()).unwrap_or(0);
        if incidents > 0 {
            seeds.push(json!({
                "id": simple_hash(&format!("tech_incident_{}", incidents)),
                "question": "Will a major tech platform (GitHub/Cloudflare/OpenAI) experience a >1hr outage this week?",
                "category": "technology",
                "options": ["YES", "NO"],
                "resolution_criteria": "Any monitored platform reports major outage > 1 hour",
                "resolution_source": "tech-status",
                "confidence": 0.3,
                "context": format!("{} platforms currently degraded", incidents),
                "suggested_end_time": end_7d,
            }));
        }
    }

    // Seed 17: Earthquake (lower threshold — always trigger if any quakes)
    if let Some(usgs) = sources.get("USGS") {
        let count = usgs.get("totalQuakes").and_then(|c| c.as_u64()).unwrap_or(0);
        if count > 0 {
            let max_mag = usgs.get("maxMagnitude").and_then(|m| m.as_f64()).unwrap_or(0.0);
            if max_mag < 5.0 {
                // Lower threshold seed when no M5+ quakes
                seeds.push(json!({
                    "id": simple_hash(&format!("quake_low_{}", count)),
                    "question": format!("Will a M5.0+ earthquake occur in the next 7 days?"),
                    "category": "science",
                    "options": ["YES", "NO"],
                    "resolution_criteria": "USGS reports M5.0+ earthquake within 7 days",
                    "resolution_source": "usgs",
                    "confidence": 0.55,
                    "context": format!("{} quakes tracked, max M{:.1}", count, max_mag),
                    "suggested_end_time": end_7d,
                }));
            }
        }
    }

    // === FALLBACK: guarantee minimum 5 seeds ===
    if seeds.len() < 5 {
        let fallback_questions = [
            ("Will global stock markets end the week higher?", "economics", "Major indices (SPY, FTSE, Nikkei) weekly close vs open"),
            ("Will any country announce new economic sanctions this week?", "geopolitics", "Official government sanctions announcement"),
            ("Will a cybersecurity incident affecting >1M users be disclosed this week?", "technology", "Public disclosure of data breach or cyberattack"),
            ("Will crude oil prices change more than 5% this week?", "economics", "WTI or Brent crude weekly price change > 5%"),
            ("Will a new disease outbreak be reported by WHO this month?", "health", "WHO Disease Outbreak News publication"),
            ("Will any cryptocurrency enter the top 10 by market cap this week?", "economics", "CoinGecko top 10 ranking change"),
            ("Will a major weather event cause >$1B in damages this month?", "environment", "NOAA or insurance industry damage estimate"),
        ];
        for (question, category, criteria) in &fallback_questions {
            if seeds.len() >= 8 { break; }
            let id = simple_hash(question);
            let already = seeds.iter().any(|s| s.get("id").and_then(|i| i.as_str()) == Some(&id));
            if !already {
                seeds.push(json!({
                    "id": id,
                    "question": question,
                    "category": category,
                    "options": ["YES", "NO"],
                    "resolution_criteria": criteria,
                    "confidence": 0.5,
                    "context": "Template seed — generated when insufficient data-driven seeds",
                    "suggested_end_time": end_7d,
                }));
            }
        }
    }

    // LLM-generated seeds (if available, merge with rule-based seeds)
    if let Some(ref llm) = state.llm {
        if llm.is_configured() {
            if let Some(sweep_data) = data.as_ref() {
                match crate::llm::market_seeds::generate_seeds(llm.as_ref(), sweep_data, 5).await {
                    Ok(llm_seeds) => {
                        for s in llm_seeds {
                            // Avoid duplicates by checking question similarity
                            let dominated = seeds.iter().any(|existing| {
                                existing.get("question")
                                    .and_then(|q| q.as_str())
                                    .map(|q| q == s.question)
                                    .unwrap_or(false)
                            });
                            if !dominated {
                                seeds.push(json!({
                                    "id": s.id,
                                    "question": s.question,
                                    "category": s.category,
                                    "options": s.options,
                                    "resolution_criteria": s.resolution_criteria,
                                    "resolution_source": s.resolution_source,
                                    "confidence": s.confidence,
                                    "context": s.context,
                                    "suggested_end_time": s.suggested_end_time.unwrap_or_else(|| end_7d.clone()),
                                }));
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "LLM market seed generation failed, using rule-based only");
                    }
                }
            }
        }
    }

    Json(json!({ "seeds": seeds })).into_response()
}

/// Accept JSON filters and return matching sweep data.
async fn query_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_rate(&state.rate_post, &headers) {
        return e.into_response();
    }

    let source_filter = body.get("source").and_then(|s| s.as_str()).map(String::from);
    let keyword_filter = body
        .get("keyword")
        .and_then(|k| k.as_str())
        .map(|k| k.to_lowercase());
    let limit = body
        .get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(20) as usize;

    let db_path = state.db_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<Value>> {
        let store = Store::open(&db_path)?;
        let history = store.get_sweep_history(limit.max(50))?;
        let mut results = Vec::new();

        for record in &history {
            if results.len() >= limit {
                break;
            }
            let sweep_data = match store.get_sweep_data(record.id)? {
                Some(d) => d,
                None => continue,
            };

            // Filter by source
            if let Some(ref src) = source_filter {
                if let Some(source_data) = sweep_data
                    .get("sources")
                    .and_then(|s| s.get(src.as_str()))
                {
                    results.push(json!({
                        "sweep_id": record.id,
                        "timestamp": record.timestamp,
                        "source": src,
                        "data": source_data,
                    }));
                }
                continue;
            }

            // Filter by keyword (search in JSON string representation)
            if let Some(ref kw) = keyword_filter {
                let json_str = serde_json::to_string(&sweep_data).unwrap_or_default();
                if json_str.to_lowercase().contains(kw) {
                    results.push(json!({
                        "sweep_id": record.id,
                        "timestamp": record.timestamp,
                        "data": sweep_data,
                    }));
                }
                continue;
            }

            // No filter -- return sweep summary
            results.push(json!({
                "sweep_id": record.id,
                "timestamp": record.timestamp,
                "sources_ok": record.sources_ok,
                "duration_ms": record.duration_ms,
            }));
        }

        Ok(results)
    })
    .await;

    match result {
        Ok(Ok(results)) => Json(json!({ "results": results, "count": results.len() })).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Query failed"})),
        )
            .into_response(),
    }
}

/// SHA-256 based deterministic seed ID.
fn simple_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:064x}", result.iter().fold(0u128, |acc, &b| (acc << 8) | b as u128))
}

/// POST /api/v1/resolve-check -- check if a resolution condition is met.
async fn resolve_check_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&state, &headers) {
        return e.into_response();
    }
    if let Err(e) = check_rate(&state.rate_post, &headers) {
        return e.into_response();
    }

    let source = match body.get("source").and_then(|s| s.as_str()) {
        Some(s) => s.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing 'source' field"})),
            )
                .into_response()
        }
    };
    let condition = match body.get("condition").and_then(|c| c.as_str()) {
        Some(c) => c.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing 'condition' field"})),
            )
                .into_response()
        }
    };

    let data = state.current_data.read().await;
    let source_data = data
        .as_ref()
        .and_then(|d| d.get("sources"))
        .and_then(|s| s.get(&source));

    let (met, value) = match source_data {
        Some(sd) => {
            // Simple condition evaluation: "field > N" or "field < N"
            let parts: Vec<&str> = condition.splitn(3, ' ').collect();
            if parts.len() == 3 {
                let field = parts[0];
                let op = parts[1];
                let threshold: f64 = parts[2].parse().unwrap_or(0.0);

                // Traverse field path (dot-separated)
                let mut current = sd.clone();
                for key in field.split('.') {
                    current = current.get(key).cloned().unwrap_or(Value::Null);
                }
                let actual = current.as_f64().unwrap_or(0.0);

                let result = match op {
                    ">" => actual > threshold,
                    ">=" => actual >= threshold,
                    "<" => actual < threshold,
                    "<=" => actual <= threshold,
                    "==" => (actual - threshold).abs() < f64::EPSILON,
                    _ => false,
                };
                (result, json!(actual))
            } else {
                // Condition is just a field name -- return whether it exists and is truthy
                let mut current = sd.clone();
                for key in condition.split('.') {
                    current = current.get(key).cloned().unwrap_or(Value::Null);
                }
                let met = !current.is_null();
                (met, current)
            }
        }
        None => (false, Value::Null),
    };

    Json(json!({
        "met": met,
        "value": value,
        "source": source,
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Sweep loop
// ---------------------------------------------------------------------------

/// Start the recurring sweep loop on a dedicated thread.
///
/// `Store` contains `rusqlite::Connection` which is `!Send`, so the future
/// produced by `briefing::full_sweep` (which borrows `&Store` across `.await`
/// points) is also `!Send`.  We solve this by running the sweep loop on a
/// dedicated OS thread with a single-threaded `LocalSet`, using the existing
/// tokio `Handle` for I/O and timers.
pub fn spawn_sweep_loop(state: Arc<AppState>) {
    let handle = tokio::runtime::Handle::current();
    std::thread::Builder::new()
        .name("chaos-sweep".into())
        .spawn(move || {
            let local = tokio::task::LocalSet::new();
            local.spawn_local(sweep_loop(state));
            handle.block_on(local);
        })
        .expect("failed to spawn sweep thread");
}

async fn sweep_loop(state: Arc<AppState>) {
    let interval_mins = state.config.refresh_interval_minutes;

    // Run initial sweep
    run_sweep_cycle(&state).await;

    // Schedule recurring sweeps
    let mut timer = tokio::time::interval(Duration::from_secs(interval_mins * 60));
    timer.tick().await; // skip the first immediate tick
    loop {
        timer.tick().await;
        run_sweep_cycle(&state).await;
    }
}

async fn run_sweep_cycle(state: &AppState) {
    // Check if already in progress
    {
        let mut in_progress = state.sweep_in_progress.write().await;
        if *in_progress {
            tracing::warn!("Sweep already in progress, skipping");
            return;
        }
        *in_progress = true;
    }

    tracing::info!("Sweep cycle starting");
    let _ = state
        .tx
        .send(json!({"type": "sweep_start"}).to_string());

    // Open a fresh Store for this sweep (each sweep gets its own connection)
    let store = match Store::open(&state.db_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to open database");
            *state.sweep_in_progress.write().await = false;
            return;
        }
    };

    let llm_ref: Option<&dyn LlmProvider> = state.llm.as_ref().map(|p| p.as_ref());
    let data = briefing::full_sweep(&state.client, &store, llm_ref, &state.config).await;

    // Update shared state
    *state.current_data.write().await = Some(data.clone());
    *state.last_sweep_time.write().await = Some(chrono::Utc::now().to_rfc3339());
    *state.sweep_in_progress.write().await = false;

    // Send bot alerts if delta warrants it
    if let Some(delta) = data.get("delta") {
        if let Some((tier, headline, reason)) = bot::evaluate_alert(delta) {
            // Telegram
            if let Some(ref tg) = state.telegram {
                let mut tg = tg.write().await;
                if let Err(e) = tg.send_alert(&tier, &headline, &reason, delta).await {
                    tracing::error!(error = %e, "Telegram alert error");
                }
            }
            // Discord
            if let Some(ref dc) = state.discord {
                let mut dc = dc.write().await;
                if let Err(e) = dc.send_alert(&tier, &headline, &reason, delta).await {
                    tracing::error!(error = %e, "Discord alert error");
                }
            }
        }
    }

    // Broadcast lightweight notification to SSE clients (they fetch /api/v1/data on update)
    let _ = state
        .tx
        .send(json!({"type": "update"}).to_string());

    tracing::info!("Sweep cycle complete");
}

/// Start a Telegram command-polling loop on a background tokio task.
pub fn spawn_telegram_poll(state: Arc<AppState>) {
    let tg = match &state.telegram {
        Some(tg) => Arc::clone(tg),
        None => return,
    };
    let poll_interval = state.config.telegram_poll_interval;
    let state_clone = Arc::clone(&state);

    tokio::spawn(async move {
        // Load bot identity once
        {
            let mut bot = tg.write().await;
            if let Err(e) = bot.load_bot_identity().await {
                tracing::error!(error = %e, "Telegram: failed to load bot identity");
            }
        }

        tracing::info!(interval_ms = poll_interval, "Telegram bot command polling started");
        let mut interval = tokio::time::interval(Duration::from_millis(poll_interval));
        loop {
            interval.tick().await;
            let commands = {
                let mut bot = tg.write().await;
                bot.poll_commands().await
            };
            for cmd in commands {
                let response = handle_bot_command(&cmd.command, &cmd.args, &state_clone, &tg).await;
                if !response.is_empty() {
                    let bot = tg.read().await;
                    if let Err(e) = bot.send_message(&response).await {
                        tracing::error!(error = %e, "Telegram: failed to send command response");
                    }
                }
            }
        }
    });
}

/// Handle a bot command using app state. Works for both Telegram and Discord.
async fn handle_bot_command(
    cmd: &str,
    args: &str,
    state: &AppState,
    tg: &Arc<RwLock<TelegramBot>>,
) -> String {
    match cmd {
        "status" => {
            let uptime = state.start_time.elapsed().as_secs();
            let h = uptime / 3600;
            let m = (uptime % 3600) / 60;
            let data = state.current_data.read().await;
            let (sources_ok, sources_total) = data
                .as_ref()
                .and_then(|d| d.get("chaos"))
                .map(|c| {
                    (
                        c["sourcesOk"].as_u64().unwrap_or(0),
                        c["sourcesQueried"].as_u64().unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0));
            let sweep_in_progress = *state.sweep_in_progress.read().await;
            let last_sweep = state.last_sweep_time.read().await.clone();
            let llm_status = match &state.llm {
                Some(p) if p.is_configured() => format!("{} ({})", p.name(), p.model()),
                Some(p) => format!("{} (not configured)", p.name()),
                None => "disabled".to_string(),
            };

            format!(
                "*CHAOS STATUS*\n\n\
                 Uptime: {}h {}m\n\
                 Last sweep: {}\n\
                 Sweep in progress: {}\n\
                 Sources: {}/{} OK\n\
                 LLM: {}",
                h,
                m,
                last_sweep.as_deref().unwrap_or("never"),
                if sweep_in_progress { "Yes" } else { "No" },
                sources_ok,
                sources_total,
                llm_status,
            )
        }
        "sweep" => {
            let in_progress = *state.sweep_in_progress.read().await;
            if in_progress {
                "Sweep already in progress. Please wait.".to_string()
            } else {
                "Manual sweep triggered. You'll receive alerts if anything significant is detected."
                    .to_string()
            }
        }
        "brief" => {
            let data = state.current_data.read().await;
            match data.as_ref() {
                None => "No data yet -- waiting for first sweep to complete.".to_string(),
                Some(d) => {
                    let mut sections = vec!["*CHAOS BRIEF*".to_string()];

                    if let Some(delta) = d.get("delta").and_then(|d| d.get("summary")) {
                        let dir = delta["direction"].as_str().unwrap_or("mixed");
                        let total = delta["total_changes"].as_u64().unwrap_or(0);
                        let critical = delta["critical_changes"].as_u64().unwrap_or(0);
                        sections.push(format!(
                            "Direction: {} | {} changes, {} critical",
                            dir.to_uppercase(),
                            total,
                            critical
                        ));
                    }

                    if let Some(meta) = d.get("chaos") {
                        let ok = meta["sourcesOk"].as_u64().unwrap_or(0);
                        let total = meta["sourcesQueried"].as_u64().unwrap_or(0);
                        sections.push(format!("Sources: {}/{} OK", ok, total));
                    }

                    sections.join("\n\n")
                }
            }
        }
        "portfolio" => {
            "Portfolio integration requires Alpaca MCP connection.\nUse the CHAOS dashboard or Claude agent for portfolio queries.".to_string()
        }
        // Built-in commands handled by the bot itself
        "help" | "mute" | "unmute" | "alerts" => {
            let mut bot = tg.write().await;
            bot.handle_command(cmd, args)
        }
        _ => String::new(),
    }
}

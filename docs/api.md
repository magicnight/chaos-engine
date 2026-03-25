# CHAOS Engine API Documentation

## Base URL
- Local: `http://localhost:3117`
- Public: `https://your-domain.com` (with Caddy reverse proxy, see `Caddyfile.example`)

## Authentication
Public mode (`--public --api-key KEY`) requires the `X-CHAOS-Key` header on extended endpoints.
In local mode (the default), no authentication is required on any endpoint.

## Rate Limits
- GET endpoints: 60 requests per minute per IP
- POST endpoints: 20 requests per minute per IP

---

## Endpoints

### Core Endpoints (no auth required in local mode)

#### GET /
Returns the CHAOS MONITOR dashboard HTML page.

**Response:** `text/html`

---

#### GET /api/v1/data
Returns the latest sweep data including all source results, analysis, delta, and correlations.

**Response:**
```json
{
  "chaos": {
    "version": "0.1.0",
    "timestamp": "2026-03-24T18:00:00Z",
    "sourcesQueried": 35,
    "sourcesOk": 33,
    "sourcesFailed": 2,
    "totalDurationMs": 30000
  },
  "sources": {
    "YFinance": { "source": "YFinance", "quotes": { "SPY": { "price": 580.12, "change_pct": 0.34 } } },
    "FRED": { "source": "FRED", "indicators": [...] },
    "USGS": { "source": "USGS", "earthquakes": [...] },
    "..."
  },
  "errors": [
    { "name": "OpenSky", "error": "timed out after 15s", "status": "timeout" }
  ],
  "timing": {
    "YFinance": { "status": "ok", "ms": 1234 },
    "FRED": { "status": "ok", "ms": 567 }
  },
  "delta": {
    "summary": { "total_changes": 5, "critical_changes": 1, "direction": "deteriorating" },
    "changes": [...]
  },
  "correlations": [
    { "name": "Oil-Conflict Correlation", "severity": "high", "sources": ["EIA", "ACLED"], "description": "...", "indicators": [...] }
  ],
  "watchlist_matches": [
    { "type": "keyword", "matched": "Ukraine", "source": "GDELT", "context": "..." }
  ],
  "analysis": {
    "text": "## SITUATION OVERVIEW\n...",
    "model": "glm-5",
    "input_tokens": 12000,
    "output_tokens": 2048
  }
}
```

**Error (503):** No data yet (first sweep in progress).

---

#### GET /api/v1/health
System health check with uptime, source status, LLM info, and database stats.

**Response:**
```json
{
  "status": "ok",
  "uptime_seconds": 3600,
  "last_sweep": "2026-03-24T18:00:00Z",
  "next_sweep": "2026-03-24T18:15:00Z",
  "sources": { "ok": 33, "error": 2 },
  "llm": { "provider": "ollama", "model": "glm-5", "configured": true },
  "sweep_in_progress": false,
  "database_path": "runs/chaos.db",
  "database_size_bytes": 1048576,
  "degraded_sources": ["OpenSky"]
}
```

---

#### GET /api/v1/trends
Historical sweep data (last 50 sweeps).

**Response:**
```json
{
  "sweeps": [
    {
      "id": 42,
      "timestamp": "2026-03-24T18:00:00Z",
      "duration_ms": 30000,
      "sources_ok": 33,
      "sources_err": 2,
      "data": { "..." }
    }
  ],
  "count": 42
}
```

---

#### GET /api/v1/analysis
Latest LLM analysis text.

**Response:**
```json
{ "analysis": "## SITUATION OVERVIEW\n..." }
```

**Error (404):** No analysis available yet.

---

#### GET /api/v1/sources
Source list with tier, description, and reliability statistics.

**Response:**
```json
{
  "sources": [
    { "name": "ACLED", "description": "Armed Conflict Location & Event Data", "tier": 1, "reliability": "95%" },
    { "name": "FRED", "description": "Federal Reserve Economic Data", "tier": 2, "reliability": "100%" }
  ]
}
```

---

#### GET /api/v1/sse
Server-Sent Events stream. Sends events when new data is available.

**Event format:**
```
data: {"type":"update"}

data: {"type":"sweep_start"}
```

Connect with `EventSource`:
```javascript
const es = new EventSource("/api/v1/sse");
es.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  if (msg.type === "update") {
    // Fetch fresh data from /api/v1/data
  }
};
```

---

### Extended Endpoints (require `X-CHAOS-Key` in public mode)

#### GET /api/v1/events
Structured events extracted from all sources with category and geo tags.

**Response:**
```json
{
  "events": [
    { "source": "ACLED", "category": "conflict", "title": "...", "location": "...", "timestamp": "..." }
  ]
}
```

---

#### GET /api/v1/correlations
Cross-source correlation signals from the correlation engine.

**Response:**
```json
{
  "correlations": [
    {
      "name": "Oil-Conflict Correlation",
      "severity": "high",
      "sources": ["EIA", "ACLED"],
      "description": "WTI crude spike coincides with escalation in armed conflict events",
      "indicators": [...]
    }
  ]
}
```

---

#### GET /api/v1/market-seeds
Prediction market seed suggestions generated via LLM or rule-based heuristics.

**Response:**
```json
{
  "seeds": [
    { "question": "Will WTI crude exceed $85 by end of week?", "category": "commodities", "confidence": 0.65 }
  ]
}
```

---

#### POST /api/v1/query
Query historical data with filters.

**Request body:**
```json
{ "source": "USGS", "limit": 10 }
```

**Response:**
```json
{
  "results": [
    { "sweep_id": 42, "timestamp": "2026-03-24T18:00:00Z", "data": { "..." } }
  ]
}
```

---

#### POST /api/v1/resolve-check
Check if a condition is met in current data. Useful for prediction market resolution.

**Request body:**
```json
{ "source": "YFinance", "condition": "SPY > 600" }
```

**Response:**
```json
{ "met": true, "value": 605.23, "source": "YFinance" }
```

---

## Source Output Schemas (Key Fields)

Each source returns a JSON object under `sources.<SourceName>` in the sweep data. Below are the key fields for each source.

### Tier 1 -- Core OSINT

| Source | Key Fields |
|--------|------------|
| **ACLED** | `events[]` -- armed conflict events with `event_type`, `country`, `fatalities`, `date` |
| **ADS-B** | `aircraft[]` -- tracked military/notable aircraft with `icao24`, `callsign`, `latitude`, `longitude`, `altitude` |
| **FIRMS** | `hotspots[]` -- active fire detections with `latitude`, `longitude`, `brightness`, `confidence` |
| **GDACS** | `alerts[]` -- natural disaster alerts with `type`, `severity`, `country`, `description` |
| **GDELT** | `articles[]` -- global news events with `title`, `url`, `tone`, `themes`, `locations` |
| **OpenSky** | `states[]` -- aircraft state vectors with position and velocity |
| **ReliefWeb** | `reports[]` -- humanitarian reports with `title`, `country`, `disaster_type` |
| **Safecast** | `measurements[]` -- radiation readings with `latitude`, `longitude`, `value`, `unit` |
| **Sanctions** | `entries[]` -- sanctioned entities with `name`, `program`, `country` |
| **Ships** | `vessels[]` -- maritime vessel positions with `mmsi`, `name`, `latitude`, `longitude` |
| **SWPC** | `alerts[]` -- space weather alerts with `type`, `severity`, `issue_time` |
| **Telegram** | `messages[]` -- OSINT channel messages with `channel`, `text`, `date` |
| **USGS** | `earthquakes[]` -- seismic events with `magnitude`, `place`, `latitude`, `longitude`, `depth` |
| **WHO** | `outbreaks[]` -- disease outbreak reports with `title`, `country`, `disease` |

### Tier 2 -- Economic/Financial

| Source | Key Fields |
|--------|------------|
| **BLS** | `data` -- labor statistics with series data |
| **Comtrade** | `trades[]` -- international trade flows with `reporter`, `partner`, `value` |
| **EIA** | `data` -- energy data including `wti.value`, `brent.value`, `natural_gas.value` |
| **FRED** | `indicators[]` -- Federal Reserve indicators (VIX, DGS10, etc.) with `series_id`, `value`, `date` |
| **GSCPI** | `data` -- Global Supply Chain Pressure Index |
| **Treasury** | `yields[]` -- US Treasury yield curve data |
| **USASpending** | `awards[]` -- federal spending awards with `amount`, `agency`, `recipient` |
| **WorldNews** | `articles[]` -- aggregated world news |

### Tier 3

| Source | Key Fields |
|--------|------------|
| **Bluesky** | `posts[]` -- social media posts with sentiment |
| **CISA-KEV** | `vulnerabilities[]` -- known exploited vulnerabilities with `cve_id`, `vendor`, `product` |
| **Cloudflare Radar** | `data` -- internet traffic anomalies and attack trends |
| **CVE** | `vulnerabilities[]` -- recent CVEs with `id`, `description`, `severity` |
| **EPA** | `data` -- environmental monitoring data |
| **ISC** | `data` -- Internet Storm Center threat level |
| **KiwiSDR** | `receivers[]` -- software-defined radio receivers and signals |
| **NOAA** | `alerts[]` -- weather alerts with `headline`, `severity`, `area` |
| **Patents** | `patents[]` -- recent patent filings in strategic domains |
| **Reddit** | `posts[]` -- OSINT-relevant subreddit posts with `title`, `subreddit`, `score` |

### Tier 4

| Source | Key Fields |
|--------|------------|
| **CelesTrak** | `satellites[]` -- satellite tracking data with TLE elements |

### Tier 5

| Source | Key Fields |
|--------|------------|
| **YFinance** | `quotes` -- market quotes keyed by ticker (SPY, BTC-USD, GC=F, etc.) with `price`, `change_pct`, `volume` |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level filter (e.g., `chaos=debug`) | `chaos=info` |
| `CHAOS_LOG_FORMAT` | Set to `json` for structured JSON log output | text (compact) |

See `.env.example` for the full list of source API keys and configuration options.

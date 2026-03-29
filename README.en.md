> **[中文](README.md)** | English

<div align="center">

# C.H.A.O.S.

**Connected Human-Augmented OSINT Suite**

*Order from chaos. Insight before impact.*

```
   ██████╗ ██╗  ██╗  █████╗   ██████╗  ███████╗
  ██╔════╝ ██║  ██║ ██╔══██╗ ██╔═══██╗ ██╔════╝
  ██║      ███████║ ███████║ ██║   ██║ ███████╗
  ██║      ██╔══██║ ██╔══██║ ██║   ██║ ╚════██║
  ╚██████╗ ██║  ██║ ██║  ██║ ╚██████╔╝ ███████║
   ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═════╝  ╚══════╝
```

[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPLv3-blue.svg)](LICENSE)
[![Sources](https://img.shields.io/badge/OSINT%20sources-46-cyan)](#data-sources-46)
[![LLM](https://img.shields.io/badge/LLM-multi--provider-green)](#ai-analysis)
[![Docker](https://img.shields.io/badge/docker-ready-2496ED)](#deployment)

</div>

---

## What is CHAOS?

CHAOS pulls from **46 open-source intelligence feeds** in parallel -- satellite fire detection, flight tracking, radiation monitoring, earthquake data, economic indicators, conflict events, cyber vulnerabilities, sanctions lists, disease outbreaks, social sentiment, and more -- then synthesizes everything into a single actionable picture updated every 15 minutes.

Connect an LLM and it becomes a **multilingual intelligence analyst** generating structured briefings in English, Chinese, Japanese, or Spanish, with cross-domain correlation detection and anomaly flagging. Alerts push to Telegram and Discord bots with three-tier severity classification (FLASH / PRIORITY / ROUTINE), and the bots accept commands back -- sweep on demand, request a briefing, check system status, all from your phone.

Everything renders on a self-contained **Jarvis-style dashboard** with a 3D globe, draggable Gridstack panels, 15 live news streams, real-time SSE updates, and full API access for downstream consumers. Single binary. Embedded SQLite. Zero cloud dependency. Zero telemetry. Zero subscriptions.

The companion **NewsPredict prediction market** adds a notification center, achievement badges, copy-trading, market sentiment analysis, Sentry error monitoring, and API rate limiting.

![CHAOS MONITOR](docs/dashboard.png)

---

## Quick Start

```bash
git clone https://github.com/magicnight/chaos-engine.git && cd chaos-engine
cargo build --release
cp .env.example .env          # edit with your API keys (optional)
./target/release/chaos serve
```

Dashboard at `http://localhost:3117`. First sweep completes in ~30 seconds.

### Docker (Recommended)

**Windows (Podman):**
```powershell
.\scripts\dev-start.ps1              # One-click start (auto .env, DB migration, seed)
.\scripts\dev-start.ps1 -Rebuild     # Rebuild and start
```

**Linux / Mac (Docker):**
```bash
./scripts/dev-start.sh               # One-click start
./scripts/dev-start.sh --rebuild     # Rebuild and start
```

Visit `http://localhost:8080`

---

## Features

### Intelligence Collection (46 Sources)

All sources run in parallel via `tokio::join_all` with per-tier timeouts (T1: 30s, T2: 24s, T3: 15s). 20+ sources work with **zero API keys**.

| Tier | Focus | Count | Sources |
|------|-------|------:|---------|
| **T1** Core OSINT | Conflict, disasters, health, transport | 16 | ACLED, ADS-B, FIRMS, GDACS, GDELT, OpenSky, ProMED-mail, ReliefWeb, Safecast, Sanctions (OFAC+OpenSanctions), Ships, SWPC, Telegram, Tsunami, USGS, WHO |
| **T2** Economic | Markets, trade, fiscal | 11 | BLS, CoinGecko, Comtrade, ECB, EIA, ExchangeRates, FRED, GSCPI, Treasury, USAspending, WorldNews |
| **T3** Supplementary | Cyber, environment, social, tech | 16 | Bluesky, CISA-KEV, Cloudflare Radar, Copernicus, CVE/NVD, EPA RadNet, EU Sanctions, Google Trends, ISC/SANS, KiwiSDR, NASA NEO, NOAA, NTSB, Patents, Reddit, RIPE Atlas, TechStatus |
| **T4** Space | Orbital tracking | 1 | CelesTrak |
| **T5** Markets | Live quotes | 1 | Yahoo Finance |

### AI Analysis

- **LLM fallback chain**: primary provider -> fallback provider -> Ollama local (automatic failover, 60s timeout)
- **10 provider backends**: OpenAI, Anthropic, Gemini, Ollama, DeepSeek, Moonshot, OpenRouter, Mistral, MiniMax, ZhipuAI
- **4-language analysis**: `--lang en|zh|ja|es` -- full military-style briefing prompts in each language
- **Structured output**: Situation Overview, Key Developments, Risk Matrix, Actionable Intelligence, Cross-Domain Correlations
- **Market prediction seeds**: 17 rules + LLM generation + 7 template fallbacks, guaranteeing >=5 seeds per sweep

### CHAOS MONITOR Dashboard

- **22 draggable panels** organized into 9 categories, all toggleable in Settings
- **Gridstack.js** panel system with drag, resize, and layout persistence (localStorage)
- **3D globe** with real-time event plotting (quakes, fires, conflicts, weather)
- **15 live news streams**: Bloomberg, Al Jazeera, France 24, DW, Sky News, CNBC, NHK, CCTV4, Phoenix TV, and more via embedded YouTube live
- **Server-Sent Events** for live data streaming
- **Rate-limited public API** mode with API key authentication
- **tower-http static file serving**: standalone `static/` directory with `ServeDir` dynamic loading

| Category | Panels |
|----------|--------|
| Situational | Situation Map, Transport & Airspace |
| Financial | Market Data, Risk Gauges, Energy & Macro, Global Economy |
| Security | Conflicts, OSINT Stream, Sanctions Watch |
| News | News Feed, Trends & Innovation |
| Natural | Seismic Monitor, Nuclear Watch, Climate & Environment |
| Cyber | Cyber Threats, Network Intel |
| Space | Space Watch, NEO Tracker |
| System | Source Health, Delta / Changes |
| AI | AI Intelligence Brief, Cross-Source Signals |

### NewsPredict Prediction Market

- **Notification center**: trade confirmations, market settlements, achievement unlocks, copy-trade alerts -- viewable on profile page
- **Achievement badges**: auto-unlocked based on trading behavior (first trade, win streaks, copy-trading, etc.)
- **Copy-trading**: follow top traders and automatically mirror their trades with configurable per-trade limits
- **Market sentiment analysis**: auto-computed bullish/bearish/neutral sentiment from comments, displayed as sentiment badges with confidence scores
- **SSE real-time updates**: live price and market data via CHAOS Engine SSE feed
- **API rate limiting**: comments, trades, and other endpoints protected by rate limiting
- **Sentry error monitoring**: @sentry/nextjs integration for production error tracking

### Multi-Tier Alerts

| Tier | Label | Trigger | Cooldown | Max/Hour |
|------|-------|---------|----------|----------|
| FLASH | Immediate action | 2+ critical signals or 5+ critical changes | 5 min | 6 |
| PRIORITY | Act within hours | 1 critical or 2+ high severity signals | 30 min | 4 |
| ROUTINE | Informational | 3+ total changes or 2+ new signals | 60 min | 2 |

- **Telegram bot**: two-way -- receives commands (`/status`, `/sweep`, `/brief`, `/mute`, `/help`), sends tiered alerts with Markdown formatting
- **Discord bot**: webhook mode or full bot token mode with rich embeds and color-coded severity
- **Desktop notifications**: native toast on Windows, macOS, and Linux
- **Webhook**: Slack / Discord / Feishu compatible generic webhook

### Historical Analysis

- **Delta engine**: 25 tracked metrics (14 numeric + 11 count) with severity scoring (Moderate / High / Critical)
- **6 correlation rules**: Natural Disaster Cascade, Geopolitical Risk Convergence, Cyber Threat Convergence, Infrastructure Stress, Market Panic, Humanitarian Crisis
- **Anomaly detection**: z-score analysis against historical sweep data, flags deviations > 2 sigma
- **Trend sparklines**: ASCII visualization of metric history in terminal
- **Risk direction**: automatic classification (risk-on / risk-off / mixed) based on weighted key indicators

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `chaos status` | Engine status, config, source availability, LLM connection |
| `chaos sweep` | Run full intelligence sweep across all 46 sources |
| `chaos sweep --json` | JSON output for piping to other tools |
| `chaos sweep --lang zh` | Sweep with Chinese-language LLM analysis |
| `chaos sweep --no-llm` | Skip LLM analysis even if configured |
| `chaos serve` | Start web dashboard on localhost:3117 |
| `chaos serve --public --api-key KEY` | Public API mode (binds 0.0.0.0, rate-limited) |
| `chaos serve --port 8080` | Custom port |
| `chaos watch --interval 15` | Continuous headless sweep loop |
| `chaos source <name>` | Test a single source by name |
| `chaos test-llm` | Test LLM connection and get a sample response |
| `chaos history` | Show sweep history (default: last 10) |
| `chaos history --show 42` | Show full data for a specific sweep ID |
| `chaos trends` | ASCII sparklines for all tracked metrics |
| `chaos trends --anomalies` | Flag statistically anomalous values |
| `chaos report` | Generate Markdown intelligence report |
| `chaos export --format csv` | Export trend data as CSV |
| `chaos export --format json` | Export trend data as JSON |
| `chaos market-seeds --count 10` | Generate prediction market seed questions |
| `chaos migrate --from-json ./runs` | Migrate data from legacy Node.js JSON files |

---

## Why Rust?

The CHAOS Engine core is written entirely in Rust. For an intelligence system that must collect 46 data sources in parallel, analyze streaming data in real time, and run reliably around the clock, Rust is the only language that simultaneously delivers:

| Advantage | What it means for CHAOS |
|-----------|------------------------|
| **Zero-cost concurrency** | `tokio` async runtime + `join_all` parallel collection across 46 sources. A single thread handles thousands of concurrent connections with no GC pauses |
| **Memory safety without GC** | Ownership system eliminates data races and memory leaks at compile time. Runs 24/7 with zero crashes -- no Go-style GC latency or Python-style memory bloat |
| **Single binary** | `cargo build --release` produces one ~15MB static binary containing the web server, dashboard, SQLite, and all 46 source parsers. No runtime dependencies. Container image is ~30MB |
| **C/C++ performance** | JSON parsing (serde), regex matching, and data aggregation run at C speed. CPU usage is near-zero while waiting for LLM responses |
| **Type system as documentation** | Every data source response is strongly typed at compile time. `enum` + `match` exhaustiveness ensures no branch is missed |
| **Cargo ecosystem** | axum (web), rusqlite (SQLite), reqwest (HTTP), tokio (async) -- mature, production-grade libraries with no left-pad-style supply chain risk |
| **Cross-platform** | Same codebase compiles to Linux / macOS / Windows / ARM. Runs on a Raspberry Pi |

**Compared to alternatives:**
- **Python**: Fast to write but slow to run. GIL limits concurrency, high memory usage, deployment requires virtual environments
- **Go**: Good concurrency but unpredictable GC, weak generics, verbose error handling
- **Node.js**: Single-threaded event loop works for I/O but blocks on CPU-intensive analysis, weak type safety
- **Java/C#**: Slow JVM/CLR startup, heavy memory footprint, poor fit for edge devices and lightweight containers

Rust's trade-off is a steeper learning curve and longer compile times, but for an intelligence engine that must be **reliable, efficient, and run indefinitely**, the investment pays for itself.

---

## Architecture

```
                        ┌─────────────────────────────────┐
                        │         CLI (clap v4)           │
                        │  status│sweep│serve│watch│...   │
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
     ┌────────▼────────┐    ┌─────────▼─────────┐   ┌─────────▼─────────┐
     │   46 Sources     │    │   LLM Fallback    │   │   Dashboard       │
     │  (async parallel)│    │   Chain            │   │  (Axum+tower-http)│
     │                  │    │                    │   │                   │
     │  T1: Core OSINT  │    │  OpenAI-compat     │   │  Gridstack panels │
     │  T2: Economic    │    │  Anthropic         │   │  3D Globe         │
     │  T3: Supplement  │    │  Gemini            │   │  Real-time SSE    │
     │  T4: Space       │    │  Ollama (local)    │   │  ServeDir static  │
     │  T5: Markets     │    └─────────┬─────────┘   └─────────┬─────────┘
     └────────┬────────┘              │                        │
              │                        │                        │
              └────────────────────────┼────────────────────────┘
                                       │
                        ┌──────────────▼──────────────────┐
                        │          Core Engine             │
                        │                                  │
                        │  Briefing ─ Delta ─ Correlation  │
                        │  Store (SQLite) ─ Watchlist      │
                        │  Anomaly Detection               │
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
     ┌────────▼────────┐    ┌─────────▼─────────┐   ┌─────────▼─────────┐
     │   Telegram Bot   │    │   Discord Bot     │   │   Notifications   │
     │   (two-way)      │    │   (webhook/bot)   │   │   (desktop/hook)  │
     │                  │    │                    │   │                   │
     │  Commands + Push │    │  Rich embeds       │   │  Win/Mac/Linux    │
     │  FLASH/PRI/RTN   │    │  Color-coded tiers │   │  Slack/Feishu     │
     └─────────────────┘    └───────────────────┘   └───────────────────┘

              ┌─────────────────────────────────────────────────┐
              │        NewsPredict Prediction Market (Next.js)  │
              │  LMSR ─ Notifications ─ Achievements ─ Copy-Trade│
              │  Sentiment ─ SSE Real-time ─ Sentry ─ Web3 (BSC)│
              │  PostgreSQL 18 ─ Drizzle ORM ─ Rate Limiting    │
              └─────────────────────────────────────────────────┘
```

**Key modules:**

| Module | Path | Purpose |
|--------|------|---------|
| Sources | `src/sources/` | 46 intelligence source adapters (trait `IntelSource`) |
| LLM | `src/llm/` | Multi-provider LLM with fallback chain |
| Delta | `src/delta/` | Change detection, severity scoring, anomaly detection |
| Correlation | `src/correlation.rs` | 6 cross-source pattern detection rules |
| Briefing | `src/briefing.rs` | Sweep orchestration, LLM prompt templates (4 languages) |
| Dashboard | `src/dashboard/` | Axum web server, SSE, tower-http static serving, rate limiting, public API |
| Bot | `src/bot/` | Telegram + Discord bots with tiered alerting |
| Store | `src/store.rs` | SQLite persistence (rusqlite, bundled) |
| Logging | `src/logging.rs` | Structured logging (tracing crate, JSON or compact) |

---

## Configuration

All configuration is via environment variables. Copy `.env.example` to `.env` and edit as needed.

```bash
# -- Core ------------------------------------------------
REFRESH_INTERVAL_MINUTES=15       # Sweep interval
SOURCE_TIMEOUT_SECS=30            # Per-source timeout

# -- LLM (primary) ---------------------------------------
LLM_PROVIDER=zhipuai              # openai|anthropic|gemini|ollama|deepseek|zhipuai|...
LLM_API_KEY=your-key
DEFAULT_MODEL=glm-4-flash
SWEEP_LANG=en                     # en|zh|ja|es

# -- LLM (fallback) --------------------------------------
FALLBACK_PROVIDER=gemini
FALLBACK_MODEL=gemini-3.1-flash-lite-preview
GEMINI_API_KEY=your-key

# -- LLM (local fallback) --------------------------------
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=qwen3:8b

# -- Data Source Keys (optional) --------------------------
FRED_API_KEY=                     # fred.stlouisfed.org
FIRMS_MAP_KEY=                    # firms.modaps.eosdis.nasa.gov
EIA_API_KEY=DEMO_KEY              # api.eia.gov
WORLDNEWS_API_KEY=                # worldnewsapi.com

# -- Telegram Bot -----------------------------------------
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
TELEGRAM_POLL_INTERVAL=5000

# -- Discord Bot ------------------------------------------
DISCORD_BOT_TOKEN=
DISCORD_CHANNEL_ID=
DISCORD_GUILD_ID=
DISCORD_WEBHOOK_URL=              # Alternative: webhook-only mode

# -- Notifications ----------------------------------------
WEBHOOK_URL=                      # Generic webhook (Slack/Discord/Feishu)

# -- Watchlist --------------------------------------------
WATCH_REGIONS=Taiwan,Ukraine,Iran
ALERT_KEYWORDS=nuclear,pandemic,coup
WATCH_TICKERS=SPY,BTC-USD,GC=F
```

---

## Data Sources (46)

| # | Source | Tier | Auth | Description |
|--:|--------|:----:|:----:|-------------|
| 1 | ACLED | T1 | Free | Armed Conflict Location & Event Data |
| 2 | ADS-B | T1 | Free | ADS-B Exchange military flight tracking |
| 3 | FIRMS | T1 | Key | NASA satellite fire/thermal detection |
| 4 | GDACS | T1 | Free | Global Disaster Alert and Coordination System |
| 5 | GDELT | T1 | Free | Global news events (100+ languages) |
| 6 | OpenSky | T1 | Free | Real-time ADS-B flight tracking |
| 7 | ProMED | T1 | Free | ProMED-mail disease outbreak early warning |
| 8 | ReliefWeb | T1 | Free | UN humanitarian crisis tracking |
| 9 | Safecast | T1 | Free | Nuclear radiation monitoring network |
| 10 | Sanctions | T1 | Free | OFAC SDN + OpenSanctions monitoring |
| 11 | Ships | T1 | Free | Maritime AIS vessel tracking |
| 12 | SWPC | T1 | Free | NOAA space weather monitoring |
| 13 | Telegram | T1 | Free | OSINT channel monitoring (public web preview) |
| 14 | Tsunami | T1 | Free | Pacific Tsunami Warning Center alerts |
| 15 | USGS | T1 | Free | Earthquake monitoring (M2.5+) |
| 16 | WHO | T1 | Free | Disease outbreak news |
| 17 | BLS | T2 | Free | Bureau of Labor Statistics -- CPI, unemployment |
| 18 | CoinGecko | T2 | Free | Top 20 cryptocurrency market data |
| 19 | Comtrade | T2 | Free | UN strategic commodity trade flows |
| 20 | ECB | T2 | Free | European Central Bank exchange rates and EURIBOR |
| 21 | EIA | T2 | Key | US Energy Information Administration |
| 22 | ExchangeRates | T2 | Free | Foreign exchange rate data |
| 23 | FRED | T2 | Key | Federal Reserve Economic Data |
| 24 | GSCPI | T2 | Free | NY Fed Global Supply Chain Pressure Index |
| 25 | Treasury | T2 | Free | US Treasury fiscal data -- debt and rates |
| 26 | USAspending | T2 | Free | Federal spending and defense contracts |
| 27 | WorldNews | T2 | Key | World News API -- global news with sentiment |
| 28 | Bluesky | T3 | Free | Social sentiment intelligence |
| 29 | CISA-KEV | T3 | Free | Known Exploited Vulnerabilities catalog |
| 30 | Cloudflare Radar | T3 | Free | Internet traffic anomalies |
| 31 | Copernicus | T3 | Free | Climate Change Service monthly bulletin |
| 32 | CVE/NVD | T3 | Free | Vulnerability intelligence |
| 33 | EPA RadNet | T3 | Free | Radiation monitoring network |
| 34 | EU Sanctions | T3 | Free | EU consolidated sanctions list |
| 35 | Google Trends | T3 | Free | Daily trending searches (US) |
| 36 | ISC/SANS | T3 | Free | Internet Storm Center threat level |
| 37 | KiwiSDR | T3 | Free | Global HF radio receiver network |
| 38 | NASA NEO | T3 | Free | Near Earth Object close approach tracking |
| 39 | NOAA | T3 | Free | NWS severe weather alerts |
| 40 | NTSB | T3 | Free | Aviation safety incident reports |
| 41 | Patents | T3 | Free | USPTO filings in strategic technology areas |
| 42 | Reddit | T3 | Free | Social sentiment monitoring |
| 43 | RIPE Atlas | T3 | Free | Global internet measurement network |
| 44 | TechStatus | T3 | Free | Major tech service status monitoring |
| 45 | CelesTrak | T4 | Free | Satellite orbit tracking and launch monitoring |
| 46 | YFinance | T5 | Free | Yahoo Finance live market quotes |

**Auth legend**: Free = no API key required. Key = optional API key for higher limits or full access.

---

## Adding Custom Sources

CHAOS uses a plugin-based source architecture. Every source implements the `IntelSource` trait:

```rust
#[async_trait]
pub trait IntelSource: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn tier(&self) -> u8;           // 1-5
    async fn sweep(&self) -> Result<Value>;
}
```

A full template is provided at `src/sources/_template.rs`. See [`docs/source-plugin-guide.md`](docs/source-plugin-guide.md) for the step-by-step guide.

---

## API

The dashboard exposes a RESTful API with SSE streaming. In public mode (`--public --api-key KEY`), all endpoints are rate-limited and require `Authorization: Bearer <key>`.

| Endpoint | Method | Auth | Description |
|----------|--------|:----:|-------------|
| `/api/v1/data` | GET | No | Latest sweep data (all sources, delta, correlations, analysis) |
| `/api/v1/health` | GET | No | Health check (uptime, LLM, DB, degraded sources) |
| `/api/v1/trends` | GET | No | Historical sweep data (last 50) |
| `/api/v1/analysis` | GET | No | Latest LLM intelligence briefing |
| `/api/v1/sources` | GET | No | Source list with tier, description, reliability |
| `/api/v1/sse` | GET | No | Server-Sent Events (real-time updates) |
| `/api/v1/events` | GET | Public | Structured events with category and geo tags |
| `/api/v1/correlations` | GET | Public | Cross-source correlation signals |
| `/api/v1/market-seeds` | GET | Public | Prediction market seed questions |
| `/api/v1/query` | POST | Public | Query historical data with filters |
| `/api/v1/resolve-check` | POST | Public | Check condition against current data |

Full OpenAPI specifications: [`docs/api/chaos-engine-openapi.yaml`](docs/api/chaos-engine-openapi.yaml) | [`docs/api/newspredict-openapi.yaml`](docs/api/newspredict-openapi.yaml)

---

## Deployment

### Production (Linux Server)

Full-stack deployment: CHAOS Engine + NewsPredict + PostgreSQL 18. Caddy/Nginx reverse proxy must be **configured separately** (not in containers).

```bash
git clone https://github.com/magicnight/chaos-engine.git && cd chaos-engine
cp .env.example .env              # Configure production values (see required fields below)
./scripts/deploy.sh               # One-click deploy (build, migrate, health check, seed)
./scripts/deploy.sh --rebuild     # Rebuild and deploy
```

**Required `.env` fields:**
```bash
DOMAIN=chaos.yourdomain.com           # Set for auto HTTPS via Let's Encrypt
NEXTAUTH_URL=https://chaos.yourdomain.com
NEXTAUTH_SECRET=$(openssl rand -hex 32)
CRON_SECRET=$(openssl rand -hex 16)
POSTGRES_PASSWORD=$(openssl rand -hex 16)
```

This starts 3 container services (Caddy runs externally):

```
Internet -> Caddy (external, :80/:443, auto HTTPS)
              |-- /api/v1/*  -> CHAOS Engine (:3117)
              +-- /*         -> NewsPredict (:3000)
                                   +-- PostgreSQL 18 (:5432)
```

**Without a domain**: Leave `DOMAIN` empty -- accessible via `http://server-ip`.

### Ops Scripts

| Script | Description |
|--------|-------------|
| `scripts/deploy.sh` | Production one-click deploy (build, migrate, health check, seed) |
| `scripts/dev-start.sh` / `dev-start.ps1` | Development environment one-click start |
| `scripts/backup.sh` | Database backup |
| `scripts/update-live-ids.sh` | Update YouTube live stream IDs (15 news channels) |

### Development Environment

| Platform | Command | Runtime |
|----------|---------|---------|
| Windows | `.\scripts\dev-start.ps1` | Podman |
| Linux / Mac | `./scripts/dev-start.sh` | Docker |

Dev uses port `8080` (HTTP), auto-generates random secrets, auto-runs DB migration.

### CHAOS Engine Only (no frontend)

```bash
cargo build --release
./target/release/chaos serve --public --api-key YOUR_SECRET --port 3117
```

### Configuration

Copy `.env.example` to `.env` and configure. All variables are optional unless noted.

#### Core

| Variable | Default | Description |
|----------|---------|-------------|
| `DOMAIN` | *(empty)* | Your domain for auto HTTPS (e.g. `chaos.example.com`) |
| `POSTGRES_PASSWORD` | `chaos_secret` | PostgreSQL password -- **change in production** |
| `REFRESH_INTERVAL_MINUTES` | `15` | OSINT sweep interval |
| `SOURCE_TIMEOUT_SECS` | `30` | Per-source timeout (T1: 100%, T2: 80%, T3: 50%) |

#### LLM (optional -- enables AI analysis and richer market seeds)

| Variable | Example | Description |
|----------|---------|-------------|
| `LLM_PROVIDER` | `openai` | Primary: openai, anthropic, gemini, ollama, deepseek, zhipuai, openrouter, mistral, minimax |
| `LLM_API_KEY` | | API key for primary provider |
| `DEFAULT_MODEL` | `gpt-4o` | Model name |
| `SWEEP_LANG` | `en` | Briefing language: en, zh, ja, es |
| `FALLBACK_PROVIDER` | `gemini` | Fallback provider (auto-failover) |
| `FALLBACK_MODEL` | `gemini-2.0-flash` | Fallback model |
| `OLLAMA_URL` | `http://localhost:11434` | Local Ollama URL (zero-cloud fallback) |
| `OLLAMA_MODEL` | `qwen3:8b` | Local model name |

#### Data Source API Keys (optional -- more keys = more sources)

| Variable | Source | Free? |
|----------|--------|:-----:|
| `FRED_API_KEY` | Federal Reserve Economic Data | Yes |
| `FIRMS_MAP_KEY` | NASA Fire Detection | Yes |
| `EIA_API_KEY` | US Energy Information | Yes |
| `WORLDNEWS_API_KEY` | World News API | Yes |
| `ACLED_EMAIL` + `ACLED_PASSWORD` | Armed Conflict Data | Yes |
| `CLOUDFLARE_API_TOKEN` | Cloudflare Radar | Yes |

20+ sources work with **zero API keys**. Each key unlocks additional data.

#### NewsPredict

| Variable | Default | Description |
|----------|---------|-------------|
| `NEXTAUTH_SECRET` | | **Required** -- random secret for session signing |
| `CRON_SECRET` | | Secret for auto-seed/resolve API calls |
| `NEXT_PUBLIC_CHAOS_URL` | | Public CHAOS API URL (for client-side SSE) |
| `NEXT_PUBLIC_REOWN_PROJECT_ID` | | WalletConnect project ID (for Web3 login) |

#### Bots (optional)

| Variable | Description |
|----------|-------------|
| `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` | Telegram alerts + commands |
| `DISCORD_BOT_TOKEN` + `DISCORD_CHANNEL_ID` | Discord bot mode |
| `DISCORD_WEBHOOK_URL` | Discord webhook mode (simpler) |
| `WEBHOOK_URL` | Generic webhook (Slack/Feishu) |

#### Runtime

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_PATH` | `./chaos.db` | SQLite database location (CHAOS Engine) |
| `CHAOS_LOG_FORMAT` | `compact` | Set to `json` for structured log output |
| `RUST_LOG` | `chaos=info` | Log level filter |

---

## NewsPredict

Companion prediction market PWA at [`newspredict/`](newspredict/). Built with Next.js, LMSR scoring, Web3 wallet integration (BSC), and Drizzle ORM. Consumes the CHAOS public API to generate and resolve prediction markets from real-world intelligence data.

- **Economic model**: [`docs/economics.md`](docs/economics.md) | [`docs/economics-zh.md`](docs/economics-zh.md)
- **Smart contracts**: ChaosToken (C.H.A.O.S.) + ChaosPredictionMarket -- deployed on BSC mainnet and testnet, verified on BscScan
- **Product guide**: [`newspredict/docs/usage-guide.md`](newspredict/docs/usage-guide.md)

---

## Structured Logging

CHAOS uses the `tracing` crate with configurable output:

```bash
# Default: compact human-readable
chaos serve

# JSON for log aggregation (ELK, Datadog, etc.)
CHAOS_LOG_FORMAT=json chaos serve

# Debug verbosity
RUST_LOG=chaos=debug chaos sweep
```

---

## License

[AGPL-3.0](LICENSE)

---

<div align="center">
<sub>Built with Rust. No cloud required.</sub>
</div>

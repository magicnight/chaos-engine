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
[![Sources](https://img.shields.io/badge/OSINT%20sources-44-cyan)](#data-sources-44)
[![LLM](https://img.shields.io/badge/LLM-multi--provider-green)](#ai-analysis)
[![Docker](https://img.shields.io/badge/docker-ready-2496ED)](#docker)

</div>

---

## What is CHAOS?

CHAOS pulls from **44 open-source intelligence feeds** in parallel -- satellite fire detection, flight tracking, radiation monitoring, earthquake data, economic indicators, conflict events, cyber vulnerabilities, sanctions lists, disease outbreaks, social sentiment, and more -- then synthesizes everything into a single actionable picture updated every 15 minutes.

Connect an LLM and it becomes a **multilingual intelligence analyst** generating structured briefings in English, Chinese, Japanese, or Spanish, with cross-domain correlation detection and anomaly flagging. Alerts push to Telegram and Discord bots with three-tier severity classification (FLASH / PRIORITY / ROUTINE), and the bots accept commands back -- sweep on demand, request a briefing, check system status, all from your phone.

Everything renders on a self-contained **Jarvis-style dashboard** with a 3D globe, draggable Gridstack panels, real-time SSE updates, and full API access for downstream consumers. Single binary. Embedded SQLite. Zero cloud dependency. Zero telemetry. Zero subscriptions.

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

### Docker

```bash
cp .env.example .env
docker compose up -d
```

The container exposes port `3117` with a built-in healthcheck at `/api/v1/health`.

---

## Features

### Intelligence Collection (44 Sources)

All sources run in parallel via `tokio::join_all` with per-source timeouts. 20+ sources work with **zero API keys**.

| Tier | Focus | Count | Sources |
|------|-------|------:|---------|
| **T1** Core OSINT | Conflict, disasters, health, transport | 16 | ACLED, ADS-B, FIRMS, GDACS, GDELT, OpenSky, ProMED-mail, ReliefWeb, Safecast, Sanctions (OFAC+OpenSanctions), Ships, SWPC, Telegram, Tsunami, USGS, WHO |
| **T2** Economic | Markets, trade, fiscal | 10 | BLS, CoinGecko, Comtrade, ECB, EIA, FRED, GSCPI, Treasury, USAspending, WorldNews |
| **T3** Supplementary | Cyber, environment, social, tech | 15 | Bluesky, CISA-KEV, Cloudflare Radar, Copernicus, CVE/NVD, EPA RadNet, EU Sanctions, Google Trends, ISC/SANS, KiwiSDR, NASA NEO, NOAA, NTSB, Patents, Reddit, RIPE Atlas |
| **T4** Space | Orbital tracking | 1 | CelesTrak |
| **T5** Markets | Live quotes | 1 | Yahoo Finance |

### AI Analysis

- **LLM fallback chain**: primary provider -> fallback provider -> Ollama local (automatic failover)
- **10 provider backends**: OpenAI, Anthropic, Gemini, Ollama, DeepSeek, Moonshot, OpenRouter, Mistral, MiniMax, ZhipuAI
- **4-language analysis**: `--lang en|zh|ja|es` -- full military-style briefing prompts in each language
- **Structured output**: Situation Overview, Key Developments, Risk Matrix, Actionable Intelligence, Cross-Domain Correlations
- **Market prediction seeds**: LLM-generated or rule-based prediction market questions

### CHAOS MONITOR Dashboard

- **22 draggable panels** organized into 9 categories, all toggleable in Settings
- **Gridstack.js** panel system with drag, resize, and layout persistence (localStorage)
- **3D globe** with real-time event plotting (quakes, fires, conflicts, weather)
- **Server-Sent Events** for live data streaming
- **Rate-limited public API** mode with API key authentication
- **Embedded in binary** via `include_str!` -- no external static files needed

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
| `chaos sweep` | Run full intelligence sweep across all 44 sources |
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
     │   44 Sources     │    │   LLM Fallback    │   │   Dashboard       │
     │  (async parallel)│    │   Chain            │   │   (Axum + SSE)    │
     │                  │    │                    │   │                   │
     │  T1: Core OSINT  │    │  OpenAI-compat     │   │  Gridstack panels │
     │  T2: Economic    │    │  Anthropic         │   │  3D Globe         │
     │  T3: Supplement  │    │  Gemini            │   │  Real-time SSE    │
     │  T4: Space       │    │  Ollama (local)    │   │  Rate-limited API │
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
```

**Key modules:**

| Module | Path | Purpose |
|--------|------|---------|
| Sources | `src/sources/` | 44 intelligence source adapters (trait `IntelSource`) |
| LLM | `src/llm/` | Multi-provider LLM with fallback chain |
| Delta | `src/delta/` | Change detection, severity scoring, anomaly detection |
| Correlation | `src/correlation.rs` | 6 cross-source pattern detection rules |
| Briefing | `src/briefing.rs` | Sweep orchestration, LLM prompt templates (4 languages) |
| Dashboard | `src/dashboard/` | Axum web server, SSE, rate limiting, public API |
| Bot | `src/bot/` | Telegram + Discord bots with tiered alerting |
| Store | `src/store.rs` | SQLite persistence (rusqlite, bundled) |
| Logging | `src/logging.rs` | Structured logging (tracing crate, JSON or compact) |

---

## Configuration

All configuration is via environment variables. Copy `.env.example` to `.env` and edit as needed.

```bash
# ── Core ────────────────────────────────────────
REFRESH_INTERVAL_MINUTES=15       # Sweep interval
SOURCE_TIMEOUT_SECS=30            # Per-source timeout

# ── LLM (primary) ──────────────────────────────
LLM_PROVIDER=zhipuai              # openai|anthropic|gemini|ollama|deepseek|zhipuai|...
LLM_API_KEY=your-key
DEFAULT_MODEL=glm-4-flash
SWEEP_LANG=en                     # en|zh|ja|es

# ── LLM (fallback) ─────────────────────────────
FALLBACK_PROVIDER=gemini
FALLBACK_MODEL=gemini-3.1-flash-lite-preview
GEMINI_API_KEY=your-key

# ── LLM (local fallback) ───────────────────────
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=qwen3:8b

# ── Data Source Keys (optional) ─────────────────
FRED_API_KEY=                     # fred.stlouisfed.org
FIRMS_MAP_KEY=                    # firms.modaps.eosdis.nasa.gov
EIA_API_KEY=DEMO_KEY              # api.eia.gov
WORLDNEWS_API_KEY=                # worldnewsapi.com

# ── Telegram Bot ────────────────────────────────
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
TELEGRAM_POLL_INTERVAL=5000

# ── Discord Bot ─────────────────────────────────
DISCORD_BOT_TOKEN=
DISCORD_CHANNEL_ID=
DISCORD_GUILD_ID=
DISCORD_WEBHOOK_URL=              # Alternative: webhook-only mode

# ── Notifications ───────────────────────────────
WEBHOOK_URL=                      # Generic webhook (Slack/Discord/Feishu)

# ── Watchlist ───────────────────────────────────
WATCH_REGIONS=Taiwan,Ukraine,Iran
ALERT_KEYWORDS=nuclear,pandemic,coup
WATCH_TICKERS=SPY,BTC-USD,GC=F
```

---

## Data Sources (44)

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
| 22 | FRED | T2 | Key | Federal Reserve Economic Data |
| 23 | GSCPI | T2 | Free | NY Fed Global Supply Chain Pressure Index |
| 24 | Treasury | T2 | Free | US Treasury fiscal data -- debt and rates |
| 25 | USAspending | T2 | Free | Federal spending and defense contracts |
| 26 | WorldNews | T2 | Key | World News API -- global news with sentiment |
| 27 | Bluesky | T3 | Free | Social sentiment intelligence |
| 28 | CISA-KEV | T3 | Free | Known Exploited Vulnerabilities catalog |
| 29 | Cloudflare Radar | T3 | Free | Internet traffic anomalies |
| 30 | Copernicus | T3 | Free | Climate Change Service monthly bulletin |
| 31 | CVE/NVD | T3 | Free | Vulnerability intelligence |
| 32 | EPA RadNet | T3 | Free | Radiation monitoring network |
| 33 | EU Sanctions | T3 | Free | EU consolidated sanctions list |
| 34 | Google Trends | T3 | Free | Daily trending searches (US) |
| 35 | ISC/SANS | T3 | Free | Internet Storm Center threat level |
| 36 | KiwiSDR | T3 | Free | Global HF radio receiver network |
| 37 | NASA NEO | T3 | Free | Near Earth Object close approach tracking |
| 38 | NOAA | T3 | Free | NWS severe weather alerts |
| 39 | NTSB | T3 | Free | Aviation safety incident reports |
| 40 | Patents | T3 | Free | USPTO filings in strategic technology areas |
| 41 | Reddit | T3 | Free | Social sentiment monitoring |
| 42 | RIPE Atlas | T3 | Free | Global internet measurement network |
| 43 | CelesTrak | T4 | Free | Satellite orbit tracking and launch monitoring |
| 44 | YFinance | T5 | Free | Yahoo Finance live market quotes |

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

Full specification: [`docs/api.md`](docs/api.md) | [`docs/openapi.yaml`](docs/openapi.yaml)

---

## Deployment

### Production (Podman / Docker Compose)

Full-stack deployment: CHAOS Engine + NewsPredict + PostgreSQL + Caddy reverse proxy.

```bash
git clone https://github.com/magicnight/chaos-engine.git && cd chaos-engine
cp .env.example .env              # configure (see below)
podman-compose up -d              # or: docker compose up -d
```

This starts 4 services:

```
Internet → Caddy (:80/:443, auto HTTPS)
             ├─ /api/v1/*  → CHAOS Engine (:3117)
             └─ /*         → NewsPredict (:3000)
                                └─ PostgreSQL (:5432)
```

**With a domain** (auto HTTPS):
```bash
# In .env:
DOMAIN=chaos.yourdomain.com
# Caddy automatically provisions Let's Encrypt certificate
```

**Without a domain** (HTTP only):
```bash
# Leave DOMAIN empty in .env — accessible via http://server-ip
```

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
| `POSTGRES_PASSWORD` | `chaos_secret` | PostgreSQL password — **change in production** |
| `REFRESH_INTERVAL_MINUTES` | `15` | OSINT sweep interval |
| `SOURCE_TIMEOUT_SECS` | `30` | Per-source timeout (T1: 100%, T2: 80%, T3: 50%) |

#### LLM (optional — enables AI analysis and richer market seeds)

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

#### Data Source API Keys (optional — more keys = more sources)

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
| `NEXTAUTH_SECRET` | | **Required** — random secret for session signing |
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
- **Smart contracts**: ChaosToken (C.H.A.O.S.) + ChaosPredictionMarket — deployed on BSC mainnet and testnet, verified on BscScan

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

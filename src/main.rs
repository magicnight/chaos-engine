mod bot;
mod briefing;
mod config;
mod correlation;
mod dashboard;
mod delta;
mod error;
mod http;
mod llm;
mod logging;
mod notify;
mod report;
mod sources;
mod store;
#[allow(dead_code)]
mod term;
mod util;
mod watchlist;

use std::sync::Arc;

use clap::{Parser, Subcommand};
use tokio::sync::{broadcast, RwLock};

use config::Config;
use http::HttpClient;
use llm::LlmOptions;
use store::Store;
use term::*;

#[derive(Parser)]
#[command(
    name = "chaos",
    version = env!("CARGO_PKG_VERSION"),
    about = "C.H.A.O.S. -- Connected Human-Augmented OSINT Suite -- 44 sources, single binary, zero cloud"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show engine status: version, config, and API key availability
    Status,
    /// Run a full intelligence sweep across all enabled sources
    Sweep {
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Language for LLM analysis: en, zh, ja, es
        #[arg(long, default_value = "en", value_parser = ["en", "zh", "ja", "es"])]
        lang: String,
        /// Skip LLM analysis even if configured
        #[arg(long)]
        no_llm: bool,
    },
    /// Fetch latest data from a single named source
    Source {
        /// Name of the source to query
        name: String,
    },
    /// Test LLM connection
    TestLlm,
    /// Show sweep history
    History {
        /// Number of history entries to show
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Show specific sweep ID data
        #[arg(long)]
        show: Option<i64>,
    },
    /// Show trend sparklines (ASCII)
    Trends {
        /// Show anomalies compared to historical data
        #[arg(long)]
        anomalies: bool,
    },
    /// Generate Markdown intelligence report
    Report,
    /// Export trend data as CSV or JSON
    Export {
        /// Output format: csv or json
        #[arg(long, default_value = "csv")]
        format: String,
        /// Number of sweeps to include
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Start web dashboard server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "3117")]
        port: u16,
        /// Enable public API mode (binds 0.0.0.0, requires --api-key)
        #[arg(long)]
        public: bool,
        /// API key for public mode authentication
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Generate market seeds via LLM or rule-based heuristics
    MarketSeeds {
        /// Number of seeds to generate
        #[arg(long, default_value = "10")]
        count: usize,
    },
    /// Migrate data from old Node.js JSON runs directory
    Migrate {
        /// Path to the JSON files directory
        #[arg(long)]
        from_json: String,
    },
    /// Continuous sweep loop (headless, no dashboard)
    Watch {
        /// Interval between sweeps in minutes
        #[arg(long, default_value = "15")]
        interval: u64,
    },
}

const BOX_W: usize = 48;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let cfg = Config::load()?;
            let client = HttpClient::new(cfg.source_timeout_secs, 3)?;
            let sources = briefing::list_sources(&client);

            // -- header with ASCII art --
            eprintln!();
            eprintln!("{CYAN}{BOLD}   ██████╗ ██╗  ██╗  █████╗   ██████╗  ███████╗{RESET}");
            eprintln!("{CYAN}{BOLD}  ██╔════╝ ██║  ██║ ██╔══██╗ ██╔═══██╗ ██╔════╝{RESET}");
            eprintln!("{CYAN}{BOLD}  ██║      ███████║ ███████║ ██║   ██║ ███████╗{RESET}");
            eprintln!("{CYAN}{BOLD}  ██║      ██╔══██║ ██╔══██║ ██║   ██║ ╚════██║{RESET}");
            eprintln!("{CYAN}{BOLD}  ╚██████╗ ██║  ██║ ██║  ██║ ╚██████╔╝ ███████║{RESET}");
            eprintln!("{CYAN}{BOLD}   ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═════╝  ╚══════╝{RESET}");
            eprintln!();
            eprintln!("  {BOLD}{WHITE}CHAOS Engine{RESET} {DIM}v{}{RESET} {DIM}—{RESET} {CYAN}Connected Human-Augmented OSINT Suite{RESET}", env!("CARGO_PKG_VERSION"));
            eprintln!();
            box_top(BOX_W);
            box_sep(BOX_W);

            // -- config --
            box_line(
                &format!("{BOLD}{WHITE}Config{RESET}"),
                BOX_W,
            );
            box_line(
                &format!(
                    "  Refresh:  {CYAN}every {} min{RESET}",
                    cfg.refresh_interval_minutes
                ),
                BOX_W,
            );
            box_line(
                &format!(
                    "  Timeout:  {CYAN}{}s per source{RESET}",
                    cfg.source_timeout_secs
                ),
                BOX_W,
            );
            box_line(
                &format!("  Language: {CYAN}{}{RESET}", cfg.sweep_lang),
                BOX_W,
            );
            box_empty(BOX_W);

            // LLM status
            let llm_str = match llm::create_provider(&cfg) {
                Some(provider) => {
                    if provider.is_configured() {
                        format!(
                            "{GREEN}{} ({}){RESET}",
                            provider.name(),
                            provider.model()
                        )
                    } else {
                        format!(
                            "{YELLOW}{} (not configured){RESET}",
                            provider.name()
                        )
                    }
                }
                None => format!("{RED}disabled{RESET}"),
            };
            box_line(&format!("  LLM:      {llm_str}"), BOX_W);

            // Notifications
            let notifier = notify::Notifier::new(cfg.webhook_url.clone());
            let alert_str = if notifier.is_configured() {
                format!("{GREEN}webhook configured{RESET}")
            } else {
                format!("{RED}disabled{RESET}")
            };
            box_line(&format!("  Alerts:   {alert_str}"), BOX_W);

            // Telegram
            let tg_str = match (&cfg.telegram_bot_token, &cfg.telegram_chat_id) {
                (Some(_), Some(chat)) => {
                    format!("{GREEN}enabled (chat: {}){RESET}", chat)
                }
                _ => format!("{RED}disabled{RESET}"),
            };
            box_line(&format!("  Telegram: {tg_str}"), BOX_W);

            // Discord
            let dc_str = if cfg.discord_bot_token.is_some() && cfg.discord_channel_id.is_some() {
                format!("{GREEN}bot mode{RESET}")
            } else if cfg.discord_webhook_url.is_some() {
                format!("{GREEN}webhook mode{RESET}")
            } else {
                format!("{RED}disabled{RESET}")
            };
            box_line(&format!("  Discord:  {dc_str}"), BOX_W);

            // Watchlist
            let wl = watchlist::Watchlist::from_config(&cfg);
            let wl_str = if !wl.is_empty() {
                format!(
                    "{GREEN}{} regions, {} keywords, {} tickers{RESET}",
                    wl.regions.len(),
                    wl.keywords.len(),
                    wl.tickers.len()
                )
            } else {
                format!("{DIM}empty{RESET}")
            };
            box_line(&format!("  Watchlist: {wl_str}"), BOX_W);

            box_sep(BOX_W);

            // -- sources by tier --
            box_line(
                &format!(
                    "{BOLD}{WHITE}Sources ({}){RESET}",
                    sources.len()
                ),
                BOX_W,
            );
            box_empty(BOX_W);

            for tier in 1..=5u64 {
                let tier_sources: Vec<&str> = sources
                    .iter()
                    .filter(|s| s["tier"].as_u64() == Some(tier))
                    .filter_map(|s| s["name"].as_str())
                    .collect();
                if tier_sources.is_empty() {
                    continue;
                }
                let tc = tier_color(tier);

                // First line with tier label
                let mut line = format!("  {BOLD}T{tier}{RESET} ");
                let mut col = 0;
                for (i, name) in tier_sources.iter().enumerate() {
                    if i == 0 {
                        line.push_str(&format!("{tc}{BOLD}\u{25cf}{RESET} {tc}{name}{RESET}"));
                        col = 1;
                    } else if col >= 3 {
                        box_line(&line, BOX_W);
                        line = format!("     {tc}{BOLD}\u{25cf}{RESET} {tc}{name}{RESET}");
                        col = 1;
                    } else {
                        line.push_str(&format!("  {tc}{BOLD}\u{25cf}{RESET} {tc}{name}{RESET}"));
                        col += 1;
                    }
                }
                box_line(&line, BOX_W);
                box_empty(BOX_W);
            }

            box_bottom(BOX_W);
        }
        Commands::Sweep { json, lang, no_llm } => {
            let mut cfg = Config::load()?;
            if lang != "en" {
                cfg.sweep_lang = lang;
            }
            let client = HttpClient::new(cfg.source_timeout_secs, 3)?;
            let store = Store::open("runs/chaos.db")?;

            let llm_provider = if no_llm {
                None
            } else {
                llm::create_provider(&cfg)
            };

            let data = briefing::full_sweep(
                &client,
                &store,
                llm_provider
                    .as_ref()
                    .map(|p| p.as_ref() as &dyn llm::LlmProvider),
                &cfg,
            )
            .await;

            if json {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                let meta = &data["chaos"];
                let ts = meta["timestamp"]
                    .as_str()
                    .unwrap_or("unknown");
                let total_ms = meta["totalDurationMs"].as_u64().unwrap_or(0);
                let queried = meta["sourcesQueried"].as_u64().unwrap_or(0) as usize;
                let ok = meta["sourcesOk"].as_u64().unwrap_or(0) as usize;
                let _failed = meta["sourcesFailed"].as_u64().unwrap_or(0) as usize;

                // -- header --
                eprintln!();
                eprintln!(
                    "{BOLD}{CYAN}{}{RESET}",
                    "\u{2550}".repeat(50)
                );
                eprintln!(
                    "{BOLD}{CYAN}  CHAOS SWEEP \u{2014} {}{RESET}",
                    ts
                );
                eprintln!(
                    "{BOLD}{CYAN}{}{RESET}",
                    "\u{2550}".repeat(50)
                );
                eprintln!();

                // -- progress bar --
                let pbar = progress_bar(ok, queried, 26);
                let pct = if queried > 0 {
                    (ok * 100) / queried
                } else {
                    0
                };
                eprintln!(
                    "{BOLD}Sources [{}{ok}/{queried} OK{RESET}{BOLD}]{RESET} {pbar} {BOLD}{pct}%{RESET}",
                    if ok == queried { GREEN } else { YELLOW },
                );
                eprintln!();

                // -- per-source timing in columns --
                if let Some(timing) = data["timing"].as_object() {
                    let mut entries: Vec<_> = timing.iter().collect();
                    entries.sort_by_key(|(n, _)| (*n).clone());
                    let mut col = 0;
                    let mut line = String::new();
                    for (name, info) in &entries {
                        let status = info["status"].as_str().unwrap_or("unknown");
                        let ms = info["ms"].as_u64().unwrap_or(0);
                        let (icon, color) = if status == "ok" {
                            ("\u{2713}", GREEN)
                        } else {
                            ("\u{2717}", RED)
                        };
                        let ms_str = if status == "ok" {
                            format!("{:.1}s", ms as f64 / 1000.0)
                        } else {
                            "\u{2014}".to_string()
                        };
                        let entry = format!(
                            "  {color}{icon}{RESET} {:<10} {DIM}{:<6}{RESET}",
                            name, ms_str
                        );
                        line.push_str(&entry);
                        col += 1;
                        if col >= 2 {
                            eprintln!("{line}");
                            line.clear();
                            col = 0;
                        }
                    }
                    if !line.is_empty() {
                        eprintln!("{line}");
                    }
                }

                // -- errors --
                if let Some(errors) = data["errors"].as_array() {
                    if !errors.is_empty() {
                        eprintln!();
                        eprintln!("{BOLD}{RED}Errors{RESET}");
                        for e in errors {
                            eprintln!(
                                "  {RED}\u{2717}{RESET} {BOLD}{}{RESET} {DIM}\u{2014} {}{RESET}",
                                e["name"].as_str().unwrap_or("?"),
                                e["error"].as_str().unwrap_or("unknown error"),
                            );
                        }
                    }
                }

                // -- market snapshot from YFinance if available --
                if let Some(yf) = data.get("sources").and_then(|s| s.get("YFinance")) {
                    if let Some(quotes) = yf.get("quotes").and_then(|q| q.as_object()) {
                        eprintln!();
                        eprintln!("{BOLD}{WHITE}Market Snapshot{RESET}");
                        for (ticker, info) in quotes {
                            if let Some(price) = info.get("price").and_then(|p| p.as_f64()) {
                                let change = info
                                    .get("change_pct")
                                    .and_then(|c| c.as_f64())
                                    .unwrap_or(0.0);
                                let (cc, _) = color_change(change);
                                let sign = if change >= 0.0 { "+" } else { "" };
                                let price_str = format_number(price);
                                eprintln!(
                                    "  {BOLD}{ticker:<8}{RESET} {CYAN}${price_str}{RESET}  {cc}{sign}{change:.1}%{RESET}",
                                );
                            }
                        }
                    }
                }

                // -- delta summary --
                if let Some(delta) = data.get("delta") {
                    if let Some(changes) = delta.get("changes").and_then(|c| c.as_array()) {
                        if !changes.is_empty() {
                            eprintln!();
                            eprintln!(
                                "{BOLD}{WHITE}Delta [{} changes]{RESET}",
                                changes.len()
                            );
                            for c in changes {
                                let metric = c["metric"].as_str().unwrap_or("?");
                                let severity = c["severity"].as_str().unwrap_or("LOW");
                                let sc = severity_color(severity);
                                let direction = c["direction"].as_str().unwrap_or("?");
                                let arrow = match direction {
                                    "up" => "\u{25b2}",
                                    "down" => "\u{25bc}",
                                    _ => "\u{2605}",
                                };
                                let detail = c["detail"].as_str().unwrap_or("");
                                eprintln!(
                                    "  {sc}{arrow}{RESET} {BOLD}{:<10}{RESET} {DIM}{}{RESET}  {sc}{BOLD}{severity}{RESET}",
                                    metric, detail,
                                );
                            }
                        }
                    } else if let Some(summary) = delta.get("summary") {
                        let total = summary["total_changes"].as_u64().unwrap_or(0);
                        let critical = summary["critical_changes"].as_u64().unwrap_or(0);
                        let dir = summary["direction"].as_str().unwrap_or("unknown");
                        if total > 0 {
                            eprintln!();
                            eprintln!(
                                "{BOLD}{WHITE}Delta:{RESET} {CYAN}{total}{RESET} changes ({RED}{critical} critical{RESET}) \u{2014} {DIM}{dir}{RESET}",
                            );
                        }
                    }
                }

                // -- correlations --
                if let Some(corrs) = data.get("correlations").and_then(|c| c.as_array()) {
                    if !corrs.is_empty() {
                        eprintln!();
                        eprintln!("{BOLD}{WHITE}Correlations{RESET}");
                        for c in corrs {
                            let severity = c["severity"].as_str().unwrap_or("low");
                            let sc = severity_color(severity);
                            eprintln!(
                                "  {sc}\u{26a0}{RESET} {BOLD}{}{RESET} {DIM}({}){RESET}",
                                c["name"].as_str().unwrap_or("?"),
                                severity,
                            );
                            if let Some(desc) = c["description"].as_str() {
                                if !desc.is_empty() {
                                    eprintln!("    {DIM}{desc}{RESET}");
                                }
                            }
                        }
                    }
                }

                // -- watchlist matches --
                if let Some(matches) = data.get("watchlist_matches").and_then(|w| w.as_array()) {
                    if !matches.is_empty() {
                        eprintln!();
                        eprintln!("{BOLD}{WHITE}Watchlist Matches{RESET}");
                        for m in matches {
                            eprintln!(
                                "  {YELLOW}\u{25cf}{RESET} [{BOLD}{}{RESET}] {CYAN}'{}'{RESET} in {DIM}{}{RESET}",
                                m["type"].as_str().unwrap_or("?"),
                                m["matched"].as_str().unwrap_or("?"),
                                m["source"].as_str().unwrap_or("?"),
                            );
                        }
                    }
                }

                // -- analysis --
                if let Some(analysis) = data.get("analysis") {
                    if let Some(text) = analysis.get("text").and_then(|t| t.as_str()) {
                        eprintln!();
                        eprintln!(
                            "{BOLD}{MAGENTA}\u{2500}\u{2500}\u{2500} Analysis \u{2500}\u{2500}\u{2500}{RESET}"
                        );
                        eprintln!("{text}");
                    } else if let Some(err) = analysis.get("error").and_then(|e| e.as_str()) {
                        eprintln!();
                        eprintln!("{RED}Analysis failed:{RESET} {err}");
                    }
                }

                // -- footer --
                eprintln!();
                eprintln!(
                    "{DIM}Sweep complete in {RESET}{BOLD}{CYAN}{:.1}s{RESET}",
                    total_ms as f64 / 1000.0
                );
            }
        }
        Commands::Source { name } => {
            let cfg = Config::load()?;
            let client = HttpClient::new(cfg.source_timeout_secs, 3)?;

            match briefing::single_source(&client, &name, cfg.source_timeout_secs).await {
                Some(data) => {
                    println!("{}", serde_json::to_string_pretty(&data)?);
                }
                None => {
                    eprintln!(
                        "{RED}Unknown source:{RESET} '{BOLD}{name}{RESET}'"
                    );
                    eprintln!();
                    eprintln!("{DIM}Available sources:{RESET}");
                    for s in briefing::list_sources(&client) {
                        let sn = s["name"].as_str().unwrap_or("");
                        let tier = s["tier"].as_u64().unwrap_or(0);
                        let tc = tier_color(tier);
                        eprintln!("  {tc}\u{25cf}{RESET} {sn}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::TestLlm => {
            let cfg = Config::load()?;
            match llm::create_provider(&cfg) {
                None => {
                    eprintln!(
                        "{YELLOW}No LLM configured.{RESET} Set {BOLD}LLM_PROVIDER{RESET} environment variable."
                    );
                    eprintln!(
                        "{DIM}Supported: ollama, anthropic, gemini, openai, deepseek, moonshot, openrouter, mistral, minimax{RESET}"
                    );
                }
                Some(provider) => {
                    eprintln!(
                        "{BOLD}Testing LLM:{RESET} {CYAN}{}{RESET} (model: {CYAN}{}{RESET})",
                        provider.name(),
                        provider.model()
                    );

                    let opts = LlmOptions::default();
                    match provider
                        .complete("You are a test.", "Say hello in one sentence.", &opts)
                        .await
                    {
                        Ok(response) => {
                            eprintln!("{GREEN}\u{2713} Success{RESET}");
                            eprintln!("  Response: {}", response.text);
                            eprintln!(
                                "  Model:    {CYAN}{}{RESET}",
                                response.model
                            );
                            eprintln!(
                                "  Tokens:   {DIM}{} in, {} out{RESET}",
                                response.input_tokens, response.output_tokens
                            );
                        }
                        Err(e) => {
                            eprintln!("{RED}\u{2717} LLM test failed:{RESET} {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::History { limit, show } => {
            let store = Store::open("runs/chaos.db")?;

            if let Some(sweep_id) = show {
                match store.get_sweep_data(sweep_id)? {
                    Some(data) => {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    }
                    None => {
                        eprintln!(
                            "{RED}No sweep found with ID {BOLD}{sweep_id}{RESET}"
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                let history = store.get_sweep_history(limit)?;
                if history.is_empty() {
                    eprintln!(
                        "{YELLOW}No sweep history yet.{RESET} Run {BOLD}chaos sweep{RESET} first."
                    );
                } else {
                    eprintln!();
                    eprintln!(
                        "  {BOLD}{WHITE}{:<6} {:<22} {:<10} {:<12} {}{RESET}",
                        "ID", "Timestamp", "Duration", "Sources", "Status"
                    );
                    eprintln!(
                        "  {DIM}{:<6} {:<22} {:<10} {:<12} {}{RESET}",
                        "\u{2500}".repeat(4),
                        "\u{2500}".repeat(20),
                        "\u{2500}".repeat(8),
                        "\u{2500}".repeat(10),
                        "\u{2500}".repeat(6),
                    );
                    for record in &history {
                        let ts = &record.timestamp[..record.timestamp.len().min(19)];
                        let dur = format!("{:.1}s", record.duration_ms as f64 / 1000.0);
                        let src = format!("{}/{}", record.sources_ok, record.total_sources);
                        let status_color = if record.sources_ok == record.total_sources {
                            GREEN
                        } else if record.sources_ok > record.total_sources / 2 {
                            YELLOW
                        } else {
                            RED
                        };
                        let status_icon = if record.sources_ok == record.total_sources {
                            "\u{2713}"
                        } else {
                            "\u{25cf}"
                        };
                        eprintln!(
                            "  {DIM}{:<6}{RESET} {:<22} {CYAN}{:<10}{RESET} {:<12} {status_color}{status_icon} OK{RESET}",
                            record.id, ts, dur, src,
                        );
                    }
                    eprintln!();
                }
            }
        }
        Commands::Trends { anomalies } => {
            let store = Store::open("runs/chaos.db")?;
            let history = store.get_sweep_history(20)?;

            if history.is_empty() {
                eprintln!(
                    "{YELLOW}No sweep history yet.{RESET} Run {BOLD}chaos sweep{RESET} first."
                );
                return Ok(());
            }

            // Collect metrics from each sweep
            let mut vix_vals: Vec<Option<f64>> = Vec::new();
            let mut wti_vals: Vec<Option<f64>> = Vec::new();
            let mut btc_vals: Vec<Option<f64>> = Vec::new();
            let mut spy_vals: Vec<Option<f64>> = Vec::new();
            let mut gold_vals: Vec<Option<f64>> = Vec::new();
            let mut conflict_vals: Vec<Option<f64>> = Vec::new();
            let mut quake_vals: Vec<Option<f64>> = Vec::new();
            let mut source_vals: Vec<Option<f64>> = Vec::new();

            // Process in chronological order
            for record in history.iter().rev() {
                let data = match store.get_sweep_data(record.id) {
                    Ok(Some(d)) => d,
                    _ => continue,
                };

                vix_vals.push(extract_fred_value(&data, "VIXCLS"));
                wti_vals.push(
                    data.get("sources")
                        .and_then(|s| s.get("EIA"))
                        .and_then(|e| e.get("data"))
                        .and_then(|d| d.get("wti"))
                        .and_then(|w| w.get("value"))
                        .and_then(|v| v.as_f64()),
                );
                btc_vals.push(
                    data.get("sources")
                        .and_then(|s| s.get("YFinance"))
                        .and_then(|y| y.get("quotes"))
                        .and_then(|q| q.get("BTC-USD"))
                        .and_then(|b| b.get("price"))
                        .and_then(|v| v.as_f64()),
                );
                spy_vals.push(
                    data.get("sources")
                        .and_then(|s| s.get("YFinance"))
                        .and_then(|y| y.get("quotes"))
                        .and_then(|q| q.get("SPY"))
                        .and_then(|b| b.get("price"))
                        .and_then(|v| v.as_f64()),
                );
                gold_vals.push(
                    data.get("sources")
                        .and_then(|s| s.get("YFinance"))
                        .and_then(|y| y.get("quotes"))
                        .and_then(|q| q.get("GC=F"))
                        .and_then(|b| b.get("price"))
                        .and_then(|v| v.as_f64()),
                );
                conflict_vals.push(
                    data.get("sources")
                        .and_then(|s| s.get("ACLED"))
                        .and_then(|a| a.get("totalEvents"))
                        .and_then(|v| v.as_u64())
                        .map(|v| v as f64),
                );
                quake_vals.push(
                    data.get("sources")
                        .and_then(|s| s.get("USGS"))
                        .and_then(|u| u.get("quakes"))
                        .and_then(|q| q.as_array())
                        .map(|a| a.len() as f64),
                );
                source_vals.push(Some(record.sources_ok as f64));
            }

            eprintln!();
            eprintln!(
                "{BOLD}{CYAN}  Trends (last {} sweeps){RESET}",
                history.len()
            );
            eprintln!(
                "  {DIM}{}{RESET}",
                "\u{2500}".repeat(42)
            );
            eprintln!();
            print_sparkline("VIX", &vix_vals, "", YELLOW);
            print_sparkline("WTI", &wti_vals, "$", CYAN);
            print_sparkline("BTC", &btc_vals, "$", CYAN);
            print_sparkline("SPY", &spy_vals, "$", CYAN);
            print_sparkline("Gold", &gold_vals, "$", CYAN);
            print_sparkline("Conflicts", &conflict_vals, "", RED);
            print_sparkline("Quakes", &quake_vals, "", MAGENTA);
            print_sparkline("Sources", &source_vals, "", GREEN);
            eprintln!();

            // Anomaly detection
            if anomalies {
                // Collect sweep data for anomaly analysis
                let mut history_data: Vec<serde_json::Value> = Vec::new();
                for record in history.iter().rev() {
                    if let Ok(Some(d)) = store.get_sweep_data(record.id) {
                        history_data.push(d);
                    }
                }

                if history_data.len() >= 4 {
                    let current = &history_data[history_data.len() - 1];
                    let past = &history_data[..history_data.len() - 1];
                    let anomaly_signals = delta::detect_anomalies(current, past);

                    if anomaly_signals.is_empty() {
                        eprintln!(
                            "{DIM}No anomalies detected vs historical data.{RESET}"
                        );
                    } else {
                        eprintln!(
                            "{BOLD}{YELLOW}  Anomalies ({} detected){RESET}",
                            anomaly_signals.len()
                        );
                        eprintln!(
                            "  {DIM}{}{RESET}",
                            "\u{2500}".repeat(42)
                        );
                        for sig in &anomaly_signals {
                            let sc = if sig.severity == "extreme" {
                                RED
                            } else {
                                YELLOW
                            };
                            let arrow = if sig.current > sig.mean {
                                "\u{25b2}"
                            } else {
                                "\u{25bc}"
                            };
                            eprintln!(
                                "  {sc}{arrow}{RESET} {BOLD}{:<14}{RESET} {CYAN}{:.2}{RESET} (mean {DIM}{:.2}{RESET}, z={sc}{:.1}{RESET}) {sc}{BOLD}{}{RESET}",
                                sig.key, sig.current, sig.mean, sig.z_score, sig.severity,
                            );
                        }
                        eprintln!();
                    }
                } else {
                    eprintln!(
                        "{DIM}Need at least 4 sweeps for anomaly detection (have {}).{RESET}",
                        history_data.len()
                    );
                }
            }
        }
        Commands::Report => {
            let store = Store::open("runs/chaos.db")?;

            match store.get_latest_sweep()? {
                Some((_id, data)) => {
                    let analysis = store.get_latest_analysis()?;
                    let delta = data.get("delta").cloned();

                    let md = report::generate_markdown_report(
                        &data,
                        analysis.as_deref(),
                        delta.as_ref(),
                    );
                    // Colorize markdown headers for terminal output
                    for line in md.lines() {
                        if line.starts_with("# ") {
                            eprintln!(
                                "\n{BOLD}{CYAN}{}{RESET}",
                                &line[2..]
                            );
                            eprintln!(
                                "{CYAN}{}{RESET}",
                                "\u{2550}".repeat(50)
                            );
                        } else if line.starts_with("## ") {
                            eprintln!(
                                "\n{BOLD}{MAGENTA}{}{RESET}",
                                &line[3..]
                            );
                            eprintln!(
                                "{DIM}{}{RESET}",
                                "\u{2500}".repeat(40)
                            );
                        } else if line.starts_with("### ") {
                            eprintln!(
                                "{BOLD}{YELLOW}{}{RESET}",
                                &line[4..]
                            );
                        } else if line.starts_with("- **") {
                            // Bold list items
                            println!("{BOLD}{line}{RESET}");
                        } else {
                            println!("{line}");
                        }
                    }
                }
                None => {
                    eprintln!(
                        "{YELLOW}No sweep data available.{RESET} Run {BOLD}chaos sweep{RESET} first."
                    );
                }
            }
        }
        Commands::Export { format, limit } => {
            let store = Store::open("runs/chaos.db")?;

            match format.as_str() {
                "csv" => {
                    let csv = report::generate_csv_trends(&store, limit)?;
                    print!("{}", csv);
                }
                "json" => {
                    let history = store.get_sweep_history(limit)?;
                    if history.is_empty() {
                        println!("[]");
                    } else {
                        let mut sweeps: Vec<serde_json::Value> = Vec::new();
                        for record in &history {
                            if let Ok(Some(data)) = store.get_sweep_data(record.id) {
                                sweeps.push(data);
                            }
                        }
                        println!("{}", serde_json::to_string_pretty(&sweeps)?);
                    }
                }
                _ => {
                    eprintln!(
                        "{RED}Unknown format '{BOLD}{format}{RESET}{RED}'.{RESET} Use {BOLD}csv{RESET} or {BOLD}json{RESET}."
                    );
                    std::process::exit(1);
                }
            }
        }
        Commands::Serve {
            port,
            public,
            api_key,
        } => {
            if public && api_key.is_none() {
                eprintln!(
                    "{RED}[CHAOS] Error:{RESET} --public mode requires --api-key"
                );
                std::process::exit(1);
            }

            let cfg = Config::load()?;
            let client = HttpClient::new(cfg.source_timeout_secs, 3)?;
            let llm_provider = llm::create_provider(&cfg);
            let sources = briefing::list_sources(&client);

            let (tx, _rx) = broadcast::channel::<String>(64);

            let bind_addr = if public { "0.0.0.0" } else { "127.0.0.1" };
            let addr = format!("{}:{}", bind_addr, port);

            let llm_status = if llm_provider.is_some() {
                format!("{GREEN}enabled{RESET}")
            } else {
                format!("{RED}disabled{RESET}")
            };

            // Build bot instances
            let telegram = match (&cfg.telegram_bot_token, &cfg.telegram_chat_id) {
                (Some(token), Some(chat_id)) => {
                    let tg = bot::telegram::TelegramBot::new(token, chat_id);
                    if tg.is_configured() {
                        eprintln!("  {DIM}[CHAOS]{RESET} {GREEN}Telegram bot enabled{RESET}");
                        Some(Arc::new(RwLock::new(tg)))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let discord = {
                let dc = bot::discord::DiscordBot::from_config(&cfg);
                if dc.is_configured() {
                    eprintln!("  {DIM}[CHAOS]{RESET} {GREEN}Discord {} enabled{RESET}", dc.mode());
                    Some(Arc::new(RwLock::new(dc)))
                } else {
                    None
                }
            };

            let state = Arc::new(dashboard::AppState {
                config: cfg.clone(),
                client,
                db_path: "runs/chaos.db".to_string(),
                llm: llm_provider.map(|p| Arc::from(p) as Arc<dyn llm::LlmProvider>),
                current_data: RwLock::new(None),
                sweep_in_progress: RwLock::new(false),
                last_sweep_time: RwLock::new(None),
                start_time: std::time::Instant::now(),
                tx,
                api_key: api_key.clone(),
                rate_get: dashboard::RateLimiter::new(60),
                rate_post: dashboard::RateLimiter::new(20),
                telegram,
                discord,
            });

            let app = dashboard::create_router(state.clone(), public, api_key);

            // -- startup banner with ASCII art --
            eprintln!();
            eprintln!("{CYAN}{BOLD}   ██████╗ ██╗  ██╗  █████╗   ██████╗  ███████╗{RESET}");
            eprintln!("{CYAN}{BOLD}  ██╔════╝ ██║  ██║ ██╔══██╗ ██╔═══██╗ ██╔════╝{RESET}");
            eprintln!("{CYAN}{BOLD}  ██║      ███████║ ███████║ ██║   ██║ ███████╗{RESET}");
            eprintln!("{CYAN}{BOLD}  ██║      ██╔══██║ ██╔══██║ ██║   ██║ ╚════██║{RESET}");
            eprintln!("{CYAN}{BOLD}  ╚██████╗ ██║  ██║ ██║  ██║ ╚██████╔╝ ███████║{RESET}");
            eprintln!("{CYAN}{BOLD}   ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═════╝  ╚══════╝{RESET}");
            eprintln!();
            eprintln!("  {BOLD}{WHITE}CHAOS Engine{RESET} {DIM}v{}{RESET} {DIM}—{RESET} {CYAN}Connected Human-Augmented OSINT Suite{RESET}", env!("CARGO_PKG_VERSION"));
            eprintln!("  {DIM}{} sources{RESET} {DIM}│{RESET} {BOLD}one{RESET} binary {DIM}│{RESET} {BOLD}zero{RESET} cloud", sources.len());
            eprintln!();
            eprintln!("  {DIM}╭──────────────────────────────────────────╮{RESET}");
            eprintln!("  {DIM}│{RESET} {GREEN}●{RESET} Dashboard   {BOLD}{CYAN}http://{addr}{RESET}{DIM}{}│{RESET}", " ".repeat(20usize.saturating_sub(addr.len())));
            eprintln!("  {DIM}│{RESET} {CYAN}◆{RESET} Mode        {BOLD}{}{RESET}{DIM}{}│{RESET}",
                if public { "public (API key required)" } else { "local" },
                if public { "    " } else { "                        " });
            eprintln!("  {DIM}│{RESET} {CYAN}◆{RESET} Refresh     every {CYAN}{} min{RESET}{DIM}{}│{RESET}",
                cfg.refresh_interval_minutes,
                " ".repeat(16usize.saturating_sub(format!("{}", cfg.refresh_interval_minutes).len())));
            eprintln!("  {DIM}│{RESET} {}{RESET}{DIM}{}│{RESET}",
                if llm_status.contains("disabled") { format!("{RED}○{RESET} LLM         disabled") } else { format!("{GREEN}●{RESET} LLM         {llm_status}") },
                " ".repeat(10));
            eprintln!("  {DIM}│{RESET} {CYAN}◆{RESET} Telegram    {}{RESET}{DIM}{}│{RESET}",
                if cfg.telegram_bot_token.is_some() { format!("{GREEN}enabled{RESET}") } else { format!("{DIM}disabled{RESET}") },
                " ".repeat(if cfg.telegram_bot_token.is_some() { 21 } else { 22 }));
            eprintln!("  {DIM}│{RESET} {CYAN}◆{RESET} Discord     {}{RESET}{DIM}{}│{RESET}",
                if cfg.discord_bot_token.is_some() || cfg.discord_webhook_url.is_some() { format!("{GREEN}enabled{RESET}") } else { format!("{DIM}disabled{RESET}") },
                " ".repeat(if cfg.discord_bot_token.is_some() || cfg.discord_webhook_url.is_some() { 21 } else { 22 }));
            eprintln!("  {DIM}╰──────────────────────────────────────────╯{RESET}");
            eprintln!();
            eprintln!(
                "  {DIM}[CHAOS]{RESET} {GREEN}Server running...{RESET}"
            );
            eprintln!(
                "  {DIM}[CHAOS]{RESET} Running initial sweep..."
            );

            // Start Telegram command polling (if configured)
            dashboard::spawn_telegram_poll(state.clone());

            // Start sweep loop on dedicated thread (Store is !Send)
            dashboard::spawn_sweep_loop(state);

            // Start HTTP server
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }
        Commands::MarketSeeds { count } => {
            let cfg = Config::load()?;
            let client = HttpClient::new(cfg.source_timeout_secs, 3)?;
            let store = Store::open("runs/chaos.db")?;
            let llm_provider = llm::create_provider(&cfg);

            let data = briefing::full_sweep(
                &client,
                &store,
                llm_provider
                    .as_ref()
                    .map(|p| p.as_ref() as &dyn llm::LlmProvider),
                &cfg,
            )
            .await;

            // Use LLM if available, otherwise use rule-based seeds from sweep data
            if let Some(provider) = &llm_provider {
                let opts = LlmOptions {
                    max_tokens: 2048,
                    temperature: 0.7,
                    model_override: None,
                };
                let prompt = format!(
                    "Generate {} prediction market questions based on this intelligence data. \
                     Each question should be a YES/NO question resolvable within 7-30 days. \
                     Return JSON array with fields: question, category (one of: geopolitics, economics, \
                     science, technology, health, environment, sports, entertainment, politics, other), \
                     resolution_criteria, confidence (0-1). Data:\n{}",
                    count,
                    serde_json::to_string(&data.get("sources").unwrap_or(&serde_json::json!({})))
                        .unwrap_or_default()
                        .chars()
                        .take(4000)
                        .collect::<String>()
                );

                match provider
                    .complete("You are a prediction market analyst.", &prompt, &opts)
                    .await
                {
                    Ok(response) => println!("{}", response.text),
                    Err(e) => {
                        eprintln!("{RED}LLM seed generation failed:{RESET} {e}");
                        eprintln!("{DIM}Falling back to rule-based seeds from sweep data.{RESET}");
                        // Print what we have from the sweep
                        println!("{}", serde_json::to_string_pretty(&data.get("sources"))?);
                    }
                }
            } else {
                eprintln!(
                    "{YELLOW}No LLM configured.{RESET} Run {BOLD}chaos serve{RESET} and use GET /api/v1/market-seeds for rule-based seeds."
                );
            }
        }
        Commands::Migrate { from_json } => {
            let store = Store::open("runs/chaos.db")?;
            let json_dir = std::path::Path::new(&from_json);
            if !json_dir.is_dir() {
                eprintln!(
                    "{RED}Directory not found:{RESET} {BOLD}{}{RESET}",
                    from_json
                );
                std::process::exit(1);
            }

            let mut imported = 0;
            let mut entries: Vec<_> = std::fs::read_dir(json_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "json")
                        .unwrap_or(false)
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in &entries {
                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!(
                            "  {RED}\u{2717}{RESET} {}: {}",
                            entry.file_name().to_string_lossy(),
                            e
                        );
                        continue;
                    }
                };
                let data: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!(
                            "  {RED}\u{2717}{RESET} {}: invalid JSON: {}",
                            entry.file_name().to_string_lossy(),
                            e
                        );
                        continue;
                    }
                };
                let timestamp = data
                    .get("timestamp")
                    .or_else(|| data.get("chaos").and_then(|c| c.get("timestamp")))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();

                if let Err(e) = store.save_migrated_sweep(&timestamp, &data) {
                    eprintln!(
                        "  {RED}\u{2717}{RESET} {}: {}",
                        entry.file_name().to_string_lossy(),
                        e
                    );
                } else {
                    imported += 1;
                    eprintln!(
                        "  {GREEN}\u{2713}{RESET} {}",
                        entry.file_name().to_string_lossy()
                    );
                }
            }
            eprintln!();
            eprintln!(
                "{GREEN}Imported {imported} sweep(s){RESET} from {BOLD}{}{RESET}",
                from_json
            );
        }
        Commands::Watch { interval } => {
            let mut cfg = Config::load()?;
            cfg.refresh_interval_minutes = interval;
            let client = HttpClient::new(cfg.source_timeout_secs, 3)?;
            let llm_provider = llm::create_provider(&cfg);

            eprintln!(
                "{BOLD}{CYAN}[CHAOS]{RESET} Watch mode -- sweeping every {BOLD}{interval}{RESET} minutes"
            );
            eprintln!("{DIM}Press Ctrl+C to stop.{RESET}");

            loop {
                let store = Store::open("runs/chaos.db")?;
                let start = std::time::Instant::now();
                eprintln!();
                eprintln!(
                    "{DIM}[CHAOS]{RESET} Starting sweep at {}",
                    chrono::Utc::now().format("%H:%M:%S")
                );

                let _data = briefing::full_sweep(
                    &client,
                    &store,
                    llm_provider
                        .as_ref()
                        .map(|p| p.as_ref() as &dyn llm::LlmProvider),
                    &cfg,
                )
                .await;

                let elapsed = start.elapsed();
                eprintln!(
                    "{GREEN}\u{2713}{RESET} Sweep complete in {CYAN}{:.1}s{RESET}",
                    elapsed.as_secs_f64()
                );

                tokio::time::sleep(std::time::Duration::from_secs(interval * 60)).await;
            }
        }
    }

    Ok(())
}

/// Render a sparkline from optional f64 values with color.
fn print_sparkline(label: &str, values: &[Option<f64>], prefix: &str, color: &str) {
    let chars = [
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];

    let concrete: Vec<f64> = values.iter().filter_map(|v| *v).collect();
    if concrete.is_empty() {
        eprintln!("  {BOLD}{:<12}{RESET} {DIM}(no data){RESET}", label);
        return;
    }

    let min = concrete.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = concrete.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    let sparkline: String = values
        .iter()
        .map(|v| match v {
            Some(val) => {
                let idx = if range > f64::EPSILON {
                    (((val - min) / range) * 7.0).round() as usize
                } else {
                    4
                };
                chars[idx.min(7)]
            }
            None => ' ',
        })
        .collect();

    let current = concrete.last().unwrap();

    // Format the current value
    let current_str = if *current > 10_000.0 {
        format!("{}{}K", prefix, format_number(*current / 1000.0))
    } else if *current > 1000.0 {
        format!("{}{}", prefix, format_number(*current))
    } else {
        format!("{}{:.1}", prefix, current)
    };

    eprintln!(
        "  {BOLD}{label:<12}{RESET} {color}{sparkline}{RESET}  {CYAN}{current_str}{RESET}",
    );
}

/// Format large numbers with comma separators.
fn format_number(n: f64) -> String {
    let integer = n as u64;
    let s = integer.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Extract a FRED indicator value from sweep data.
fn extract_fred_value(data: &serde_json::Value, series_id: &str) -> Option<f64> {
    let indicators = data
        .get("sources")?
        .get("FRED")?
        .get("indicators")?
        .as_array()?;
    for item in indicators {
        if item.get("id")?.as_str()? == series_id {
            return item.get("value")?.as_f64();
        }
    }
    None
}

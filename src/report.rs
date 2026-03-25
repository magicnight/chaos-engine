use serde_json::Value;

use crate::store::Store;

/// Generate a Markdown intelligence report from sweep data.
pub fn generate_markdown_report(
    sweep_data: &Value,
    analysis: Option<&str>,
    delta: Option<&Value>,
) -> String {
    let mut report = String::with_capacity(4096);

    let timestamp = sweep_data
        .get("chaos")
        .and_then(|c| c.get("timestamp"))
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    report.push_str("# CHAOS Intelligence Report\n\n");

    // Format timestamp for readability
    let display_ts = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|_| timestamp.to_string());
    report.push_str(&format!("**Date:** {}\n\n", display_ts));

    // Situation overview
    report.push_str("## Situation Overview\n\n");
    if let Some(text) = analysis {
        report.push_str(text);
        report.push_str("\n\n");
    } else {
        let sources_ok = sweep_data
            .get("chaos")
            .and_then(|c| c.get("sourcesOk"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let sources_total = sweep_data
            .get("chaos")
            .and_then(|c| c.get("sourcesQueried"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let duration = sweep_data
            .get("chaos")
            .and_then(|c| c.get("totalDurationMs"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        report.push_str(&format!(
            "Automated sweep completed in {}ms. {}/{} sources reporting normally.\n\n",
            duration, sources_ok, sources_total
        ));
    }

    // Source status table
    report.push_str("## Source Status\n\n");
    report.push_str("| Source | Status | Duration |\n");
    report.push_str("|--------|--------|----------|\n");
    if let Some(timing) = sweep_data.get("timing").and_then(|t| t.as_object()) {
        for (name, info) in timing {
            let status = info
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            let ms = info.get("ms").and_then(|m| m.as_u64()).unwrap_or(0);
            report.push_str(&format!("| {} | {} | {}ms |\n", name, status, ms));
        }
    }
    report.push('\n');

    // Source reliability stats
    report.push_str("## Source Reliability\n\n");
    if let Some(timing) = sweep_data.get("timing").and_then(|t| t.as_object()) {
        let total = timing.len();
        let ok_count = timing.values().filter(|v| {
            v.get("status").and_then(|s| s.as_str()) == Some("ok")
        }).count();
        let err_count = timing.values().filter(|v| {
            v.get("status").and_then(|s| s.as_str()) == Some("error")
        }).count();
        let tmo_count = timing.values().filter(|v| {
            v.get("status").and_then(|s| s.as_str()) == Some("timeout")
        }).count();
        let avg_ms: u64 = if total > 0 {
            timing.values()
                .filter_map(|v| v.get("ms").and_then(|m| m.as_u64()))
                .sum::<u64>() / total as u64
        } else {
            0
        };
        report.push_str(&format!(
            "- **Success rate:** {}/{} ({:.0}%)\n- **Errors:** {} | **Timeouts:** {}\n- **Avg response:** {}ms\n\n",
            ok_count, total,
            if total > 0 { ok_count as f64 / total as f64 * 100.0 } else { 0.0 },
            err_count, tmo_count, avg_ms
        ));
    }

    // Key metrics
    report.push_str("## Key Metrics\n\n");
    if let Some(sources) = sweep_data.get("sources").and_then(|s| s.as_object()) {
        // FRED indicators
        if let Some(fred) = sources.get("FRED").and_then(|f| f.get("indicators")).and_then(|i| i.as_array()) {
            for item in fred {
                let id = item.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                if let Some(val) = item.get("value").and_then(|v| v.as_f64()) {
                    report.push_str(&format!("- **{}**: {:.2}\n", id, val));
                }
            }
        }

        // EIA
        if let Some(eia_data) = sources.get("EIA").and_then(|e| e.get("data")) {
            for (key, label) in &[("wti", "WTI"), ("brent", "BRENT"), ("henryHub", "NATGAS")] {
                if let Some(val) = eia_data.get(*key).and_then(|s| s.get("value")).and_then(|v| v.as_f64()) {
                    report.push_str(&format!("- **EIA {}**: ${:.2}\n", label, val));
                }
            }
        }

        // YFinance
        if let Some(quotes) = sources.get("YFinance").and_then(|y| y.get("quotes")) {
            for (symbol, label) in &[("SPY", "SPY"), ("BTC-USD", "BTC"), ("GC=F", "GOLD")] {
                if let Some(val) = quotes.get(*symbol).and_then(|q| q.get("price")).and_then(|v| v.as_f64()) {
                    report.push_str(&format!("- **{}**: ${:.2}\n", label, val));
                }
            }
        }
    }
    report.push('\n');

    // WorldNews headlines
    if let Some(sources) = sweep_data.get("sources").and_then(|s| s.as_object()) {
        if let Some(wn) = sources.get("WorldNews") {
            if let Some(top_neg) = wn.get("topNegative").and_then(|t| t.as_array()) {
                if !top_neg.is_empty() {
                    report.push_str("## Top News Headlines\n\n");
                    for article in top_neg.iter().take(5) {
                        let title = article.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled");
                        let category = article.get("category").and_then(|c| c.as_str()).unwrap_or("");
                        let sentiment = article.get("sentiment").and_then(|s| s.as_f64());
                        let sentiment_str = sentiment
                            .map(|s| format!(" (sentiment: {:.2})", s))
                            .unwrap_or_default();
                        report.push_str(&format!("- **[{}]** {}{}\n", category, title, sentiment_str));
                    }
                    report.push('\n');
                }
            }
        }
    }

    // Delta changes
    if let Some(delta_val) = delta {
        report.push_str("## Delta Changes\n\n");
        if let Some(summary) = delta_val.get("summary") {
            let total = summary.get("total_changes").and_then(|v| v.as_u64()).unwrap_or(0);
            let critical = summary.get("critical_changes").and_then(|v| v.as_u64()).unwrap_or(0);
            let direction = summary.get("direction").and_then(|v| v.as_str()).unwrap_or("unknown");
            report.push_str(&format!(
                "**Direction:** {} | **Changes:** {} | **Critical:** {}\n\n",
                direction, total, critical
            ));
        }

        if let Some(signals) = delta_val.get("signals") {
            for category in &["escalated", "deescalated"] {
                if let Some(items) = signals.get(*category).and_then(|v| v.as_array()) {
                    if !items.is_empty() {
                        let header = if *category == "escalated" { "Escalated" } else { "De-escalated" };
                        report.push_str(&format!("### {}\n\n", header));
                        for item in items {
                            let label = item.get("label").and_then(|v| v.as_str()).unwrap_or("?");
                            let pct = item.get("pct_change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let sev = item.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
                            report.push_str(&format!("- **{}**: {:+.1}% [{}]\n", label, pct, sev));
                        }
                        report.push('\n');
                    }
                }
            }
        }
    }

    // Correlations
    if let Some(corrs) = sweep_data.get("correlations").and_then(|c| c.as_array()) {
        if !corrs.is_empty() {
            report.push_str("## Correlations\n\n");
            for c in corrs {
                let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let severity = c.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
                let desc = c.get("description").and_then(|v| v.as_str()).unwrap_or("");
                report.push_str(&format!("- **{}** [{}]: {}\n", name, severity, desc));
            }
            report.push('\n');
        }
    }

    // Watchlist matches
    if let Some(matches) = sweep_data.get("watchlist_matches").and_then(|w| w.as_array()) {
        if !matches.is_empty() {
            report.push_str("## Watchlist Matches\n\n");
            for m in matches {
                let mtype = m.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                let matched = m.get("matched").and_then(|v| v.as_str()).unwrap_or("?");
                let source = m.get("source").and_then(|v| v.as_str()).unwrap_or("?");
                report.push_str(&format!("- [{}] **{}** in {}\n", mtype, matched, source));
            }
            report.push('\n');
        }
    }

    // Errors
    if let Some(errors) = sweep_data.get("errors").and_then(|e| e.as_array()) {
        if !errors.is_empty() {
            report.push_str("## Source Errors\n\n");
            for e in errors {
                let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let error = e.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                report.push_str(&format!("- **{}**: {}\n", name, error));
            }
            report.push('\n');
        }
    }

    report.push_str("---\n*Generated by CHAOS v");
    report.push_str(env!("CARGO_PKG_VERSION"));
    report.push_str("*\n");

    report
}

/// Export trend data as CSV from the last N sweeps.
pub fn generate_csv_trends(store: &Store, limit: usize) -> anyhow::Result<String> {
    let mut csv = String::from("timestamp,vix,wti,btc,spy,gold,conflicts,quakes,sources_ok\n");

    let history = store.get_sweep_history(limit)?;

    // Process in chronological order (history is newest-first)
    for record in history.iter().rev() {
        let data = match store.get_sweep_data(record.id)? {
            Some(d) => d,
            None => continue,
        };

        let timestamp = &record.timestamp;
        let vix = extract_metric_f64(&data, "FRED", "VIXCLS");
        let wti = data.get("sources")
            .and_then(|s| s.get("EIA"))
            .and_then(|e| e.get("data"))
            .and_then(|d| d.get("wti"))
            .and_then(|w| w.get("value"))
            .and_then(|v| v.as_f64());
        let btc = data.get("sources")
            .and_then(|s| s.get("YFinance"))
            .and_then(|y| y.get("quotes"))
            .and_then(|q| q.get("BTC-USD"))
            .and_then(|b| b.get("price"))
            .and_then(|v| v.as_f64());
        let spy = data.get("sources")
            .and_then(|s| s.get("YFinance"))
            .and_then(|y| y.get("quotes"))
            .and_then(|q| q.get("SPY"))
            .and_then(|b| b.get("price"))
            .and_then(|v| v.as_f64());
        let gold = data.get("sources")
            .and_then(|s| s.get("YFinance"))
            .and_then(|y| y.get("quotes"))
            .and_then(|q| q.get("GC=F"))
            .and_then(|b| b.get("price"))
            .and_then(|v| v.as_f64());
        let conflicts = data.get("sources")
            .and_then(|s| s.get("ACLED"))
            .and_then(|a| a.get("totalEvents"))
            .and_then(|v| v.as_u64());
        let quakes = data.get("sources")
            .and_then(|s| s.get("USGS"))
            .and_then(|u| u.get("quakes"))
            .and_then(|q| q.as_array())
            .map(|a| a.len() as u64);
        let sources_ok = record.sources_ok;

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            timestamp,
            fmt_opt_f64(vix),
            fmt_opt_f64(wti),
            fmt_opt_f64(btc),
            fmt_opt_f64(spy),
            fmt_opt_f64(gold),
            fmt_opt_u64(conflicts),
            fmt_opt_u64(quakes),
            sources_ok,
        ));
    }

    Ok(csv)
}

fn extract_metric_f64(data: &Value, source: &str, series_id: &str) -> Option<f64> {
    let indicators = data.get("sources")?.get(source)?.get("indicators")?.as_array()?;
    for item in indicators {
        if item.get("id")?.as_str()? == series_id {
            return item.get("value")?.as_f64();
        }
    }
    None
}

fn fmt_opt_f64(v: Option<f64>) -> String {
    match v {
        Some(val) => format!("{:.2}", val),
        None => String::new(),
    }
}

fn fmt_opt_u64(v: Option<u64>) -> String {
    match v {
        Some(val) => val.to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_markdown_report_basic() {
        let sweep = json!({
            "chaos": {
                "version": "0.1.0",
                "timestamp": "2025-01-01T00:00:00Z",
                "totalDurationMs": 3400,
                "sourcesQueried": 33,
                "sourcesOk": 30,
                "sourcesFailed": 3,
            },
            "sources": {},
            "errors": [],
            "timing": {},
        });

        let report = generate_markdown_report(&sweep, None, None);
        assert!(report.contains("# CHAOS Intelligence Report"));
        assert!(report.contains("2025-01-01 00:00:00 UTC"));
        assert!(report.contains("30/33 sources"));
    }

    #[test]
    fn test_generate_markdown_report_with_analysis() {
        let sweep = json!({
            "chaos": {
                "timestamp": "2025-01-01T00:00:00Z",
                "sourcesQueried": 10,
                "sourcesOk": 10,
                "sourcesFailed": 0,
                "totalDurationMs": 1000,
            },
            "sources": {},
            "errors": [],
            "timing": {},
        });

        let report = generate_markdown_report(&sweep, Some("Test analysis content"), None);
        assert!(report.contains("Test analysis content"));
    }

    #[test]
    fn test_generate_csv_trends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_trends.db");
        let store = Store::open(path.to_str().unwrap()).unwrap();

        let data = json!({
            "chaos": { "timestamp": "2025-01-01T00:00:00Z", "sourcesOk": 10 },
            "sources": {
                "FRED": { "indicators": [{ "id": "VIXCLS", "value": 18.5 }] }
            }
        });
        store.save_sweep(&data, 100, 10, 0, 10).unwrap();

        let csv = generate_csv_trends(&store, 10).unwrap();
        assert!(csv.starts_with("timestamp,vix,wti,btc,spy,gold,conflicts,quakes,sources_ok\n"));
        assert!(csv.contains("18.50"));
    }
}

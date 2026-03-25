pub mod memory;

use serde_json::Value;

// ─── Numeric Thresholds: % change to flag ────────────────────────────────────

const NUMERIC_THRESHOLDS: &[(&str, f64)] = &[
    ("vix", 5.0),
    ("hy_spread", 5.0),
    ("10y2y", 10.0),
    ("wti", 3.0),
    ("brent", 3.0),
    ("natgas", 5.0),
    ("unemployment", 2.0),
    ("fed_funds", 1.0),
    ("10y_yield", 3.0),
    ("usd_index", 1.0),
    ("mortgage", 2.0),
    ("spy", 2.0),
    ("btc", 5.0),
    ("gold", 2.0),
];

// ─── Count Thresholds: absolute change to flag ──────────────────────────────

const COUNT_THRESHOLDS: &[(&str, u64)] = &[
    ("urgent_posts", 2),
    ("thermal_total", 500),
    ("air_total", 50),
    ("who_alerts", 1),
    ("conflict_events", 5),
    ("conflict_fatalities", 10),
    ("sdr_online", 3),
    ("news_count", 5),
    ("sources_ok", 1),
    ("quakes", 10),
    ("cve_critical", 3),
];

/// Risk-sensitive keys used for determining overall direction.
const RISK_KEYS: &[&str] = &[
    "vix",
    "hy_spread",
    "urgent_posts",
    "conflict_events",
    "thermal_total",
];

// ─── Metric Definitions ─────────────────────────────────────────────────────

struct NumericMetric {
    key: &'static str,
    label: &'static str,
    extract: fn(&Value) -> Option<f64>,
}

struct CountMetric {
    key: &'static str,
    label: &'static str,
    extract: fn(&Value) -> Option<u64>,
}

// Helper: find a FRED indicator by id and return its value
fn fred_value(data: &Value, series_id: &str) -> Option<f64> {
    let indicators = data.get("sources")?.get("FRED")?.get("indicators")?.as_array()?;
    for item in indicators {
        if item.get("id")?.as_str()? == series_id {
            return item.get("value")?.as_f64();
        }
    }
    None
}

const NUMERIC_METRICS: &[NumericMetric] = &[
    NumericMetric { key: "vix", label: "VIX", extract: |d| fred_value(d, "VIXCLS") },
    NumericMetric { key: "hy_spread", label: "HY Spread", extract: |d| fred_value(d, "BAMLH0A0HYM2") },
    NumericMetric { key: "10y2y", label: "10Y-2Y Spread", extract: |d| fred_value(d, "T10Y2Y") },
    NumericMetric {
        key: "wti",
        label: "WTI Crude",
        extract: |d| d.get("sources")?.get("EIA")?.get("data")?.get("wti")?.get("value")?.as_f64(),
    },
    NumericMetric {
        key: "brent",
        label: "Brent Crude",
        extract: |d| d.get("sources")?.get("EIA")?.get("data")?.get("brent")?.get("value")?.as_f64(),
    },
    NumericMetric {
        key: "natgas",
        label: "Natural Gas",
        extract: |d| d.get("sources")?.get("EIA")?.get("data")?.get("henryHub")?.get("value")?.as_f64(),
    },
    NumericMetric {
        key: "unemployment",
        label: "Unemployment",
        extract: |d| {
            let indicators = d.get("sources")?.get("BLS")?.get("indicators")?.as_array()?;
            for item in indicators {
                let id = item.get("id")?.as_str()?;
                if id == "LNS14000000" || id == "UNRATE" {
                    return item.get("value")?.as_f64();
                }
            }
            None
        },
    },
    NumericMetric { key: "fed_funds", label: "Fed Funds Rate", extract: |d| fred_value(d, "DFF") },
    NumericMetric { key: "10y_yield", label: "10Y Yield", extract: |d| fred_value(d, "DGS10") },
    NumericMetric { key: "usd_index", label: "USD Index", extract: |d| fred_value(d, "DTWEXBGS") },
    NumericMetric { key: "mortgage", label: "30Y Mortgage", extract: |d| fred_value(d, "MORTGAGE30US") },
    NumericMetric {
        key: "spy",
        label: "S&P 500 ETF",
        extract: |d| d.get("sources")?.get("YFinance")?.get("quotes")?.get("SPY")?.get("price")?.as_f64(),
    },
    NumericMetric {
        key: "btc",
        label: "Bitcoin",
        extract: |d| d.get("sources")?.get("YFinance")?.get("quotes")?.get("BTC-USD")?.get("price")?.as_f64(),
    },
    NumericMetric {
        key: "gold",
        label: "Gold",
        extract: |d| d.get("sources")?.get("YFinance")?.get("quotes")?.get("GC=F")?.get("price")?.as_f64(),
    },
];

const COUNT_METRICS: &[CountMetric] = &[
    CountMetric {
        key: "urgent_posts",
        label: "Urgent OSINT Posts",
        extract: |d| {
            d.get("sources")?
                .get("Telegram")?
                .get("urgent")?
                .as_array()
                .map(|a| a.len() as u64)
        },
    },
    CountMetric {
        key: "thermal_total",
        label: "Thermal Detections",
        extract: |d| {
            let items = d.get("sources")?.get("FIRMS")?.get("hotspots")?.as_array()?;
            let sum: u64 = items.iter().filter_map(|t| t.get("totalDetections")?.as_u64()).sum();
            Some(sum)
        },
    },
    CountMetric {
        key: "air_total",
        label: "Air Activity",
        extract: |d| {
            d.get("sources")?
                .get("OpenSky")?
                .get("totalAircraft")?
                .as_u64()
        },
    },
    CountMetric {
        key: "who_alerts",
        label: "WHO Alerts",
        extract: |d| {
            d.get("sources")?
                .get("WHO")?
                .get("alerts")?
                .as_array()
                .map(|a| a.len() as u64)
        },
    },
    CountMetric {
        key: "conflict_events",
        label: "Conflict Events",
        extract: |d| {
            d.get("sources")?
                .get("ACLED")?
                .get("totalEvents")?
                .as_u64()
        },
    },
    CountMetric {
        key: "conflict_fatalities",
        label: "Conflict Fatalities",
        extract: |d| {
            d.get("sources")?
                .get("ACLED")?
                .get("totalFatalities")?
                .as_u64()
        },
    },
    CountMetric {
        key: "sdr_online",
        label: "SDR Receivers",
        extract: |d| {
            d.get("sources")?
                .get("KiwiSDR")?
                .get("online")?
                .as_u64()
        },
    },
    CountMetric {
        key: "news_count",
        label: "News Items",
        extract: |d| {
            d.get("sources")?
                .get("GDELT")?
                .get("articles")?
                .as_array()
                .map(|a| a.len() as u64)
        },
    },
    CountMetric {
        key: "sources_ok",
        label: "Sources OK",
        extract: |d| {
            d.get("chaos")?.get("sourcesOk")?.as_u64()
        },
    },
    CountMetric {
        key: "quakes",
        label: "Earthquakes",
        extract: |d| {
            d.get("sources")?
                .get("USGS")?
                .get("quakes")?
                .as_array()
                .map(|a| a.len() as u64)
        },
    },
    CountMetric {
        key: "cve_critical",
        label: "Critical CVEs",
        extract: |d| {
            d.get("sources")?
                .get("NVD-CVE")?
                .get("criticalCount")?
                .as_u64()
        },
    },
];

// ─── Public Types ───────────────────────────────────────────────────────────

/// Severity level for a delta signal.
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Moderate,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Moderate => write!(f, "moderate"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// A single detected change between sweeps.
#[derive(Debug, Clone)]
pub struct DeltaSignal {
    pub key: String,
    pub label: String,
    pub from: f64,
    pub to: f64,
    pub pct_change: f64,
    pub direction: String,
    pub severity: Severity,
}

/// Categorized delta signals.
#[derive(Debug, Clone)]
pub struct DeltaSignals {
    pub new: Vec<DeltaSignal>,
    pub escalated: Vec<DeltaSignal>,
    pub deescalated: Vec<DeltaSignal>,
    pub unchanged: Vec<String>,
}

/// Summary of all changes in this delta.
#[derive(Debug, Clone)]
pub struct DeltaSummary {
    pub total_changes: usize,
    pub critical_changes: usize,
    pub direction: String,
}

/// Full result of a delta computation.
#[derive(Debug, Clone)]
pub struct DeltaResult {
    pub timestamp: String,
    pub previous: Option<String>,
    pub signals: DeltaSignals,
    pub summary: DeltaSummary,
}

// ─── Threshold Lookup Helpers ───────────────────────────────────────────────

fn numeric_threshold(key: &str) -> f64 {
    NUMERIC_THRESHOLDS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or(5.0)
}

fn count_threshold(key: &str) -> u64 {
    COUNT_THRESHOLDS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or(1)
}

fn is_risk_key(key: &str) -> bool {
    RISK_KEYS.contains(&key)
}

// ─── Core Delta Computation ─────────────────────────────────────────────────

/// Compare two sweep data values and produce a structured delta result.
///
/// Returns `None` if either input is null/empty (mirrors Node.js behavior on first run).
pub fn compute_delta(current: &Value, previous: &Value) -> Option<DeltaResult> {
    if current.is_null() || previous.is_null() {
        return None;
    }

    let mut signals = DeltaSignals {
        new: Vec::new(),
        escalated: Vec::new(),
        deescalated: Vec::new(),
        unchanged: Vec::new(),
    };
    let mut critical_changes: usize = 0;

    // ─── Numeric metrics: track % change ─────────────────────────────

    for m in NUMERIC_METRICS {
        let curr = (m.extract)(current);
        let prev = (m.extract)(previous);

        let (curr_val, prev_val) = match (curr, prev) {
            (Some(c), Some(p)) => (c, p),
            _ => continue,
        };

        let threshold = numeric_threshold(m.key);
        let pct_change = if prev_val.abs() > f64::EPSILON {
            ((curr_val - prev_val) / prev_val.abs()) * 100.0
        } else {
            0.0
        };

        if pct_change.abs() > threshold {
            let severity = if pct_change.abs() > threshold * 3.0 {
                Severity::Critical
            } else if pct_change.abs() > threshold * 2.0 {
                Severity::High
            } else {
                Severity::Moderate
            };

            let signal = DeltaSignal {
                key: m.key.to_string(),
                label: m.label.to_string(),
                from: prev_val,
                to: curr_val,
                pct_change: (pct_change * 100.0).round() / 100.0,
                direction: if pct_change > 0.0 {
                    "up".to_string()
                } else {
                    "down".to_string()
                },
                severity,
            };

            if pct_change.abs() > 10.0 {
                critical_changes += 1;
            }

            if pct_change > 0.0 {
                signals.escalated.push(signal);
            } else {
                signals.deescalated.push(signal);
            }
        } else {
            signals.unchanged.push(m.key.to_string());
        }
    }

    // ─── Count metrics: track absolute change ────────────────────────

    for m in COUNT_METRICS {
        let curr = (m.extract)(current);
        let prev = (m.extract)(previous);

        let (curr_val, prev_val) = match (curr, prev) {
            (Some(c), Some(p)) => (c, p),
            _ => continue,
        };

        let diff = curr_val as i64 - prev_val as i64;
        let threshold = count_threshold(m.key);

        if (diff.unsigned_abs()) >= threshold {
            let pct_change = if prev_val > 0 {
                (diff as f64 / prev_val as f64) * 100.0
            } else if diff > 0 {
                100.0
            } else {
                0.0
            };

            let severity = if diff.unsigned_abs() >= threshold * 5 {
                Severity::Critical
            } else if diff.unsigned_abs() >= threshold * 2 {
                Severity::High
            } else {
                Severity::Moderate
            };

            if severity == Severity::Critical {
                critical_changes += 1;
            }

            let signal = DeltaSignal {
                key: m.key.to_string(),
                label: m.label.to_string(),
                from: prev_val as f64,
                to: curr_val as f64,
                pct_change: (pct_change * 10.0).round() / 10.0,
                direction: if diff > 0 {
                    "up".to_string()
                } else {
                    "down".to_string()
                },
                severity,
            };

            if diff > 0 {
                signals.escalated.push(signal);
            } else {
                signals.deescalated.push(signal);
            }
        } else {
            signals.unchanged.push(m.key.to_string());
        }
    }

    // ─── Overall direction ───────────────────────────────────────────

    let risk_up = signals
        .escalated
        .iter()
        .filter(|s| is_risk_key(&s.key))
        .count();
    let risk_down = signals
        .deescalated
        .iter()
        .filter(|s| is_risk_key(&s.key))
        .count();

    let direction = if risk_up > risk_down + 1 {
        "risk-off".to_string()
    } else if risk_down > risk_up + 1 {
        "risk-on".to_string()
    } else {
        "mixed".to_string()
    };

    let total_changes =
        signals.new.len() + signals.escalated.len() + signals.deescalated.len();

    let timestamp = current
        .get("chaos")
        .and_then(|c| c.get("timestamp"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let prev_timestamp = previous
        .get("chaos")
        .and_then(|c| c.get("timestamp"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());

    Some(DeltaResult {
        timestamp,
        previous: prev_timestamp,
        signals,
        summary: DeltaSummary {
            total_changes,
            critical_changes,
            direction,
        },
    })
}

// ─── Anomaly Detection ──────────────────────────────────────────────────────

/// An anomaly detected by comparing current values to historical data.
#[derive(Debug, Clone)]
pub struct AnomalySignal {
    pub key: String,
    pub current: f64,
    pub mean: f64,
    #[allow(dead_code)]
    pub std_dev: f64,
    pub z_score: f64,
    pub severity: String,
}

/// Compare current sweep values against a history of previous sweeps.
///
/// For each numeric and count metric, compute the mean and standard deviation
/// from `history`, then flag metrics where the current value exceeds 2 standard
/// deviations from the mean.
///
/// Returns anomalies sorted by absolute z-score descending.
pub fn detect_anomalies(current: &Value, history: &[Value]) -> Vec<AnomalySignal> {
    if history.is_empty() || current.is_null() {
        return Vec::new();
    }

    let mut anomalies = Vec::new();

    // Check numeric metrics
    for m in NUMERIC_METRICS {
        let curr_val = match (m.extract)(current) {
            Some(v) => v,
            None => continue,
        };

        let hist_vals: Vec<f64> = history
            .iter()
            .filter_map(|h| (m.extract)(h))
            .collect();

        if hist_vals.len() < 3 {
            continue; // need enough data points for meaningful stats
        }

        if let Some(signal) = compute_anomaly(m.key, curr_val, &hist_vals) {
            anomalies.push(signal);
        }
    }

    // Check count metrics
    for m in COUNT_METRICS {
        let curr_val = match (m.extract)(current) {
            Some(v) => v as f64,
            None => continue,
        };

        let hist_vals: Vec<f64> = history
            .iter()
            .filter_map(|h| (m.extract)(h).map(|v| v as f64))
            .collect();

        if hist_vals.len() < 3 {
            continue;
        }

        if let Some(signal) = compute_anomaly(m.key, curr_val, &hist_vals) {
            anomalies.push(signal);
        }
    }

    // Sort by absolute z-score descending
    anomalies.sort_by(|a, b| {
        b.z_score
            .abs()
            .partial_cmp(&a.z_score.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    anomalies
}

/// Compute anomaly signal for a single metric given its current value and
/// historical values. Returns `Some` if z-score exceeds 2.0.
fn compute_anomaly(key: &str, current: f64, hist_vals: &[f64]) -> Option<AnomalySignal> {
    let n = hist_vals.len() as f64;
    let mean = hist_vals.iter().sum::<f64>() / n;
    let variance = hist_vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    if std_dev < f64::EPSILON {
        return None; // all values identical, no variation
    }

    let z_score = (current - mean) / std_dev;

    if z_score.abs() >= 2.0 {
        let severity = if z_score.abs() >= 3.0 {
            "extreme".to_string()
        } else {
            "unusual".to_string()
        };

        Some(AnomalySignal {
            key: key.to_string(),
            current,
            mean: (mean * 100.0).round() / 100.0,
            std_dev: (std_dev * 100.0).round() / 100.0,
            z_score: (z_score * 100.0).round() / 100.0,
            severity,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_sweep(vix: f64, sources_ok: u64) -> Value {
        json!({
            "chaos": {
                "timestamp": "2025-01-01T00:00:00Z",
                "sourcesOk": sources_ok
            },
            "sources": {
                "FRED": {
                    "indicators": [
                        { "id": "VIXCLS", "value": vix }
                    ]
                }
            }
        })
    }

    #[test]
    fn test_null_inputs_return_none() {
        let data = json!({"test": true});
        assert!(compute_delta(&Value::Null, &data).is_none());
        assert!(compute_delta(&data, &Value::Null).is_none());
    }

    #[test]
    fn test_no_change_below_threshold() {
        let prev = make_sweep(20.0, 10);
        let curr = make_sweep(20.5, 10); // 2.5% change, below 5% threshold
        let result = compute_delta(&curr, &prev).unwrap();
        assert!(result.signals.escalated.is_empty());
        assert!(result.signals.deescalated.is_empty());
        assert!(result.signals.unchanged.contains(&"vix".to_string()));
    }

    #[test]
    fn test_numeric_escalation() {
        let prev = make_sweep(20.0, 10);
        let curr = make_sweep(22.0, 10); // 10% up, above 5% threshold
        let result = compute_delta(&curr, &prev).unwrap();
        assert_eq!(result.signals.escalated.len(), 1);
        let sig = &result.signals.escalated[0];
        assert_eq!(sig.key, "vix");
        assert_eq!(sig.direction, "up");
        assert_eq!(sig.severity, Severity::Moderate);
    }

    #[test]
    fn test_numeric_deescalation() {
        let prev = make_sweep(25.0, 10);
        let curr = make_sweep(22.0, 10); // -12%, above 5% threshold
        let result = compute_delta(&curr, &prev).unwrap();
        assert_eq!(result.signals.deescalated.len(), 1);
        let sig = &result.signals.deescalated[0];
        assert_eq!(sig.key, "vix");
        assert_eq!(sig.direction, "down");
    }

    #[test]
    fn test_severity_high() {
        let prev = make_sweep(20.0, 10);
        let curr = make_sweep(22.5, 10); // 12.5% — above 2x threshold (10%)
        let result = compute_delta(&curr, &prev).unwrap();
        let sig = &result.signals.escalated[0];
        assert_eq!(sig.severity, Severity::High);
    }

    #[test]
    fn test_severity_critical() {
        let prev = make_sweep(20.0, 10);
        let curr = make_sweep(24.0, 10); // 20% — above 3x threshold (15%)
        let result = compute_delta(&curr, &prev).unwrap();
        let sig = &result.signals.escalated[0];
        assert_eq!(sig.severity, Severity::Critical);
    }

    #[test]
    fn test_count_metric_change() {
        let prev = make_sweep(20.0, 10);
        let curr = make_sweep(20.0, 8); // sources_ok dropped by 2, threshold is 1
        let result = compute_delta(&curr, &prev).unwrap();
        let sig = result
            .signals
            .deescalated
            .iter()
            .find(|s| s.key == "sources_ok");
        assert!(sig.is_some());
    }

    #[test]
    fn test_risk_direction_risk_off() {
        // VIX escalated + conflict escalated = risk-off
        let prev = json!({
            "chaos": { "timestamp": "2025-01-01T00:00:00Z", "sourcesOk": 10 },
            "sources": {
                "FRED": { "indicators": [{ "id": "VIXCLS", "value": 20.0 }] },
                "ACLED": { "totalEvents": 100, "totalFatalities": 50 }
            }
        });
        let curr = json!({
            "chaos": { "timestamp": "2025-01-01T01:00:00Z", "sourcesOk": 10 },
            "sources": {
                "FRED": { "indicators": [{ "id": "VIXCLS", "value": 25.0 }] },
                "ACLED": { "totalEvents": 120, "totalFatalities": 70 }
            }
        });
        let result = compute_delta(&curr, &prev).unwrap();
        assert_eq!(result.summary.direction, "risk-off");
    }

    #[test]
    fn test_summary_total_changes() {
        let prev = make_sweep(20.0, 10);
        let curr = make_sweep(22.0, 10);
        let result = compute_delta(&curr, &prev).unwrap();
        assert_eq!(result.summary.total_changes, 1);
    }

    #[test]
    fn test_detect_anomalies_empty_history() {
        let current = make_sweep(20.0, 10);
        let anomalies = detect_anomalies(&current, &[]);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_detect_anomalies_null_current() {
        let history = vec![make_sweep(20.0, 10)];
        let anomalies = detect_anomalies(&Value::Null, &history);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_detect_anomalies_no_anomaly() {
        // All values close to 20.0 — current at 20.5 is within normal
        let history = vec![
            make_sweep(19.5, 10),
            make_sweep(20.0, 10),
            make_sweep(20.5, 10),
            make_sweep(20.0, 10),
        ];
        let current = make_sweep(20.3, 10);
        let anomalies = detect_anomalies(&current, &history);
        let vix_anomaly = anomalies.iter().find(|a| a.key == "vix");
        assert!(vix_anomaly.is_none());
    }

    #[test]
    fn test_detect_anomalies_extreme() {
        // History: VIX around 20, current: VIX at 40 — extreme anomaly
        let history = vec![
            make_sweep(20.0, 10),
            make_sweep(20.5, 10),
            make_sweep(19.5, 10),
            make_sweep(20.0, 10),
            make_sweep(20.2, 10),
        ];
        let current = make_sweep(40.0, 10);
        let anomalies = detect_anomalies(&current, &history);
        let vix_anomaly = anomalies.iter().find(|a| a.key == "vix");
        assert!(vix_anomaly.is_some());
        let sig = vix_anomaly.unwrap();
        assert_eq!(sig.severity, "extreme");
        assert!(sig.z_score > 3.0);
    }
}

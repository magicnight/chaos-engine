use serde_json::Value;

/// A cross-source correlation signal detected by rule-based analysis.
#[derive(Debug, Clone)]
pub struct CorrelationSignal {
    pub name: String,
    pub severity: String,
    pub sources: Vec<String>,
    pub description: String,
    pub indicators: Vec<String>,
}

/// Analyze sweep data for cross-source correlation signals (6 rules).
pub fn analyze_correlations(sweep_data: &Value) -> Vec<CorrelationSignal> {
    let sources = &sweep_data["sources"];
    let mut signals = Vec::new();

    // Rule 1: NATURAL_DISASTER_CASCADE
    // Significant earthquake (M6+) + (tsunami warning OR GDACS Red alert)
    if let Some(sig) = check_natural_disaster_cascade(sources) {
        signals.push(sig);
    }

    // Rule 2: GEOPOLITICAL_RISK_CONVERGENCE
    // VIX > 25 + conflict events > 50 + WTI > $85
    if let Some(sig) = check_geopolitical_risk(sources) {
        signals.push(sig);
    }

    // Rule 3: CYBER_THREAT_CONVERGENCE
    // Critical CVEs >= 3 + ISC threat level elevated
    if let Some(sig) = check_cyber_threat(sources) {
        signals.push(sig);
    }

    // Rule 4: INFRASTRUCTURE_STRESS
    // Space weather Kp >= 2 + (active fires > 1000 OR extreme weather events > 5)
    if let Some(sig) = check_infrastructure_stress(sources) {
        signals.push(sig);
    }

    // Rule 5: MARKET_PANIC
    // VIX > 30 + Treasury yields dropping (flight to safety)
    if let Some(sig) = check_market_panic(sources) {
        signals.push(sig);
    }

    // Rule 6: HUMANITARIAN_CRISIS
    // WHO alerts >= 3 + conflict fatalities > 50
    if let Some(sig) = check_humanitarian_crisis(sources) {
        signals.push(sig);
    }

    signals
}

// --- Helper extractors ---

fn extract_f64(v: &Value, path: &[&str]) -> Option<f64> {
    let mut current = v;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_f64()
}

fn extract_u64(v: &Value, path: &[&str]) -> Option<u64> {
    let mut current = v;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_u64()
}

fn extract_str<'a>(v: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = v;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn extract_array_len(v: &Value, path: &[&str]) -> Option<usize> {
    let mut current = v;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_array().map(|a| a.len())
}

/// Count earthquakes with magnitude >= threshold.
fn count_significant_quakes(sources: &Value, min_mag: f64) -> usize {
    sources
        .get("USGS")
        .and_then(|v| v.get("quakes"))
        .and_then(|v| v.as_array())
        .map(|quakes| {
            quakes
                .iter()
                .filter(|q| {
                    q.get("mag")
                        .and_then(|m| m.as_f64())
                        .unwrap_or(0.0)
                        >= min_mag
                })
                .count()
        })
        .unwrap_or(0)
}

// --- Rule implementations ---

fn check_natural_disaster_cascade(sources: &Value) -> Option<CorrelationSignal> {
    let sig_quakes = count_significant_quakes(sources, 6.0);
    if sig_quakes == 0 {
        return None;
    }

    // Check for tsunami or GDACS red alert
    let has_tsunami = sources
        .get("NOAA")
        .and_then(|v| v.get("topAlerts"))
        .and_then(|v| v.as_array())
        .map(|alerts| {
            alerts.iter().any(|a| {
                a.get("event")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .contains("tsunami")
            })
        })
        .unwrap_or(false);

    let has_gdacs_red = sources
        .get("GDACS")
        .and_then(|v| v.get("alerts"))
        .and_then(|v| v.as_array())
        .map(|alerts| {
            alerts.iter().any(|a| {
                a.get("alertLevel")
                    .and_then(|l| l.as_str())
                    .unwrap_or("")
                    .eq_ignore_ascii_case("red")
            })
        })
        .unwrap_or(false);

    if !has_tsunami && !has_gdacs_red {
        return None;
    }

    let mut indicators = vec![format!("{} earthquake(s) M6+", sig_quakes)];
    let mut src = vec!["USGS".to_string()];

    if has_tsunami {
        indicators.push("Tsunami warning active".to_string());
        src.push("NOAA".to_string());
    }
    if has_gdacs_red {
        indicators.push("GDACS Red alert".to_string());
        src.push("GDACS".to_string());
    }

    Some(CorrelationSignal {
        name: "NATURAL_DISASTER_CASCADE".to_string(),
        severity: "critical".to_string(),
        sources: src,
        description: "Major earthquake with cascading natural disaster indicators".to_string(),
        indicators,
    })
}

fn check_geopolitical_risk(sources: &Value) -> Option<CorrelationSignal> {
    // VIX > 25
    let vix = extract_f64(sources, &["YFinance", "quotes", "^VIX", "price"]).unwrap_or(0.0);
    if vix <= 25.0 {
        return None;
    }

    // Conflict events > 50
    let conflict_events =
        extract_u64(sources, &["ACLED", "totalEvents"]).unwrap_or(0);
    if conflict_events <= 50 {
        return None;
    }

    // WTI > $85
    let wti = extract_f64(sources, &["EIA", "data", "wti", "value"]).unwrap_or(0.0);
    if wti <= 85.0 {
        return None;
    }

    Some(CorrelationSignal {
        name: "GEOPOLITICAL_RISK_CONVERGENCE".to_string(),
        severity: "high".to_string(),
        sources: vec![
            "YFinance".to_string(),
            "ACLED".to_string(),
            "EIA".to_string(),
        ],
        description: "Elevated fear index, active conflicts, and rising oil prices converging"
            .to_string(),
        indicators: vec![
            format!("VIX: {:.1}", vix),
            format!("Conflict events: {}", conflict_events),
            format!("WTI: ${:.2}", wti),
        ],
    })
}

fn check_cyber_threat(sources: &Value) -> Option<CorrelationSignal> {
    // Critical CVEs >= 3
    let critical_cves = sources
        .get("NVD-CVE")
        .and_then(|v| v.get("topVulnerabilities"))
        .and_then(|v| v.as_array())
        .map(|cves| {
            cves.iter()
                .filter(|c| {
                    c.get("severity")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .eq_ignore_ascii_case("critical")
                })
                .count()
        })
        .unwrap_or(0);

    if critical_cves < 3 {
        return None;
    }

    // ISC threat level elevated
    let isc_level = extract_str(sources, &["ISC-SANS", "infocon", "status"]).unwrap_or("green");
    let isc_elevated = matches!(
        isc_level.to_lowercase().as_str(),
        "yellow" | "orange" | "red"
    );

    if !isc_elevated {
        return None;
    }

    Some(CorrelationSignal {
        name: "CYBER_THREAT_CONVERGENCE".to_string(),
        severity: "high".to_string(),
        sources: vec!["NVD-CVE".to_string(), "ISC-SANS".to_string()],
        description: "Multiple critical vulnerabilities with elevated internet threat level"
            .to_string(),
        indicators: vec![
            format!("{} critical CVEs", critical_cves),
            format!("ISC threat level: {}", isc_level),
        ],
    })
}

fn check_infrastructure_stress(sources: &Value) -> Option<CorrelationSignal> {
    // Space weather G-scale (geomagnetic storm level)
    // SWPC output: current.G.scale e.g. "G2", "none"
    // Parse numeric level from scale string like "G2" -> 2.0
    let g_scale_str = extract_str(sources, &["SWPC", "current", "G", "scale"]).unwrap_or("none");
    let kp = if g_scale_str.starts_with('G') || g_scale_str.starts_with('g') {
        g_scale_str[1..].parse::<f64>().unwrap_or(0.0)
    } else {
        0.0
    };

    if kp < 2.0 {
        return None;
    }

    // Active fires > 1000 OR extreme weather events > 5
    // FIRMS output: hotspots array, each with totalDetections
    let fire_count = sources
        .get("FIRMS")
        .and_then(|v| v.get("hotspots"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|h| h.get("totalDetections").and_then(|t| t.as_u64()))
                .sum::<u64>()
        })
        .unwrap_or(0);
    let weather_events = extract_array_len(sources, &["NOAA", "topAlerts"]).unwrap_or(0);

    let fires_high = fire_count > 1000;
    let weather_high = weather_events > 5;

    if !fires_high && !weather_high {
        return None;
    }

    let mut indicators = vec![format!("Space weather G-scale: {}", g_scale_str)];
    let mut src = vec!["SWPC".to_string()];

    if fires_high {
        indicators.push(format!("{} active fire hotspots", fire_count));
        src.push("FIRMS".to_string());
    }
    if weather_high {
        indicators.push(format!("{} extreme weather alerts", weather_events));
        src.push("NOAA".to_string());
    }

    Some(CorrelationSignal {
        name: "INFRASTRUCTURE_STRESS".to_string(),
        severity: "high".to_string(),
        sources: src,
        description: "Space weather and environmental stress threatening infrastructure"
            .to_string(),
        indicators,
    })
}

fn check_market_panic(sources: &Value) -> Option<CorrelationSignal> {
    // VIX > 30
    let vix = extract_f64(sources, &["YFinance", "quotes", "^VIX", "price"]).unwrap_or(0.0);
    if vix <= 30.0 {
        return None;
    }

    // Treasury yields dropping (flight to safety) — compare 10Y yield < 3.5 as proxy
    // US Treasury output: interestRates array with { security, rate, date }
    let yield_10y = sources
        .get("US Treasury")
        .and_then(|v| v.get("interestRates"))
        .and_then(|v| v.as_array())
        .and_then(|rates| {
            rates.iter().find_map(|r| {
                let sec = r.get("security").and_then(|s| s.as_str()).unwrap_or("");
                if sec.contains("10-Year") || sec.contains("10 Year") {
                    r.get("rate")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| r.get("rate").and_then(|v| v.as_f64()))
                } else {
                    None
                }
            })
        })
        .unwrap_or(f64::MAX);

    // Flight to safety: yields drop below 3.5% when investors flee to bonds
    if yield_10y >= 3.5 {
        return None;
    }

    Some(CorrelationSignal {
        name: "MARKET_PANIC".to_string(),
        severity: "critical".to_string(),
        sources: vec!["YFinance".to_string(), "US Treasury".to_string()],
        description: "Extreme market fear with flight-to-safety bond buying".to_string(),
        indicators: vec![
            format!("VIX: {:.1}", vix),
            format!("10Y Treasury yield: {:.2}%", yield_10y),
        ],
    })
}

fn check_humanitarian_crisis(sources: &Value) -> Option<CorrelationSignal> {
    // WHO alerts >= 3
    let who_alerts = extract_array_len(sources, &["WHO", "alerts"]).unwrap_or(0);

    if who_alerts < 3 {
        return None;
    }

    // Conflict fatalities > 50
    let fatalities = extract_u64(sources, &["ACLED", "totalFatalities"]).unwrap_or(0);
    if fatalities <= 50 {
        return None;
    }

    Some(CorrelationSignal {
        name: "HUMANITARIAN_CRISIS".to_string(),
        severity: "critical".to_string(),
        sources: vec!["WHO".to_string(), "ACLED".to_string()],
        description: "Multiple health emergencies coinciding with significant conflict casualties"
            .to_string(),
        indicators: vec![
            format!("{} WHO alerts", who_alerts),
            format!("{} conflict fatalities", fatalities),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_signals_on_empty_data() {
        let data = json!({ "sources": {} });
        let signals = analyze_correlations(&data);
        assert!(signals.is_empty());
    }

    #[test]
    fn test_natural_disaster_cascade() {
        let data = json!({
            "sources": {
                "USGS": {
                    "quakes": [
                        { "mag": 7.2 },
                        { "mag": 3.1 }
                    ]
                },
                "GDACS": { "alerts": [{ "alertLevel": "Red" }] }
            }
        });
        let signals = analyze_correlations(&data);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].name, "NATURAL_DISASTER_CASCADE");
        assert_eq!(signals[0].severity, "critical");
    }

    #[test]
    fn test_geopolitical_risk_convergence() {
        let data = json!({
            "sources": {
                "YFinance": { "quotes": { "^VIX": { "price": 28.5 } } },
                "EIA": { "data": { "wti": { "value": 92.0 } } },
                "ACLED": { "totalEvents": 120 }
            }
        });
        let signals = analyze_correlations(&data);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].name, "GEOPOLITICAL_RISK_CONVERGENCE");
    }

    #[test]
    fn test_cyber_threat_convergence() {
        let data = json!({
            "sources": {
                "NVD-CVE": {
                    "topVulnerabilities": [
                        { "severity": "CRITICAL" },
                        { "severity": "CRITICAL" },
                        { "severity": "CRITICAL" },
                        { "severity": "HIGH" }
                    ]
                },
                "ISC-SANS": { "infocon": { "status": "yellow" } }
            }
        });
        let signals = analyze_correlations(&data);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].name, "CYBER_THREAT_CONVERGENCE");
    }

    #[test]
    fn test_no_signal_below_thresholds() {
        let data = json!({
            "sources": {
                "YFinance": { "quotes": { "^VIX": { "price": 15.0 } } },
                "EIA": { "data": { "wti": { "value": 70.0 } } },
                "ACLED": { "totalEvents": 10 }
            }
        });
        let signals = analyze_correlations(&data);
        assert!(signals.is_empty());
    }

    #[test]
    fn test_market_panic() {
        let data = json!({
            "sources": {
                "YFinance": { "quotes": { "^VIX": { "price": 35.0 } } },
                "US Treasury": { "interestRates": [{ "security": "Treasury 10-Year", "rate": "2.80" }] }
            }
        });
        let signals = analyze_correlations(&data);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].name, "MARKET_PANIC");
        assert_eq!(signals[0].severity, "critical");
    }

    #[test]
    fn test_humanitarian_crisis() {
        let data = json!({
            "sources": {
                "WHO": { "alerts": [{}, {}, {}] },
                "ACLED": { "totalFatalities": 100 }
            }
        });
        let signals = analyze_correlations(&data);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].name, "HUMANITARIAN_CRISIS");
    }
}

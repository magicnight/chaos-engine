use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const GSCPI_CSV_URL: &str =
    "https://www.newyorkfed.org/medialibrary/research/interactives/data/gscpi/gscpi_interactive_data.csv";

pub struct Gscpi {
    client: HttpClient,
}

impl Gscpi {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

/// Parse NY Fed date format "31-Jan-2026" -> "2026-01"
fn parse_nyfed_date(s: &str) -> Option<String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let month = match parts[1] {
        "Jan" => "01", "Feb" => "02", "Mar" => "03", "Apr" => "04",
        "May" => "05", "Jun" => "06", "Jul" => "07", "Aug" => "08",
        "Sep" => "09", "Oct" => "10", "Nov" => "11", "Dec" => "12",
        _ => return None,
    };
    Some(format!("{}-{}", parts[2], month))
}

/// Detect trend from values sorted newest-first.
fn detect_trend(values: &[(String, f64)]) -> &'static str {
    if values.len() < 3 {
        return "insufficient data";
    }
    let mut rising = 0u32;
    let mut falling = 0u32;
    for i in 0..2 {
        if values[i].1 > values[i + 1].1 {
            rising += 1;
        } else if values[i].1 < values[i + 1].1 {
            falling += 1;
        }
    }
    if rising > falling {
        "rising"
    } else if falling > rising {
        "falling"
    } else {
        "stable"
    }
}

#[async_trait]
impl IntelSource for Gscpi {
    fn name(&self) -> &str {
        "NY Fed GSCPI"
    }

    fn description(&self) -> &str {
        "Global Supply Chain Pressure Index"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let text = self.client.fetch_text(GSCPI_CSV_URL).await?;

        let lines: Vec<&str> = text
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with(','))
            .collect();

        if lines.len() < 2 {
            return Ok(json!({
                "source": "NY Fed GSCPI",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "Could not parse GSCPI CSV data",
            }));
        }

        // Parse rows: each row has date in col 0, then multiple vintage columns.
        // We want the last non-empty, non-#N/A numeric value per row.
        let mut history: Vec<(String, f64)> = Vec::new();

        for line in lines.iter().skip(1) {
            let cols: Vec<&str> = line.split(',').collect();
            let date_str = cols.first().map(|s| s.trim()).unwrap_or("");
            if date_str.is_empty() {
                continue;
            }

            // Find last valid numeric value
            let mut value: Option<f64> = None;
            for j in (1..cols.len()).rev() {
                let v = cols[j].trim();
                if !v.is_empty() && v != "#N/A" {
                    if let Ok(num) = v.parse::<f64>() {
                        value = Some(num);
                        break;
                    }
                }
            }

            if let (Some(date), Some(val)) = (parse_nyfed_date(date_str), value) {
                history.push((date, val));
            }
        }

        // Sort newest first
        history.sort_by(|a, b| b.0.cmp(&a.0));
        history.truncate(12);

        let trend = detect_trend(&history);
        let mut signals = Vec::new();

        let latest = history.first().cloned();

        if let Some((_, val)) = &latest {
            let val = *val;
            if val > 2.0 {
                signals.push(format!(
                    "GSCPI extremely elevated at {:.2} — severe supply chain stress",
                    val
                ));
            } else if val > 1.0 {
                signals.push(format!(
                    "GSCPI elevated at {:.2} — above-normal supply chain pressure",
                    val
                ));
            } else if val < -1.0 {
                signals.push(format!(
                    "GSCPI at {:.2} — unusually loose supply chains",
                    val
                ));
            }

            if trend == "rising" && val > 0.0 {
                signals.push("Supply chain pressure trending higher".to_string());
            }
            if trend == "falling" && val > 1.0 {
                signals.push("Supply chain pressure elevated but improving".to_string());
            }
        }

        // Month-over-month change
        if history.len() >= 2 {
            let mom = history[0].1 - history[1].1;
            if mom.abs() > 0.5 {
                let dir = if mom > 0.0 { "surged" } else { "dropped" };
                signals.push(format!(
                    "GSCPI {} {:.2} points month-over-month",
                    dir,
                    mom.abs()
                ));
            }
        }

        let history_json: Vec<Value> = history
            .iter()
            .map(|(date, val)| json!({"date": date, "value": val}))
            .collect();

        let latest_json = latest.map(|(date, val)| {
            let interpretation = if val > 1.0 {
                "elevated"
            } else if val > 0.0 {
                "above average"
            } else if val > -1.0 {
                "below average"
            } else {
                "unusually loose"
            };
            json!({
                "value": val,
                "date": date,
                "interpretation": interpretation,
            })
        });

        Ok(json!({
            "source": "NY Fed GSCPI",
            "timestamp": Utc::now().to_rfc3339(),
            "latest": latest_json,
            "trend": trend,
            "history": history_json,
            "signals": signals,
        }))
    }
}

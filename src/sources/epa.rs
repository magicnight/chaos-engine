use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

// EPA Envirofacts RadNet API — the RADNET_ANALYTICAL_RESULTS table is no
// longer available (404). Use RADNET_GROSS which contains gross alpha/beta
// air-filter results and is still served by the Envirofacts REST API.
//
// If this endpoint also stops working, alternatives:
//   - Safecast (already integrated) as primary radiation source.
//   - EPA RadNet CSV downloads: https://www.epa.gov/radnet/radnet-csv-file-downloads
const RADNET_URL: &str =
    "https://enviro.epa.gov/enviro/efservice/RADNET_GROSS/ROWS/0:100/JSON";

/// Normal background radiation thresholds (pCi/m3).
const THRESHOLD_GROSS_BETA: f64 = 5.0;
const THRESHOLD_GROSS_ALPHA: f64 = 0.15;
const THRESHOLD_IODINE_131: f64 = 0.1;
const THRESHOLD_CESIUM_137: f64 = 0.1;

pub struct Epa {
    client: HttpClient,
}

impl Epa {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn check_threshold(analyte: &str, result: f64) -> Option<(&'static str, f64)> {
    match analyte.to_uppercase().as_str() {
        "GROSS BETA" if result > THRESHOLD_GROSS_BETA => {
            Some(("GROSS BETA", THRESHOLD_GROSS_BETA))
        }
        "GROSS ALPHA" if result > THRESHOLD_GROSS_ALPHA => {
            Some(("GROSS ALPHA", THRESHOLD_GROSS_ALPHA))
        }
        "IODINE-131" if result > THRESHOLD_IODINE_131 => {
            Some(("IODINE-131", THRESHOLD_IODINE_131))
        }
        "CESIUM-137" if result > THRESHOLD_CESIUM_137 => {
            Some(("CESIUM-137", THRESHOLD_CESIUM_137))
        }
        _ => None,
    }
}

#[async_trait]
impl IntelSource for Epa {
    fn name(&self) -> &str {
        "EPA-RadNet"
    }

    fn description(&self) -> &str {
        "EPA RadNet radiation monitoring network"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let data = match self.client.fetch_json(RADNET_URL).await {
            Ok(d) => d,
            Err(e) => {
                // The EPA Envirofacts RadNet API tables change over time.
                // Return a structured error so the engine keeps running;
                // Safecast serves as the primary radiation source.
                return Ok(json!({
                    "source": "EPA-RadNet",
                    "timestamp": Utc::now().to_rfc3339(),
                    "error": format!("EPA RadNet API unavailable: {}", e),
                    "note": "EPA RadNet endpoint may have changed. Safecast serves as the primary radiation source. For US government radiation data, see https://www.epa.gov/radnet/radnet-csv-file-downloads",
                    "totalReadings": 0,
                    "readings": [],
                    "signals": ["EPA RadNet API currently unavailable"],
                }));
            }
        };

        let records = data.as_array().cloned().unwrap_or_default();

        let mut readings = Vec::new();
        let mut signals = Vec::new();
        let mut by_state: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        for record in records.iter().take(100) {
            let location = record
                .get("ANA_CITY")
                .or_else(|| record.get("LOCATION"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let state = record
                .get("ANA_STATE")
                .or_else(|| record.get("STATE"))
                .and_then(|v| v.as_str())
                .unwrap_or("UNK");
            let analyte = record
                .get("ANA_TYPE")
                .or_else(|| record.get("ANALYTE_NAME"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let result_val = record
                .get("ANA_RESULT")
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| v.as_f64()))
                .unwrap_or(0.0);
            let unit = record
                .get("RESULT_UNIT")
                .or_else(|| record.get("ANA_UNIT"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let collect_date = record
                .get("COLLECT_DATE")
                .or_else(|| record.get("SAMPLE_DATE"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            *by_state.entry(state.to_string()).or_insert(0) += 1;

            if let Some((name, threshold)) = check_threshold(analyte, result_val) {
                let ratio = result_val / threshold;
                signals.push(format!(
                    "ELEVATED {} at {}, {}: {:.3} {} ({:.1}x threshold) [{}]",
                    name, location, state, result_val, unit, ratio, collect_date
                ));
            }

            if readings.len() < 50 {
                readings.push(json!({
                    "location": location,
                    "state": state,
                    "analyte": analyte,
                    "result": result_val,
                    "unit": unit,
                    "collectDate": collect_date,
                }));
            }
        }

        if signals.is_empty() {
            signals.push("All EPA RadNet readings within normal background levels".to_string());
        }

        Ok(json!({
            "source": "EPA-RadNet",
            "timestamp": Utc::now().to_rfc3339(),
            "totalReadings": records.len(),
            "readings": readings,
            "stateSummary": by_state,
            "signals": signals,
            "note": "RadNet data may lag by hours to days",
        }))
    }
}

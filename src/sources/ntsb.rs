// NTSB — National Transportation Safety Board aviation incidents
// API may return empty responses; gracefully handled
// No auth required

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const API_URL: &str =
    "https://data.ntsb.gov/carol-repgen/api/Aviation/GetEvents?ResultsPerPage=10&OrderBy=EventDate%20desc";

pub struct Ntsb {
    client: HttpClient,
}

impl Ntsb {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for Ntsb {
    fn name(&self) -> &str { "NTSB" }
    fn description(&self) -> &str { "NTSB aviation safety incidents" }
    fn tier(&self) -> u8 { 3 }

    async fn sweep(&self) -> Result<Value> {
        // NTSB API requires Accept header and may return empty body
        let resp = self.client.raw_client()
            .get(API_URL)
            .header("Accept", "application/json")
            .header("User-Agent", "CHAOS/0.1.0 IntelligenceEngine")
            .send()
            .await;

        let data: Value = match resp {
            Ok(r) if r.status().is_success() => {
                let text = r.text().await.unwrap_or_default();
                if text.trim().is_empty() {
                    return Ok(json!({
                        "source": self.name(),
                        "timestamp": Utc::now().to_rfc3339(),
                        "error": "NTSB API returned empty response",
                        "hint": "The NTSB CAROL API may be intermittently unavailable"
                    }));
                }
                serde_json::from_str(&text).unwrap_or(json!([]))
            }
            Ok(r) => return Ok(json!({
                "source": self.name(),
                "timestamp": Utc::now().to_rfc3339(),
                "error": format!("NTSB API returned HTTP {}", r.status()),
            })),
            Err(e) => return Ok(json!({
                "source": self.name(),
                "timestamp": Utc::now().to_rfc3339(),
                "error": format!("NTSB API request failed: {}", e),
            })),
        };

        let events = data.as_array().cloned().unwrap_or_default();
        let mut incidents = Vec::new();
        let mut fatal_count = 0u32;

        for event in events.iter().take(10) {
            let get_str = |keys: &[&str]| -> String {
                for k in keys {
                    if let Some(s) = event.get(k).and_then(|v| v.as_str()) {
                        return s.to_string();
                    }
                }
                String::new()
            };

            let total_fatal = event.get("TotalFatalInjuries")
                .or_else(|| event.get("totalFatalInjuries"))
                .and_then(|v| v.as_str().and_then(|s| s.parse::<u32>().ok()).or_else(|| v.as_u64().map(|n| n as u32)))
                .unwrap_or(0);
            fatal_count += total_fatal;

            let make = get_str(&["Make", "make"]);
            let model = get_str(&["Model", "model"]);
            incidents.push(json!({
                "date": get_str(&["EventDate", "eventDate"]),
                "location": format!("{}, {}", get_str(&["City", "city"]), get_str(&["State", "state"])).trim_matches(',').trim().to_string(),
                "country": get_str(&["Country", "country"]),
                "severity": get_str(&["InjurySeverity", "injurySeverity"]),
                "aircraft": format!("{} {}", make, model).trim().to_string(),
                "fatalInjuries": total_fatal,
            }));
        }

        Ok(json!({
            "source": self.name(),
            "timestamp": Utc::now().to_rfc3339(),
            "totalIncidents": incidents.len(),
            "fatalInjuries": fatal_count,
            "incidents": incidents,
        }))
    }
}

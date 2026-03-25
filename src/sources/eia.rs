use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BASE: &str = "https://api.eia.gov/v2";

struct SeriesDef {
    key: &'static str,
    label: &'static str,
    path: &'static str,
    frequency: &'static str,
    facet_series: &'static str,
}

const SERIES: &[SeriesDef] = &[
    SeriesDef {
        key: "wti",
        label: "WTI Crude Oil ($/bbl)",
        path: "/petroleum/pri/spt/data/",
        frequency: "daily",
        facet_series: "RWTC",
    },
    SeriesDef {
        key: "brent",
        label: "Brent Crude Oil ($/bbl)",
        path: "/petroleum/pri/spt/data/",
        frequency: "daily",
        facet_series: "RBRTE",
    },
    SeriesDef {
        key: "natgas",
        label: "Henry Hub Natural Gas ($/MMBtu)",
        path: "/natural-gas/pri/fut/data/",
        frequency: "daily",
        facet_series: "RNGWHHD",
    },
    SeriesDef {
        key: "crudeStocks",
        label: "US Crude Oil Inventories (thousand barrels)",
        path: "/petroleum/stoc/wstk/data/",
        frequency: "weekly",
        facet_series: "WCESTUS1",
    },
];

pub struct Eia {
    client: HttpClient,
}

impl Eia {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    async fn fetch_series(&self, def: &SeriesDef, api_key: &str) -> Option<Value> {
        let url = format!(
            "{}{}?api_key={}&frequency={}&data[0]=value&sort[0][column]=period&sort[0][direction]=desc&length=10&facets[series][]={}",
            BASE, def.path, api_key, def.frequency, def.facet_series
        );
        let data = self.client.fetch_json(&url).await.ok()?;
        let records = data.pointer("/response/data")?.as_array()?;
        if records.is_empty() {
            return None;
        }

        let latest = &records[0];
        let value = latest.get("value")
            .and_then(|v| v.as_str().or_else(|| v.as_f64().map(|_| "")))
            .and_then(|s| if s.is_empty() {
                latest.get("value").and_then(|v| v.as_f64())
            } else {
                s.parse::<f64>().ok()
            })?;
        let period = latest.get("period").and_then(|v| v.as_str()).unwrap_or("");

        let recent: Vec<Value> = records.iter().take(5).filter_map(|r| {
            let v = r.get("value")
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| v.as_f64()))?;
            let p = r.get("period").and_then(|v| v.as_str()).unwrap_or("");
            Some(json!({"value": v, "period": p}))
        }).collect();

        Some(json!({
            "key": def.key,
            "label": def.label,
            "value": value,
            "period": period,
            "recent": recent,
        }))
    }
}

#[async_trait]
impl IntelSource for Eia {
    fn name(&self) -> &str {
        "EIA"
    }

    fn description(&self) -> &str {
        "US Energy Information Administration"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let api_key = std::env::var("EIA_API_KEY").unwrap_or_default();

        if api_key.is_empty() {
            return Ok(json!({
                "source": "EIA",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "No EIA API key. Register free at https://www.eia.gov/opendata/register.php",
                "hint": "Set EIA_API_KEY environment variable (DEMO_KEY works)",
            }));
        }

        let futures: Vec<_> = SERIES
            .iter()
            .map(|def| self.fetch_series(def, &api_key))
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut series_data = serde_json::Map::new();
        let mut signals = Vec::new();
        let mut wti_val: Option<f64> = None;
        let mut brent_val: Option<f64> = None;

        for result in results.into_iter().flatten() {
            let key = result.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let value = result.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);

            match key {
                "wti" => {
                    wti_val = Some(value);
                    if value > 100.0 {
                        signals.push(format!("WTI crude above $100 at ${}/bbl", value));
                    } else if value < 50.0 {
                        signals.push(format!("WTI crude below $50 at ${}/bbl — supply glut or demand destruction", value));
                    }
                }
                "brent" => { brent_val = Some(value); }
                "natgas" => {
                    if value > 6.0 {
                        signals.push(format!("Natural gas elevated at ${}/MMBtu", value));
                    }
                }
                "crudeStocks" => {
                    let recent = result.get("recent").and_then(|v| v.as_array());
                    if let Some(arr) = recent {
                        if arr.len() >= 2 {
                            let v0 = arr[0].get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let v1 = arr[1].get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = v0 - v1;
                            if change.abs() > 5000.0 {
                                let direction = if change > 0.0 { "build" } else { "draw" };
                                signals.push(format!(
                                    "Large crude inventory {}: {:.1}M barrels",
                                    direction, change / 1000.0
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }

            series_data.insert(key.to_string(), result);
        }

        if let (Some(wti), Some(brent)) = (wti_val, brent_val) {
            let spread = brent - wti;
            if spread > 10.0 {
                signals.push(format!(
                    "Brent-WTI spread wide at ${:.2} — supply/logistics divergence",
                    spread
                ));
            }
            series_data.insert("brentWtiSpread".to_string(), json!(spread));
        }

        Ok(json!({
            "source": "EIA",
            "timestamp": Utc::now().to_rfc3339(),
            "data": series_data,
            "signals": signals,
        }))
    }
}

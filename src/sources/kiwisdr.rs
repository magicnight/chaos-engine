use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const RECEIVERBOOK_URL: &str = "https://www.receiverbook.de/map?type=kiwisdr";

pub struct KiwiSdr {
    client: HttpClient,
}

impl KiwiSdr {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn get_continent(lat: f64, lon: f64) -> &'static str {
    if lat >= 15.0 && lat <= 72.0 && lon >= -170.0 && lon <= -50.0 {
        "North America"
    } else if lat >= -60.0 && lat < 15.0 && lon >= -90.0 && lon <= -30.0 {
        "South America"
    } else if lat >= 35.0 && lat <= 72.0 && lon >= -25.0 && lon <= 45.0 {
        "Europe"
    } else if lat >= -35.0 && lat <= 37.0 && lon >= -25.0 && lon <= 55.0 {
        "Africa"
    } else if lat >= 0.0 && lat <= 72.0 && lon >= 45.0 && lon <= 180.0 {
        "Asia"
    } else if lat >= -50.0 && lat <= 0.0 && lon >= 95.0 && lon <= 180.0 {
        "Oceania"
    } else {
        "Other"
    }
}

#[async_trait]
impl IntelSource for KiwiSdr {
    fn name(&self) -> &str {
        "KiwiSDR"
    }

    fn description(&self) -> &str {
        "KiwiSDR global HF radio receiver network"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let html = self.client.fetch_text(RECEIVERBOOK_URL).await?;

        // Extract embedded JS: var receivers = [...];
        let receivers_json = html
            .find("var receivers = ")
            .and_then(|start| {
                let json_start = start + "var receivers = ".len();
                let rest = &html[json_start..];
                rest.find("];").map(|end| &rest[..end + 1])
            });

        let sites: Vec<Value> = match receivers_json {
            Some(js) => serde_json::from_str(js).unwrap_or_default(),
            None => {
                return Ok(json!({
                    "source": "KiwiSDR",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "error",
                    "message": "Could not parse receiver data from page",
                }));
            }
        };

        // Flatten sites into individual receivers
        let mut total = 0u32;
        let mut by_continent: std::collections::HashMap<&str, u32> =
            std::collections::HashMap::new();
        let mut by_country: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut sample_receivers = Vec::new();

        for site in &sites {
            let coords = site
                .get("location")
                .and_then(|l| l.get("coordinates"))
                .and_then(|c| c.as_array());

            let (lon, lat) = match coords {
                Some(c) => {
                    let lo = c.first().and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
                    let la = c.get(1).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
                    (lo, la)
                }
                None => (f64::NAN, f64::NAN),
            };

            let label = site
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let country = label.split(',').last().map(|s| s.trim()).unwrap_or("");

            let receivers = site
                .get("receivers")
                .and_then(|v| v.as_array())
                .map(|a| a.len() as u32)
                .unwrap_or(1);

            total += receivers;

            if !lat.is_nan() && !lon.is_nan() {
                let continent = get_continent(lat, lon);
                *by_continent.entry(continent).or_insert(0) += receivers;
            }

            if !country.is_empty() {
                *by_country.entry(country.to_string()).or_insert(0) += receivers;
            }

            if sample_receivers.len() < 15 {
                sample_receivers.push(json!({
                    "name": label,
                    "lat": if lat.is_nan() { Value::Null } else { json!(lat) },
                    "lon": if lon.is_nan() { Value::Null } else { json!(lon) },
                    "country": country,
                }));
            }
        }

        // Top countries
        let mut top_countries: Vec<_> = by_country.into_iter().collect();
        top_countries.sort_by(|a, b| b.1.cmp(&a.1));
        let top_countries: Vec<Value> = top_countries
            .into_iter()
            .take(15)
            .map(|(country, count)| json!({"country": country, "count": count}))
            .collect();

        Ok(json!({
            "source": "KiwiSDR",
            "timestamp": Utc::now().to_rfc3339(),
            "network": {
                "totalSites": sites.len(),
                "totalReceivers": total,
            },
            "geographic": {
                "byContinent": by_continent,
                "topCountries": top_countries,
            },
            "sampleReceivers": sample_receivers,
        }))
    }
}

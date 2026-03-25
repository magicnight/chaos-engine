use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const EUR_USD_URL: &str =
    "https://data-api.ecb.europa.eu/service/data/EXR/D.USD.EUR.SP00.A?lastNObservations=5&format=jsondata";
const EURIBOR_URL: &str =
    "https://data-api.ecb.europa.eu/service/data/FM/D.U2.EUR.RT.MM.EURIBOR3MD_.HSTA?lastNObservations=5&format=jsondata";

pub struct Ecb {
    client: HttpClient,
}

impl Ecb {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

/// Extract the most recent observation value from ECB SDMX-JSON response.
fn extract_latest_obs(data: &Value) -> Option<(f64, String)> {
    let datasets = data.get("dataSets")?.as_array()?;
    let dataset = datasets.first()?;
    let series = dataset.get("series")?.as_object()?;
    // Take the first (and usually only) series
    let first_series = series.values().next()?;
    let observations = first_series.get("observations")?.as_object()?;

    // Keys are string indices like "0", "1", etc. Find the highest index.
    let max_key = observations
        .keys()
        .filter_map(|k| k.parse::<u64>().ok())
        .max()?;

    let obs = observations.get(&max_key.to_string())?.as_array()?;
    let value = obs.first()?.as_f64()?;

    // Try to get the date from structure dimensions
    let structures = data.get("structure")?;
    let dims = structures.get("dimensions")?.get("observation")?.as_array()?;
    let time_dim = dims.first()?;
    let values_arr = time_dim.get("values")?.as_array()?;
    let date = values_arr
        .get(max_key as usize)
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some((value, date))
}

#[async_trait]
impl IntelSource for Ecb {
    fn name(&self) -> &str {
        "ECB"
    }

    fn description(&self) -> &str {
        "European Central Bank exchange rates and EURIBOR"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        use futures::future::join_all;

        let client = self.client.clone();
        let fx_future = {
            let c = client.clone();
            async move { c.fetch_json(EUR_USD_URL).await }
        };
        let euribor_future = {
            let c = client;
            async move { c.fetch_json(EURIBOR_URL).await }
        };

        let results = join_all(vec![
            Box::pin(fx_future) as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>,
            Box::pin(euribor_future),
        ])
        .await;

        let mut eur_usd_rate: Option<f64> = None;
        let mut eur_usd_date = String::new();
        let mut euribor_rate: Option<f64> = None;
        let mut euribor_date = String::new();

        if let Some(Ok(data)) = results.first() {
            if let Some((val, date)) = extract_latest_obs(data) {
                eur_usd_rate = Some(val);
                eur_usd_date = date;
            }
        }

        if let Some(Ok(data)) = results.get(1) {
            if let Some((val, date)) = extract_latest_obs(data) {
                euribor_rate = Some(val);
                euribor_date = date;
            }
        }

        let mut signals = Vec::new();
        if let Some(rate) = eur_usd_rate {
            if rate > 1.15 {
                signals.push(format!("EUR/USD at {:.4} — strong euro", rate));
            } else if rate < 1.02 {
                signals.push(format!("EUR/USD at {:.4} — approaching parity", rate));
            }
        }
        if let Some(rate) = euribor_rate {
            if rate > 4.0 {
                signals.push(format!(
                    "EURIBOR 3M at {:.3}% — elevated eurozone rates",
                    rate
                ));
            }
        }
        if signals.is_empty() {
            signals.push("ECB rates within normal range".to_string());
        }

        Ok(json!({
            "source": "ECB",
            "timestamp": Utc::now().to_rfc3339(),
            "eurUsd": {
                "rate": eur_usd_rate,
                "date": eur_usd_date,
            },
            "euribor3m": {
                "rate": euribor_rate,
                "date": euribor_date,
            },
            "signals": signals,
        }))
    }
}

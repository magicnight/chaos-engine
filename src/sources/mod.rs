pub mod acled;
pub mod adsb;
pub mod bluesky;
pub mod bls;
pub mod celestrak;
pub mod cisa_kev;
pub mod cloudflare_radar;
pub mod coingecko;
pub mod comtrade;
pub mod copernicus;
pub mod cve;
pub mod ecb;
pub mod eia;
pub mod epa;
pub mod eu_sanctions;
pub mod firms;
pub mod fred;
pub mod gdacs;
pub mod gdelt;
pub mod google_trends;
pub mod gscpi;
pub mod isc;
pub mod kiwisdr;
pub mod nasa_neo;
pub mod noaa;
pub mod ntsb;
pub mod opensky;
pub mod patents;
pub mod promedmail;
pub mod reddit;
pub mod reliefweb;
pub mod ripe_atlas;
pub mod safecast;
pub mod sanctions;
pub mod ships;
pub mod swpc;
pub mod telegram;
pub mod treasury;
pub mod tsunami;
pub mod usaspending;
pub mod usgs;
pub mod who;
pub mod worldnews;
pub mod yfinance;

use std::fmt;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::http::HttpClient;

/// Core trait every intelligence source must implement.
#[async_trait]
pub trait IntelSource: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn tier(&self) -> u8;
    async fn sweep(&self) -> Result<Value>;
}

/// Outcome of a single source run.
#[derive(PartialEq)]
pub enum SourceStatus {
    Ok,
    Error,
    Timeout,
}

impl fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceStatus::Ok => write!(f, "ok"),
            SourceStatus::Error => write!(f, "error"),
            SourceStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Result returned by `run_source`.
pub struct SourceResult {
    pub name: String,
    pub status: SourceStatus,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tier: u8,
}

/// Run a single source with a wall-clock timeout.
///
/// Returns `SourceStatus::Ok` on success, `SourceStatus::Timeout` when the
/// future does not complete within `timeout`, and `SourceStatus::Error` when
/// the source itself returns an error.
pub async fn run_source(source: &dyn IntelSource, timeout: Duration) -> SourceResult {
    let name = source.name().to_string();
    let tier = source.tier();
    let start = Instant::now();

    match tokio::time::timeout(timeout, source.sweep()).await {
        Ok(Ok(value)) => SourceResult {
            name,
            status: SourceStatus::Ok,
            data: Some(value),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
            tier,
        },
        Ok(Err(e)) => SourceResult {
            name,
            status: SourceStatus::Error,
            data: None,
            error: Some(e.to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
            tier,
        },
        Err(_elapsed) => SourceResult {
            name,
            status: SourceStatus::Timeout,
            data: None,
            error: Some(format!("timed out after {}s", timeout.as_secs())),
            duration_ms: start.elapsed().as_millis() as u64,
            tier,
        },
    }
}

/// Construct all available sources.
pub fn build_sources(client: &HttpClient) -> Vec<Box<dyn IntelSource>> {
    vec![
        // Tier 1 — Core OSINT
        Box::new(acled::Acled::new(client.clone())),
        Box::new(adsb::Adsb::new(client.clone())),
        Box::new(firms::Firms::new(client.clone())),
        Box::new(gdacs::Gdacs::new(client.clone())),
        Box::new(gdelt::Gdelt::new(client.clone())),
        Box::new(opensky::OpenSky::new(client.clone())),
        Box::new(promedmail::PromedMail::new(client.clone())),
        Box::new(reliefweb::ReliefWeb::new(client.clone())),
        Box::new(safecast::Safecast::new(client.clone())),
        Box::new(sanctions::Sanctions::new(client.clone())),
        Box::new(ships::Ships::new(client.clone())),
        Box::new(swpc::Swpc::new(client.clone())),
        Box::new(telegram::Telegram::new(client.clone())),
        Box::new(tsunami::Tsunami::new(client.clone())),
        Box::new(usgs::Usgs::new(client.clone())),
        Box::new(who::Who::new(client.clone())),
        // Tier 2 — Economic/Financial
        Box::new(bls::Bls::new(client.clone())),
        Box::new(coingecko::CoinGecko::new(client.clone())),
        Box::new(comtrade::Comtrade::new(client.clone())),
        Box::new(ecb::Ecb::new(client.clone())),
        Box::new(eia::Eia::new(client.clone())),
        Box::new(fred::Fred::new(client.clone())),
        Box::new(gscpi::Gscpi::new(client.clone())),
        Box::new(treasury::Treasury::new(client.clone())),
        Box::new(usaspending::UsaSpending::new(client.clone())),
        Box::new(worldnews::WorldNews::new(client.clone())),
        // Tier 3
        Box::new(bluesky::Bluesky::new(client.clone())),
        Box::new(cisa_kev::CisaKev::new(client.clone())),
        Box::new(cloudflare_radar::CloudflareRadar::new(client.clone())),
        Box::new(copernicus::Copernicus::new(client.clone())),
        Box::new(cve::Cve::new(client.clone())),
        Box::new(epa::Epa::new(client.clone())),
        Box::new(eu_sanctions::EuSanctions::new(client.clone())),
        Box::new(google_trends::GoogleTrends::new(client.clone())),
        Box::new(isc::Isc::new(client.clone())),
        Box::new(kiwisdr::KiwiSdr::new(client.clone())),
        Box::new(nasa_neo::NasaNeo::new(client.clone())),
        Box::new(noaa::Noaa::new(client.clone())),
        Box::new(ntsb::Ntsb::new(client.clone())),
        Box::new(patents::Patents::new(client.clone())),
        Box::new(reddit::Reddit::new(client.clone())),
        Box::new(ripe_atlas::RipeAtlas::new(client.clone())),
        // Tier 4
        Box::new(celestrak::CelesTrak::new(client.clone())),
        // Tier 5
        Box::new(yfinance::YFinance::new(client.clone())),
    ]
}

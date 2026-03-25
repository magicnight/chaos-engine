use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const RAPIDAPI_MIL_URL: &str = "https://adsbexchange-com1.p.rapidapi.com/v2/mil";

const MILITARY_TYPES: &[(&str, &str)] = &[
    ("RC135", "RC-135 Rivet Joint (SIGINT)"),
    ("E3CF", "E-3 Sentry AWACS"),
    ("E6B", "E-6B Mercury (TACAMO)"),
    ("P8", "P-8 Poseidon (Maritime Patrol)"),
    ("P8A", "P-8A Poseidon"),
    ("RQ4", "RQ-4 Global Hawk (UAV)"),
    ("U2", "U-2 Dragon Lady"),
    ("MQ9", "MQ-9 Reaper (UAV)"),
    ("E8", "E-8 JSTARS"),
    ("KC135", "KC-135 Stratotanker"),
    ("KC46", "KC-46 Pegasus"),
    ("B52", "B-52 Stratofortress"),
    ("B1", "B-1B Lancer"),
    ("B2", "B-2 Spirit"),
    ("C17", "C-17 Globemaster III"),
    ("C5", "C-5 Galaxy"),
    ("C130", "C-130 Hercules"),
    ("VC25", "VC-25 (Air Force One)"),
    ("E4B", "E-4B Nightwatch (Doomsday Plane)"),
];

const MIL_CALLSIGN_PREFIXES: &[&str] = &[
    "RCH", "REACH", "DUKE", "IRON", "NAVY", "TOPCAT",
    "GORDO", "BISON", "DEATH", "DOOM", "SAM", "EXEC",
];

pub struct Adsb {
    client: HttpClient,
}

impl Adsb {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn is_military_callsign(callsign: &str) -> bool {
    let upper = callsign.trim().to_uppercase();
    MIL_CALLSIGN_PREFIXES.iter().any(|p| upper.starts_with(p))
}

fn lookup_military_type(type_code: &str) -> Option<&'static str> {
    let clean: String = type_code
        .to_uppercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    MILITARY_TYPES
        .iter()
        .find(|(k, _)| *k == clean.as_str())
        .map(|(_, v)| *v)
}

fn classify_aircraft(ac: &Value) -> Value {
    let hex = ac
        .get("hex")
        .or_else(|| ac.get("icao"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let callsign = ac
        .get("flight")
        .or_else(|| ac.get("callsign"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let type_code = ac
        .get("t")
        .or_else(|| ac.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mil_flag = ac
        .get("mil")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mil_call = is_military_callsign(callsign);
    let mil_type = lookup_military_type(type_code);
    let is_military = mil_flag || mil_call || mil_type.is_some();

    let lat = ac.get("lat").and_then(|v| v.as_f64());
    let lon = ac.get("lon").and_then(|v| v.as_f64());
    let alt = ac
        .get("alt_baro")
        .or_else(|| ac.get("alt_geom"))
        .and_then(|v| v.as_f64());
    let speed = ac.get("gs").and_then(|v| v.as_f64());
    let registration = ac.get("r").and_then(|v| v.as_str()).unwrap_or("");

    json!({
        "hex": hex,
        "callsign": callsign.trim(),
        "type": type_code,
        "typeDescription": mil_type,
        "lat": lat,
        "lon": lon,
        "altitude": alt,
        "speed": speed,
        "isMilitary": is_military,
        "registration": registration,
    })
}

#[async_trait]
impl IntelSource for Adsb {
    fn name(&self) -> &str {
        "ADS-B"
    }

    fn description(&self) -> &str {
        "ADS-B Exchange military flight tracking"
    }

    fn tier(&self) -> u8 {
        1
    }

    async fn sweep(&self) -> Result<Value> {
        let api_key = std::env::var("ADSB_API_KEY")
            .or_else(|_| std::env::var("RAPIDAPI_KEY"))
            .unwrap_or_default();

        if api_key.is_empty() {
            return Ok(json!({
                "source": "ADS-B Exchange",
                "timestamp": Utc::now().to_rfc3339(),
                "error": "ADSB_API_KEY required (paid RapidAPI). Sign up at https://rapidapi.com/adsbexchange/api/adsbexchange-com1",
                "status": "no_key",
                "militaryAircraft": [],
                "signals": ["ADS-B data unavailable - cannot assess military flight activity"],
            }));
        }

        let resp = self
            .client
            .raw_client()
            .get(RAPIDAPI_MIL_URL)
            .header("X-RapidAPI-Key", &api_key)
            .header("X-RapidAPI-Host", "adsbexchange-com1.p.rapidapi.com")
            .send()
            .await;

        let data: Value = match resp {
            Ok(r) => {
                let text = r.text().await?;
                serde_json::from_str(&text)?
            }
            Err(e) => {
                return Ok(json!({
                    "source": "ADS-B Exchange",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "error",
                    "error": e.to_string(),
                    "militaryAircraft": [],
                }));
            }
        };

        let aircraft_raw = data
            .get("ac")
            .or_else(|| data.get("aircraft"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let military: Vec<Value> = aircraft_raw
            .iter()
            .map(classify_aircraft)
            .filter(|a| a.get("isMilitary").and_then(|v| v.as_bool()).unwrap_or(false))
            .collect();

        let total_mil = military.len();

        // Categorize
        let mut recon = Vec::new();
        let mut bombers = Vec::new();
        let mut tankers = Vec::new();

        for ac in &military {
            let desc = ac
                .get("typeDescription")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            if desc.contains("sigint")
                || desc.contains("awacs")
                || desc.contains("patrol")
                || desc.contains("global hawk")
                || desc.contains("jstars")
            {
                recon.push(ac.clone());
            } else if desc.contains("stratofortress")
                || desc.contains("lancer")
                || desc.contains("spirit")
            {
                bombers.push(ac.clone());
            } else if desc.contains("tanker")
                || desc.contains("pegasus")
            {
                tankers.push(ac.clone());
            }
        }

        let mut signals = Vec::new();
        if recon.len() > 5 {
            signals.push(format!(
                "HIGH ISR ACTIVITY: {} reconnaissance aircraft airborne",
                recon.len()
            ));
        }
        if !bombers.is_empty() {
            signals.push(format!(
                "BOMBERS AIRBORNE: {} strategic bombers detected",
                bombers.len()
            ));
        }
        if tankers.len() > 8 {
            signals.push(format!(
                "ELEVATED TANKER OPS: {} aerial refueling aircraft active",
                tankers.len()
            ));
        }
        if signals.is_empty() {
            signals.push("Military flight activity within normal patterns".to_string());
        }

        Ok(json!({
            "source": "ADS-B Exchange",
            "timestamp": Utc::now().to_rfc3339(),
            "status": "live",
            "totalMilitary": total_mil,
            "categories": {
                "reconnaissance": recon.into_iter().take(20).collect::<Vec<_>>(),
                "bombers": bombers.into_iter().take(10).collect::<Vec<_>>(),
                "tankers": tankers.into_iter().take(10).collect::<Vec<_>>(),
            },
            "militaryAircraft": military.into_iter().take(50).collect::<Vec<_>>(),
            "signals": signals,
        }))
    }
}

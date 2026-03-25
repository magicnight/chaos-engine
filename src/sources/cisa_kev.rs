use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const KEV_URL: &str =
    "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";

pub struct CisaKev {
    client: HttpClient,
}

impl CisaKev {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for CisaKev {
    fn name(&self) -> &str {
        "CISA-KEV"
    }

    fn description(&self) -> &str {
        "CISA Known Exploited Vulnerabilities catalog"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let data = self.client.fetch_json(KEV_URL).await?;

        let vulns = data
            .get("vulnerabilities")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let catalog_version = data
            .get("catalogVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let date_released = data
            .get("dateReleased")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let now = Utc::now();
        let seven_days_ago = now - chrono::Duration::days(7);
        let seven_days_str = seven_days_ago.format("%Y-%m-%d").to_string();

        let mut recent = Vec::new();
        let mut vendor_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut ransomware_count = 0u32;

        for vuln in &vulns {
            let date_added = vuln
                .get("dateAdded")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let vendor = vuln
                .get("vendorProject")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            if date_added >= seven_days_str.as_str() {
                recent.push(json!({
                    "cveID": vuln.get("cveID").and_then(|v| v.as_str()).unwrap_or(""),
                    "vendorProject": vendor,
                    "product": vuln.get("product").and_then(|v| v.as_str()).unwrap_or(""),
                    "vulnerabilityName": vuln.get("vulnerabilityName").and_then(|v| v.as_str()).unwrap_or(""),
                    "dateAdded": date_added,
                    "dueDate": vuln.get("dueDate").and_then(|v| v.as_str()).unwrap_or(""),
                    "knownRansomware": vuln.get("knownRansomwareCampaignUse").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                }));

                *vendor_counts.entry(vendor.to_string()).or_insert(0) += 1;

                if vuln
                    .get("knownRansomwareCampaignUse")
                    .and_then(|v| v.as_str())
                    == Some("Known")
                {
                    ransomware_count += 1;
                }
            }
        }

        // Top vendors sorted by count
        let mut top_vendors: Vec<_> = vendor_counts.into_iter().collect();
        top_vendors.sort_by(|a, b| b.1.cmp(&a.1));
        let top_vendors: Vec<Value> = top_vendors
            .into_iter()
            .take(10)
            .map(|(vendor, count)| json!({"vendor": vendor, "count": count}))
            .collect();

        let mut signals = Vec::new();
        if recent.len() > 5 {
            signals.push(format!(
                "{} new KEV entries in last 7 days -- elevated exploit activity",
                recent.len()
            ));
        }
        if ransomware_count > 0 {
            signals.push(format!(
                "{} recently added CVEs linked to ransomware campaigns",
                ransomware_count
            ));
        }
        if signals.is_empty() {
            signals.push("CISA KEV catalog activity within normal levels".to_string());
        }

        Ok(json!({
            "source": "CISA-KEV",
            "timestamp": Utc::now().to_rfc3339(),
            "catalogVersion": catalog_version,
            "dateReleased": date_released,
            "totalInCatalog": vulns.len(),
            "recentAdditions": recent.len(),
            "ransomwareLinked": ransomware_count,
            "topVendors": top_vendors,
            "vulnerabilities": recent,
            "signals": signals,
        }))
    }
}

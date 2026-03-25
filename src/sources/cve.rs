use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

pub struct Cve {
    client: HttpClient,
}

impl Cve {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

fn severity_from_score(score: f64) -> &'static str {
    if score >= 9.0 {
        "Critical"
    } else if score >= 7.0 {
        "High"
    } else if score >= 4.0 {
        "Medium"
    } else {
        "Low"
    }
}

#[async_trait]
impl IntelSource for Cve {
    fn name(&self) -> &str {
        "NVD-CVE"
    }

    fn description(&self) -> &str {
        "NVD/CVE vulnerability intelligence"
    }

    fn tier(&self) -> u8 {
        3
    }

    async fn sweep(&self) -> Result<Value> {
        let now = Utc::now();
        let seven_days_ago = now - chrono::Duration::days(7);
        let start = seven_days_ago.format("%Y-%m-%dT00:00:00.000").to_string();
        let end = now.format("%Y-%m-%dT23:59:59.999").to_string();

        let url = format!(
            "https://services.nvd.nist.gov/rest/json/cves/2.0?pubStartDate={}&pubEndDate={}",
            start, end
        );

        let api_key = std::env::var("NVD_API_KEY").unwrap_or_default();

        let resp = if api_key.is_empty() {
            self.client.raw_client().get(&url).send().await?
        } else {
            self.client
                .raw_client()
                .get(&url)
                .header("apiKey", &api_key)
                .send()
                .await?
        };

        let text = resp.text().await?;
        let data: Value = serde_json::from_str(&text)?;

        let total_results = data
            .get("totalResults")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let vulnerabilities = data
            .get("vulnerabilities")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut critical_count = 0u32;
        let mut high_count = 0u32;
        let mut top_vulns = Vec::new();

        for vuln_wrapper in &vulnerabilities {
            let cve = match vuln_wrapper.get("cve") {
                Some(c) => c,
                None => continue,
            };

            let cve_id = cve
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let description = cve
                .get("descriptions")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    arr.iter().find(|d| {
                        d.get("lang").and_then(|l| l.as_str()) == Some("en")
                    })
                })
                .and_then(|d| d.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Extract CVSS v3.1 score
            let metrics = cve.get("metrics");
            let cvss_v31 = metrics
                .and_then(|m| m.get("cvssMetricV31"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first());

            let (score, severity) = if let Some(metric) = cvss_v31 {
                let s = metric
                    .get("cvssData")
                    .and_then(|d| d.get("baseScore"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                (s, severity_from_score(s))
            } else {
                (0.0, "Unknown")
            };

            match severity {
                "Critical" => critical_count += 1,
                "High" => high_count += 1,
                _ => {}
            }

            // Only collect Critical and High for the top list
            if (severity == "Critical" || severity == "High") && top_vulns.len() < 20 {
                let truncated_desc = if description.chars().count() > 300 {
                    format!("{}...", description.chars().take(300).collect::<String>())
                } else {
                    description.to_string()
                };

                top_vulns.push(json!({
                    "cveId": cve_id,
                    "description": truncated_desc,
                    "cvssScore": score,
                    "severity": severity,
                }));
            }
        }

        let mut signals = Vec::new();
        if critical_count > 0 {
            signals.push(format!(
                "{} CRITICAL vulnerabilities published in last 7 days",
                critical_count
            ));
        }
        if high_count > 0 {
            signals.push(format!(
                "{} HIGH severity vulnerabilities published in last 7 days",
                high_count
            ));
        }
        if signals.is_empty() {
            signals.push("No critical or high severity CVEs in last 7 days".to_string());
        }

        Ok(json!({
            "source": "NVD-CVE",
            "timestamp": Utc::now().to_rfc3339(),
            "totalResults": total_results,
            "criticalCount": critical_count,
            "highCount": high_count,
            "topVulnerabilities": top_vulns,
            "signals": signals,
        }))
    }
}

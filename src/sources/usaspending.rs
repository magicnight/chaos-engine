use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

use crate::http::HttpClient;
use super::IntelSource;

const BASE: &str = "https://api.usaspending.gov/api/v2";

pub struct UsaSpending {
    client: HttpClient,
}

impl UsaSpending {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntelSource for UsaSpending {
    fn name(&self) -> &str {
        "USAspending"
    }

    fn description(&self) -> &str {
        "Federal spending and defense contracts"
    }

    fn tier(&self) -> u8 {
        2
    }

    async fn sweep(&self) -> Result<Value> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let days_ago_14 = (Utc::now() - chrono::Duration::days(14))
            .format("%Y-%m-%d")
            .to_string();

        let defense_body = json!({
            "filters": {
                "keywords": ["defense", "military", "missile", "ammunition", "aircraft", "naval"],
                "time_period": [{"start_date": days_ago_14, "end_date": today}],
                "award_type_codes": ["A", "B", "C", "D"],
            },
            "fields": [
                "Award ID",
                "Recipient Name",
                "Award Amount",
                "Description",
                "Awarding Agency",
                "Start Date",
                "Award Type",
            ],
            "limit": 20,
            "page": 1,
            "sort": "Award Amount",
            "order": "desc",
        });

        let agencies_url = format!("{}/references/toptier_agencies/", BASE);

        let defense_fut = async {
            let resp = self
                .client
                .raw_client()
                .post(&format!("{}/search/spending_by_award/", BASE))
                .header("Content-Type", "application/json")
                .json(&defense_body)
                .send()
                .await
                .ok()?;
            let text = resp.text().await.ok()?;
            serde_json::from_str::<Value>(&text).ok()
        };

        let agencies_fut = async {
            self.client.fetch_json(&agencies_url).await.ok()
        };

        let (defense_resp, agencies_resp) = tokio::join!(defense_fut, agencies_fut);

        let defense_contracts: Vec<Value> = defense_resp
            .as_ref()
            .and_then(|v| v.get("results"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().take(10).map(|r| {
                    json!({
                        "awardId": r.get("Award ID").and_then(|v| v.as_str()).unwrap_or(""),
                        "recipient": r.get("Recipient Name").and_then(|v| v.as_str()).unwrap_or(""),
                        "amount": r.get("Award Amount"),
                        "description": r.get("Description").and_then(|v| v.as_str()).unwrap_or(""),
                        "agency": r.get("Awarding Agency").and_then(|v| v.as_str()).unwrap_or(""),
                        "date": r.get("Start Date").and_then(|v| v.as_str()).unwrap_or(""),
                        "type": r.get("Award Type").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }).collect()
            })
            .unwrap_or_default();

        let top_agencies: Vec<Value> = agencies_resp
            .as_ref()
            .and_then(|v| v.get("results"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().take(10).map(|a| {
                    json!({
                        "name": a.get("agency_name").and_then(|v| v.as_str()).unwrap_or(""),
                        "budget": a.get("budget_authority_amount"),
                        "obligations": a.get("obligated_amount"),
                        "outlays": a.get("outlay_amount"),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(json!({
            "source": "USAspending",
            "timestamp": Utc::now().to_rfc3339(),
            "recentDefenseContracts": defense_contracts,
            "topAgencies": top_agencies,
        }))
    }
}

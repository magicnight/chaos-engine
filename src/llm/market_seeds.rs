use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{LlmOptions, LlmProvider};

/// A prediction market seed generated from intelligence data.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSeed {
    pub id: String,
    pub question: String,
    pub category: String,
    pub options: Vec<String>,
    pub resolution_criteria: String,
    pub resolution_source: Option<String>,
    pub confidence: f64,
    pub context: String,
    pub suggested_end_time: Option<String>,
}

#[allow(dead_code)]
impl MarketSeed {
    /// Generate a deterministic ID from the question text using SHA-256.
    pub fn generate_id(question: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(question.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..16])
    }
}

/// Generate market seeds using an LLM provider.
#[allow(dead_code)]
pub async fn generate_seeds(
    provider: &dyn LlmProvider,
    sweep_data: &Value,
    count: usize,
) -> Result<Vec<MarketSeed>> {
    let opts = LlmOptions {
        max_tokens: 2048,
        temperature: 0.7,
        model_override: None,
    };

    let data_summary = serde_json::to_string(
        sweep_data.get("sources").unwrap_or(&Value::Null),
    )?;
    // Truncate to avoid exceeding context
    let truncated: String = data_summary.chars().take(4000).collect();

    let prompt = format!(
        "Generate exactly {count} prediction market questions based on this OSINT intelligence data.\n\
         Each question must be a YES/NO question resolvable within 7-30 days.\n\
         Return a JSON array where each object has:\n\
         - question: the YES/NO question\n\
         - category: one of geopolitics, economics, science, technology, health, environment, sports, entertainment, politics, other\n\
         - resolution_criteria: how to verify the outcome\n\
         - confidence: 0.0-1.0 initial probability estimate\n\
         - context: brief explanation of why this question is relevant\n\n\
         Data:\n{truncated}"
    );

    let response = provider
        .complete(
            "You are a prediction market analyst. Output valid JSON only.",
            &prompt,
            &opts,
        )
        .await?;

    // Try to parse the response as JSON array
    let text = response.text.trim();
    // Strip markdown code fences if present
    let json_text = if text.starts_with("```") {
        text.lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        text.to_string()
    };

    let parsed: Vec<Value> = serde_json::from_str(&json_text).unwrap_or_default();

    let seeds: Vec<MarketSeed> = parsed
        .into_iter()
        .take(count)
        .map(|v| {
            let question = v["question"].as_str().unwrap_or("").to_string();
            let id = MarketSeed::generate_id(&question);
            MarketSeed {
                id,
                question,
                category: v["category"]
                    .as_str()
                    .unwrap_or("other")
                    .to_string(),
                options: vec!["YES".to_string(), "NO".to_string()],
                resolution_criteria: v["resolution_criteria"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                resolution_source: v["resolution_source"]
                    .as_str()
                    .map(String::from),
                confidence: v["confidence"].as_f64().unwrap_or(0.5),
                context: v["context"].as_str().unwrap_or("").to_string(),
                suggested_end_time: v["suggested_end_time"]
                    .as_str()
                    .map(String::from),
            }
        })
        .collect();

    Ok(seeds)
}

/// hex encoding helper (avoids adding the `hex` crate).
#[allow(dead_code)]
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

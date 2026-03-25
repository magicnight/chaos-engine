use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{LlmOptions, LlmProvider, LlmResponse};

/// Google Gemini provider — native generateContent API.
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    default_model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            default_model: model,
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.default_model
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        opts: &LlmOptions,
    ) -> Result<LlmResponse> {
        let model = opts
            .model_override
            .as_deref()
            .unwrap_or(&self.default_model);

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        );

        let body = json!({
            "systemInstruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": [{
                "parts": [{ "text": user_message }]
            }],
            "generationConfig": {
                "maxOutputTokens": opts.max_tokens,
                "temperature": opts.temperature,
            }
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "Gemini API {}: {}",
                status,
                &text[..text.len().min(200)]
            );
        }

        let data: serde_json::Value = resp.json().await?;

        let text = data["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let input_tokens =
            data["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0) as u32;
        let output_tokens =
            data["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0) as u32;

        Ok(LlmResponse {
            text,
            model: model.to_string(),
            input_tokens,
            output_tokens,
        })
    }
}

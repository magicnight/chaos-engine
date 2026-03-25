use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{LlmOptions, LlmProvider, LlmResponse};

/// Anthropic Claude provider — native Messages API.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    default_model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            default_model: model,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
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

        let body = json!({
            "model": model,
            "max_tokens": opts.max_tokens,
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": user_message }
            ]
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "Anthropic API {}: {}",
                status,
                &text[..text.len().min(200)]
            );
        }

        let data: serde_json::Value = resp.json().await?;

        let text = data["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let input_tokens = data["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = data["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(LlmResponse {
            text,
            model: data["model"]
                .as_str()
                .unwrap_or(model)
                .to_string(),
            input_tokens,
            output_tokens,
        })
    }
}

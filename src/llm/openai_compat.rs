use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{LlmOptions, LlmProvider, LlmResponse};

/// OpenAI-compatible provider. Covers: OpenAI, DeepSeek, Moonshot, GLM-4,
/// Qwen, OpenRouter, Mistral, MiniMax — any service with /v1/chat/completions.
pub struct OpenAiCompatProvider {
    client: reqwest::Client,
    provider_name: String,
    base_url: String,
    api_key: String,
    default_model: String,
}

impl OpenAiCompatProvider {
    pub fn new(
        provider_name: String,
        base_url: String,
        api_key: String,
        default_model: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            provider_name,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            default_model,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        &self.provider_name
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

        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": model,
            "max_tokens": opts.max_tokens,
            "temperature": opts.temperature,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_message },
            ]
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "{} API {}: {}",
                self.provider_name,
                status,
                &text[..text.len().min(200)]
            );
        }

        let data: serde_json::Value = resp.json().await?;

        let text = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let input_tokens = data["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = data["usage"]["completion_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

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

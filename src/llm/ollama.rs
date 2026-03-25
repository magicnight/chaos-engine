use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{LlmOptions, LlmProvider, LlmResponse};

/// Local Ollama provider — uses OpenAI-compatible /v1/chat/completions endpoint.
/// No API key required.
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    default_model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            default_model: model,
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.default_model
    }

    fn is_configured(&self) -> bool {
        true // local, no key needed
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

        let url = format!("{}/v1/chat/completions", self.base_url);

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
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "Ollama API {}: {}",
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

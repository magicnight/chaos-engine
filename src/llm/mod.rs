pub mod anthropic;
pub mod fallback;
pub mod gemini;
pub mod market_seeds;
pub mod ollama;
pub mod openai_compat;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct LlmOptions {
    pub max_tokens: u32,
    pub temperature: f32,
    pub model_override: Option<String>,
}

impl Default for LlmOptions {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            temperature: 0.3,
            model_override: None,
        }
    }
}

#[derive(Debug)]
pub struct LlmResponse {
    pub text: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    fn is_configured(&self) -> bool;
    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        opts: &LlmOptions,
    ) -> Result<LlmResponse>;
}

/// Create a single provider from name + config.
fn create_single_provider(
    provider_name: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
    model: Option<&str>,
    config: &Config,
) -> Option<Box<dyn LlmProvider>> {
    match provider_name {
        "ollama" => {
            let url = config
                .ollama_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            let m = model
                .map(|s| s.to_string())
                .or_else(|| config.ollama_model.clone())
                .unwrap_or_else(|| "qwen3:8b".to_string());
            Some(Box::new(ollama::OllamaProvider::new(url, m)))
        }
        "anthropic" => {
            let key = api_key?.to_string();
            let m = model
                .map(|s| s.to_string())
                .unwrap_or_else(|| "claude-sonnet-4-6".to_string());
            Some(Box::new(anthropic::AnthropicProvider::new(key, m)))
        }
        "gemini" => {
            let key = api_key
                .map(|s| s.to_string())
                .or_else(|| config.gemini_api_key.clone())?;
            let m = model
                .map(|s| s.to_string())
                .unwrap_or_else(|| "gemini-3.1-flash-lite-preview".to_string());
            Some(Box::new(gemini::GeminiProvider::new(key, m)))
        }
        // openai, deepseek, moonshot, openrouter, mistral, minimax, zhipuai, etc.
        _ => {
            let key = api_key?.to_string();
            let url = base_url
                .map(|s| s.to_string())
                .unwrap_or_else(|| match provider_name {
                    "deepseek" => "https://api.deepseek.com/v1".to_string(),
                    "moonshot" => "https://api.moonshot.cn/v1".to_string(),
                    "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                    "mistral" => "https://api.mistral.ai/v1".to_string(),
                    "minimax" => "https://api.minimax.chat/v1".to_string(),
                    "zhipuai" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
                    _ => "https://api.openai.com/v1".to_string(),
                });
            let m = model
                .map(|s| s.to_string())
                .unwrap_or_else(|| match provider_name {
                    "deepseek" => "deepseek-chat".to_string(),
                    "zhipuai" => "glm-4-flash".to_string(),
                    _ => "gpt-4o".to_string(),
                });
            Some(Box::new(openai_compat::OpenAiCompatProvider::new(
                provider_name.to_string(),
                url,
                key,
                m,
            )))
        }
    }
}

/// Factory: create provider with fallback chain from config.
///
/// Fallback chain: primary → fallback → ollama (if configured)
pub fn create_provider(config: &Config) -> Option<Box<dyn LlmProvider>> {
    let mut providers: Vec<Box<dyn LlmProvider>> = Vec::new();

    // 1. Primary provider
    if let Some(ref provider_name) = config.llm_provider {
        if let Some(p) = create_single_provider(
            provider_name,
            config.llm_api_key.as_deref(),
            config.llm_base_url.as_deref(),
            config.llm_model.as_deref(),
            config,
        ) {
            eprintln!("  [CHAOS] LLM primary: {} ({})", p.name(), p.model());
            providers.push(p);
        }
    }

    // 2. Fallback provider (if configured)
    if let Some(ref fallback_name) = config.fallback_provider {
        let fallback_key = match fallback_name.as_str() {
            "gemini" => config.gemini_api_key.as_deref(),
            _ => config.llm_api_key.as_deref(),
        };
        if let Some(p) = create_single_provider(
            fallback_name,
            fallback_key,
            None,
            config.fallback_model.as_deref(),
            config,
        ) {
            eprintln!("  [CHAOS] LLM fallback: {} ({})", p.name(), p.model());
            providers.push(p);
        }
    }

    // 3. Ollama as last resort (if URL is set and not already primary)
    let primary_is_ollama = config
        .llm_provider
        .as_deref()
        .map(|p| p == "ollama")
        .unwrap_or(false);
    if !primary_is_ollama && config.ollama_url.is_some() {
        if let Some(p) = create_single_provider("ollama", None, None, None, config) {
            eprintln!("  [CHAOS] LLM local fallback: {} ({})", p.name(), p.model());
            providers.push(p);
        }
    }

    if providers.is_empty() {
        return None;
    }

    // Single provider: return directly (no overhead)
    if providers.len() == 1 {
        return Some(providers.into_iter().next().unwrap());
    }

    // Multiple: wrap in FallbackProvider
    Some(Box::new(fallback::FallbackProvider::new(providers)))
}

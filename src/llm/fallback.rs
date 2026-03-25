// Fallback LLM Provider — tries providers in order until one succeeds

use anyhow::Result;
use async_trait::async_trait;

use super::{LlmOptions, LlmProvider, LlmResponse};

/// Wraps multiple LLM providers, trying each in order until one succeeds.
pub struct FallbackProvider {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl FallbackProvider {
    pub fn new(providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl LlmProvider for FallbackProvider {
    fn name(&self) -> &str {
        "fallback-chain"
    }

    fn model(&self) -> &str {
        self.providers
            .first()
            .map(|p| p.model())
            .unwrap_or("none")
    }

    fn is_configured(&self) -> bool {
        self.providers.iter().any(|p| p.is_configured())
    }

    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        opts: &LlmOptions,
    ) -> Result<LlmResponse> {
        let mut last_err = None;

        for (i, provider) in self.providers.iter().enumerate() {
            if !provider.is_configured() {
                continue;
            }

            match provider.complete(system_prompt, user_message, opts).await {
                Ok(response) => {
                    if i > 0 {
                        eprintln!(
                            "  [CHAOS] LLM fallback: {} failed, succeeded with {}",
                            self.providers[0].name(),
                            provider.name()
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    eprintln!(
                        "  [CHAOS] LLM {} failed: {}, trying next...",
                        provider.name(),
                        e
                    );
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No LLM providers configured")))
    }
}

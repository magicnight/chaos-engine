use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use tokio::time::sleep;

use crate::error::ChaosError;

const USER_AGENT: &str = "CHAOS/3.0";

/// Wraps a reqwest::Client with retry logic and a fixed user-agent.
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    max_retries: u32,
}

impl HttpClient {
    /// Build a client with a connection pool of 10 per host and the given
    /// per-request timeout.
    pub fn new(timeout_secs: u64, max_retries: u32) -> Result<Self, ChaosError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(10)
            .user_agent(USER_AGENT)
            .build()
            .map_err(ChaosError::Http)?;

        Ok(HttpClient { client, max_retries })
    }

    /// Expose the underlying reqwest::Client for sources that need custom requests.
    pub fn raw_client(&self) -> &Client {
        &self.client
    }

    /// GET the URL and parse the response body as JSON.
    pub async fn fetch_json(&self, url: &str) -> Result<Value, ChaosError> {
        let text = self.get_with_retry(url).await?;
        serde_json::from_str(&text).map_err(ChaosError::Json)
    }

    /// GET the URL and return the response body as a String.
    pub async fn fetch_text(&self, url: &str) -> Result<String, ChaosError> {
        self.get_with_retry(url).await
    }

    /// Performs a GET request with exponential backoff: 500ms, 1s, 2s, …
    ///
    /// 4xx errors (except 429) are treated as immediate failures (no retry).
    /// 5xx and 429 errors are retried with backoff.
    async fn get_with_retry(&self, url: &str) -> Result<String, ChaosError> {
        let mut last_err: Option<ChaosError> = None;
        let backoff_ms: &[u64] = &[500, 1_000, 2_000];

        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let text = resp.text().await.map_err(ChaosError::Http)?;
                        return Ok(text);
                    }
                    // 4xx (except 429 Too Many Requests) — fail immediately, no retry
                    if status.is_client_error() && status.as_u16() != 429 {
                        let text = resp.text().await.unwrap_or_default();
                        return Err(ChaosError::Other(format!(
                            "HTTP {} for {}: {}",
                            status.as_u16(),
                            url,
                            text.chars().take(200).collect::<String>()
                        )));
                    }
                    // 5xx or 429 — retriable
                    last_err = Some(ChaosError::Other(format!(
                        "HTTP {} for {}",
                        status.as_u16(),
                        url
                    )));
                }
                Err(e) => {
                    last_err = Some(ChaosError::Http(e));
                }
            }

            if attempt < self.max_retries {
                let delay = backoff_ms
                    .get(attempt as usize)
                    .copied()
                    .unwrap_or(2_000);
                sleep(Duration::from_millis(delay)).await;
            }
        }

        Err(last_err.unwrap_or_else(|| ChaosError::Other("Unknown HTTP error".into())))
    }
}

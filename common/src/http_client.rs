use crate::errors::AppError;
use reqwest::Client;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

/// HTTP client with retry logic and timeout
pub struct HttpClient {
    client: Client,
    max_retries: u32,
    timeout: Duration,
}

impl HttpClient {
    pub fn new(timeout_secs: u64, max_retries: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            max_retries,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Fetch JSON from URL with retry and exponential backoff
    #[instrument(skip(self), fields(url = %url))]
    pub async fn get_json<T>(&self, url: &str) -> Result<T, AppError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let span = tracing::span!(tracing::Level::INFO, "http_request", attempt = attempt + 1);
            let _enter = span.enter();

            match self.fetch_with_timeout(url).await {
                Ok(response) => {
                    info!(url = %url, attempt = attempt + 1, "Request successful");
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        let backoff = Duration::from_millis(2_u64.pow(attempt) * 100);
                        warn!(
                            url = %url,
                            attempt = attempt + 1,
                            backoff_ms = backoff.as_millis(),
                            "Request failed, retrying with exponential backoff"
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        error!(
            url = %url,
            attempts = self.max_retries + 1,
            "All retry attempts exhausted"
        );
        Err(last_error.unwrap_or_else(|| AppError::internal("Unknown error after retries")))
    }

    async fn fetch_with_timeout<T>(&self, url: &str) -> Result<T, AppError>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = tokio::time::timeout(self.timeout, self.client.get(url).send())
            .await
            .map_err(|_| AppError::timeout(format!("Request to {} timed out", url)))?
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::timeout(format!("Request to {} timed out", url))
                } else {
                    AppError::NetworkError(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::http(
                status.as_u16(),
                format!("HTTP error: {}", status),
            ));
        }

        let text = response.text().await.map_err(AppError::NetworkError)?;
        let json: T = serde_json::from_str(&text).map_err(AppError::ParseError)?;

        Ok(json)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(2, 2) // 2 second timeout, 2 retries
    }
}

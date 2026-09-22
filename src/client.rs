use std::env;
use std::time::Duration;

use reqwest::{Client as HttpClient, StatusCode};
use serde::Serialize;
use serde_json::Value;

use crate::answer::{ModelsResponse, SystemOneResponse};
use crate::error::Error;
use crate::question::Question;
use crate::serde_map;

const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai/v1";
const ENV_API_KEY: &str = "TYPESAFE_API_KEY";

/// Retry behavior for `429 Too Many Requests` and `529 Overloaded` responses.
/// TypeSafe's own client SDKs retry these with exponential backoff by default;
/// this mirrors that.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(8),
        }
    }
}

impl RetryPolicy {
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            initial_backoff: Duration::from_millis(0),
            max_backoff: Duration::from_millis(0),
        }
    }

    fn backoff_for(&self, attempt: u32) -> Duration {
        let scale = 2u32.saturating_pow(attempt);
        (self.initial_backoff * scale).min(self.max_backoff)
    }
}

/// A client for TypeSafe's System One API (`https://api.typesafe.ai/v1`).
///
/// Every model is served by the same evaluation endpoint; the `model` field
/// on each request picks which one handles it. See <https://docs.typesafe.ai/api>.
#[derive(Debug, Clone)]
pub struct TypeSafeClient {
    http: HttpClient,
    base_url: String,
    api_key: String,
    retry_policy: RetryPolicy,
}

impl TypeSafeClient {
    /// Build a client with an explicit API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: api_key.into(),
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Build a client using the `TYPESAFE_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, Error> {
        let api_key = env::var(ENV_API_KEY).map_err(|_| {
            Error::Authentication(format!("{ENV_API_KEY} is not set"))
        })?;
        Ok(Self::new(api_key))
    }

    /// Override the API base URL (useful for testing against a mock server).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Override the retry policy for `429`/`529` responses.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Evaluate `state` against a map of typed `questions` using `model`
    /// (e.g. `"jev-latest"`). Every question is evaluated independently and
    /// in parallel against the same state; batch everything you might need
    /// into one call rather than issuing several. See
    /// <https://docs.typesafe.ai/primitives#ask-multiple-questions-together>.
    pub async fn system_one(
        &self,
        state: impl Serialize,
        model: impl Into<String>,
        questions: Vec<(String, Question)>,
    ) -> Result<SystemOneResponse, Error> {
        let body = SystemOneRequest {
            state: serde_json::to_value(state).map_err(Error::Decode)?,
            model: model.into(),
            questions,
        };

        let url = format!("{}/systemone", self.base_url);
        let response = self.send_with_retry(|| self.http.post(&url).json(&body)).await?;
        response.json().await.map_err(Error::Transport)
    }

    /// `GET /v1/models` — the model names and aliases your account can send
    /// in the `model` field.
    pub async fn list_models(&self) -> Result<ModelsResponse, Error> {
        let url = format!("{}/models", self.base_url);
        let response = self.send_with_retry(|| self.http.get(&url)).await?;
        response.json().await.map_err(Error::Transport)
    }

    async fn send_with_retry(
        &self,
        build: impl Fn() -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, Error> {
        let mut attempt = 0;
        loop {
            let response = build()
                .bearer_auth(&self.api_key)
                .send()
                .await
                .map_err(Error::Transport)?;

            let status = response.status();
            if status.is_success() {
                return Ok(response);
            }

            let retryable = status == StatusCode::TOO_MANY_REQUESTS || status.as_u16() == 529;
            if retryable && attempt < self.retry_policy.max_retries {
                let wait = retry_after(&response).unwrap_or_else(|| self.retry_policy.backoff_for(attempt));
                tokio::time::sleep(wait).await;
                attempt += 1;
                continue;
            }

            return Err(to_error(status, response.text().await.unwrap_or_default()));
        }
    }
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

fn to_error(status: StatusCode, body: String) -> Error {
    match status.as_u16() {
        401 => Error::Authentication(body),
        422 => Error::Validation(body),
        429 => Error::RateLimited(body),
        529 => Error::Overloaded(body),
        code => Error::Api { status: code, body },
    }
}

#[derive(Serialize)]
struct SystemOneRequest {
    state: Value,
    model: String,
    #[serde(with = "serde_map")]
    questions: Vec<(String, Question)>,
}

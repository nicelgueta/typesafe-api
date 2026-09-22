use thiserror::Error;

/// Errors returned by the TypeSafe API or the underlying transport.
#[derive(Debug, Error)]
pub enum Error {
    /// `401 Unauthorized` — missing or invalid API key.
    #[error("authentication failed: {0}")]
    Authentication(String),

    /// `422 Unprocessable Entity` — the request body failed validation.
    #[error("request validation failed: {0}")]
    Validation(String),

    /// `429 Too Many Requests` — rate limit exceeded, even after retries.
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// `529 Overloaded` — TypeSafe is temporarily overloaded, even after retries.
    #[error("service overloaded: {0}")]
    Overloaded(String),

    /// Any other non-2xx response.
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },

    /// The transport itself failed (DNS, TLS, connection reset, etc).
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// The response body could not be decoded as the expected JSON shape.
    #[error("failed to decode response: {0}")]
    Decode(serde_json::Error),
}

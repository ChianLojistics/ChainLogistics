use thiserror::Error;

/// SDK error types
#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid API key format")]
    InvalidApiKey,

    #[error("Network timeout")]
    Timeout,

    #[error("Server error: {0}")]
    Server(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Error {
    /// Create an API error from status code and message
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        match self {
            Error::Api { status, .. } => (400..500).contains(status),
            Error::Unauthorized | Error::RateLimit | Error::NotFound(_) | Error::Validation(_) => {
                true
            }
            _ => false,
        }
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        match self {
            Error::Api { status, .. } => (500..600).contains(status),
            Error::Server(_) => true,
            _ => false,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Http(_) | Error::Timeout | Error::RateLimit | Error::Server(_)
        )
    }

    /// Actionable guidance for recovering from this error.
    pub fn recovery_guidance(&self) -> &'static str {
        match self {
            Error::Unauthorized => "Check API key validity and permissions, then retry.",
            Error::RateLimit => "Back off and retry with exponential delay.",
            Error::Timeout | Error::Http(_) => "Check network connectivity and retry.",
            Error::Validation(_) => "Fix request payload fields before retrying.",
            Error::Config(_) | Error::InvalidApiKey => {
                "Review SDK configuration and environment variables."
            }
            Error::Server(_)
            | Error::Api {
                status: 500..=599, ..
            } => "Service-side failure. Retry with backoff and contact support if persistent.",
            Error::NotFound(_) => "Verify resource identifiers before retrying.",
            _ => "Inspect the error context and retry only if the operation is idempotent.",
        }
    }

    /// Key/value pairs suitable for structured logging.
    pub fn log_fields(&self) -> Vec<(&'static str, String)> {
        let kind = match self {
            Error::Http(_) => "http",
            Error::Json(_) => "json",
            Error::Url(_) => "url",
            Error::Api { .. } => "api",
            Error::Unauthorized => "unauthorized",
            Error::RateLimit => "rate_limit",
            Error::NotFound(_) => "not_found",
            Error::Validation(_) => "validation",
            Error::Config(_) => "config",
            Error::Io(_) => "io",
            Error::InvalidApiKey => "invalid_api_key",
            Error::Timeout => "timeout",
            Error::Server(_) => "server",
            Error::Unknown(_) => "unknown",
        };

        let mut fields = vec![
            ("error.kind", kind.to_string()),
            ("error.message", self.to_string()),
            ("error.retryable", self.is_retryable().to_string()),
            ("error.guidance", self.recovery_guidance().to_string()),
        ];

        if let Error::Api { status, .. } = self {
            fields.push(("error.status", status.to_string()));
        }
        fields
    }
}

pub type Result<T> = std::result::Result<T, Error>;

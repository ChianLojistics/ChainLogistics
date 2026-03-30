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
            Error::Unauthorized | Error::RateLimit | Error::NotFound(_) | Error::Validation(_) => true,
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
}

pub type Result<T> = std::result::Result<T, Error>;

use std::time::Duration;

/// Configuration for the ChainLogistics client
#[derive(Debug, Clone)]
pub struct Config {
    api_key: String,
    base_url: String,
    timeout: Duration,
    user_agent: String,
}

impl Config {
    /// Create a new configuration with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: crate::DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
            user_agent: format!("chainlogistics-sdk-rust/{}", crate::VERSION),
        }
    }

    /// Set the base URL for the API
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a custom user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Get the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the user agent string
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.api_key.is_empty() {
            return Err(crate::Error::Config("API key cannot be empty".to_string()));
        }

        if self.base_url.is_empty() {
            return Err(crate::Error::Config("Base URL cannot be empty".to_string()));
        }

        // Validate URL format
        url::Url::parse(&self.base_url)
            .map_err(|e| crate::Error::Config(format!("Invalid base URL: {}", e)))?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::new("test-key");
        assert_eq!(config.api_key(), "test-key");
        assert_eq!(config.base_url(), crate::DEFAULT_BASE_URL);
        assert_eq!(config.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new("test-key")
            .with_base_url("https://example.com")
            .with_timeout(Duration::from_secs(60))
            .with_user_agent("custom-agent");

        assert_eq!(config.api_key(), "test-key");
        assert_eq!(config.base_url(), "https://example.com");
        assert_eq!(config.timeout(), Duration::from_secs(60));
        assert_eq!(config.user_agent(), "custom-agent");
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = Config::new("test-key").with_base_url("https://api.example.com");
        assert!(config.validate().is_ok());

        // Empty API key
        let config = Config::new("").with_base_url("https://api.example.com");
        assert!(config.validate().is_err());

        // Empty base URL
        let config = Config::new("test-key").with_base_url("");
        assert!(config.validate().is_err());

        // Invalid URL
        let config = Config::new("test-key").with_base_url("not-a-url");
        assert!(config.validate().is_err());
    }
}

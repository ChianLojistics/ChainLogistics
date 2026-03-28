use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{Config, Error, Result};

/// Main ChainLogistics API client
#[derive(Debug, Clone)]
pub struct ChainLogisticsClient {
    config: Config,
    client: Client,
    products: crate::products::ProductsService,
    events: crate::events::EventsService,
    stats: crate::stats::StatsService,
}

impl ChainLogisticsClient {
    /// Create a new client with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let client = Client::builder()
            .timeout(config.timeout())
            .user_agent(config.user_agent())
            .build()?;

        let products = crate::products::ProductsService::new(client.clone(), config.clone());
        let events = crate::events::EventsService::new(client.clone(), config.clone());
        let stats = crate::stats::StatsService::new(client.clone(), config.clone());

        Ok(Self {
            config,
            client,
            products,
            events,
            stats,
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the products service
    pub fn products(&self) -> &crate::products::ProductsService {
        &self.products
    }

    /// Get the events service
    pub fn events(&self) -> &crate::events::EventsService {
        &self.events
    }

    /// Get the stats service
    pub fn stats(&self) -> &crate::stats::StatsService {
        &self.stats
    }

    /// Perform a health check
    pub async fn health_check(&self) -> Result<crate::models::HealthResponse> {
        let response = self
            .client
            .get(format!("{}/health", self.config.base_url()))
            .send()
            .await?;

        if response.status().is_success() {
            let health = response.json().await?;
            Ok(health)
        } else {
            Err(Error::api(
                response.status().as_u16(),
                "Health check failed",
            ))
        }
    }

    /// Perform a database health check
    pub async fn db_health_check(&self) -> Result<crate::models::DbHealthResponse> {
        let response = self
            .client
            .get(format!("{}/health/db", self.config.base_url()))
            .send()
            .await?;

        if response.status().is_success() {
            let health = response.json().await?;
            Ok(health)
        } else {
            Err(Error::api(
                response.status().as_u16(),
                "Database health check failed",
            ))
        }
    }
}

/// Internal HTTP client helper
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: Config,
}

impl HttpClient {
    pub fn new(client: Client, config: Config) -> Self {
        Self { client, config }
    }

    /// Create an authenticated GET request
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.client.get(self.url(path)).bearer_auth(self.config.api_key())
    }

    /// Create an authenticated POST request
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.client.post(self.url(path)).bearer_auth(self.config.api_key())
    }

    /// Create an authenticated PUT request
    pub fn put(&self, path: &str) -> RequestBuilder {
        self.client.put(self.url(path)).bearer_auth(self.config.api_key())
    }

    /// Create an authenticated DELETE request
    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.client
            .delete(self.url(path))
            .bearer_auth(self.config.api_key())
    }

    /// Execute a request and parse the JSON response
    pub async fn execute<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Execute a request with a body and parse the JSON response
    pub async fn execute_with_body<T: DeserializeOwned, B: Serialize>(
        &self,
        request: RequestBuilder,
        body: &B,
    ) -> Result<T> {
        let request = request.json(body);
        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Execute a request that returns no body
    pub async fn execute_no_body(&self, request: RequestBuilder) -> Result<()> {
        let response = request.send().await?;
        self.handle_empty_response(response).await
    }

    /// Handle HTTP response
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            response.json().await.map_err(Error::from)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            let error = match status.as_u16() {
                401 => Error::Unauthorized,
                429 => Error::RateLimit,
                404 => Error::NotFound(error_text),
                400..=499 => Error::Api {
                    status: status.as_u16(),
                    message: error_text,
                },
                _ => Error::Server(error_text),
            };
            Err(error)
        }
    }

    /// Handle empty response
    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<()> {
        let status = response.status();
        
        if status.is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            let error = match status.as_u16() {
                401 => Error::Unauthorized,
                429 => Error::RateLimit,
                404 => Error::NotFound(error_text),
                400..=499 => Error::Api {
                    status: status.as_u16(),
                    message: error_text,
                },
                _ => Error::Server(error_text),
            };
            Err(error)
        }
    }

    /// Build full URL from path
    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.config.base_url().trim_end_matches('/'), path.trim_start_matches('/'))
    }
}

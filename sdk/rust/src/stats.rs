use crate::{
    client::HttpClient,
    models::GlobalStats,
    Config, Result,
};

/// Service for accessing statistics and analytics
#[derive(Debug, Clone)]
pub struct StatsService {
    client: HttpClient,
}

impl StatsService {
    pub(crate) fn new(client: reqwest::Client, config: Config) -> Self {
        Self {
            client: HttpClient::new(client, config),
        }
    }

    /// Get global statistics
    pub async fn get_global(&self) -> Result<GlobalStats> {
        let request = self.client.get("api/v1/stats");
        self.client.execute(request).await
    }

    /// Get system health status
    pub async fn health(&self) -> Result<crate::models::HealthResponse> {
        let request = self.client.get("health");
        self.client.execute(request).await
    }

    /// Get database health status
    pub async fn db_health(&self) -> Result<crate::models::DbHealthResponse> {
        let request = self.client.get("health/db");
        self.client.execute(request).await
    }
}

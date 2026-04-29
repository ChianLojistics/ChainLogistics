use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub security: SecurityConfig,
    pub audit: AuditConfig,
    pub encryption_key: String,
    pub jwt_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub read_replica_urls: Vec<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enforce_https: bool,
    pub hsts_max_age: u64,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub hmac_key: String,
    pub retention_days: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://chainlogistics:password@localhost/chainlogistics".to_string()
                }),
                read_replica_urls: env::var("DATABASE_READ_REPLICA_URLS")
                    .unwrap_or_else(|_| "".to_string())
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect(),
                max_connections: 20,
                min_connections: 5,
                connect_timeout: 30,
                idle_timeout: 600,
            },
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3001".to_string())
                    .parse()
                    .unwrap_or(3001),
                tls_enabled: env::var("TLS_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                tls_cert_path: env::var("TLS_CERT_PATH").ok(),
                tls_key_path: env::var("TLS_KEY_PATH").ok(),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            security: SecurityConfig {
                enforce_https: env::var("ENFORCE_HTTPS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                hsts_max_age: env::var("HSTS_MAX_AGE")
                    .unwrap_or_else(|_| "31536000".to_string())
                    .parse()
                    .unwrap_or(31536000), // 1 year
                allowed_origins: env::var("ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "https://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            audit: AuditConfig {
                enabled: env::var("AUDIT_LOGGING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                hmac_key: env::var("AUDIT_HMAC_KEY")
                    .unwrap_or_else(|_| "default_audit_hmac_key_change_me_in_production".to_string()),
                retention_days: env::var("AUDIT_RETENTION_DAYS")
                    .unwrap_or_else(|_| "365".to_string())
                    .parse()
                    .unwrap_or(365),
            },
            encryption_key: env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef".to_string()), // 32 chars for AES-256
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default_jwt_secret_change_me_in_production".to_string()),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let profile = env::var("CHAINLOGISTICS_ENV")
            .or_else(|_| env::var("APP_ENV"))
            .unwrap_or_else(|_| "development".to_string());

        let cfg = config::Config::builder()
            .add_source(config::Config::try_from(&Config::default())?)
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name(&format!("config/{}", profile)).required(false))
            .add_source(
                config::Environment::with_prefix("CHAINLOGISTICS")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let config: Config = cfg.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }
    fn validate(&self) -> Result<(), config::ConfigError> {
        if self.database.url.trim().is_empty() {
            return Err(config::ConfigError::Message(
                "database.url must not be empty".to_string(),
            ));
        }
        if self.redis.url.trim().is_empty() {
            return Err(config::ConfigError::Message(
                "redis.url must not be empty".to_string(),
            ));
        }
        if self.jwt_secret.trim().len() < 16 {
            return Err(config::ConfigError::Message(
                "jwt_secret must be at least 16 characters".to_string(),
            ));
        }
        if self.encryption_key.trim().len() != 32 {
            return Err(config::ConfigError::Message(
                "encryption_key must be exactly 32 characters (AES-256 key)".to_string(),
            ));
        }
        if self.server.tls_enabled
            && (self.server.tls_cert_path.is_none() || self.server.tls_key_path.is_none())
        {
            return Err(config::ConfigError::Message(
                "tls_cert_path and tls_key_path are required when tls_enabled=true".to_string(),
            ));
        }
        Ok(())
    }
}

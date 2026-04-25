use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub security: SecurityConfig,
    pub encryption_key: String,
    pub jwt_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://chainlogistics:password@localhost/chainlogistics".to_string()),
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
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
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
            encryption_key: env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef".to_string()), // 32 chars for AES-256
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default_jwt_secret_change_me_in_production".to_string()),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Config::try_from(&Config::default())?)
            .add_source(config::Environment::with_prefix("CHAINLOGISTICS"))
            .build()?;

        cfg.try_deserialize()
    }
}
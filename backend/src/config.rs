use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid PORT value '{0}': must be a valid u16 (1-65535)")]
    InvalidPort(String),
    #[error("Invalid {0} value '{1}': expected 'true' or 'false'")]
    InvalidBool(String, String),
    #[error("Invalid {0} value '{1}': must be a valid u64")]
    InvalidU64(String, String),
    #[error("Missing required environment variable: {0}")]
    MissingVar(String),
}

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

impl Config {
    /// Parse configuration from environment variables with proper error handling and logging.
    /// This function validates all configuration values and logs the parsing process.
    pub fn from_env() -> Result<Self, ConfigError> {
        info!("Loading configuration from environment variables");

        let database = Self::parse_database_config();
        let server = Self::parse_server_config()?;
        let redis = Self::parse_redis_config();
        let security = Self::parse_security_config()?;
        let encryption_key = Self::parse_encryption_key();
        let jwt_secret = Self::parse_jwt_secret();

        let config = Config {
            database,
            server,
            redis,
            security,
            encryption_key,
            jwt_secret,
        };

        Self::validate(&config)?;

        info!("Configuration loaded successfully - port: {}, host: {}, tls: {}", 
              config.server.port, config.server.host, config.server.tls_enabled);

        Ok(config)
    }

    fn parse_database_config() -> DatabaseConfig {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                info!("DATABASE_URL not set, using default PostgreSQL connection string");
                "postgres://chainlogistics:password@localhost/chainlogistics".to_string()
            });
        info!("Database URL configured");

        DatabaseConfig {
            url,
            max_connections: 20,
            min_connections: 5,
            connect_timeout: 30,
            idle_timeout: 600,
        }
    }

    fn parse_server_config() -> Result<ServerConfig, ConfigError> {
        let host = env::var("HOST").unwrap_or_else(|_| {
            info!("HOST not set, using default 0.0.0.0");
            "0.0.0.0".to_string()
        });

        let port = Self::parse_port("PORT", 3001)?;

        let tls_enabled = Self::parse_bool("TLS_ENABLED", false)?;

        let tls_cert_path = env::var("TLS_CERT_PATH").ok();
        let tls_key_path = env::var("TLS_KEY_PATH").ok();

        if tls_enabled {
            if tls_cert_path.is_none() || tls_key_path.is_none() {
                warn!("TLS enabled but certificate paths not configured");
            }
        }

        Ok(ServerConfig {
            host,
            port,
            tls_enabled,
            tls_cert_path,
            tls_key_path,
        })
    }

    fn parse_redis_config() -> RedisConfig {
        let url = env::var("REDIS_URL")
            .unwrap_or_else(|_| {
                info!("REDIS_URL not set, using default redis://localhost:6379");
                "redis://localhost:6379".to_string()
            });
        info!("Redis URL configured");

        RedisConfig { url }
    }

    fn parse_security_config() -> Result<SecurityConfig, ConfigError> {
        let enforce_https = Self::parse_bool("ENFORCE_HTTPS", true)?;
        let hsts_max_age = Self::parse_u64("HSTS_MAX_AGE", 31536000)?;

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| {
                info!("ALLOWED_ORIGINS not set, using default https://localhost:3000");
                "https://localhost:3000".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        info!("Security configuration loaded - enforce_https: {}, hsts_max_age: {}, origins: {}",
              enforce_https, hsts_max_age, allowed_origins.len());

        Ok(SecurityConfig {
            enforce_https,
            hsts_max_age,
            allowed_origins,
        })
    }

    fn parse_encryption_key() -> String {
        let key = env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| {
                warn!("ENCRYPTION_KEY not set, using default insecure key - CHANGE IN PRODUCTION");
                "0123456789abcdef0123456789abcdef".to_string() // 32 chars for AES-256
            });
        info!("Encryption key configured (length: {})", key.len());
        key
    }

    fn parse_jwt_secret() -> String {
        let secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                warn!("JWT_SECRET not set, using default insecure secret - CHANGE IN PRODUCTION");
                "default_jwt_secret_change_me_in_production".to_string()
            });
        info!("JWT secret configured (length: {})", secret.len());
        secret
    }

    /// Parse a port number from an environment variable with validation.
    /// Valid port range: 1-65535 (u16)
    fn parse_port(env_var: &str, default: u16) -> Result<u16, ConfigError> {
        match env::var(env_var) {
            Ok(port_str) => {
                match port_str.parse::<u16>() {
                    Ok(port) => {
                        if port == 0 {
                            Err(ConfigError::InvalidPort(env_var.to_string()))
                        } else {
                            info!("Using {} from environment: {}", env_var, port);
                            Ok(port)
                        }
                    }
                    Err(_) => {
                        warn!("Invalid {} value '{}': must be a valid u16 (1-65535). Using default: {}", 
                              env_var, port_str, default);
                        Ok(default)
                    }
                }
            }
            Err(_) => {
                info!("{} not set, using default: {}", env_var, default);
                Ok(default)
            }
        }
    }

    /// Parse a boolean value from an environment variable with validation.
    fn parse_bool(env_var: &str, default: bool) -> Result<bool, ConfigError> {
        match env::var(env_var) {
            Ok(value) => {
                match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => {
                        info!("Using {} from environment: true", env_var);
                        Ok(true)
                    }
                    "false" | "0" | "no" | "off" => {
                        info!("Using {} from environment: false", env_var);
                        Ok(false)
                    }
                    _ => Err(ConfigError::InvalidBool(env_var.to_string(), value)),
                }
            }
            Err(_) => {
                info!("{} not set, using default: {}", env_var, default);
                Ok(default)
            }
        }
    }

    /// Parse a u64 value from an environment variable with validation.
    fn parse_u64(env_var: &str, default: u64) -> Result<u64, ConfigError> {
        match env::var(env_var) {
            Ok(value) => {
                match value.parse::<u64>() {
                    Ok(num) => {
                        info!("Using {} from environment: {}", env_var, num);
                        Ok(num)
                    }
                    Err(_) => Err(ConfigError::InvalidU64(env_var.to_string(), value)),
                }
            }
            Err(_) => {
                info!("{} not set, using default: {}", env_var, default);
                Ok(default)
            }
        }
    }

    /// Validate the complete configuration.
    fn validate(config: &Config) -> Result<(), ConfigError> {
        // Validate port is in valid range
        if config.server.port == 0 {
            return Err(ConfigError::InvalidPort("PORT".to_string()));
        }

        // Validate encryption key length (should be 32 chars for AES-256)
        if config.encryption_key.len() != 32 {
            warn!("Encryption key length is {} (expected 32 for AES-256)", config.encryption_key.len());
        }

        // Validate JWT secret is not empty
        if config.jwt_secret.is_empty() {
            return Err(ConfigError::MissingVar("JWT_SECRET".to_string()));
        }

        // Validate TLS configuration consistency
        if config.server.tls_enabled {
            if config.server.tls_cert_path.is_none() || config.server.tls_key_path.is_none() {
                warn!("TLS enabled but certificate paths missing");
            }
        }

        Ok(())
    }

    /// Load configuration with the config crate (legacy method for compatibility).
    /// This method uses the Default implementation for fallback values.
    pub fn from_env_legacy() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Config::try_from(&Config::default())?)
            .add_source(config::Environment::with_prefix("CHAINLOGISTICS"))
            .build()?;

        cfg.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://chainlogistics:password@localhost/chainlogistics".to_string(),
                max_connections: 20,
                min_connections: 5,
                connect_timeout: 30,
                idle_timeout: 600,
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3001,
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
            },
            security: SecurityConfig {
                enforce_https: true,
                hsts_max_age: 31536000,
                allowed_origins: vec!["https://localhost:3000".to_string()],
            },
            encryption_key: "0123456789abcdef0123456789abcdef".to_string(),
            jwt_secret: "default_jwt_secret_change_me_in_production".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_port_valid_values() {
        // Test valid port values
        env::set_var("PORT", "8080");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8080);

        env::set_var("PORT", "1");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        env::set_var("PORT", "65535");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 65535);

        env::remove_var("PORT");
    }

    #[test]
    fn test_parse_port_invalid_values() {
        // Test non-numeric values - should return default with warning
        env::set_var("PORT", "abc");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3001); // Uses default
        env::remove_var("PORT");

        env::set_var("PORT", "8080abc");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3001); // Uses default
        env::remove_var("PORT");

        // Test negative values (will fail u16 parse)
        env::set_var("PORT", "-1");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3001); // Uses default
        env::remove_var("PORT");

        // Test out-of-range values
        env::set_var("PORT", "99999");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3001); // Uses default
        env::remove_var("PORT");
    }

    #[test]
    fn test_parse_port_zero_value() {
        // Test port 0 - should return error
        env::set_var("PORT", "0");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidPort(_))));
        env::remove_var("PORT");
    }

    #[test]
    fn test_parse_port_missing_env_var() {
        // Test missing environment variable - should use default
        env::remove_var("PORT");
        let result = Config::parse_port("PORT", 3001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3001);
    }

    #[test]
    fn test_parse_bool_valid_values() {
        // Test valid true values
        for value in ["true", "TRUE", "True", "1", "yes", "YES", "on", "ON"] {
            env::set_var("TLS_ENABLED", value);
            let result = Config::parse_bool("TLS_ENABLED", false);
            assert!(result.is_ok(), "Failed for value: {}", value);
            assert_eq!(result.unwrap(), true);
        }

        // Test valid false values
        for value in ["false", "FALSE", "False", "0", "no", "NO", "off", "OFF"] {
            env::set_var("TLS_ENABLED", value);
            let result = Config::parse_bool("TLS_ENABLED", true);
            assert!(result.is_ok(), "Failed for value: {}", value);
            assert_eq!(result.unwrap(), false);
        }

        env::remove_var("TLS_ENABLED");
    }

    #[test]
    fn test_parse_bool_invalid_values() {
        // Test invalid boolean values
        for value in ["maybe", "2", "invalid", "t", "f"] {
            env::set_var("TLS_ENABLED", value);
            let result = Config::parse_bool("TLS_ENABLED", false);
            assert!(result.is_err(), "Should fail for value: {}", value);
            assert!(matches!(result, Err(ConfigError::InvalidBool(_, _))));
        }
        env::remove_var("TLS_ENABLED");
    }

    #[test]
    fn test_parse_bool_missing_env_var() {
        // Test missing environment variable - should use default
        env::remove_var("TLS_ENABLED");
        let result = Config::parse_bool("TLS_ENABLED", true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_parse_u64_valid_values() {
        // Test valid u64 values
        env::set_var("HSTS_MAX_AGE", "31536000");
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 31536000);

        env::set_var("HSTS_MAX_AGE", "0");
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        env::set_var("HSTS_MAX_AGE", "18446744073709551615"); // u64::MAX
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 18446744073709551615);

        env::remove_var("HSTS_MAX_AGE");
    }

    #[test]
    fn test_parse_u64_invalid_values() {
        // Test invalid u64 values
        env::set_var("HSTS_MAX_AGE", "abc");
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidU64(_, _))));
        env::remove_var("HSTS_MAX_AGE");

        env::set_var("HSTS_MAX_AGE", "-1");
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidU64(_, _))));
        env::remove_var("HSTS_MAX_AGE");

        env::set_var("HSTS_MAX_AGE", "1.5");
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidU64(_, _))));
        env::remove_var("HSTS_MAX_AGE");
    }

    #[test]
    fn test_parse_u64_missing_env_var() {
        // Test missing environment variable - should use default
        env::remove_var("HSTS_MAX_AGE");
        let result = Config::parse_u64("HSTS_MAX_AGE", 31536000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 31536000);
    }

    #[test]
    fn test_validate_port_zero() {
        // Test validation rejects port 0
        let mut config = Config::default();
        config.server.port = 0;
        let result = Config::validate(&config);
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidPort(_))));
    }

    #[test]
    fn test_validate_empty_jwt_secret() {
        // Test validation rejects empty JWT secret
        let mut config = Config::default();
        config.jwt_secret = "".to_string();
        let result = Config::validate(&config);
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::MissingVar(_))));
    }

    #[test]
    fn test_validate_encryption_key_length() {
        // Test validation warns about incorrect encryption key length
        let mut config = Config::default();
        config.encryption_key = "short".to_string();
        let result = Config::validate(&config);
        // Should succeed but with warning (can't test warning directly)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tls_configuration() {
        // Test TLS configuration validation
        let mut config = Config::default();
        config.server.tls_enabled = true;
        config.server.tls_cert_path = None;
        config.server.tls_key_path = None;
        let result = Config::validate(&config);
        // Should succeed but with warning (can't test warning directly)
        assert!(result.is_ok());

        // Complete TLS configuration
        config.server.tls_cert_path = Some("/path/to/cert.pem".to_string());
        config.server.tls_key_path = Some("/path/to/key.pem".to_string());
        let result = Config::validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_env_with_valid_config() {
        // Test loading configuration with valid environment variables
        env::set_var("PORT", "8080");
        env::set_var("HOST", "127.0.0.1");
        env::set_var("TLS_ENABLED", "true");
        env::set_var("ENFORCE_HTTPS", "false");
        env::set_var("HSTS_MAX_AGE", "86400");
        env::set_var("JWT_SECRET", "test_secret_key_123456789012345");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef");

        let result = Config::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.tls_enabled, true);
        assert_eq!(config.security.enforce_https, false);
        assert_eq!(config.security.hsts_max_age, 86400);

        // Cleanup
        env::remove_var("PORT");
        env::remove_var("HOST");
        env::remove_var("TLS_ENABLED");
        env::remove_var("ENFORCE_HTTPS");
        env::remove_var("HSTS_MAX_AGE");
        env::remove_var("JWT_SECRET");
        env::remove_var("ENCRYPTION_KEY");
    }

    #[test]
    fn test_from_env_with_invalid_port() {
        // Test loading configuration with invalid port
        env::set_var("PORT", "invalid");
        env::set_var("JWT_SECRET", "test_secret_key_123456789012345");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef");

        let result = Config::from_env();
        assert!(result.is_ok()); // Should use default with warning

        let config = result.unwrap();
        assert_eq!(config.server.port, 3001); // Default value

        // Cleanup
        env::remove_var("PORT");
        env::remove_var("JWT_SECRET");
        env::remove_var("ENCRYPTION_KEY");
    }

    #[test]
    fn test_from_env_with_invalid_bool() {
        // Test loading configuration with invalid boolean
        env::set_var("TLS_ENABLED", "maybe");
        env::set_var("JWT_SECRET", "test_secret_key_123456789012345");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidBool(_, _))));

        // Cleanup
        env::remove_var("TLS_ENABLED");
        env::remove_var("JWT_SECRET");
        env::remove_var("ENCRYPTION_KEY");
    }

    #[test]
    fn test_from_env_with_invalid_u64() {
        // Test loading configuration with invalid u64
        env::set_var("HSTS_MAX_AGE", "invalid");
        env::set_var("JWT_SECRET", "test_secret_key_123456789012345");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::InvalidU64(_, _))));

        // Cleanup
        env::remove_var("HSTS_MAX_AGE");
        env::remove_var("JWT_SECRET");
        env::remove_var("ENCRYPTION_KEY");
    }

    #[test]
    fn test_from_env_with_missing_jwt_secret() {
        // Test loading configuration with empty JWT secret
        env::set_var("JWT_SECRET", "");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::MissingVar(_))));

        // Cleanup
        env::remove_var("JWT_SECRET");
        env::remove_var("ENCRYPTION_KEY");
    }

    #[test]
    fn test_config_error_display() {
        // Test ConfigError display formatting
        let err = ConfigError::InvalidPort("PORT".to_string());
        assert_eq!(err.to_string(), "Invalid PORT value 'PORT': must be a valid u16 (1-65535)");

        let err = ConfigError::InvalidBool("TLS_ENABLED".to_string(), "maybe".to_string());
        assert_eq!(err.to_string(), "Invalid TLS_ENABLED value 'maybe': expected 'true' or 'false'");

        let err = ConfigError::InvalidU64("HSTS_MAX_AGE".to_string(), "abc".to_string());
        assert_eq!(err.to_string(), "Invalid HSTS_MAX_AGE value 'abc': must be a valid u64");

        let err = ConfigError::MissingVar("JWT_SECRET".to_string());
        assert_eq!(err.to_string(), "Missing required environment variable: JWT_SECRET");
    }

    #[test]
    fn test_default_config() {
        // Test default configuration values
        let config = Config::default();
        assert_eq!(config.server.port, 3001);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.tls_enabled, false);
        assert_eq!(config.security.enforce_https, true);
        assert_eq!(config.security.hsts_max_age, 31536000);
        assert_eq!(config.encryption_key.len(), 32);
        assert!(!config.jwt_secret.is_empty());
    }
}
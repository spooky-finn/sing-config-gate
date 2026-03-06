//! Application configuration management.
//!
//! This module handles loading and validating environment variables
//! required for the application to run.

pub use crate::errors::EnvError;

/// Initializes the environment from .env file.
/// Should be called once at application startup.
fn init_env() {
    dotenvy::dotenv().ok();
}

/// Loads an environment variable, returning an error if not set.
pub fn get_env(name: &str) -> Result<String, EnvError> {
    std::env::var(name).map_err(|_| EnvError::Missing(name.to_string()))
}

/// Loads an environment variable or returns a default value.
pub fn get_env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Parses an environment variable into a specific type.
fn parse_env<T: std::str::FromStr>(name: &str) -> Result<T, EnvError> {
    let val = get_env(name)?;
    val.parse::<T>()
        .map_err(|_| EnvError::Invalid(name.to_string(), val))
}

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub tg_bot_token: String,
    pub tg_admin_id: i64,
    pub client_config_endpoint: String,
    pub db_location: String,
    pub log_level: String,
    pub log_disable_timestamp: bool,
    pub sing_box_private_key: Option<String>,
    pub sing_box_short_id: Option<String>,
    pub sing_box_server_name: String,
}

impl AppConfig {
    /// Loads configuration from environment variables.
    ///
    /// # Panics
    /// Panics if required environment variables are missing or invalid.
    pub fn load() -> Self {
        init_env();
        match Self::from_env() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                eprintln!("Required environment variables: TELOXIDE_TOKEN, TG_ADMIN_ID, CLIENT_CONFIG_ENDPOINT");
                eprintln!("Set them via docker --env-file or -e flags");
                std::process::exit(1);
            }
        }
    }

    fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            tg_bot_token: get_env("TELOXIDE_TOKEN")?,
            tg_admin_id: parse_env("TG_ADMIN_ID")?,
            client_config_endpoint: get_env("CLIENT_CONFIG_ENDPOINT")?,
            db_location: get_env_or("DB_LOCATION", "./src/db/vpn_signaling_server.db"),
            log_level: get_env_or("LOG_LEVEL", "info"),
            log_disable_timestamp: get_env_or("LOG_DISABLE_TIMESTAMP", "false") == "true",
            sing_box_private_key: get_env("SING_BOX_PRIVATE_KEY").ok(),
            sing_box_short_id: get_env("SING_BOX_SHORT_ID").ok(),
            sing_box_server_name: get_env_or("SING_BOX_SERVER_NAME", "google.com"),
        })
    }
}

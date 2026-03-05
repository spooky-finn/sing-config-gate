#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Missing required environment variable: {0}")]
    Missing(String),
    #[error("Invalid environment variable {0}: {1}")]
    Invalid(String, String),
}

fn get_env(name: &str) -> Result<String, EnvError> {
    dotenvy::dotenv().ok();
    std::env::var(name).map_err(|_| EnvError::Missing(name.to_string()))
}

fn get_env_or(name: &str, default: &str) -> String {
    dotenvy::dotenv().ok();
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn parse_env<T: std::str::FromStr>(name: &str) -> Result<T, EnvError> {
    let val = get_env(name)?;
    val.parse::<T>()
        .map_err(|_| EnvError::Invalid(name.to_string(), val))
}

fn parse_env_or<T: std::str::FromStr>(name: &str, default: &str) -> T {
    let val = std::env::var(name).unwrap_or_else(|_| default.to_string());
    val.parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid default value for {}", name))
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub tg_bot_token: String,
    pub tg_admin_id: i64,
    pub client_config_endpoint: String,
    pub db_location: String,
    pub log_level: String,
    pub log_disable_timestamp: bool,
    pub sing_box_private_key: String,
    pub sing_box_short_id: String,
    pub sing_box_server_name: String,
    pub sing_box_server_port: u16,
}

impl AppConfig {
    pub fn load() -> Self {
        AppConfig::from_env().expect("Failed to load environment configuration")
    }

    fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            tg_bot_token: get_env("TELOXIDE_TOKEN")?,
            tg_admin_id: parse_env("TG_ADMIN_ID")?,
            client_config_endpoint: get_env("CLIENT_CONFIG_ENDPOINT")?,
            db_location: get_env_or("DB_LOCATION", "./src/db/vpn_signaling_server.db"),
            log_level: get_env_or("LOG_LEVEL", "info"),
            log_disable_timestamp: get_env_or("LOG_DISABLE_TIMESTAMP", "false") == "true",
            sing_box_private_key: get_env("SING_BOX_PRIVATE_KEY")?,
            sing_box_short_id: get_env("SING_BOX_SHORT_ID")?,
            sing_box_server_name: get_env_or("SING_BOX_SERVER_NAME", "google.com"),
            sing_box_server_port: parse_env_or("SING_BOX_SERVER_PORT", "443"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub deploy_host: String,
    pub deploy_keyfile: String,
    pub deploy_user: String,
    pub deploy_command: String,
    pub deploy_cwd: String,
}

impl DeployConfig {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            deploy_host: get_env("DEPLOY_HOST")?,
            deploy_keyfile: get_env("DEPLOY_KEYFILE")?,
            deploy_user: get_env("DEPLOY_USER")?,
            deploy_command: get_env("DEPLOY_COMMAND")?,
            deploy_cwd: get_env("DEPLOY_CWD")?,
        })
    }
}

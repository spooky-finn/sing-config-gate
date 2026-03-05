use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Missing required environment variable: {0}")]
    Missing(String),
    #[error("Invalid environment variable {0}: {1}")]
    #[allow(dead_code)]
    Invalid(String, String),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub tg_bot_token: String,
    pub tg_admin_id: i64,
    pub client_config_endpoint: String,
    pub db_location: String,
    pub log_level: String,
    pub log_disable_timestamp: bool,
    #[allow(dead_code)]
    pub sing_box_private_key: String,
    #[allow(dead_code)]
    pub sing_box_short_id: String,
    #[allow(dead_code)]
    pub sing_box_server_name: String,
    #[allow(dead_code)]
    pub sing_box_server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, EnvError> {
        dotenvy::dotenv().ok();

        let tg_bot_token = std::env::var("TG_BOT_TOKEN")
            .map_err(|_| EnvError::Missing("TG_BOT_TOKEN".to_string()))?;

        let tg_admin_id_str = std::env::var("TG_ADMIN_ID")
            .map_err(|_| EnvError::Missing("TG_ADMIN_ID".to_string()))?;
        let tg_admin_id = tg_admin_id_str
            .parse::<i64>()
            .map_err(|_| EnvError::Invalid("TG_ADMIN_ID".to_string(), tg_admin_id_str))?;

        let client_config_endpoint = std::env::var("CLIENT_CONFIG_ENDPOINT")
            .map_err(|_| EnvError::Missing("CLIENT_CONFIG_ENDPOINT".to_string()))?;

        let db_location = std::env::var("DB_LOCATION")
            .unwrap_or_else(|_| "./db/vpn_signaling_server.db".to_string());

        let log_level =
            std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let log_disable_timestamp = std::env::var("LOG_DISABLE_TIMESTAMP")
            .map(|v| v == "true")
            .unwrap_or(false);

        let sing_box_private_key = std::env::var("SING_BOX_PRIVATE_KEY")
            .map_err(|_| EnvError::Missing("SING_BOX_PRIVATE_KEY".to_string()))?;

        let sing_box_short_id = std::env::var("SING_BOX_SHORT_ID")
            .map_err(|_| EnvError::Missing("SING_BOX_SHORT_ID".to_string()))?;

        let sing_box_server_name = std::env::var("SING_BOX_SERVER_NAME")
            .unwrap_or_else(|_| "google.com".to_string());

        let sing_box_server_port = std::env::var("SING_BOX_SERVER_PORT")
            .unwrap_or_else(|_| "443".to_string())
            .parse::<u16>()
            .map_err(|_| EnvError::Invalid("SING_BOX_SERVER_PORT".to_string(), std::env::var("SING_BOX_SERVER_PORT").unwrap_or_default()))?;

        Ok(Self {
            tg_bot_token,
            tg_admin_id,
            client_config_endpoint,
            db_location,
            log_level,
            log_disable_timestamp,
            sing_box_private_key,
            sing_box_short_id,
            sing_box_server_name,
            sing_box_server_port,
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
        dotenvy::dotenv().ok();

        let deploy_host = std::env::var("DEPLOY_HOST")
            .map_err(|_| EnvError::Missing("DEPLOY_HOST".to_string()))?;

        let deploy_keyfile = std::env::var("DEPLOY_KEYFILE")
            .map_err(|_| EnvError::Missing("DEPLOY_KEYFILE".to_string()))?;

        let deploy_user = std::env::var("DEPLOY_USER")
            .map_err(|_| EnvError::Missing("DEPLOY_USER".to_string()))?;

        let deploy_command = std::env::var("DEPLOY_COMMAND")
            .map_err(|_| EnvError::Missing("DEPLOY_COMMAND".to_string()))?;

        let deploy_cwd = std::env::var("DEPLOY_CWD")
            .map_err(|_| EnvError::Missing("DEPLOY_CWD".to_string()))?;

        Ok(Self {
            deploy_host,
            deploy_keyfile,
            deploy_user,
            deploy_command,
            deploy_cwd,
        })
    }
}

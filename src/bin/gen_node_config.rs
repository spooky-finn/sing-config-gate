// generate sing box proxy config
use serde::{Deserialize, Serialize};
use sing_box_config_bot::{
    adapters::VlessIdentityRepo,
    config::AppConfig,
    connect,
    ports::vless_identity::VlessIdentityRepoTrait,
    singbox::{
        builders::{ServerReality, ServerTls},
        server::{Inbound, Outbound, ServerConfig, VlessUser},
        shared::{DnsConfig, DnsServer, LogConfig},
    },
    utils::logger,
};
use std::fs;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsServerMinimal {
    pub tag: String,
    pub address: String,
}

#[tokio::main]
async fn main() {
    let config = AppConfig::load();
    logger::init(&config.log_level, config.log_disable_timestamp);
    info!("Generating sing-box server config");

    // Validate required sing-box configuration for node config generation
    let private_key = config
        .sing_box_private_key
        .expect("SING_BOX_PRIVATE_KEY is required for node config generation");
    let short_id = config
        .sing_box_short_id
        .expect("SING_BOX_SHORT_ID is required for node config generation");

    let pool = connect(&config.db_location).expect("Failed to initialize database");
    let vless_identity_repo = VlessIdentityRepo::new(pool.clone());
    let identities = vless_identity_repo.get_all().unwrap();
    let users: Vec<VlessUser> = identities
        .iter()
        .map(|identity| VlessUser {
            uuid: identity.uuid.clone(),
            flow: "xtls-rprx-vision".to_string(),
        })
        .collect();

    // Build the config
    let reality = ServerReality::new(&private_key, &short_id, &config.sing_box_server_name, 443);

    let sing_box_config = ServerConfig {
        log: LogConfig {
            level: "info".to_string(),
            timestamp: false,
        },
        dns: DnsConfig {
            servers: vec![DnsServer {
                tag: "cf".to_string(),
                address: "1.0.0.1".to_string(),
                address_resolver: None,
                strategy: None,
                detour: None,
            }],
            rules: None,
            final_field: None,
        },
        inbounds: vec![Inbound {
            listen: "::".to_string(),
            listen_port: 443,
            r#type: "vless".to_string(),
            tag: "vless-in".to_string(),
            users,
            tls: ServerTls::new(&config.sing_box_server_name, reality),
        }],
        outbounds: vec![Outbound {
            r#type: "direct".to_string(),
            tag: "direct-out".to_string(),
        }],
    };

    let json = serde_json::to_string_pretty(&sing_box_config).expect("Failed to serialize config");

    let output_path = "config/sing-box.server.json";
    fs::write(output_path, &json).expect("Failed to write config file");

    info!(path = %output_path, "Config file generated successfully");
}

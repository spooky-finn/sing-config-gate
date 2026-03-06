// generate sing box proxy config
mod adapters;
mod config;
mod db;
mod ports;
mod singbox;
mod utils;

use std::fs;
use tracing::info;

use crate::{
    adapters::db::VlessIdentityRepo,
    config::AppConfig,
    ports::vless_identity::VlessIdentityRepoTrait,
    singbox::{
        server::{Inbound, Outbound, Reality, RealityHandshake, ServerConfig, Tls, VlessUser},
        shared::{DnsConfig, DnsServer, LogConfig},
    },
    utils::logger,
};

#[tokio::main]
async fn main() {
    let config = AppConfig::load();
    logger::init(&config.log_level, config.log_disable_timestamp);
    info!("Generating sing-box server config");

    let pool = db::connect(&config.db_location).expect("Failed to initialize database");
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
    let sing_box_config = ServerConfig {
        log: LogConfig {
            level: "info".to_string(),
            timestamp: false,
        },
        dns: DnsConfig {
            servers: vec![DnsServer {
                tag: "cf".to_string(),
                address: "1.0.0.1".to_string(),
            }],
        },
        inbounds: vec![Inbound {
            listen: "::".to_string(),
            listen_port: config.sing_box_server_port,
            r#type: "vless".to_string(),
            tag: "vless-in".to_string(),
            users,
            tls: Tls {
                enabled: true,
                server_name: config.sing_box_server_name.clone(),
                reality: Reality {
                    enabled: true,
                    handshake: RealityHandshake {
                        server: config.sing_box_server_name,
                        server_port: 443,
                    },
                    private_key: config.sing_box_private_key,
                    short_id: vec![config.sing_box_short_id],
                },
            },
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

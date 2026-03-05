// generate sing box proxy config
mod adapters;
mod config;
mod db;
mod ports;
mod utils;

use std::fs;

use adapters::db::UserRepo;
use config::AppConfig;
use diesel::prelude::*;
use ports::user::UserRepoTrait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utils::logger;

use crate::db::models::VlessIdentity;

#[derive(Debug, Serialize, Deserialize)]
struct SingBoxUser {
    uuid: String,
    flow: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RealityConfig {
    enabled: bool,
    handshake: HandshakeConfig,
    private_key: String,
    short_id: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HandshakeConfig {
    server: String,
    server_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct TlsConfig {
    enabled: bool,
    server_name: String,
    reality: RealityConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct Inbound {
    listen: String,
    listen_port: u16,
    r#type: String,
    tag: String,
    users: Vec<SingBoxUser>,
    tls: TlsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct Outbound {
    r#type: String,
    tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogConfig {
    level: String,
    timestamp: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnsServer {
    tag: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnsConfig {
    servers: Vec<DnsServer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingBoxConfig {
    log: LogConfig,
    dns: DnsConfig,
    inbounds: Vec<Inbound>,
    outbounds: Vec<Outbound>,
}

#[tokio::main]
async fn main() {
    let config = AppConfig::load();
    logger::init(&config.log_level, config.log_disable_timestamp);
    info!("Generating sing-box server config");

    let pool = db::connect(&config.db_location).expect("Failed to initialize database");
    let user_repo = UserRepo::new(pool);

    // Get all accepted users
    let accepted_users = user_repo
        .get_by_status(db::enums::UserStatus::Accepted)
        .expect("Failed to get accepted users");

    // Get VPN UUIDs for accepted users
    let mut conn = user_repo.conn().expect("Failed to get database connection");

    let mut users: Vec<SingBoxUser> = Vec::new();

    for user in &accepted_users {
        // Get or create UUID for this user
        use crate::db::schema::vless_identity::dsl as vless_dsl;
        let uuid_record = vless_dsl::vless_identity
            .filter(vless_dsl::user_id.eq(user.id))
            .first::<VlessIdentity>(&mut conn)
            .optional();

        match uuid_record {
            Ok(Some(uuid_record)) => {
                users.push(SingBoxUser {
                    uuid: uuid_record.uuid,
                    flow: "xtls-rprx-vision".to_string(),
                });
            }
            Ok(None) => {
                // Generate new UUID for this user
                let new_uuid = uuid::Uuid::new_v4().to_string();
                let new_vless_identity = VlessIdentity {
                    uuid: new_uuid.clone(),
                    user_id: Some(user.id),
                };

                // Insert the new UUID
                diesel::insert_into(vless_dsl::vless_identity)
                    .values(&new_vless_identity)
                    .execute(&mut conn)
                    .expect("Failed to insert VPN UUID");

                users.push(SingBoxUser {
                    uuid: new_uuid,
                    flow: "xtls-rprx-vision".to_string(),
                });
            }
            Err(e) => {
                error!(error = %e, "Failed to query VPN UUID");
            }
        }
    }

    // Build the config
    let sing_box_config = SingBoxConfig {
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
            tls: TlsConfig {
                enabled: true,
                server_name: config.sing_box_server_name.clone(),
                reality: RealityConfig {
                    enabled: true,
                    handshake: HandshakeConfig {
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
    println!("{}", json);
}

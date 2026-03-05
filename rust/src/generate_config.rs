mod adapters;
mod db;
mod ports;
mod utils;

use adapters::db::{init_db, DieselUserRepo};
use db::{enums::UserStatus, NewVpnUuid, VpnUuidRow};
use diesel::prelude::*;
use ports::user::IUserRepo;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{error, info};
use utils::env::AppConfig;
use utils::log::init_logger;

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
    // Load and validate environment
    let config = AppConfig::from_env().expect("Failed to load environment configuration");

    // Initialize logger
    init_logger(&config.log_level, config.log_disable_timestamp);

    info!("Generating sing-box server config");

    // Initialize database
    let pool = init_db(&config.db_location).expect("Failed to initialize database");
    let user_repo = DieselUserRepo::new(pool);

    // Get all accepted users
    let accepted_users = user_repo
        .get_by_status(db::enums::UserStatus::Accepted)
        .expect("Failed to get accepted users");

    // Get VPN UUIDs for accepted users
    use crate::db::schema::vpn_uuid::dsl as vpn_dsl;
    let mut conn = user_repo
        .get_connection()
        .expect("Failed to get database connection");

    let mut users: Vec<SingBoxUser> = Vec::new();

    for user in &accepted_users {
        // Get or create UUID for this user
        let vpn_uuid = diesel::dsl::select(diesel::dsl::sql::<diesel::sql_types::BigInt>("1"))
            .get_result::<i64>(&mut conn);

        use crate::db::schema::vpn_uuid::dsl as vpn_dsl;
        let uuid_record = vpn_dsl::vpn_uuid
            .filter(vpn_dsl::user_id.eq(user.id))
            .first::<VpnUuidRow>(&mut conn)
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
                let new_vpn_uuid = NewVpnUuid {
                    id: user.id,
                    uuid: new_uuid.clone(),
                    user_id: user.id,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                // Insert the new UUID
                diesel::insert_into(vpn_dsl::vpn_uuid)
                    .values(&new_vpn_uuid)
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

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&sing_box_config).expect("Failed to serialize config");

    // Write to file
    let output_path = "config/sing-box.server.json";
    fs::write(output_path, &json).expect("Failed to write config file");

    info!(path = %output_path, "Config file generated successfully");
    println!("{}", json);
}

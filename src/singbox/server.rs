use serde::{Deserialize, Serialize};

use crate::singbox::shared::{DnsConfig, LogConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct VlessUser {
    pub uuid: String,
    pub flow: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reality {
    pub enabled: bool,
    pub handshake: RealityHandshake,
    pub private_key: String,
    pub short_id: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealityHandshake {
    pub server: String,
    pub server_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tls {
    pub enabled: bool,
    pub server_name: String,
    pub reality: Reality,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inbound {
    pub listen: String,
    pub listen_port: u16,
    pub r#type: String,
    pub tag: String,
    pub users: Vec<VlessUser>,
    pub tls: Tls,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outbound {
    pub r#type: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub log: LogConfig,
    pub dns: DnsConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
}

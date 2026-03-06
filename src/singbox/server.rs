//! Sing-box server configuration types.

use serde::{Deserialize, Serialize};

use crate::singbox::shared::{DnsConfig, LogConfig};
pub use crate::singbox::builders::{ServerReality, RealityHandshake, ServerTls};

#[derive(Debug, Serialize, Deserialize)]
pub struct VlessUser {
    pub uuid: String,
    pub flow: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inbound {
    pub listen: String,
    pub listen_port: u16,
    pub r#type: String,
    pub tag: String,
    pub users: Vec<VlessUser>,
    pub tls: ServerTls,
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

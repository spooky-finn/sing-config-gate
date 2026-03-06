use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub timestamp: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsServer {
    pub tag: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<DnsServer>,
}

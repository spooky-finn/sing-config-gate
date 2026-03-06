//! Shared sing-box configuration types.

use serde::{Deserialize, Serialize};

/// Log configuration for sing-box.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub timestamp: bool,
}

/// DNS server configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct DnsServer {
    pub tag: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_resolver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detour: Option<String>,
}

/// DNS rule configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct DnsRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_keyword: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_regex: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
}

/// DNS configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<DnsServer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<DnsRule>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "final")]
    pub final_field: Option<String>,
}

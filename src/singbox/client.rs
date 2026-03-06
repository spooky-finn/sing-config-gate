//! Sing-box client configuration types.

use serde::{Deserialize, Serialize};

use crate::singbox::shared::{DnsConfig, LogConfig};
use crate::singbox::builders::ClientTls;

#[derive(Debug, Serialize, Deserialize)]
pub struct TunInboundPlatformHttpProxy {
    pub enabled: bool,
    pub server: String,
    pub server_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TunInbound {
    #[serde(rename = "type")]
    pub r#type: String,
    pub mtu: u16,
    pub tag: String,
    pub interface_name: String,
    pub address: Vec<String>,
    pub auto_route: bool,
    pub strict_route: bool,
    pub endpoint_independent_nat: bool,
    pub stack: String,
    pub sniff: bool,
    pub platform: TunInboundPlatformHttpProxy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MixedInbound {
    #[serde(rename = "type")]
    pub r#type: String,
    pub tag: String,
    pub listen: String,
    pub listen_port: u16,
    pub sniff: bool,
    pub users: Vec<()>,
    pub set_system_proxy: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VlessOutbound {
    #[serde(rename = "type")]
    pub r#type: String,
    pub tag: String,
    pub server: String,
    pub server_port: u16,
    pub uuid: String,
    pub flow: String,
    pub tls: ClientTls,
    pub packet_encoding: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectOutbound {
    #[serde(rename = "type")]
    pub r#type: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RouteRule {
    DomainSuffix {
        domain_suffix: Vec<String>,
        outbound: String,
    },
    DomainKeyword {
        domain_keyword: Vec<String>,
        outbound: String,
    },
    Sniff {
        action: String,
        timeout: String,
    },
    Logical {
        r#type: String,
        mode: String,
        rules: Vec<LogicalRule>,
        action: String,
    },
    IpIsPrivate {
        ip_is_private: bool,
        outbound: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum LogicalRule {
    Protocol {
        protocol: String,
    },
    Port {
        port: u16,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteConfig {
    pub rules: Vec<RouteRule>,
    pub auto_detect_interface: bool,
    pub override_android_vpn: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub log: LogConfig,
    pub dns: DnsConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub route: RouteConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Inbound {
    Tun(TunInbound),
    Mixed(MixedInbound),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Outbound {
    Vless(VlessOutbound),
    Direct(DirectOutbound),
}

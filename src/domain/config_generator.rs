use crate::{
    config::AppConfig,
    singbox::{
        builders::{ClientReality, ClientTls, Utls},
        client::{
            ClientConfig, DirectOutbound, Inbound, LogicalRule, MixedInbound, Outbound,
            RouteConfig, RouteRule, TunInbound, TunInboundPlatformHttpProxy, VlessOutbound,
        },
        shared::{DnsConfig, DnsRule, DnsServer, LogConfig},
    },
};
use super::routing_config::RoutingConfig;

pub fn generate_config(
    app_config: &AppConfig,
    routing_config: &RoutingConfig,
    uuid: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let dns_servers = vec![
        DnsServer {
            tag: "dns_proxy".to_string(),
            address: "https://mozilla.cloudflare-dns.com/dns-query".to_string(),
            address_resolver: Some("dns_resolver".to_string()),
            strategy: Some("ipv4_only".to_string()),
            detour: Some("direct".to_string()),
        },
        DnsServer {
            tag: "dns_direct".to_string(),
            address: "https://common.dot.dns.yandex.net/dns-query".to_string(),
            address_resolver: Some("dns_resolver".to_string()),
            strategy: Some("ipv4_only".to_string()),
            detour: Some("direct".to_string()),
        },
        DnsServer {
            tag: "dns_resolver".to_string(),
            address: "77.88.8.8".to_string(),
            address_resolver: None,
            strategy: Some("ipv4_only".to_string()),
            detour: Some("direct".to_string()),
        },
        DnsServer {
            tag: "dns_block".to_string(),
            address: "rcode://refused".to_string(),
            address_resolver: None,
            strategy: None,
            detour: None,
        },
    ];

    let dns_rules = vec![
        DnsRule {
            domain_keyword: Some(routing_config.dns_proxy_keywords.clone()),
            domain_regex: None,
            server: Some("dns_proxy".to_string()),
        },
        DnsRule {
            domain_keyword: Some(routing_config.dns_direct_keywords.clone()),
            domain_regex: Some(routing_config.dns_direct_regex.clone()),
            server: Some("dns_direct".to_string()),
        },
    ];

    let dns = DnsConfig {
        servers: dns_servers,
        rules: Some(dns_rules),
        final_field: Some("dns_direct".to_string()),
    };

    let tun_inbound = TunInbound {
        r#type: "tun".to_string(),
        mtu: 9000,
        tag: "tun-in".to_string(),
        interface_name: "tun125".to_string(),
        address: vec![
            "172.19.0.1/30".to_string(),
            "fdfe:dcba:9876::1/126".to_string(),
        ],
        auto_route: true,
        strict_route: false,
        endpoint_independent_nat: true,
        stack: "mixed".to_string(),
        sniff: true,
        platform: TunInboundPlatformHttpProxy {
            enabled: true,
            server: "127.0.0.1".to_string(),
            server_port: 2412,
        },
    };

    let mixed_inbound = MixedInbound {
        r#type: "mixed".to_string(),
        tag: "mixed-in".to_string(),
        listen: "127.0.0.1".to_string(),
        listen_port: 2412,
        sniff: true,
        users: vec![],
        set_system_proxy: false,
    };

    let inbounds = vec![Inbound::Tun(tun_inbound), Inbound::Mixed(mixed_inbound)];

    let vless_outbound = VlessOutbound {
        r#type: "vless".to_string(),
        tag: "vless-out".to_string(),
        server: app_config.client_config_endpoint.clone(),
        server_port: 443,
        uuid: uuid.to_string(),
        flow: "xtls-rprx-vision".to_string(),
        tls: ClientTls::builder(app_config.sing_box_server_name.clone())
            .insecure(true)
            .utls(Utls::chrome())
            .reality(ClientReality::new(
                &app_config.sing_box_private_key,
                &app_config.sing_box_short_id,
            ))
            .build(),
        packet_encoding: "xudp".to_string(),
    };

    let direct_outbound = DirectOutbound {
        r#type: "direct".to_string(),
        tag: "direct".to_string(),
    };

    let outbounds = vec![
        Outbound::Vless(vless_outbound),
        Outbound::Direct(direct_outbound),
    ];

    let route_rules = vec![
        RouteRule::DomainSuffix {
            domain_suffix: vec![".ru".to_string()],
            outbound: "direct".to_string(),
        },
        RouteRule::DomainKeyword {
            domain_keyword: routing_config.direct_route_keywords.clone(),
            outbound: "direct".to_string(),
        },
        RouteRule::Sniff {
            action: "sniff".to_string(),
            timeout: "100ms".to_string(),
        },
        RouteRule::Logical {
            r#type: "logical".to_string(),
            mode: "or".to_string(),
            rules: vec![
                LogicalRule::Protocol {
                    protocol: "dns".to_string(),
                },
                LogicalRule::Port { port: 53 },
            ],
            action: "hijack-dns".to_string(),
        },
        RouteRule::IpIsPrivate {
            ip_is_private: true,
            outbound: "direct".to_string(),
        },
    ];

    let route = RouteConfig {
        rules: route_rules,
        auto_detect_interface: true,
        override_android_vpn: true,
    };

    let sing_box_config = ClientConfig {
        log: LogConfig {
            level: "warn".to_string(),
            timestamp: false,
        },
        dns,
        inbounds,
        outbounds,
        route,
    };

    Ok(serde_json::to_value(&sing_box_config)?)
}

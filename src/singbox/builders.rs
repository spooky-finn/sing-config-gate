//! Builder patterns for sing-box TLS and Reality configurations.

use serde::{Deserialize, Serialize};

/// UTLS fingerprint configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Utls {
    pub enabled: bool,
    pub fingerprint: String,
}

impl Utls {
    /// Creates a new UTLS configuration with Chrome fingerprint.
    pub fn chrome() -> Self {
        Self {
            enabled: true,
            fingerprint: "chrome".to_string(),
        }
    }

    /// Creates a new UTLS configuration with a custom fingerprint.
    pub fn with_fingerprint(fingerprint: impl Into<String>) -> Self {
        Self {
            enabled: true,
            fingerprint: fingerprint.into(),
        }
    }
}

/// Reality configuration for client.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientReality {
    pub enabled: bool,
    pub public_key: String,
    pub short_id: String,
}

impl ClientReality {
    /// Creates a new client Reality configuration.
    pub fn new(public_key: impl Into<String>, short_id: impl Into<String>) -> Self {
        Self {
            enabled: true,
            public_key: public_key.into(),
            short_id: short_id.into(),
        }
    }
}

/// Reality configuration for server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerReality {
    pub enabled: bool,
    pub handshake: RealityHandshake,
    pub private_key: String,
    pub short_id: Vec<String>,
}

/// Reality handshake configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RealityHandshake {
    pub server: String,
    pub server_port: u16,
}

impl ServerReality {
    /// Creates a new server Reality configuration.
    pub fn new(
        private_key: impl Into<String>,
        short_id: impl Into<String>,
        server: impl Into<String>,
        server_port: u16,
    ) -> Self {
        Self {
            enabled: true,
            handshake: RealityHandshake {
                server: server.into(),
                server_port,
            },
            private_key: private_key.into(),
            short_id: vec![short_id.into()],
        }
    }
}

/// TLS configuration for client.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientTls {
    pub enabled: bool,
    pub insecure: bool,
    pub server_name: String,
    pub utls: Utls,
    pub reality: ClientReality,
}

impl ClientTls {
    /// Creates a new client TLS configuration builder.
    pub fn builder(server_name: impl Into<String>) -> ClientTlsBuilder {
        ClientTlsBuilder::new(server_name)
    }
}

/// Builder for client TLS configuration.
pub struct ClientTlsBuilder {
    server_name: String,
    insecure: bool,
    utls: Utls,
    reality: Option<ClientReality>,
}

impl ClientTlsBuilder {
    pub fn new(server_name: impl Into<String>) -> Self {
        Self {
            server_name: server_name.into(),
            insecure: false,
            utls: Utls::chrome(),
            reality: None,
        }
    }

    pub fn insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    pub fn utls(mut self, utls: Utls) -> Self {
        self.utls = utls;
        self
    }

    pub fn reality(mut self, reality: ClientReality) -> Self {
        self.reality = Some(reality);
        self
    }

    pub fn build(self) -> ClientTls {
        ClientTls {
            enabled: true,
            insecure: self.insecure,
            server_name: self.server_name,
            utls: self.utls,
            reality: self.reality.expect("Reality config is required"),
        }
    }
}

/// TLS configuration for server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerTls {
    pub enabled: bool,
    pub server_name: String,
    pub reality: ServerReality,
}

impl ServerTls {
    /// Creates a new server TLS configuration.
    pub fn new(server_name: impl Into<String>, reality: ServerReality) -> Self {
        Self {
            enabled: true,
            server_name: server_name.into(),
            reality,
        }
    }
}

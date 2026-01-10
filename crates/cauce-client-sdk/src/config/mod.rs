//! Configuration types for the Cauce Client SDK.
//!
//! This module provides the main [`ClientConfig`] type for configuring
//! client connections, along with supporting types for authentication,
//! reconnection, and TLS settings.
//!
//! ## Example
//!
//! ```rust
//! use cauce_client_sdk::{ClientConfig, AuthConfig, ReconnectConfig};
//! use cauce_core::ClientType;
//! use std::time::Duration;
//!
//! let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
//!     .client_type(ClientType::Agent)
//!     .auth(AuthConfig::bearer("my-token"))
//!     .reconnect(ReconnectConfig::default()
//!         .with_max_attempts(10))
//!     .connect_timeout(Duration::from_secs(30))
//!     .build()
//!     .expect("valid config");
//! ```

mod auth;
mod reconnect;
mod tls;

pub use auth::AuthConfig;
pub use reconnect::ReconnectConfig;
pub use tls::TlsConfig;

use crate::error::ClientError;
use cauce_core::{ClientType, Transport as TransportType};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for connecting to a Cauce Hub.
///
/// Use [`ClientConfig::builder`] to create a configuration with a fluent API.
///
/// # Example
///
/// ```rust
/// use cauce_client_sdk::{ClientConfig, AuthConfig};
/// use cauce_core::ClientType;
///
/// let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
///     .client_type(ClientType::Agent)
///     .auth(AuthConfig::api_key("secret-key"))
///     .build()
///     .expect("valid config");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Hub URL (ws://, wss://, http://, or https://).
    pub hub_url: String,

    /// Unique client identifier.
    pub client_id: String,

    /// Type of client (Adapter, Agent, or A2aAgent).
    pub client_type: ClientType,

    /// Authentication configuration.
    pub auth: Option<AuthConfig>,

    /// Preferred transport type.
    pub transport: TransportType,

    /// Reconnection configuration.
    pub reconnect: ReconnectConfig,

    /// TLS configuration.
    pub tls: Option<TlsConfig>,

    /// Connection timeout.
    #[serde(with = "duration_secs")]
    pub connect_timeout: Duration,

    /// Request timeout.
    #[serde(with = "duration_secs")]
    pub request_timeout: Duration,

    /// Keepalive ping interval.
    #[serde(with = "duration_secs")]
    pub keepalive_interval: Duration,

    /// Protocol version to use.
    pub protocol_version: String,

    /// Minimum protocol version to accept from server.
    pub min_protocol_version: String,
}

impl ClientConfig {
    /// Create a new builder for [`ClientConfig`].
    ///
    /// # Arguments
    ///
    /// * `hub_url` - The URL of the Cauce Hub (ws://, wss://, http://, https://)
    /// * `client_id` - A unique identifier for this client
    ///
    /// # Example
    ///
    /// ```rust
    /// use cauce_client_sdk::ClientConfig;
    ///
    /// let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
    ///     .build()
    ///     .expect("valid config");
    /// ```
    pub fn builder(
        hub_url: impl Into<String>,
        client_id: impl Into<String>,
    ) -> ClientConfigBuilder {
        ClientConfigBuilder::new(hub_url, client_id)
    }

    /// Validate the configuration.
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<(), ClientError> {
        // Validate URL format
        if self.hub_url.is_empty() {
            return Err(ClientError::config_error("hub_url cannot be empty"));
        }

        // Check URL scheme
        let valid_schemes = ["ws://", "wss://", "http://", "https://"];
        if !valid_schemes.iter().any(|s| self.hub_url.starts_with(s)) {
            return Err(ClientError::config_error(format!(
                "hub_url must start with one of: {}",
                valid_schemes.join(", ")
            )));
        }

        // Validate client_id
        if self.client_id.is_empty() {
            return Err(ClientError::config_error("client_id cannot be empty"));
        }

        // Validate TLS config if present
        if let Some(ref tls) = self.tls {
            tls.validate()
                .map_err(|e| ClientError::config_error(format!("TLS config error: {}", e)))?;
        }

        Ok(())
    }

    /// Returns the WebSocket URL for this configuration.
    ///
    /// Converts http:// to ws:// and https:// to wss:// if necessary.
    pub fn websocket_url(&self) -> String {
        if let Some(rest) = self.hub_url.strip_prefix("http://") {
            format!("ws://{}", rest)
        } else if let Some(rest) = self.hub_url.strip_prefix("https://") {
            format!("wss://{}", rest)
        } else {
            self.hub_url.clone()
        }
    }

    /// Returns the HTTP URL for this configuration.
    ///
    /// Converts ws:// to http:// and wss:// to https:// if necessary.
    pub fn http_url(&self) -> String {
        if let Some(rest) = self.hub_url.strip_prefix("ws://") {
            format!("http://{}", rest)
        } else if let Some(rest) = self.hub_url.strip_prefix("wss://") {
            format!("https://{}", rest)
        } else {
            self.hub_url.clone()
        }
    }

    /// Returns true if the connection should use TLS.
    pub fn is_secure(&self) -> bool {
        self.hub_url.starts_with("wss://") || self.hub_url.starts_with("https://")
    }
}

/// Builder for [`ClientConfig`].
///
/// Created via [`ClientConfig::builder`].
#[derive(Debug, Clone)]
pub struct ClientConfigBuilder {
    hub_url: String,
    client_id: String,
    client_type: ClientType,
    auth: Option<AuthConfig>,
    transport: TransportType,
    reconnect: ReconnectConfig,
    tls: Option<TlsConfig>,
    connect_timeout: Duration,
    request_timeout: Duration,
    keepalive_interval: Duration,
    protocol_version: String,
    min_protocol_version: String,
}

impl ClientConfigBuilder {
    /// Create a new builder with required fields.
    fn new(hub_url: impl Into<String>, client_id: impl Into<String>) -> Self {
        Self {
            hub_url: hub_url.into(),
            client_id: client_id.into(),
            client_type: ClientType::Agent,
            auth: None,
            transport: TransportType::WebSocket,
            reconnect: ReconnectConfig::default(),
            tls: None,
            connect_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            keepalive_interval: Duration::from_secs(30),
            protocol_version: "1.0".to_string(),
            min_protocol_version: "1.0".to_string(),
        }
    }

    /// Set the client type.
    pub fn client_type(mut self, client_type: ClientType) -> Self {
        self.client_type = client_type;
        self
    }

    /// Set the authentication configuration.
    pub fn auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set the preferred transport type.
    pub fn transport(mut self, transport: TransportType) -> Self {
        self.transport = transport;
        self
    }

    /// Set the reconnection configuration.
    pub fn reconnect(mut self, reconnect: ReconnectConfig) -> Self {
        self.reconnect = reconnect;
        self
    }

    /// Set the TLS configuration.
    pub fn tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Set the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the request timeout.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the keepalive ping interval.
    pub fn keepalive_interval(mut self, interval: Duration) -> Self {
        self.keepalive_interval = interval;
        self
    }

    /// Set the protocol version.
    pub fn protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }

    /// Set the minimum protocol version to accept.
    pub fn min_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.min_protocol_version = version.into();
        self
    }

    /// Build the configuration.
    ///
    /// Returns an error if the configuration is invalid.
    pub fn build(self) -> Result<ClientConfig, ClientError> {
        let config = ClientConfig {
            hub_url: self.hub_url,
            client_id: self.client_id,
            client_type: self.client_type,
            auth: self.auth,
            transport: self.transport,
            reconnect: self.reconnect,
            tls: self.tls,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            keepalive_interval: self.keepalive_interval,
            protocol_version: self.protocol_version,
            min_protocol_version: self.min_protocol_version,
        };

        config.validate()?;
        Ok(config)
    }
}

/// Serde helper for serializing Duration as seconds.
mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_minimal() {
        let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
            .build()
            .expect("should build");

        assert_eq!(config.hub_url, "wss://hub.example.com");
        assert_eq!(config.client_id, "my-agent");
        assert_eq!(config.client_type, ClientType::Agent);
        assert!(config.auth.is_none());
    }

    #[test]
    fn test_builder_full() {
        let config = ClientConfig::builder("wss://hub.example.com", "my-adapter")
            .client_type(ClientType::Adapter)
            .auth(AuthConfig::api_key("secret"))
            .transport(TransportType::WebSocket)
            .reconnect(ReconnectConfig::default().with_max_attempts(5))
            .tls(TlsConfig::default())
            .connect_timeout(Duration::from_secs(60))
            .request_timeout(Duration::from_secs(120))
            .keepalive_interval(Duration::from_secs(45))
            .protocol_version("1.1")
            .build()
            .expect("should build");

        assert_eq!(config.client_type, ClientType::Adapter);
        assert!(config.auth.is_some());
        assert_eq!(config.connect_timeout, Duration::from_secs(60));
        assert_eq!(config.request_timeout, Duration::from_secs(120));
        assert_eq!(config.keepalive_interval, Duration::from_secs(45));
        assert_eq!(config.protocol_version, "1.1");
    }

    #[test]
    fn test_validate_empty_url() {
        let result = ClientConfig::builder("", "my-agent").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_scheme() {
        let result = ClientConfig::builder("ftp://hub.example.com", "my-agent").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_client_id() {
        let result = ClientConfig::builder("wss://hub.example.com", "").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_url_conversion() {
        let config = ClientConfig::builder("https://hub.example.com/cauce", "agent")
            .build()
            .unwrap();
        assert_eq!(config.websocket_url(), "wss://hub.example.com/cauce");

        let config = ClientConfig::builder("http://localhost:8080", "agent")
            .build()
            .unwrap();
        assert_eq!(config.websocket_url(), "ws://localhost:8080");

        let config = ClientConfig::builder("wss://hub.example.com", "agent")
            .build()
            .unwrap();
        assert_eq!(config.websocket_url(), "wss://hub.example.com");
    }

    #[test]
    fn test_http_url_conversion() {
        let config = ClientConfig::builder("wss://hub.example.com/cauce", "agent")
            .build()
            .unwrap();
        assert_eq!(config.http_url(), "https://hub.example.com/cauce");

        let config = ClientConfig::builder("ws://localhost:8080", "agent")
            .build()
            .unwrap();
        assert_eq!(config.http_url(), "http://localhost:8080");
    }

    #[test]
    fn test_is_secure() {
        let config = ClientConfig::builder("wss://hub.example.com", "agent")
            .build()
            .unwrap();
        assert!(config.is_secure());

        let config = ClientConfig::builder("https://hub.example.com", "agent")
            .build()
            .unwrap();
        assert!(config.is_secure());

        let config = ClientConfig::builder("ws://localhost", "agent")
            .build()
            .unwrap();
        assert!(!config.is_secure());
    }

    #[test]
    fn test_serialization() {
        let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
            .auth(AuthConfig::bearer("token"))
            .build()
            .unwrap();

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("wss://hub.example.com"));
        assert!(json.contains("my-agent"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "hub_url": "wss://hub.example.com",
            "client_id": "test-agent",
            "client_type": "agent",
            "auth": null,
            "transport": "websocket",
            "reconnect": {
                "enabled": true,
                "initial_delay": 100,
                "max_delay": 30000,
                "backoff_multiplier": 2.0,
                "max_attempts": null,
                "jitter": true
            },
            "tls": null,
            "connect_timeout": 30,
            "request_timeout": 60,
            "keepalive_interval": 30,
            "protocol_version": "1.0",
            "min_protocol_version": "1.0"
        }"#;

        let config: ClientConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.hub_url, "wss://hub.example.com");
        assert_eq!(config.client_id, "test-agent");
        assert_eq!(config.client_type, ClientType::Agent);
    }
}

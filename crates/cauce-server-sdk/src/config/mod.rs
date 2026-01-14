//! Server configuration for the Cauce Server SDK.
//!
//! This module provides configuration types for setting up a Cauce server,
//! including transport settings, resource limits, and redelivery policies.
//!
//! # Example
//!
//! ```
//! use cauce_server_sdk::config::ServerConfig;
//! use std::net::SocketAddr;
//!
//! let config = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(config.address.port(), 8080);
//! ```

mod limits;
mod redelivery;
mod transports;

pub use limits::LimitsConfig;
pub use redelivery::RedeliveryConfig;
pub use transports::TransportsConfig;

use crate::error::{ServerError, ServerResult};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// TLS configuration for secure connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to the TLS certificate file.
    pub cert_path: PathBuf,

    /// Path to the TLS private key file.
    pub key_path: PathBuf,

    /// Whether to require client certificates (mTLS).
    #[serde(default)]
    pub require_client_cert: bool,

    /// Path to CA certificate for client verification.
    #[serde(default)]
    pub ca_cert_path: Option<PathBuf>,
}

impl TlsConfig {
    /// Create a new TLS config with the given certificate and key paths.
    pub fn new(cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            require_client_cert: false,
            ca_cert_path: None,
        }
    }

    /// Enable mutual TLS (mTLS) with the given CA certificate.
    pub fn with_mtls(mut self, ca_cert_path: impl Into<PathBuf>) -> Self {
        self.require_client_cert = true;
        self.ca_cert_path = Some(ca_cert_path.into());
        self
    }

    /// Validate the TLS configuration.
    pub fn validate(&self) -> ServerResult<()> {
        if !self.cert_path.exists() {
            return Err(ServerError::config_error(format!(
                "TLS certificate not found: {:?}",
                self.cert_path
            )));
        }
        if !self.key_path.exists() {
            return Err(ServerError::config_error(format!(
                "TLS key not found: {:?}",
                self.key_path
            )));
        }
        if self.require_client_cert {
            if let Some(ref ca_path) = self.ca_cert_path {
                if !ca_path.exists() {
                    return Err(ServerError::config_error(format!(
                        "CA certificate not found: {:?}",
                        ca_path
                    )));
                }
            } else {
                return Err(ServerError::config_error(
                    "mTLS requires a CA certificate path",
                ));
            }
        }
        Ok(())
    }
}

/// Authentication configuration for API access.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is required.
    #[serde(default)]
    pub required: bool,

    /// Static API keys (for simple deployments).
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Whether to accept bearer tokens.
    #[serde(default)]
    pub accept_bearer: bool,
}

impl AuthConfig {
    /// Create a config that doesn't require authentication.
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a config that requires API key authentication.
    pub fn require_api_key(keys: Vec<String>) -> Self {
        Self {
            required: true,
            api_keys: keys,
            accept_bearer: false,
        }
    }

    /// Create a config that accepts bearer tokens.
    pub fn accept_bearer() -> Self {
        Self {
            required: true,
            api_keys: Vec::new(),
            accept_bearer: true,
        }
    }

    /// Add an API key.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_keys.push(key.into());
        self
    }

    /// Check if authentication is required.
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// Validate an API key.
    pub fn validate_api_key(&self, key: &str) -> bool {
        self.api_keys.iter().any(|k| k == key)
    }
}

/// Main server configuration.
///
/// Use [`ServerConfigBuilder`] to construct this configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to bind the server to.
    pub address: SocketAddr,

    /// Optional TLS configuration for HTTPS.
    #[serde(default)]
    pub tls: Option<TlsConfig>,

    /// Enabled transports.
    #[serde(default)]
    pub transports: TransportsConfig,

    /// Resource limits.
    #[serde(default)]
    pub limits: LimitsConfig,

    /// Authentication settings.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Redelivery settings for unacked signals.
    #[serde(default)]
    pub redelivery: RedeliveryConfig,

    /// Server name for identification.
    #[serde(default = "default_server_name")]
    pub server_name: String,
}

fn default_server_name() -> String {
    "cauce-hub".to_string()
}

impl ServerConfig {
    /// Create a new configuration builder.
    pub fn builder(address: SocketAddr) -> ServerConfigBuilder {
        ServerConfigBuilder::new(address)
    }

    /// Create a development configuration.
    ///
    /// This configuration:
    /// - Binds to localhost:8080
    /// - Disables TLS
    /// - Disables authentication
    /// - Disables rate limiting
    /// - Enables all transports
    pub fn development() -> Self {
        Self {
            address: "127.0.0.1:8080".parse().unwrap(),
            tls: None,
            transports: TransportsConfig::all(),
            limits: LimitsConfig::development(),
            auth: AuthConfig::none(),
            redelivery: RedeliveryConfig::default(),
            server_name: "cauce-hub-dev".to_string(),
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> ServerResult<()> {
        if let Some(ref tls) = self.tls {
            tls.validate()?;
        }

        if !self.transports.any_enabled() {
            return Err(ServerError::config_error(
                "At least one transport must be enabled",
            ));
        }

        if self.limits.max_connections == 0 {
            return Err(ServerError::config_error(
                "max_connections must be greater than 0",
            ));
        }

        Ok(())
    }

    /// Check if TLS is enabled.
    pub fn is_tls_enabled(&self) -> bool {
        self.tls.is_some()
    }

    /// Get the server URL scheme (http or https).
    pub fn scheme(&self) -> &'static str {
        if self.is_tls_enabled() {
            "https"
        } else {
            "http"
        }
    }

    /// Get the full base URL for the server.
    pub fn base_url(&self) -> String {
        format!("{}://{}", self.scheme(), self.address)
    }
}

/// Builder for [`ServerConfig`].
#[derive(Debug)]
pub struct ServerConfigBuilder {
    address: SocketAddr,
    tls: Option<TlsConfig>,
    transports: TransportsConfig,
    limits: LimitsConfig,
    auth: AuthConfig,
    redelivery: RedeliveryConfig,
    server_name: String,
}

impl ServerConfigBuilder {
    /// Create a new builder with the given address.
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            tls: None,
            transports: TransportsConfig::default(),
            limits: LimitsConfig::default(),
            auth: AuthConfig::default(),
            redelivery: RedeliveryConfig::default(),
            server_name: default_server_name(),
        }
    }

    /// Set TLS configuration.
    pub fn tls(mut self, config: TlsConfig) -> Self {
        self.tls = Some(config);
        self
    }

    /// Set transports configuration.
    pub fn transports(mut self, config: TransportsConfig) -> Self {
        self.transports = config;
        self
    }

    /// Set limits configuration.
    pub fn limits(mut self, config: LimitsConfig) -> Self {
        self.limits = config;
        self
    }

    /// Set authentication configuration.
    pub fn auth(mut self, config: AuthConfig) -> Self {
        self.auth = config;
        self
    }

    /// Set redelivery configuration.
    pub fn redelivery(mut self, config: RedeliveryConfig) -> Self {
        self.redelivery = config;
        self
    }

    /// Set the server name.
    pub fn server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = name.into();
        self
    }

    /// Build and validate the configuration.
    pub fn build(self) -> ServerResult<ServerConfig> {
        let config = ServerConfig {
            address: self.address,
            tls: self.tls,
            transports: self.transports,
            limits: self.limits,
            auth: self.auth,
            redelivery: self.redelivery,
            server_name: self.server_name,
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_minimal() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let config = ServerConfig::builder(addr).build().unwrap();

        assert_eq!(config.address.port(), 8080);
        assert!(config.tls.is_none());
        assert!(config.transports.websocket_enabled);
        assert!(!config.is_tls_enabled());
    }

    #[test]
    fn test_builder_full() {
        let addr: SocketAddr = "0.0.0.0:443".parse().unwrap();
        let config = ServerConfig::builder(addr)
            .transports(TransportsConfig::websocket_only())
            .limits(LimitsConfig::production())
            .auth(AuthConfig::require_api_key(vec!["secret".to_string()]))
            .redelivery(RedeliveryConfig::aggressive())
            .server_name("my-hub")
            .build()
            .unwrap();

        assert_eq!(config.address.port(), 443);
        assert!(config.transports.websocket_enabled);
        assert!(!config.transports.sse_enabled);
        assert_eq!(config.limits.max_connections, 50_000);
        assert!(config.auth.is_required());
        assert_eq!(config.server_name, "my-hub");
    }

    #[test]
    fn test_development_config() {
        let config = ServerConfig::development();

        assert_eq!(config.address.port(), 8080);
        assert!(config.tls.is_none());
        assert!(!config.auth.is_required());
        assert!(!config.limits.is_rate_limited());
    }

    #[test]
    fn test_validation_no_transports() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let result = ServerConfig::builder(addr)
            .transports(TransportsConfig::none())
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("transport"));
    }

    #[test]
    fn test_base_url() {
        let config = ServerConfig::development();
        assert_eq!(config.base_url(), "http://127.0.0.1:8080");
        assert_eq!(config.scheme(), "http");
    }

    #[test]
    fn test_auth_config() {
        let auth = AuthConfig::require_api_key(vec!["key1".to_string()])
            .with_api_key("key2");

        assert!(auth.is_required());
        assert!(auth.validate_api_key("key1"));
        assert!(auth.validate_api_key("key2"));
        assert!(!auth.validate_api_key("invalid"));
    }

    #[test]
    fn test_serialization() {
        let config = ServerConfig::development();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: ServerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.address, parsed.address);
        assert_eq!(config.server_name, parsed.server_name);
    }

    #[test]
    fn test_tls_config_creation() {
        let tls = TlsConfig::new("/path/to/cert.pem", "/path/to/key.pem");
        assert_eq!(tls.cert_path.to_str().unwrap(), "/path/to/cert.pem");
        assert_eq!(tls.key_path.to_str().unwrap(), "/path/to/key.pem");
        assert!(!tls.require_client_cert);
        assert!(tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_tls_config_with_mtls() {
        let tls = TlsConfig::new("/path/to/cert.pem", "/path/to/key.pem")
            .with_mtls("/path/to/ca.pem");
        assert!(tls.require_client_cert);
        assert!(tls.ca_cert_path.is_some());
    }

    #[test]
    fn test_tls_config_validate_cert_not_found() {
        let tls = TlsConfig::new("/nonexistent/cert.pem", "/path/to/key.pem");
        let result = tls.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("certificate"));
    }

    #[test]
    fn test_tls_config_validate_mtls_no_ca() {
        // Create a temp file for cert and key
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let cert_path = temp_dir.join("test_cert.pem");
        let key_path = temp_dir.join("test_key.pem");

        std::fs::File::create(&cert_path).unwrap().write_all(b"cert").unwrap();
        std::fs::File::create(&key_path).unwrap().write_all(b"key").unwrap();

        let mut tls = TlsConfig::new(&cert_path, &key_path);
        tls.require_client_cert = true;
        tls.ca_cert_path = None;

        let result = tls.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CA certificate"));

        // Clean up
        let _ = std::fs::remove_file(&cert_path);
        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn test_auth_config_none() {
        let auth = AuthConfig::none();
        assert!(!auth.is_required());
    }

    #[test]
    fn test_auth_config_accept_bearer() {
        let auth = AuthConfig::accept_bearer();
        // accept_bearer creates a config that requires auth
        assert!(auth.is_required());
        assert!(auth.accept_bearer);
    }

    #[test]
    fn test_auth_config_with_api_key_builder() {
        let auth = AuthConfig::none().with_api_key("key123");
        assert!(auth.api_keys.contains(&"key123".to_string()));
    }

    #[test]
    fn test_server_config_clone() {
        let config = ServerConfig::development();
        let cloned = config.clone();
        assert_eq!(cloned.address, config.address);
    }

    #[test]
    fn test_server_config_debug() {
        let config = ServerConfig::development();
        let debug = format!("{:?}", config);
        assert!(debug.contains("127.0.0.1"));
    }

    #[test]
    fn test_builder_with_server_name() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let config = ServerConfig::builder(addr)
            .server_name("test-server")
            .build()
            .unwrap();

        assert_eq!(config.server_name, "test-server");
    }

    #[test]
    fn test_tls_config_serialization() {
        let tls = TlsConfig::new("/path/to/cert.pem", "/path/to/key.pem");
        let json = serde_json::to_string(&tls).unwrap();
        assert!(json.contains("cert_path"));
        assert!(json.contains("key_path"));
    }

    #[test]
    fn test_auth_config_serialization() {
        let auth = AuthConfig::require_api_key(vec!["key1".to_string()]);
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("key1"));
    }

    #[test]
    fn test_server_config_builder_builds_twice() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let config1 = ServerConfigBuilder::new(addr).build().unwrap();
        let config2 = ServerConfigBuilder::new(addr).build().unwrap();
        assert_eq!(config1.address, config2.address);
    }

    #[test]
    fn test_auth_config_clone() {
        let auth = AuthConfig::require_api_key(vec!["key".to_string()]);
        let cloned = auth.clone();
        assert!(cloned.validate_api_key("key"));
    }

    #[test]
    fn test_auth_config_debug() {
        let auth = AuthConfig::none();
        let debug = format!("{:?}", auth);
        assert!(debug.contains("AuthConfig"));
    }
}

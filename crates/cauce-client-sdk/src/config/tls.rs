//! TLS configuration for the Cauce Client SDK.
//!
//! This module provides types for configuring TLS/SSL connections
//! when connecting to a Cauce Hub.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS/SSL configuration for secure connections.
///
/// # Example
///
/// ```rust
/// use cauce_client_sdk::TlsConfig;
///
/// // Default TLS configuration (validates certificates)
/// let tls = TlsConfig::default();
///
/// // Development configuration (allows self-signed certs)
/// let tls = TlsConfig::insecure();
///
/// // Client certificate authentication
/// let tls = TlsConfig::default()
///     .with_client_cert("./client.crt", "./client.key");
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Accept invalid certificates (self-signed, expired, wrong hostname).
    ///
    /// **Warning**: This should only be used in development environments.
    /// Never enable this in production as it makes the connection
    /// vulnerable to man-in-the-middle attacks.
    #[serde(default)]
    pub accept_invalid_certs: bool,

    /// Path to a custom CA certificate file (PEM format).
    ///
    /// Use this to trust a private CA or self-signed certificate
    /// without disabling certificate validation entirely.
    pub ca_cert: Option<PathBuf>,

    /// Path to the client certificate file (PEM format).
    ///
    /// Required for mTLS (mutual TLS) authentication.
    pub client_cert: Option<PathBuf>,

    /// Path to the client private key file (PEM format).
    ///
    /// Required for mTLS (mutual TLS) authentication.
    pub client_key: Option<PathBuf>,
}

impl TlsConfig {
    /// Create a new TLS configuration with default settings.
    ///
    /// This validates certificates and does not use client certificates.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an insecure TLS configuration for development.
    ///
    /// **Warning**: This accepts invalid certificates and should never
    /// be used in production.
    pub fn insecure() -> Self {
        Self {
            accept_invalid_certs: true,
            ..Self::default()
        }
    }

    /// Set whether to accept invalid certificates.
    ///
    /// **Warning**: Setting this to `true` should only be done in
    /// development environments.
    pub fn with_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Set a custom CA certificate for validation.
    ///
    /// Use this to trust a private CA or self-signed certificate.
    pub fn with_ca_cert(mut self, path: impl Into<PathBuf>) -> Self {
        self.ca_cert = Some(path.into());
        self
    }

    /// Set client certificate and key for mTLS authentication.
    pub fn with_client_cert(
        mut self,
        cert_path: impl Into<PathBuf>,
        key_path: impl Into<PathBuf>,
    ) -> Self {
        self.client_cert = Some(cert_path.into());
        self.client_key = Some(key_path.into());
        self
    }

    /// Returns true if mTLS client certificates are configured.
    pub fn has_client_cert(&self) -> bool {
        self.client_cert.is_some() && self.client_key.is_some()
    }

    /// Validate the TLS configuration.
    ///
    /// Returns an error if the configuration is invalid (e.g., client cert
    /// without client key).
    pub fn validate(&self) -> Result<(), String> {
        // Check that both cert and key are provided together
        match (&self.client_cert, &self.client_key) {
            (Some(_), None) => {
                return Err("client_cert specified without client_key".to_string());
            }
            (None, Some(_)) => {
                return Err("client_key specified without client_cert".to_string());
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TlsConfig::default();
        assert!(!config.accept_invalid_certs);
        assert!(config.ca_cert.is_none());
        assert!(config.client_cert.is_none());
        assert!(config.client_key.is_none());
    }

    #[test]
    fn test_insecure_config() {
        let config = TlsConfig::insecure();
        assert!(config.accept_invalid_certs);
    }

    #[test]
    fn test_builder_methods() {
        let config = TlsConfig::new()
            .with_accept_invalid_certs(true)
            .with_ca_cert("/path/to/ca.crt")
            .with_client_cert("/path/to/client.crt", "/path/to/client.key");

        assert!(config.accept_invalid_certs);
        assert_eq!(config.ca_cert, Some(PathBuf::from("/path/to/ca.crt")));
        assert_eq!(config.client_cert, Some(PathBuf::from("/path/to/client.crt")));
        assert_eq!(config.client_key, Some(PathBuf::from("/path/to/client.key")));
    }

    #[test]
    fn test_has_client_cert() {
        let config = TlsConfig::default();
        assert!(!config.has_client_cert());

        let config = TlsConfig::new().with_client_cert("cert.pem", "key.pem");
        assert!(config.has_client_cert());
    }

    #[test]
    fn test_validate_success() {
        // Default is valid
        assert!(TlsConfig::default().validate().is_ok());

        // With client cert and key is valid
        let config = TlsConfig::new().with_client_cert("cert.pem", "key.pem");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_cert_without_key() {
        let config = TlsConfig {
            client_cert: Some(PathBuf::from("cert.pem")),
            client_key: None,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_key_without_cert() {
        let config = TlsConfig {
            client_cert: None,
            client_key: Some(PathBuf::from("key.pem")),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = TlsConfig::new()
            .with_accept_invalid_certs(true)
            .with_ca_cert("/path/to/ca.crt");

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"accept_invalid_certs\":true"));
        assert!(json.contains("\"ca_cert\":\"/path/to/ca.crt\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "accept_invalid_certs": false,
            "ca_cert": "/custom/ca.crt",
            "client_cert": "/path/to/client.crt",
            "client_key": "/path/to/client.key"
        }"#;

        let config: TlsConfig = serde_json::from_str(json).unwrap();
        assert!(!config.accept_invalid_certs);
        assert_eq!(config.ca_cert, Some(PathBuf::from("/custom/ca.crt")));
        assert_eq!(config.client_cert, Some(PathBuf::from("/path/to/client.crt")));
        assert_eq!(config.client_key, Some(PathBuf::from("/path/to/client.key")));
    }
}

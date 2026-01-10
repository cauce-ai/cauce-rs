//! Transport configuration types for the Cauce Protocol.
//!
//! This module provides types for configuring how signals are delivered to clients.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::EncryptionAlgorithm;

/// Transport method for delivering signals to clients.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `WebSocket` → `"websocket"`
/// - `Sse` → `"sse"`
/// - `Polling` → `"polling"`
/// - `LongPolling` → `"long_polling"`
/// - `Webhook` → `"webhook"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    /// WebSocket connection (bidirectional, persistent)
    #[serde(rename = "websocket")]
    WebSocket,
    /// Server-Sent Events (server-to-client only)
    Sse,
    /// Short polling (client pulls periodically)
    Polling,
    /// Long polling (client waits for response)
    LongPolling,
    /// Webhook callbacks (server pushes to client URL)
    Webhook,
}

/// Webhook configuration for signal delivery.
///
/// Used when `Transport::Webhook` is selected.
///
/// # Example
///
/// ```
/// use cauce_core::methods::WebhookConfig;
/// use std::collections::HashMap;
///
/// let config = WebhookConfig {
///     url: "https://example.com/webhook".to_string(),
///     secret: Some("webhook-secret".to_string()),
///     headers: Some(HashMap::from([
///         ("X-Custom-Header".to_string(), "value".to_string()),
///     ])),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// The URL to send webhook requests to
    pub url: String,

    /// Optional secret for webhook signature verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,

    /// Optional custom headers to include in webhook requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

impl WebhookConfig {
    /// Creates a new webhook configuration with just a URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            secret: None,
            headers: None,
        }
    }

    /// Creates a new webhook configuration with a URL and secret.
    pub fn with_secret(url: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            secret: Some(secret.into()),
            headers: None,
        }
    }
}

/// End-to-end encryption configuration.
///
/// Used to negotiate E2E encryption support during subscription.
///
/// # Example
///
/// ```
/// use cauce_core::methods::E2eConfig;
/// use cauce_core::types::EncryptionAlgorithm;
///
/// let config = E2eConfig {
///     enabled: true,
///     public_key: Some("base64-encoded-public-key".to_string()),
///     supported_algorithms: vec![
///         EncryptionAlgorithm::X25519XSalsa20Poly1305,
///         EncryptionAlgorithm::A256Gcm,
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct E2eConfig {
    /// Whether E2E encryption is enabled
    pub enabled: bool,

    /// The client's public key for key exchange (base64-encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// List of supported encryption algorithms
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_algorithms: Vec<EncryptionAlgorithm>,
}

impl E2eConfig {
    /// Creates a new E2E config with encryption enabled.
    pub fn enabled(public_key: impl Into<String>) -> Self {
        Self {
            enabled: true,
            public_key: Some(public_key.into()),
            supported_algorithms: vec![],
        }
    }

    /// Creates a new E2E config with encryption disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            public_key: None,
            supported_algorithms: vec![],
        }
    }

    /// Adds a supported algorithm.
    pub fn with_algorithm(mut self, algorithm: EncryptionAlgorithm) -> Self {
        self.supported_algorithms.push(algorithm);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Transport Tests =====

    #[test]
    fn test_transport_serialization() {
        assert_eq!(
            serde_json::to_string(&Transport::WebSocket).unwrap(),
            "\"websocket\""
        );
        assert_eq!(serde_json::to_string(&Transport::Sse).unwrap(), "\"sse\"");
        assert_eq!(
            serde_json::to_string(&Transport::Polling).unwrap(),
            "\"polling\""
        );
        assert_eq!(
            serde_json::to_string(&Transport::LongPolling).unwrap(),
            "\"long_polling\""
        );
        assert_eq!(
            serde_json::to_string(&Transport::Webhook).unwrap(),
            "\"webhook\""
        );
    }

    #[test]
    fn test_transport_deserialization() {
        assert_eq!(
            serde_json::from_str::<Transport>("\"websocket\"").unwrap(),
            Transport::WebSocket
        );
        assert_eq!(
            serde_json::from_str::<Transport>("\"sse\"").unwrap(),
            Transport::Sse
        );
        assert_eq!(
            serde_json::from_str::<Transport>("\"polling\"").unwrap(),
            Transport::Polling
        );
        assert_eq!(
            serde_json::from_str::<Transport>("\"long_polling\"").unwrap(),
            Transport::LongPolling
        );
        assert_eq!(
            serde_json::from_str::<Transport>("\"webhook\"").unwrap(),
            Transport::Webhook
        );
    }

    #[test]
    fn test_transport_roundtrip() {
        for transport in [
            Transport::WebSocket,
            Transport::Sse,
            Transport::Polling,
            Transport::LongPolling,
            Transport::Webhook,
        ] {
            let json = serde_json::to_string(&transport).unwrap();
            let restored: Transport = serde_json::from_str(&json).unwrap();
            assert_eq!(transport, restored);
        }
    }

    // ===== WebhookConfig Tests =====

    #[test]
    fn test_webhook_config_new() {
        let config = WebhookConfig::new("https://example.com/webhook");
        assert_eq!(config.url, "https://example.com/webhook");
        assert!(config.secret.is_none());
        assert!(config.headers.is_none());
    }

    #[test]
    fn test_webhook_config_with_secret() {
        let config = WebhookConfig::with_secret("https://example.com/webhook", "my-secret");
        assert_eq!(config.url, "https://example.com/webhook");
        assert_eq!(config.secret, Some("my-secret".to_string()));
    }

    #[test]
    fn test_webhook_config_serialization() {
        let config = WebhookConfig::new("https://example.com");
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"url\":\"https://example.com\""));
        assert!(!json.contains("\"secret\"")); // Should be omitted
        assert!(!json.contains("\"headers\"")); // Should be omitted
    }

    #[test]
    fn test_webhook_config_serialization_with_all_fields() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());

        let config = WebhookConfig {
            url: "https://example.com".to_string(),
            secret: Some("secret123".to_string()),
            headers: Some(headers),
        };
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"url\":\"https://example.com\""));
        assert!(json.contains("\"secret\":\"secret123\""));
        assert!(json.contains("\"headers\""));
        assert!(json.contains("\"X-Custom\":\"value\""));
    }

    #[test]
    fn test_webhook_config_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            secret: Some("webhook-secret".to_string()),
            headers: Some(headers),
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: WebhookConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, restored);
    }

    // ===== E2eConfig Tests =====

    #[test]
    fn test_e2e_config_enabled() {
        let config = E2eConfig::enabled("my-public-key");
        assert!(config.enabled);
        assert_eq!(config.public_key, Some("my-public-key".to_string()));
        assert!(config.supported_algorithms.is_empty());
    }

    #[test]
    fn test_e2e_config_disabled() {
        let config = E2eConfig::disabled();
        assert!(!config.enabled);
        assert!(config.public_key.is_none());
    }

    #[test]
    fn test_e2e_config_with_algorithm() {
        let config = E2eConfig::enabled("key")
            .with_algorithm(EncryptionAlgorithm::X25519XSalsa20Poly1305)
            .with_algorithm(EncryptionAlgorithm::A256Gcm);

        assert_eq!(config.supported_algorithms.len(), 2);
        assert_eq!(
            config.supported_algorithms[0],
            EncryptionAlgorithm::X25519XSalsa20Poly1305
        );
        assert_eq!(config.supported_algorithms[1], EncryptionAlgorithm::A256Gcm);
    }

    #[test]
    fn test_e2e_config_serialization() {
        let config = E2eConfig::disabled();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"enabled\":false"));
        assert!(!json.contains("\"public_key\"")); // Should be omitted
        assert!(!json.contains("\"supported_algorithms\"")); // Should be omitted when empty
    }

    #[test]
    fn test_e2e_config_serialization_enabled() {
        let config = E2eConfig::enabled("pubkey123").with_algorithm(EncryptionAlgorithm::A256Gcm);
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"public_key\":\"pubkey123\""));
        assert!(json.contains("\"supported_algorithms\""));
        assert!(json.contains("\"a256gcm\""));
    }

    #[test]
    fn test_e2e_config_roundtrip() {
        let config = E2eConfig::enabled("test-key")
            .with_algorithm(EncryptionAlgorithm::X25519XSalsa20Poly1305);

        let json = serde_json::to_string(&config).unwrap();
        let restored: E2eConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, restored);
    }
}

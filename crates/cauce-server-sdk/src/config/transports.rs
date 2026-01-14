//! Transport configuration for the Cauce server.
//!
//! This module defines which transports are enabled for client connections.

use serde::{Deserialize, Serialize};

/// Configuration for enabled transports.
///
/// # Example
///
/// ```
/// use cauce_server_sdk::config::TransportsConfig;
///
/// let config = TransportsConfig::default();
/// assert!(config.websocket_enabled);
/// assert!(config.sse_enabled);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportsConfig {
    /// Enable WebSocket transport (recommended, full-duplex).
    #[serde(default = "default_true")]
    pub websocket_enabled: bool,

    /// Enable Server-Sent Events transport (server-to-client streaming).
    #[serde(default = "default_true")]
    pub sse_enabled: bool,

    /// Enable HTTP polling transport (short and long polling).
    #[serde(default = "default_true")]
    pub polling_enabled: bool,

    /// Enable webhook delivery transport (server pushes to client URL).
    #[serde(default = "default_true")]
    pub webhook_enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for TransportsConfig {
    fn default() -> Self {
        Self {
            websocket_enabled: true,
            sse_enabled: true,
            polling_enabled: true,
            webhook_enabled: true,
        }
    }
}

impl TransportsConfig {
    /// Create a new config with all transports enabled.
    pub fn all() -> Self {
        Self::default()
    }

    /// Create a config with only WebSocket enabled.
    pub fn websocket_only() -> Self {
        Self {
            websocket_enabled: true,
            sse_enabled: false,
            polling_enabled: false,
            webhook_enabled: false,
        }
    }

    /// Create a config with no transports enabled.
    pub fn none() -> Self {
        Self {
            websocket_enabled: false,
            sse_enabled: false,
            polling_enabled: false,
            webhook_enabled: false,
        }
    }

    /// Enable or disable WebSocket transport.
    pub fn with_websocket(mut self, enabled: bool) -> Self {
        self.websocket_enabled = enabled;
        self
    }

    /// Enable or disable SSE transport.
    pub fn with_sse(mut self, enabled: bool) -> Self {
        self.sse_enabled = enabled;
        self
    }

    /// Enable or disable polling transport.
    pub fn with_polling(mut self, enabled: bool) -> Self {
        self.polling_enabled = enabled;
        self
    }

    /// Enable or disable webhook transport.
    pub fn with_webhook(mut self, enabled: bool) -> Self {
        self.webhook_enabled = enabled;
        self
    }

    /// Check if any transport is enabled.
    pub fn any_enabled(&self) -> bool {
        self.websocket_enabled
            || self.sse_enabled
            || self.polling_enabled
            || self.webhook_enabled
    }

    /// Get the number of enabled transports.
    pub fn enabled_count(&self) -> usize {
        [
            self.websocket_enabled,
            self.sse_enabled,
            self.polling_enabled,
            self.webhook_enabled,
        ]
        .iter()
        .filter(|&&enabled| enabled)
        .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_all_enabled() {
        let config = TransportsConfig::default();
        assert!(config.websocket_enabled);
        assert!(config.sse_enabled);
        assert!(config.polling_enabled);
        assert!(config.webhook_enabled);
        assert!(config.any_enabled());
        assert_eq!(config.enabled_count(), 4);
    }

    #[test]
    fn test_websocket_only() {
        let config = TransportsConfig::websocket_only();
        assert!(config.websocket_enabled);
        assert!(!config.sse_enabled);
        assert!(!config.polling_enabled);
        assert!(!config.webhook_enabled);
        assert!(config.any_enabled());
        assert_eq!(config.enabled_count(), 1);
    }

    #[test]
    fn test_none() {
        let config = TransportsConfig::none();
        assert!(!config.websocket_enabled);
        assert!(!config.sse_enabled);
        assert!(!config.polling_enabled);
        assert!(!config.webhook_enabled);
        assert!(!config.any_enabled());
        assert_eq!(config.enabled_count(), 0);
    }

    #[test]
    fn test_builder_methods() {
        let config = TransportsConfig::none()
            .with_websocket(true)
            .with_sse(true);

        assert!(config.websocket_enabled);
        assert!(config.sse_enabled);
        assert!(!config.polling_enabled);
        assert!(!config.webhook_enabled);
        assert_eq!(config.enabled_count(), 2);
    }

    #[test]
    fn test_serialization() {
        let config = TransportsConfig::websocket_only();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: TransportsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.websocket_enabled, parsed.websocket_enabled);
        assert_eq!(config.sse_enabled, parsed.sse_enabled);
    }

    #[test]
    fn test_deserialization_defaults() {
        let json = "{}";
        let config: TransportsConfig = serde_json::from_str(json).unwrap();
        assert!(config.websocket_enabled);
        assert!(config.sse_enabled);
        assert!(config.polling_enabled);
        assert!(config.webhook_enabled);
    }
}

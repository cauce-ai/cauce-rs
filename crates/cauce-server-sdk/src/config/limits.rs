//! Server limits configuration.
//!
//! This module defines resource limits for the Cauce server.

use cauce_core::{MAX_SIGNAL_PAYLOAD_SIZE, MAX_SUBSCRIPTIONS_PER_CLIENT, MAX_TOPICS_PER_SUBSCRIPTION};
use serde::{Deserialize, Serialize};

/// Configuration for server resource limits.
///
/// # Example
///
/// ```
/// use cauce_server_sdk::config::LimitsConfig;
///
/// let config = LimitsConfig::default();
/// assert_eq!(config.max_connections, 10000);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum number of concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Maximum subscriptions per client.
    #[serde(default = "default_max_subscriptions_per_client")]
    pub max_subscriptions_per_client: usize,

    /// Maximum topics per subscription request.
    #[serde(default = "default_max_topics_per_subscription")]
    pub max_topics_per_subscription: usize,

    /// Maximum signal payload size in bytes.
    #[serde(default = "default_max_signal_size")]
    pub max_signal_size: usize,

    /// Maximum requests per second per client (0 = unlimited).
    #[serde(default = "default_rate_limit")]
    pub rate_limit_requests_per_second: u32,

    /// Maximum burst size for rate limiting.
    #[serde(default = "default_rate_limit_burst")]
    pub rate_limit_burst: u32,

    /// Maximum pending signals per subscription before dropping oldest.
    #[serde(default = "default_max_pending_signals")]
    pub max_pending_signals_per_subscription: usize,

    /// Session timeout in seconds (0 = no timeout).
    #[serde(default = "default_session_timeout")]
    pub session_timeout_seconds: u64,

    /// Long polling timeout in seconds.
    #[serde(default = "default_long_poll_timeout")]
    pub long_poll_timeout_seconds: u64,
}

fn default_max_connections() -> usize {
    10_000
}

fn default_max_subscriptions_per_client() -> usize {
    MAX_SUBSCRIPTIONS_PER_CLIENT
}

fn default_max_topics_per_subscription() -> usize {
    MAX_TOPICS_PER_SUBSCRIPTION
}

fn default_max_signal_size() -> usize {
    MAX_SIGNAL_PAYLOAD_SIZE
}

fn default_rate_limit() -> u32 {
    100 // 100 requests per second
}

fn default_rate_limit_burst() -> u32 {
    50 // Allow burst of 50 additional requests
}

fn default_max_pending_signals() -> usize {
    1000
}

fn default_session_timeout() -> u64 {
    3600 // 1 hour
}

fn default_long_poll_timeout() -> u64 {
    30 // 30 seconds
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            max_subscriptions_per_client: default_max_subscriptions_per_client(),
            max_topics_per_subscription: default_max_topics_per_subscription(),
            max_signal_size: default_max_signal_size(),
            rate_limit_requests_per_second: default_rate_limit(),
            rate_limit_burst: default_rate_limit_burst(),
            max_pending_signals_per_subscription: default_max_pending_signals(),
            session_timeout_seconds: default_session_timeout(),
            long_poll_timeout_seconds: default_long_poll_timeout(),
        }
    }
}

impl LimitsConfig {
    /// Create a new limits config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a config suitable for development/testing with relaxed limits.
    pub fn development() -> Self {
        Self {
            max_connections: 100,
            rate_limit_requests_per_second: 0, // Unlimited
            rate_limit_burst: 0,
            ..Self::default()
        }
    }

    /// Create a config suitable for production with stricter limits.
    pub fn production() -> Self {
        Self {
            max_connections: 50_000,
            rate_limit_requests_per_second: 50,
            rate_limit_burst: 25,
            ..Self::default()
        }
    }

    /// Set the maximum number of connections.
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the maximum subscriptions per client.
    pub fn with_max_subscriptions_per_client(mut self, max: usize) -> Self {
        self.max_subscriptions_per_client = max;
        self
    }

    /// Set the maximum topics per subscription.
    pub fn with_max_topics_per_subscription(mut self, max: usize) -> Self {
        self.max_topics_per_subscription = max;
        self
    }

    /// Set the maximum signal size.
    pub fn with_max_signal_size(mut self, max: usize) -> Self {
        self.max_signal_size = max;
        self
    }

    /// Set the rate limit (requests per second).
    pub fn with_rate_limit(mut self, requests_per_second: u32, burst: u32) -> Self {
        self.rate_limit_requests_per_second = requests_per_second;
        self.rate_limit_burst = burst;
        self
    }

    /// Disable rate limiting.
    pub fn without_rate_limit(mut self) -> Self {
        self.rate_limit_requests_per_second = 0;
        self.rate_limit_burst = 0;
        self
    }

    /// Set the session timeout.
    pub fn with_session_timeout(mut self, seconds: u64) -> Self {
        self.session_timeout_seconds = seconds;
        self
    }

    /// Set the long poll timeout.
    pub fn with_long_poll_timeout(mut self, seconds: u64) -> Self {
        self.long_poll_timeout_seconds = seconds;
        self
    }

    /// Check if rate limiting is enabled.
    pub fn is_rate_limited(&self) -> bool {
        self.rate_limit_requests_per_second > 0
    }

    /// Check if sessions have a timeout.
    pub fn has_session_timeout(&self) -> bool {
        self.session_timeout_seconds > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = LimitsConfig::default();
        assert_eq!(config.max_connections, 10_000);
        assert_eq!(config.max_subscriptions_per_client, MAX_SUBSCRIPTIONS_PER_CLIENT);
        assert_eq!(config.max_topics_per_subscription, MAX_TOPICS_PER_SUBSCRIPTION);
        assert_eq!(config.max_signal_size, MAX_SIGNAL_PAYLOAD_SIZE);
        assert!(config.is_rate_limited());
        assert!(config.has_session_timeout());
    }

    #[test]
    fn test_development_config() {
        let config = LimitsConfig::development();
        assert_eq!(config.max_connections, 100);
        assert!(!config.is_rate_limited());
    }

    #[test]
    fn test_production_config() {
        let config = LimitsConfig::production();
        assert_eq!(config.max_connections, 50_000);
        assert_eq!(config.rate_limit_requests_per_second, 50);
        assert!(config.is_rate_limited());
    }

    #[test]
    fn test_builder_methods() {
        let config = LimitsConfig::default()
            .with_max_connections(5000)
            .with_rate_limit(200, 100)
            .with_session_timeout(7200);

        assert_eq!(config.max_connections, 5000);
        assert_eq!(config.rate_limit_requests_per_second, 200);
        assert_eq!(config.rate_limit_burst, 100);
        assert_eq!(config.session_timeout_seconds, 7200);
    }

    #[test]
    fn test_disable_rate_limit() {
        let config = LimitsConfig::default().without_rate_limit();
        assert!(!config.is_rate_limited());
        assert_eq!(config.rate_limit_requests_per_second, 0);
    }

    #[test]
    fn test_serialization() {
        let config = LimitsConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: LimitsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.max_connections, parsed.max_connections);
    }

    #[test]
    fn test_deserialization_defaults() {
        let json = "{}";
        let config: LimitsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_connections, 10_000);
        assert!(config.is_rate_limited());
    }
}

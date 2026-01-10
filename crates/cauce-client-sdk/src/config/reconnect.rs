//! Reconnection configuration for the Cauce Client SDK.
//!
//! This module provides types for configuring automatic reconnection
//! behavior when the connection to a Hub is lost.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for automatic reconnection.
///
/// When the connection to the Hub is lost, the client will automatically
/// attempt to reconnect using exponential backoff.
///
/// # Example
///
/// ```rust
/// use cauce_client_sdk::ReconnectConfig;
/// use std::time::Duration;
///
/// let config = ReconnectConfig::default()
///     .with_initial_delay(Duration::from_millis(500))
///     .with_max_delay(Duration::from_secs(60))
///     .with_backoff_multiplier(2.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Whether automatic reconnection is enabled.
    pub enabled: bool,

    /// Initial delay before the first reconnection attempt.
    #[serde(with = "duration_millis")]
    pub initial_delay: Duration,

    /// Maximum delay between reconnection attempts.
    #[serde(with = "duration_millis")]
    pub max_delay: Duration,

    /// Multiplier for exponential backoff.
    ///
    /// After each failed attempt, the delay is multiplied by this value,
    /// up to `max_delay`.
    pub backoff_multiplier: f64,

    /// Maximum number of reconnection attempts.
    ///
    /// `None` means unlimited attempts.
    pub max_attempts: Option<u32>,

    /// Whether to add random jitter to delays.
    ///
    /// Jitter helps prevent the "thundering herd" problem when many
    /// clients try to reconnect at the same time.
    pub jitter: bool,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            max_attempts: None,
            jitter: true,
        }
    }
}

impl ReconnectConfig {
    /// Create a new reconnection configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a reconnection configuration with reconnection disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    /// Set whether automatic reconnection is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the initial delay before the first reconnection attempt.
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay between reconnection attempts.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff multiplier.
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set the maximum number of reconnection attempts.
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = Some(attempts);
        self
    }

    /// Set whether to add random jitter to delays.
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate the delay for a given attempt number (0-indexed).
    ///
    /// The delay follows exponential backoff: `initial_delay * multiplier^attempt`,
    /// capped at `max_delay`. If jitter is enabled, a random value between 0 and
    /// 25% of the calculated delay is added.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.initial_delay.as_millis() as f64;
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = (base * multiplier).min(self.max_delay.as_millis() as f64);

        let final_delay_ms = if self.jitter {
            // Add 0-25% jitter
            let jitter_factor = 1.0 + (rand_jitter() * 0.25);
            delay_ms * jitter_factor
        } else {
            delay_ms
        };

        Duration::from_millis(final_delay_ms as u64)
    }

    /// Returns true if another reconnection attempt should be made.
    pub fn should_attempt(&self, attempt: u32) -> bool {
        if !self.enabled {
            return false;
        }
        match self.max_attempts {
            Some(max) => attempt < max,
            None => true,
        }
    }
}

/// Generate a pseudo-random value between 0.0 and 1.0 for jitter.
///
/// This uses a simple approach based on system time to avoid
/// requiring a random number generator dependency.
fn rand_jitter() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

/// Serde helper for serializing Duration as milliseconds.
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReconnectConfig::default();
        assert!(config.enabled);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.max_attempts.is_none());
        assert!(config.jitter);
    }

    #[test]
    fn test_disabled_config() {
        let config = ReconnectConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_builder_methods() {
        let config = ReconnectConfig::new()
            .with_enabled(true)
            .with_initial_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(60))
            .with_backoff_multiplier(1.5)
            .with_max_attempts(10)
            .with_jitter(false);

        assert!(config.enabled);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.max_attempts, Some(10));
        assert!(!config.jitter);
    }

    #[test]
    fn test_delay_for_attempt_no_jitter() {
        let config = ReconnectConfig::new()
            .with_initial_delay(Duration::from_millis(100))
            .with_backoff_multiplier(2.0)
            .with_max_delay(Duration::from_secs(10))
            .with_jitter(false);

        // Attempt 0: 100ms
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));

        // Attempt 1: 200ms
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));

        // Attempt 2: 400ms
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));

        // Attempt 3: 800ms
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));

        // Attempt 10: capped at max_delay (10s)
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(10));
    }

    #[test]
    fn test_delay_for_attempt_with_jitter() {
        let config = ReconnectConfig::new()
            .with_initial_delay(Duration::from_millis(100))
            .with_backoff_multiplier(2.0)
            .with_jitter(true);

        let delay = config.delay_for_attempt(0);

        // With jitter, delay should be between 100ms and 125ms
        assert!(delay >= Duration::from_millis(100));
        assert!(delay <= Duration::from_millis(125));
    }

    #[test]
    fn test_should_attempt() {
        // Enabled with unlimited attempts
        let config = ReconnectConfig::new();
        assert!(config.should_attempt(0));
        assert!(config.should_attempt(100));
        assert!(config.should_attempt(1000));

        // Enabled with max attempts
        let config = ReconnectConfig::new().with_max_attempts(5);
        assert!(config.should_attempt(0));
        assert!(config.should_attempt(4));
        assert!(!config.should_attempt(5));
        assert!(!config.should_attempt(6));

        // Disabled
        let config = ReconnectConfig::disabled();
        assert!(!config.should_attempt(0));
    }

    #[test]
    fn test_serialization() {
        let config = ReconnectConfig::new()
            .with_initial_delay(Duration::from_millis(500))
            .with_max_delay(Duration::from_secs(60))
            .with_jitter(false);

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"initial_delay\":500"));
        assert!(json.contains("\"max_delay\":60000"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "enabled": true,
            "initial_delay": 200,
            "max_delay": 10000,
            "backoff_multiplier": 1.5,
            "max_attempts": 5,
            "jitter": false
        }"#;

        let config: ReconnectConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
        assert_eq!(config.max_delay, Duration::from_secs(10));
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.max_attempts, Some(5));
        assert!(!config.jitter);
    }
}

//! Redelivery configuration for unacknowledged signals.
//!
//! This module defines the retry strategy for signals that haven't been
//! acknowledged by clients.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for signal redelivery.
///
/// When a signal is delivered but not acknowledged within a timeout,
/// it will be redelivered according to this configuration.
///
/// # Example
///
/// ```
/// use cauce_server_sdk::config::RedeliveryConfig;
/// use std::time::Duration;
///
/// let config = RedeliveryConfig::default();
/// assert_eq!(config.initial_delay, Duration::from_secs(5));
/// assert_eq!(config.max_attempts, 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeliveryConfig {
    /// Initial delay before first redelivery attempt.
    #[serde(with = "duration_serde", default = "default_initial_delay")]
    pub initial_delay: Duration,

    /// Maximum delay between redelivery attempts.
    #[serde(with = "duration_serde", default = "default_max_delay")]
    pub max_delay: Duration,

    /// Multiplier for exponential backoff.
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// Maximum number of redelivery attempts before moving to dead letter.
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Optional topic to publish undeliverable signals to.
    #[serde(default)]
    pub dead_letter_topic: Option<String>,

    /// Whether redelivery is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_initial_delay() -> Duration {
    Duration::from_secs(5)
}

fn default_max_delay() -> Duration {
    Duration::from_secs(300) // 5 minutes
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_max_attempts() -> u32 {
    5
}

impl Default for RedeliveryConfig {
    fn default() -> Self {
        Self {
            initial_delay: default_initial_delay(),
            max_delay: default_max_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            max_attempts: default_max_attempts(),
            dead_letter_topic: None,
            enabled: true,
        }
    }
}

impl RedeliveryConfig {
    /// Create a new redelivery config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a disabled redelivery config.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    /// Create an aggressive redelivery config for time-sensitive signals.
    pub fn aggressive() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 1.5,
            max_attempts: 10,
            ..Self::default()
        }
    }

    /// Create a relaxed redelivery config for non-time-sensitive signals.
    pub fn relaxed() -> Self {
        Self {
            initial_delay: Duration::from_secs(30),
            max_delay: Duration::from_secs(3600), // 1 hour
            backoff_multiplier: 2.0,
            max_attempts: 3,
            ..Self::default()
        }
    }

    /// Set the initial delay.
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff multiplier.
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set the maximum attempts.
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set the dead letter topic.
    pub fn with_dead_letter_topic(mut self, topic: impl Into<String>) -> Self {
        self.dead_letter_topic = Some(topic.into());
        self
    }

    /// Enable or disable redelivery.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Calculate the delay for a given attempt number (0-indexed).
    ///
    /// Uses exponential backoff: delay = initial_delay * (multiplier ^ attempt)
    /// The result is capped at max_delay.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }

        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let delay_secs = self.initial_delay.as_secs_f64() * multiplier;
        let delay = Duration::from_secs_f64(delay_secs);

        std::cmp::min(delay, self.max_delay)
    }

    /// Check if an attempt should be made (attempt number is within max_attempts).
    pub fn should_attempt(&self, attempt: u32) -> bool {
        self.enabled && attempt < self.max_attempts
    }

    /// Check if there's a dead letter topic configured.
    pub fn has_dead_letter(&self) -> bool {
        self.dead_letter_topic.is_some()
    }
}

/// Serde module for Duration serialization as seconds.
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
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
    fn test_default_values() {
        let config = RedeliveryConfig::default();
        assert_eq!(config.initial_delay, Duration::from_secs(5));
        assert_eq!(config.max_delay, Duration::from_secs(300));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.max_attempts, 5);
        assert!(config.enabled);
        assert!(!config.has_dead_letter());
    }

    #[test]
    fn test_disabled() {
        let config = RedeliveryConfig::disabled();
        assert!(!config.enabled);
        assert!(!config.should_attempt(0));
    }

    #[test]
    fn test_aggressive() {
        let config = RedeliveryConfig::aggressive();
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_attempts, 10);
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    #[test]
    fn test_relaxed() {
        let config = RedeliveryConfig::relaxed();
        assert_eq!(config.initial_delay, Duration::from_secs(30));
        assert_eq!(config.max_attempts, 3);
    }

    #[test]
    fn test_delay_for_attempt() {
        let config = RedeliveryConfig::default();

        // Attempt 0: 5s
        assert_eq!(config.delay_for_attempt(0), Duration::from_secs(5));

        // Attempt 1: 5s * 2 = 10s
        assert_eq!(config.delay_for_attempt(1), Duration::from_secs(10));

        // Attempt 2: 5s * 4 = 20s
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(20));

        // Attempt 3: 5s * 8 = 40s
        assert_eq!(config.delay_for_attempt(3), Duration::from_secs(40));

        // Attempt 4: 5s * 16 = 80s
        assert_eq!(config.delay_for_attempt(4), Duration::from_secs(80));

        // Attempt 5: 5s * 32 = 160s
        assert_eq!(config.delay_for_attempt(5), Duration::from_secs(160));

        // Attempt 6: 5s * 64 = 320s, but capped at max_delay (300s)
        assert_eq!(config.delay_for_attempt(6), Duration::from_secs(300));
    }

    #[test]
    fn test_should_attempt() {
        let config = RedeliveryConfig::default();

        assert!(config.should_attempt(0));
        assert!(config.should_attempt(4));
        assert!(!config.should_attempt(5)); // max_attempts is 5

        let disabled = RedeliveryConfig::disabled();
        assert!(!disabled.should_attempt(0));
    }

    #[test]
    fn test_builder_methods() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_secs(10))
            .with_max_delay(Duration::from_secs(600))
            .with_backoff_multiplier(3.0)
            .with_max_attempts(10)
            .with_dead_letter_topic("cauce.dead_letter");

        assert_eq!(config.initial_delay, Duration::from_secs(10));
        assert_eq!(config.max_delay, Duration::from_secs(600));
        assert_eq!(config.backoff_multiplier, 3.0);
        assert_eq!(config.max_attempts, 10);
        assert!(config.has_dead_letter());
        assert_eq!(
            config.dead_letter_topic,
            Some("cauce.dead_letter".to_string())
        );
    }

    #[test]
    fn test_serialization() {
        let config = RedeliveryConfig::default()
            .with_dead_letter_topic("dlq");

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"initial_delay\":5"));
        assert!(json.contains("\"dead_letter_topic\":\"dlq\""));

        let parsed: RedeliveryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.initial_delay, parsed.initial_delay);
        assert_eq!(config.dead_letter_topic, parsed.dead_letter_topic);
    }

    #[test]
    fn test_deserialization_defaults() {
        let json = "{}";
        let config: RedeliveryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.initial_delay, Duration::from_secs(5));
        assert!(config.enabled);
    }
}

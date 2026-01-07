//! Topic type for the Cauce Protocol.
//!
//! The [`Topic`] type is a validated hierarchical identifier for pub/sub routing.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::constants::{TOPIC_ALLOWED_CHARS, TOPIC_MAX_LENGTH, TOPIC_MIN_LENGTH};
use crate::errors::ValidationError;

/// A validated hierarchical identifier for pub/sub routing.
///
/// Topics are dot-separated hierarchical identifiers used to route
/// signals and actions through the Cauce protocol.
///
/// # Validation Rules
///
/// - Length: 1-255 characters
/// - Valid characters: `[a-zA-Z0-9._-]`
/// - Must not start or end with a dot
/// - Must not contain consecutive dots
///
/// # Examples
///
/// Valid topics:
/// - `signal.email.received`
/// - `action.slack.send`
/// - `system.health`
/// - `user_events.login`
///
/// Invalid topics:
/// - `.leading.dot` (starts with dot)
/// - `trailing.dot.` (ends with dot)
/// - `double..dots` (consecutive dots)
/// - `space invalid` (contains space)
///
/// # Example
///
/// ```
/// use cauce_core::types::Topic;
///
/// let topic = Topic::new("signal.email.received").unwrap();
/// assert_eq!(topic.as_str(), "signal.email.received");
///
/// // Invalid topic
/// assert!(Topic::new(".invalid").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Topic(String);

impl Topic {
    /// Creates a new Topic with validation.
    ///
    /// # Arguments
    ///
    /// * `value` - The topic string to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(Topic)` if valid, or `Err(ValidationError)` if invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Topic;
    ///
    /// let topic = Topic::new("my.topic").unwrap();
    /// assert_eq!(topic.as_str(), "my.topic");
    ///
    /// // Invalid: empty
    /// assert!(Topic::new("").is_err());
    ///
    /// // Invalid: starts with dot
    /// assert!(Topic::new(".invalid").is_err());
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        Self::validate(&value)?;
        Ok(Self(value))
    }

    /// Creates a Topic without validation.
    ///
    /// # Safety
    ///
    /// This method bypasses validation. Use only when you're certain
    /// the value is valid (e.g., from a trusted source or const).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Topic;
    ///
    /// // Use for known-valid topics in tests or constants
    /// let topic = Topic::new_unchecked("known.valid.topic");
    /// ```
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the topic as a string slice.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Topic;
    ///
    /// let topic = Topic::new("my.topic").unwrap();
    /// assert_eq!(topic.as_str(), "my.topic");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the Topic and returns the inner String.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Topic;
    ///
    /// let topic = Topic::new("my.topic").unwrap();
    /// let s: String = topic.into_inner();
    /// assert_eq!(s, "my.topic");
    /// ```
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Validates a topic string.
    ///
    /// # Validation Rules
    ///
    /// 1. Length must be 1-255 characters
    /// 2. Only alphanumeric, dot, hyphen, underscore allowed
    /// 3. Must not start with a dot
    /// 4. Must not end with a dot
    /// 5. Must not contain consecutive dots
    fn validate(value: &str) -> Result<(), ValidationError> {
        // Check length
        if value.len() < TOPIC_MIN_LENGTH {
            return Err(ValidationError::InvalidTopic {
                reason: "topic cannot be empty".to_string(),
            });
        }

        if value.len() > TOPIC_MAX_LENGTH {
            return Err(ValidationError::InvalidTopic {
                reason: format!(
                    "topic length {} exceeds maximum {}",
                    value.len(),
                    TOPIC_MAX_LENGTH
                ),
            });
        }

        // Check for invalid characters
        for (i, c) in value.chars().enumerate() {
            if !TOPIC_ALLOWED_CHARS.contains(c) {
                return Err(ValidationError::InvalidTopic {
                    reason: format!("invalid character '{}' at position {}", c, i),
                });
            }
        }

        // Check for leading dot
        if value.starts_with('.') {
            return Err(ValidationError::InvalidTopic {
                reason: "topic cannot start with a dot".to_string(),
            });
        }

        // Check for trailing dot
        if value.ends_with('.') {
            return Err(ValidationError::InvalidTopic {
                reason: "topic cannot end with a dot".to_string(),
            });
        }

        // Check for consecutive dots
        if value.contains("..") {
            return Err(ValidationError::InvalidTopic {
                reason: "topic cannot contain consecutive dots".to_string(),
            });
        }

        Ok(())
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Topic {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Topic> for String {
    fn from(topic: Topic) -> Self {
        topic.0
    }
}

impl TryFrom<String> for Topic {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Topic {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_valid() {
        assert!(Topic::new("signal.email.received").is_ok());
        assert!(Topic::new("action.slack.send").is_ok());
        assert!(Topic::new("system.health").is_ok());
        assert!(Topic::new("user_events.login").is_ok());
        assert!(Topic::new("a").is_ok());
        assert!(Topic::new("simple").is_ok());
        assert!(Topic::new("with-dashes").is_ok());
        assert!(Topic::new("with_underscores").is_ok());
        assert!(Topic::new("Mixed.Case.123").is_ok());
    }

    #[test]
    fn test_topic_invalid_empty() {
        let result = Topic::new("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ValidationError::InvalidTopic { .. }));
    }

    #[test]
    fn test_topic_invalid_leading_dot() {
        let result = Topic::new(".leading");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot start with a dot"));
    }

    #[test]
    fn test_topic_invalid_trailing_dot() {
        let result = Topic::new("trailing.");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot end with a dot"));
    }

    #[test]
    fn test_topic_invalid_consecutive_dots() {
        let result = Topic::new("double..dots");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("consecutive dots"));
    }

    #[test]
    fn test_topic_invalid_characters() {
        assert!(Topic::new("space invalid").is_err());
        assert!(Topic::new("special@char").is_err());
        assert!(Topic::new("slash/not/allowed").is_err());
        assert!(Topic::new("back\\slash").is_err());
    }

    #[test]
    fn test_topic_too_long() {
        let long_topic = "a".repeat(256);
        let result = Topic::new(&long_topic);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_topic_max_length_ok() {
        let max_topic = "a".repeat(255);
        assert!(Topic::new(&max_topic).is_ok());
    }

    #[test]
    fn test_topic_as_str() {
        let topic = Topic::new("my.topic").unwrap();
        assert_eq!(topic.as_str(), "my.topic");
    }

    #[test]
    fn test_topic_into_inner() {
        let topic = Topic::new("my.topic").unwrap();
        let s = topic.into_inner();
        assert_eq!(s, "my.topic");
    }

    #[test]
    fn test_topic_display() {
        let topic = Topic::new("display.test").unwrap();
        assert_eq!(format!("{}", topic), "display.test");
    }

    #[test]
    fn test_topic_new_unchecked() {
        let topic = Topic::new_unchecked("unchecked.topic");
        assert_eq!(topic.as_str(), "unchecked.topic");
    }

    #[test]
    fn test_topic_serialization() {
        let topic = Topic::new("signal.test").unwrap();
        let json = serde_json::to_string(&topic).unwrap();
        assert_eq!(json, "\"signal.test\"");
    }

    #[test]
    fn test_topic_deserialization() {
        let json = "\"action.send\"";
        let topic: Topic = serde_json::from_str(json).unwrap();
        assert_eq!(topic.as_str(), "action.send");
    }

    #[test]
    fn test_topic_deserialization_invalid() {
        let json = "\"..invalid\"";
        let result: Result<Topic, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_topic_round_trip() {
        let topic = Topic::new("round.trip.test").unwrap();
        let json = serde_json::to_string(&topic).unwrap();
        let restored: Topic = serde_json::from_str(&json).unwrap();
        assert_eq!(topic, restored);
    }

    #[test]
    fn test_topic_try_from_string() {
        let topic: Result<Topic, _> = "valid.topic".to_string().try_into();
        assert!(topic.is_ok());

        let invalid: Result<Topic, _> = "..invalid".to_string().try_into();
        assert!(invalid.is_err());
    }

    #[test]
    fn test_topic_try_from_str() {
        let topic: Result<Topic, _> = "valid.topic".try_into();
        assert!(topic.is_ok());
    }

    #[test]
    fn test_topic_as_ref() {
        let topic = Topic::new("ref.test").unwrap();
        let s: &str = topic.as_ref();
        assert_eq!(s, "ref.test");
    }

    #[test]
    fn test_topic_from_into_string() {
        let topic = Topic::new("convert.test").unwrap();
        let s: String = topic.into();
        assert_eq!(s, "convert.test");
    }

    #[test]
    fn test_topic_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Topic::new("unique.topic").unwrap());
        set.insert(Topic::new("unique.topic").unwrap());
        assert_eq!(set.len(), 1);
    }
}

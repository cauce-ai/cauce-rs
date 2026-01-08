//! Validation utilities for the Cauce Protocol.
//!
//! This module provides validation functionality:
//!
//! - [`is_valid_signal_id`] - Validate Signal ID format
//! - [`is_valid_action_id`] - Validate Action ID format
//! - [`is_valid_topic`] - Validate Topic format
//!
//! ## ID Validation
//!
//! IDs follow specific patterns defined in the Constitution:
//! - Signal IDs: `sig_<unix_timestamp>_<random_12_chars>`
//! - Action IDs: `act_<unix_timestamp>_<random_12_chars>`

use crate::constants::{
    ACTION_ID_PATTERN, SIGNAL_ID_PATTERN, TOPIC_ALLOWED_CHARS, TOPIC_MAX_LENGTH, TOPIC_MIN_LENGTH,
};
use crate::errors::ValidationError;
use once_cell::sync::Lazy;
use regex::Regex;

// Compiled regex patterns for ID validation
static SIGNAL_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(SIGNAL_ID_PATTERN).expect("Invalid signal ID regex pattern"));

static ACTION_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(ACTION_ID_PATTERN).expect("Invalid action ID regex pattern"));

/// Validates a Signal ID against the required format.
///
/// # Arguments
///
/// * `id` - The Signal ID to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(ValidationError)` if invalid
///
/// # Example
///
/// ```
/// use cauce_core::validation::is_valid_signal_id;
///
/// assert!(is_valid_signal_id("sig_1704067200_abc123def456").is_ok());
/// assert!(is_valid_signal_id("invalid").is_err());
/// ```
pub fn is_valid_signal_id(id: &str) -> Result<(), ValidationError> {
    if SIGNAL_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(ValidationError::InvalidSignalId {
            reason: format!(
                "must match format 'sig_<timestamp>_<12 alphanumeric chars>', got '{}'",
                id
            ),
        })
    }
}

/// Validates an Action ID against the required format.
///
/// # Arguments
///
/// * `id` - The Action ID to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(ValidationError)` if invalid
///
/// # Example
///
/// ```
/// use cauce_core::validation::is_valid_action_id;
///
/// assert!(is_valid_action_id("act_1704067200_abc123def456").is_ok());
/// assert!(is_valid_action_id("invalid").is_err());
/// ```
pub fn is_valid_action_id(id: &str) -> Result<(), ValidationError> {
    if ACTION_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(ValidationError::InvalidActionId {
            reason: format!(
                "must match format 'act_<timestamp>_<12 alphanumeric chars>', got '{}'",
                id
            ),
        })
    }
}

/// Validates a Topic string against the required format.
///
/// # Rules
///
/// - Length: 1-255 characters
/// - Allowed characters: alphanumeric, dots (`.`), hyphens (`-`), underscores (`_`)
/// - No leading or trailing dots
/// - No consecutive dots
///
/// # Arguments
///
/// * `topic` - The topic string to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(ValidationError)` if invalid
///
/// # Example
///
/// ```
/// use cauce_core::validation::is_valid_topic;
///
/// assert!(is_valid_topic("signal.email.received").is_ok());
/// assert!(is_valid_topic(".leading.dot").is_err());
/// ```
pub fn is_valid_topic(topic: &str) -> Result<(), ValidationError> {
    // Check length
    if topic.len() < TOPIC_MIN_LENGTH {
        return Err(ValidationError::InvalidTopic {
            reason: "topic cannot be empty".to_string(),
        });
    }
    if topic.len() > TOPIC_MAX_LENGTH {
        return Err(ValidationError::InvalidTopic {
            reason: format!(
                "topic exceeds maximum length of {} characters",
                TOPIC_MAX_LENGTH
            ),
        });
    }

    // Check for leading dot
    if topic.starts_with('.') {
        return Err(ValidationError::InvalidTopic {
            reason: "topic cannot start with a dot".to_string(),
        });
    }

    // Check for trailing dot
    if topic.ends_with('.') {
        return Err(ValidationError::InvalidTopic {
            reason: "topic cannot end with a dot".to_string(),
        });
    }

    // Check for consecutive dots
    if topic.contains("..") {
        return Err(ValidationError::InvalidTopic {
            reason: "topic cannot contain consecutive dots".to_string(),
        });
    }

    // Check for invalid characters
    for c in topic.chars() {
        if !TOPIC_ALLOWED_CHARS.contains(c) {
            return Err(ValidationError::InvalidTopic {
                reason: format!("topic contains invalid character '{}'", c),
            });
        }
    }

    Ok(())
}

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "validation: Validation utilities and schema validation"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_signal_id() {
        assert!(is_valid_signal_id("sig_1704067200_abc123def456").is_ok());
        assert!(is_valid_signal_id("sig_0_ABCDEFGHIJKL").is_ok());
        assert!(is_valid_signal_id("sig_9999999999_123456789012").is_ok());
    }

    #[test]
    fn test_invalid_signal_id() {
        // Missing prefix
        assert!(is_valid_signal_id("1704067200_abc123def456").is_err());
        // Wrong prefix
        assert!(is_valid_signal_id("act_1704067200_abc123def456").is_err());
        // Too short random part
        assert!(is_valid_signal_id("sig_1704067200_abc").is_err());
        // Too long random part
        assert!(is_valid_signal_id("sig_1704067200_abc123def4567").is_err());
        // Non-alphanumeric in random part
        assert!(is_valid_signal_id("sig_1704067200_abc123def45!").is_err());
    }

    #[test]
    fn test_valid_action_id() {
        assert!(is_valid_action_id("act_1704067200_abc123def456").is_ok());
        assert!(is_valid_action_id("act_0_ABCDEFGHIJKL").is_ok());
    }

    #[test]
    fn test_invalid_action_id() {
        // Missing prefix
        assert!(is_valid_action_id("1704067200_abc123def456").is_err());
        // Wrong prefix
        assert!(is_valid_action_id("sig_1704067200_abc123def456").is_err());
    }

    #[test]
    fn test_valid_topic() {
        assert!(is_valid_topic("signal.email.received").is_ok());
        assert!(is_valid_topic("action.slack.send").is_ok());
        assert!(is_valid_topic("system-health").is_ok());
        assert!(is_valid_topic("topic_with_underscores").is_ok());
        assert!(is_valid_topic("a").is_ok()); // minimum length
    }

    #[test]
    fn test_invalid_topic_empty() {
        assert!(is_valid_topic("").is_err());
    }

    #[test]
    fn test_invalid_topic_too_long() {
        let long_topic = "a".repeat(256);
        assert!(is_valid_topic(&long_topic).is_err());
    }

    #[test]
    fn test_invalid_topic_leading_dot() {
        assert!(is_valid_topic(".leading.dot").is_err());
    }

    #[test]
    fn test_invalid_topic_trailing_dot() {
        assert!(is_valid_topic("trailing.dot.").is_err());
    }

    #[test]
    fn test_invalid_topic_consecutive_dots() {
        assert!(is_valid_topic("double..dots").is_err());
    }

    #[test]
    fn test_invalid_topic_invalid_chars() {
        assert!(is_valid_topic("space invalid").is_err());
        assert!(is_valid_topic("special!char").is_err());
        assert!(is_valid_topic("tab\there").is_err());
    }
}

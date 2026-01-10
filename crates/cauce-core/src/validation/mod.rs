//! Validation utilities for the Cauce Protocol.
//!
//! This module provides validation functionality:
//!
//! - [`is_valid_signal_id`] - Validate Signal ID format
//! - [`is_valid_action_id`] - Validate Action ID format
//! - [`is_valid_topic`] - Validate Topic format
//! - [`validate_topic_pattern`] - Validate topic patterns with wildcards
//! - [`is_valid_subscription_id`] - Validate Subscription ID format
//! - [`is_valid_session_id`] - Validate Session ID format
//! - [`is_valid_message_id`] - Validate Message ID format
//! - [`validate_signal`] - Validate JSON against Signal schema
//! - [`validate_action`] - Validate JSON against Action schema
//!
//! ## ID Validation
//!
//! IDs follow specific patterns defined in the Constitution:
//! - Signal IDs: `sig_<unix_timestamp>_<random_12_chars>`
//! - Action IDs: `act_<unix_timestamp>_<random_12_chars>`
//! - Subscription IDs: `sub_<uuid>`
//! - Session IDs: `sess_<uuid>`
//! - Message IDs: `msg_<uuid>`

pub mod schema;

pub use schema::{validate_action, validate_signal};

use crate::constants::{
    ACTION_ID_PATTERN, MESSAGE_ID_PATTERN, SESSION_ID_PATTERN, SIGNAL_ID_PATTERN,
    SUBSCRIPTION_ID_PATTERN, TOPIC_ALLOWED_CHARS, TOPIC_MAX_LENGTH, TOPIC_MIN_LENGTH,
};
use crate::errors::ValidationError;
use once_cell::sync::Lazy;
use regex::Regex;

// Compiled regex patterns for ID validation
static SIGNAL_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(SIGNAL_ID_PATTERN).expect("Invalid signal ID regex pattern"));

static ACTION_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(ACTION_ID_PATTERN).expect("Invalid action ID regex pattern"));

static SUBSCRIPTION_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(SUBSCRIPTION_ID_PATTERN).expect("Invalid subscription ID regex pattern")
});

static SESSION_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(SESSION_ID_PATTERN).expect("Invalid session ID regex pattern"));

static MESSAGE_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(MESSAGE_ID_PATTERN).expect("Invalid message ID regex pattern"));

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

/// Characters allowed in topic pattern wildcards
const PATTERN_WILDCARDS: &str = "*";

/// Validates a topic pattern string (allows wildcards).
///
/// # Rules
///
/// - Same rules as `is_valid_topic` plus:
/// - `*` is allowed as a single-segment wildcard
/// - `**` is allowed as a multi-segment wildcard
/// - Wildcards must be complete segments (not mixed with text)
///
/// # Arguments
///
/// * `pattern` - The topic pattern string to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(ValidationError)` if invalid
///
/// # Example
///
/// ```
/// use cauce_core::validation::validate_topic_pattern;
///
/// assert!(validate_topic_pattern("signal.*").is_ok());
/// assert!(validate_topic_pattern("signal.**").is_ok());
/// assert!(validate_topic_pattern("*foo").is_err()); // Mixed wildcard
/// ```
pub fn validate_topic_pattern(pattern: &str) -> Result<(), ValidationError> {
    // Check length
    if pattern.is_empty() {
        return Err(ValidationError::InvalidTopicPattern {
            reason: "pattern cannot be empty".to_string(),
        });
    }
    if pattern.len() > TOPIC_MAX_LENGTH {
        return Err(ValidationError::InvalidTopicPattern {
            reason: format!(
                "pattern exceeds maximum length of {} characters",
                TOPIC_MAX_LENGTH
            ),
        });
    }

    // Check for leading dot
    if pattern.starts_with('.') {
        return Err(ValidationError::InvalidTopicPattern {
            reason: "pattern cannot start with a dot".to_string(),
        });
    }

    // Check for trailing dot
    if pattern.ends_with('.') {
        return Err(ValidationError::InvalidTopicPattern {
            reason: "pattern cannot end with a dot".to_string(),
        });
    }

    // Check for consecutive dots
    if pattern.contains("..") {
        return Err(ValidationError::InvalidTopicPattern {
            reason: "pattern cannot contain consecutive dots".to_string(),
        });
    }

    // Validate each segment
    for segment in pattern.split('.') {
        if segment.is_empty() {
            return Err(ValidationError::InvalidTopicPattern {
                reason: "pattern cannot have empty segments".to_string(),
            });
        }

        // Check for valid wildcards
        if segment == "*" || segment == "**" {
            continue;
        }

        // Check for mixed wildcard (e.g., "*foo" or "foo*")
        if segment.contains('*') {
            return Err(ValidationError::InvalidTopicPattern {
                reason: format!(
                    "wildcard must be standalone segment, not mixed with text: '{}'",
                    segment
                ),
            });
        }

        // Check for invalid characters (same as topic)
        for c in segment.chars() {
            if !TOPIC_ALLOWED_CHARS.contains(c) && !PATTERN_WILDCARDS.contains(c) {
                return Err(ValidationError::InvalidTopicPattern {
                    reason: format!("pattern contains invalid character '{}'", c),
                });
            }
        }
    }

    Ok(())
}

/// Validates a Subscription ID against the required format.
///
/// # Format
///
/// `sub_<uuid>` where uuid is a standard UUID v4
///
/// # Example
///
/// ```
/// use cauce_core::validation::is_valid_subscription_id;
///
/// assert!(is_valid_subscription_id("sub_550e8400-e29b-41d4-a716-446655440000").is_ok());
/// assert!(is_valid_subscription_id("invalid").is_err());
/// ```
pub fn is_valid_subscription_id(id: &str) -> Result<(), ValidationError> {
    if SUBSCRIPTION_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(ValidationError::InvalidSubscriptionId {
            reason: format!("must match format 'sub_<uuid>', got '{}'", id),
        })
    }
}

/// Validates a Session ID against the required format.
///
/// # Format
///
/// `sess_<uuid>` where uuid is a standard UUID v4
///
/// # Example
///
/// ```
/// use cauce_core::validation::is_valid_session_id;
///
/// assert!(is_valid_session_id("sess_550e8400-e29b-41d4-a716-446655440000").is_ok());
/// assert!(is_valid_session_id("invalid").is_err());
/// ```
pub fn is_valid_session_id(id: &str) -> Result<(), ValidationError> {
    if SESSION_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(ValidationError::InvalidSessionId {
            reason: format!("must match format 'sess_<uuid>', got '{}'", id),
        })
    }
}

/// Validates a Message ID against the required format.
///
/// # Format
///
/// `msg_<uuid>` where uuid is a standard UUID v4
///
/// # Example
///
/// ```
/// use cauce_core::validation::is_valid_message_id;
///
/// assert!(is_valid_message_id("msg_550e8400-e29b-41d4-a716-446655440000").is_ok());
/// assert!(is_valid_message_id("invalid").is_err());
/// ```
pub fn is_valid_message_id(id: &str) -> Result<(), ValidationError> {
    if MESSAGE_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(ValidationError::InvalidMessageId {
            reason: format!("must match format 'msg_<uuid>', got '{}'", id),
        })
    }
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

    // ===== Topic Pattern Validation Tests =====

    #[test]
    fn test_valid_topic_pattern_exact() {
        assert!(validate_topic_pattern("signal.email.received").is_ok());
    }

    #[test]
    fn test_valid_topic_pattern_single_wildcard() {
        assert!(validate_topic_pattern("signal.*").is_ok());
        assert!(validate_topic_pattern("*.email").is_ok());
        assert!(validate_topic_pattern("signal.*.received").is_ok());
    }

    #[test]
    fn test_valid_topic_pattern_multi_wildcard() {
        assert!(validate_topic_pattern("signal.**").is_ok());
        assert!(validate_topic_pattern("**.received").is_ok());
        assert!(validate_topic_pattern("signal.**.received").is_ok());
    }

    #[test]
    fn test_valid_topic_pattern_all_wildcards() {
        assert!(validate_topic_pattern("*").is_ok());
        assert!(validate_topic_pattern("**").is_ok());
        assert!(validate_topic_pattern("*.*").is_ok());
        assert!(validate_topic_pattern("**.**").is_ok());
    }

    #[test]
    fn test_invalid_topic_pattern_empty() {
        assert!(validate_topic_pattern("").is_err());
    }

    #[test]
    fn test_invalid_topic_pattern_mixed_wildcard() {
        assert!(validate_topic_pattern("*foo").is_err());
        assert!(validate_topic_pattern("foo*").is_err());
        assert!(validate_topic_pattern("f*o").is_err());
        assert!(validate_topic_pattern("**bar").is_err());
    }

    #[test]
    fn test_invalid_topic_pattern_leading_dot() {
        assert!(validate_topic_pattern(".signal.*").is_err());
    }

    #[test]
    fn test_invalid_topic_pattern_trailing_dot() {
        assert!(validate_topic_pattern("signal.*.").is_err());
    }

    #[test]
    fn test_invalid_topic_pattern_consecutive_dots() {
        assert!(validate_topic_pattern("signal..email").is_err());
    }

    // ===== Subscription ID Validation Tests =====

    #[test]
    fn test_valid_subscription_id() {
        assert!(is_valid_subscription_id("sub_550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(is_valid_subscription_id("sub_00000000-0000-0000-0000-000000000000").is_ok());
        assert!(is_valid_subscription_id("sub_ffffffff-ffff-ffff-ffff-ffffffffffff").is_ok());
    }

    #[test]
    fn test_invalid_subscription_id() {
        assert!(is_valid_subscription_id("invalid").is_err());
        assert!(is_valid_subscription_id("sub_invalid").is_err());
        assert!(is_valid_subscription_id("sess_550e8400-e29b-41d4-a716-446655440000").is_err());
        assert!(is_valid_subscription_id("SUB_550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    // ===== Session ID Validation Tests =====

    #[test]
    fn test_valid_session_id() {
        assert!(is_valid_session_id("sess_550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(is_valid_session_id("sess_00000000-0000-0000-0000-000000000000").is_ok());
    }

    #[test]
    fn test_invalid_session_id() {
        assert!(is_valid_session_id("invalid").is_err());
        assert!(is_valid_session_id("sess_invalid").is_err());
        assert!(is_valid_session_id("sub_550e8400-e29b-41d4-a716-446655440000").is_err());
    }

    // ===== Message ID Validation Tests =====

    #[test]
    fn test_valid_message_id() {
        assert!(is_valid_message_id("msg_550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(is_valid_message_id("msg_00000000-0000-0000-0000-000000000000").is_ok());
    }

    #[test]
    fn test_invalid_message_id() {
        assert!(is_valid_message_id("invalid").is_err());
        assert!(is_valid_message_id("msg_invalid").is_err());
        assert!(is_valid_message_id("sig_550e8400-e29b-41d4-a716-446655440000").is_err());
    }
}

//! ID generation utilities for the Cauce Protocol.
//!
//! This module provides functions to generate unique identifiers
//! for various protocol entities:
//!
//! - [`generate_signal_id`] - Generate Signal IDs (`sig_<timestamp>_<random12>`)
//! - [`generate_action_id`] - Generate Action IDs (`act_<timestamp>_<random12>`)
//! - [`generate_subscription_id`] - Generate Subscription IDs (`sub_<uuid>`)
//! - [`generate_session_id`] - Generate Session IDs (`sess_<uuid>`)
//! - [`generate_message_id`] - Generate Message IDs (`msg_<uuid>`)

use chrono::Utc;
use uuid::Uuid;

/// Generates a unique Signal ID.
///
/// Format: `sig_<unix_timestamp>_<random_12_alphanumeric>`
///
/// # Example
///
/// ```
/// use cauce_core::id::generate_signal_id;
///
/// let id = generate_signal_id();
/// assert!(id.starts_with("sig_"));
/// ```
pub fn generate_signal_id() -> String {
    let timestamp = Utc::now().timestamp();
    let random: String = Uuid::new_v4()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(12)
        .collect();
    format!("sig_{}_{}", timestamp, random)
}

/// Generates a unique Action ID.
///
/// Format: `act_<unix_timestamp>_<random_12_alphanumeric>`
///
/// # Example
///
/// ```
/// use cauce_core::id::generate_action_id;
///
/// let id = generate_action_id();
/// assert!(id.starts_with("act_"));
/// ```
pub fn generate_action_id() -> String {
    let timestamp = Utc::now().timestamp();
    let random: String = Uuid::new_v4()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(12)
        .collect();
    format!("act_{}_{}", timestamp, random)
}

/// Generates a unique Subscription ID.
///
/// Format: `sub_<uuid>`
///
/// # Example
///
/// ```
/// use cauce_core::id::generate_subscription_id;
///
/// let id = generate_subscription_id();
/// assert!(id.starts_with("sub_"));
/// ```
pub fn generate_subscription_id() -> String {
    format!("sub_{}", Uuid::new_v4())
}

/// Generates a unique Session ID.
///
/// Format: `sess_<uuid>`
///
/// # Example
///
/// ```
/// use cauce_core::id::generate_session_id;
///
/// let id = generate_session_id();
/// assert!(id.starts_with("sess_"));
/// ```
pub fn generate_session_id() -> String {
    format!("sess_{}", Uuid::new_v4())
}

/// Generates a unique Message ID.
///
/// Format: `msg_<uuid>`
///
/// # Example
///
/// ```
/// use cauce_core::id::generate_message_id;
///
/// let id = generate_message_id();
/// assert!(id.starts_with("msg_"));
/// ```
pub fn generate_message_id() -> String {
    format!("msg_{}", Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_generate_signal_id_format() {
        let id = generate_signal_id();
        let re = Regex::new(r"^sig_\d+_[a-zA-Z0-9]{12}$").unwrap();
        assert!(
            re.is_match(&id),
            "Signal ID '{}' doesn't match expected format",
            id
        );
    }

    #[test]
    fn test_generate_signal_id_uniqueness() {
        let id1 = generate_signal_id();
        let id2 = generate_signal_id();
        assert_ne!(id1, id2, "Signal IDs should be unique");
    }

    #[test]
    fn test_generate_action_id_format() {
        let id = generate_action_id();
        let re = Regex::new(r"^act_\d+_[a-zA-Z0-9]{12}$").unwrap();
        assert!(
            re.is_match(&id),
            "Action ID '{}' doesn't match expected format",
            id
        );
    }

    #[test]
    fn test_generate_action_id_uniqueness() {
        let id1 = generate_action_id();
        let id2 = generate_action_id();
        assert_ne!(id1, id2, "Action IDs should be unique");
    }

    #[test]
    fn test_generate_subscription_id_format() {
        let id = generate_subscription_id();
        let re = Regex::new(r"^sub_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .unwrap();
        assert!(
            re.is_match(&id),
            "Subscription ID '{}' doesn't match expected format",
            id
        );
    }

    #[test]
    fn test_generate_subscription_id_uniqueness() {
        let id1 = generate_subscription_id();
        let id2 = generate_subscription_id();
        assert_ne!(id1, id2, "Subscription IDs should be unique");
    }

    #[test]
    fn test_generate_session_id_format() {
        let id = generate_session_id();
        let re = Regex::new(r"^sess_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .unwrap();
        assert!(
            re.is_match(&id),
            "Session ID '{}' doesn't match expected format",
            id
        );
    }

    #[test]
    fn test_generate_session_id_uniqueness() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert_ne!(id1, id2, "Session IDs should be unique");
    }

    #[test]
    fn test_generate_message_id_format() {
        let id = generate_message_id();
        let re = Regex::new(r"^msg_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .unwrap();
        assert!(
            re.is_match(&id),
            "Message ID '{}' doesn't match expected format",
            id
        );
    }

    #[test]
    fn test_generate_message_id_uniqueness() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        assert_ne!(id1, id2, "Message IDs should be unique");
    }
}

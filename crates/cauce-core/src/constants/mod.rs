//! Protocol constants for the Cauce Protocol.
//!
//! This module provides protocol-wide constants:
//!
//! - ID format patterns for Signal and Action identifiers
//! - Topic validation limits and patterns
//! - Protocol version information
//!
//! ## ID Formats
//!
//! Per the Constitution, IDs follow these patterns:
//! - Signal IDs: `sig_<unix_timestamp_seconds>_<random_12_chars>`
//! - Action IDs: `act_<unix_timestamp_seconds>_<random_12_chars>`

/// Prefix for Signal IDs
pub const SIGNAL_ID_PREFIX: &str = "sig_";

/// Prefix for Action IDs
pub const ACTION_ID_PREFIX: &str = "act_";

/// Length of the random portion of IDs
pub const ID_RANDOM_LENGTH: usize = 12;

/// Regex pattern for validating Signal IDs
/// Format: sig_<digits>_<12 alphanumeric chars>
pub const SIGNAL_ID_PATTERN: &str = r"^sig_\d+_[a-zA-Z0-9]{12}$";

/// Regex pattern for validating Action IDs
/// Format: act_<digits>_<12 alphanumeric chars>
pub const ACTION_ID_PATTERN: &str = r"^act_\d+_[a-zA-Z0-9]{12}$";

/// Minimum length for a Topic (1 character)
pub const TOPIC_MIN_LENGTH: usize = 1;

/// Maximum length for a Topic (255 characters)
pub const TOPIC_MAX_LENGTH: usize = 255;

/// Characters allowed in a Topic (alphanumeric, dots, hyphens, underscores)
pub const TOPIC_ALLOWED_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-";

/// Current protocol version
pub const PROTOCOL_VERSION: &str = "1.0";

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "constants: Protocol constants (method names, limits)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_id_prefix() {
        assert_eq!(SIGNAL_ID_PREFIX, "sig_");
    }

    #[test]
    fn test_action_id_prefix() {
        assert_eq!(ACTION_ID_PREFIX, "act_");
    }

    #[test]
    fn test_topic_limits() {
        assert_eq!(TOPIC_MIN_LENGTH, 1);
        assert_eq!(TOPIC_MAX_LENGTH, 255);
    }

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "1.0");
    }
}

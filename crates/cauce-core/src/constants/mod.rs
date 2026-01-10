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

// =============================================================================
// Method Name Constants
// =============================================================================

/// Method name for Hello handshake
pub const METHOD_HELLO: &str = "cauce.hello";

/// Method name for Goodbye
pub const METHOD_GOODBYE: &str = "cauce.goodbye";

/// Method name for Ping
pub const METHOD_PING: &str = "cauce.ping";

/// Method name for Pong
pub const METHOD_PONG: &str = "cauce.pong";

/// Method name for Publish
pub const METHOD_PUBLISH: &str = "cauce.publish";

/// Method name for Subscribe
pub const METHOD_SUBSCRIBE: &str = "cauce.subscribe";

/// Method name for Unsubscribe
pub const METHOD_UNSUBSCRIBE: &str = "cauce.unsubscribe";

/// Method name for Signal delivery notification
pub const METHOD_SIGNAL: &str = "cauce.signal";

/// Method name for Acknowledgment
pub const METHOD_ACK: &str = "cauce.ack";

/// Method name for Subscription request notification
pub const METHOD_SUBSCRIPTION_REQUEST: &str = "cauce.subscription.request";

/// Method name for Subscription approval
pub const METHOD_SUBSCRIPTION_APPROVE: &str = "cauce.subscription.approve";

/// Method name for Subscription denial
pub const METHOD_SUBSCRIPTION_DENY: &str = "cauce.subscription.deny";

/// Method name for Subscription list
pub const METHOD_SUBSCRIPTION_LIST: &str = "cauce.subscription.list";

/// Method name for Subscription revocation
pub const METHOD_SUBSCRIPTION_REVOKE: &str = "cauce.subscription.revoke";

/// Method name for Subscription status notification
pub const METHOD_SUBSCRIPTION_STATUS: &str = "cauce.subscription.status";

/// Method name for Schemas list
pub const METHOD_SCHEMAS_LIST: &str = "cauce.schemas.list";

/// Method name for Schemas get
pub const METHOD_SCHEMAS_GET: &str = "cauce.schemas.get";

// =============================================================================
// Size Limit Constants
// =============================================================================

/// Maximum size of a signal payload in bytes (10 MB)
pub const MAX_SIGNAL_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of topics per subscription
pub const MAX_TOPICS_PER_SUBSCRIPTION: usize = 100;

/// Maximum number of subscriptions per client
pub const MAX_SUBSCRIPTIONS_PER_CLIENT: usize = 1000;

/// Maximum number of signals per batch acknowledgment
pub const MAX_SIGNALS_PER_BATCH: usize = 100;

/// Maximum depth of topic segments (e.g., a.b.c.d.e.f.g.h.i.j = 10)
pub const MAX_TOPIC_DEPTH: usize = 10;

/// Alias for TOPIC_MAX_LENGTH for consistency
pub const MAX_TOPIC_LENGTH: usize = TOPIC_MAX_LENGTH;

// =============================================================================
// ID Pattern Constants
// =============================================================================

/// Regex pattern for validating Subscription IDs
/// Format: sub_<uuid>
pub const SUBSCRIPTION_ID_PATTERN: &str =
    r"^sub_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

/// Regex pattern for validating Session IDs
/// Format: sess_<uuid>
pub const SESSION_ID_PATTERN: &str =
    r"^sess_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

/// Regex pattern for validating Message IDs
/// Format: msg_<uuid>
pub const MESSAGE_ID_PATTERN: &str =
    r"^msg_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

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

    // ===== Method Name Tests =====

    #[test]
    fn test_method_hello() {
        assert_eq!(METHOD_HELLO, "cauce.hello");
    }

    #[test]
    fn test_method_goodbye() {
        assert_eq!(METHOD_GOODBYE, "cauce.goodbye");
    }

    #[test]
    fn test_method_ping_pong() {
        assert_eq!(METHOD_PING, "cauce.ping");
        assert_eq!(METHOD_PONG, "cauce.pong");
    }

    #[test]
    fn test_method_publish_subscribe() {
        assert_eq!(METHOD_PUBLISH, "cauce.publish");
        assert_eq!(METHOD_SUBSCRIBE, "cauce.subscribe");
        assert_eq!(METHOD_UNSUBSCRIBE, "cauce.unsubscribe");
    }

    #[test]
    fn test_method_signal_ack() {
        assert_eq!(METHOD_SIGNAL, "cauce.signal");
        assert_eq!(METHOD_ACK, "cauce.ack");
    }

    #[test]
    fn test_method_subscription_management() {
        assert_eq!(METHOD_SUBSCRIPTION_REQUEST, "cauce.subscription.request");
        assert_eq!(METHOD_SUBSCRIPTION_APPROVE, "cauce.subscription.approve");
        assert_eq!(METHOD_SUBSCRIPTION_DENY, "cauce.subscription.deny");
        assert_eq!(METHOD_SUBSCRIPTION_LIST, "cauce.subscription.list");
        assert_eq!(METHOD_SUBSCRIPTION_REVOKE, "cauce.subscription.revoke");
        assert_eq!(METHOD_SUBSCRIPTION_STATUS, "cauce.subscription.status");
    }

    #[test]
    fn test_method_schemas() {
        assert_eq!(METHOD_SCHEMAS_LIST, "cauce.schemas.list");
        assert_eq!(METHOD_SCHEMAS_GET, "cauce.schemas.get");
    }

    // ===== Size Limit Tests =====

    #[test]
    fn test_max_signal_payload_size() {
        assert_eq!(MAX_SIGNAL_PAYLOAD_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_max_topics_per_subscription() {
        assert_eq!(MAX_TOPICS_PER_SUBSCRIPTION, 100);
    }

    #[test]
    fn test_max_subscriptions_per_client() {
        assert_eq!(MAX_SUBSCRIPTIONS_PER_CLIENT, 1000);
    }

    #[test]
    fn test_max_signals_per_batch() {
        assert_eq!(MAX_SIGNALS_PER_BATCH, 100);
    }

    #[test]
    fn test_max_topic_depth() {
        assert_eq!(MAX_TOPIC_DEPTH, 10);
    }

    #[test]
    fn test_max_topic_length_alias() {
        assert_eq!(MAX_TOPIC_LENGTH, TOPIC_MAX_LENGTH);
        assert_eq!(MAX_TOPIC_LENGTH, 255);
    }

    // ===== ID Pattern Tests =====

    #[test]
    fn test_subscription_id_pattern() {
        use regex::Regex;
        let re = Regex::new(SUBSCRIPTION_ID_PATTERN).unwrap();
        assert!(re.is_match("sub_550e8400-e29b-41d4-a716-446655440000"));
        assert!(!re.is_match("sub_invalid"));
        assert!(!re.is_match("sess_550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_session_id_pattern() {
        use regex::Regex;
        let re = Regex::new(SESSION_ID_PATTERN).unwrap();
        assert!(re.is_match("sess_550e8400-e29b-41d4-a716-446655440000"));
        assert!(!re.is_match("sess_invalid"));
        assert!(!re.is_match("sub_550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_message_id_pattern() {
        use regex::Regex;
        let re = Regex::new(MESSAGE_ID_PATTERN).unwrap();
        assert!(re.is_match("msg_550e8400-e29b-41d4-a716-446655440000"));
        assert!(!re.is_match("msg_invalid"));
        assert!(!re.is_match("sig_550e8400-e29b-41d4-a716-446655440000"));
    }
}

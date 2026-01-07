//! Signal type for the Cauce Protocol.
//!
//! The [`Signal`] struct represents an inbound message from an adapter to the hub.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Encrypted, Metadata, Payload, Source, Topic};

/// An inbound message from an adapter to the hub.
///
/// Signals represent messages received by adapters from external platforms
/// (email, Slack, Telegram, etc.) that are forwarded to the hub for processing.
///
/// # Fields
///
/// - `id` - Unique identifier: `sig_<unix_timestamp>_<random_12>`
/// - `version` - Protocol version (e.g., "1.0")
/// - `timestamp` - When the signal was created (ISO 8601)
/// - `source` - Origin information (adapter type, adapter ID, native message ID)
/// - `topic` - Routing topic for pub/sub
/// - `payload` - Message content with type information
/// - `metadata` - Optional threading and priority info
/// - `encrypted` - Optional E2E encryption envelope
///
/// # Example
///
/// ```
/// use cauce_core::types::{Signal, Source, Topic, Payload};
/// use chrono::Utc;
/// use serde_json::json;
///
/// let signal = Signal {
///     id: "sig_1704067200_abc123def456".to_string(),
///     version: "1.0".to_string(),
///     timestamp: Utc::now(),
///     source: Source::new("email", "email-adapter-1", "msg-12345"),
///     topic: Topic::new_unchecked("signal.email.received"),
///     payload: Payload::new(json!({"from": "alice@example.com"}), "application/json"),
///     metadata: None,
///     encrypted: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    /// Unique identifier: `sig_<unix_timestamp>_<random_12>`
    pub id: String,

    /// Protocol version (e.g., "1.0")
    pub version: String,

    /// When the signal was created
    pub timestamp: DateTime<Utc>,

    /// Origin information
    pub source: Source,

    /// Routing topic for pub/sub
    pub topic: Topic,

    /// Message content with type information
    pub payload: Payload,

    /// Optional threading and priority information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// Optional end-to-end encryption envelope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<Encrypted>,
}

impl Signal {
    /// Returns the signal ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the protocol version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Returns a reference to the source.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Returns a reference to the topic.
    pub fn topic(&self) -> &Topic {
        &self.topic
    }

    /// Returns a reference to the payload.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Returns a reference to the metadata, if present.
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Returns a reference to the encrypted envelope, if present.
    pub fn encrypted(&self) -> Option<&Encrypted> {
        self.encrypted.as_ref()
    }

    /// Checks if the signal has metadata.
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Checks if the signal is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encrypted.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Priority;
    use serde_json::json;

    fn create_test_signal() -> Signal {
        Signal {
            id: "sig_1704067200_abc123def456".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: Source::new("email", "email-adapter-1", "msg-12345"),
            topic: Topic::new_unchecked("signal.email.received"),
            payload: Payload::new(
                json!({"from": "alice@example.com", "subject": "Hello"}),
                "application/json",
            ),
            metadata: None,
            encrypted: None,
        }
    }

    #[test]
    fn test_signal_creation() {
        let signal = create_test_signal();
        assert_eq!(signal.id, "sig_1704067200_abc123def456");
        assert_eq!(signal.version, "1.0");
        assert_eq!(signal.source.type_, "email");
        assert_eq!(signal.topic.as_str(), "signal.email.received");
    }

    #[test]
    fn test_signal_accessors() {
        let signal = create_test_signal();
        assert_eq!(signal.id(), "sig_1704067200_abc123def456");
        assert_eq!(signal.version(), "1.0");
        assert_eq!(signal.source().type_, "email");
        assert_eq!(signal.topic().as_str(), "signal.email.received");
        assert_eq!(signal.payload().content_type, "application/json");
    }

    #[test]
    fn test_signal_metadata_accessors() {
        let mut signal = create_test_signal();
        assert!(!signal.has_metadata());
        assert!(signal.metadata().is_none());

        signal.metadata = Some(Metadata::new().priority(Priority::High));
        assert!(signal.has_metadata());
        assert!(signal.metadata().is_some());
        assert_eq!(signal.metadata().unwrap().priority, Some(Priority::High));
    }

    #[test]
    fn test_signal_is_encrypted() {
        let signal = create_test_signal();
        assert!(!signal.is_encrypted());
        assert!(signal.encrypted().is_none());
    }

    #[test]
    fn test_signal_serialization() {
        let signal = create_test_signal();
        let json_str = serde_json::to_string(&signal).unwrap();

        assert!(json_str.contains("\"id\":\"sig_1704067200_abc123def456\""));
        assert!(json_str.contains("\"version\":\"1.0\""));
        assert!(json_str.contains("\"type\":\"email\"")); // Source type
                                                          // None fields should not appear
        assert!(!json_str.contains("\"metadata\""));
        assert!(!json_str.contains("\"encrypted\""));
    }

    #[test]
    fn test_signal_serialization_with_metadata() {
        let mut signal = create_test_signal();
        signal.metadata = Some(Metadata::with_thread("thread-1").priority(Priority::Urgent));

        let json_str = serde_json::to_string(&signal).unwrap();
        assert!(json_str.contains("\"metadata\""));
        assert!(json_str.contains("\"thread_id\":\"thread-1\""));
        assert!(json_str.contains("\"priority\":\"urgent\""));
    }

    #[test]
    fn test_signal_deserialization() {
        let json_str = r#"{
            "id": "sig_1704067200_xyz789abc123",
            "version": "1.0",
            "timestamp": "2024-01-01T12:00:00Z",
            "source": {"type": "slack", "adapter_id": "slack-1", "native_id": "msg-1"},
            "topic": "signal.slack.message",
            "payload": {"raw": {"text": "hello"}, "content_type": "application/json", "size_bytes": 18}
        }"#;

        let signal: Signal = serde_json::from_str(json_str).unwrap();
        assert_eq!(signal.id, "sig_1704067200_xyz789abc123");
        assert_eq!(signal.source.type_, "slack");
        assert_eq!(signal.topic.as_str(), "signal.slack.message");
        assert!(signal.metadata.is_none());
    }

    #[test]
    fn test_signal_round_trip() {
        let signal = Signal {
            id: "sig_1704067200_roundtrip12".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: Source::new("telegram", "tg-adapter", "tg-msg-999"),
            topic: Topic::new_unchecked("signal.telegram.message"),
            payload: Payload::new(
                json!({"chat_id": 12345, "text": "Hello!"}),
                "application/json",
            ),
            metadata: Some(
                Metadata::with_thread("tg-thread")
                    .priority(Priority::Normal)
                    .tags(vec!["bot".to_string(), "greeting".to_string()]),
            ),
            encrypted: None,
        };

        let json_str = serde_json::to_string(&signal).unwrap();
        let restored: Signal = serde_json::from_str(&json_str).unwrap();
        assert_eq!(signal, restored);
    }

    #[test]
    fn test_signal_timestamp() {
        use chrono::Datelike;

        let signal = create_test_signal();
        let ts = signal.timestamp();
        assert_eq!(ts.year(), 2024);
        assert_eq!(ts.month(), 1);
        assert_eq!(ts.day(), 1);
    }
}

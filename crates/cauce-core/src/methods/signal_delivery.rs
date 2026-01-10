//! Signal delivery notification type for the Cauce Protocol.
//!
//! Used to deliver signals to subscribers.

use serde::{Deserialize, Serialize};

use crate::types::Signal;

/// Notification payload for signal delivery.
///
/// Sent via the `cauce.signal` notification method.
///
/// # Example
///
/// ```ignore
/// use cauce_core::methods::SignalDelivery;
/// use cauce_core::types::Signal;
///
/// let delivery = SignalDelivery {
///     topic: "signal.email.received".to_string(),
///     signal,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalDelivery {
    /// The topic the signal was published to
    pub topic: String,

    /// The signal being delivered
    pub signal: Signal,
}

impl SignalDelivery {
    /// Creates a new SignalDelivery.
    pub fn new(topic: impl Into<String>, signal: Signal) -> Self {
        Self {
            topic: topic.into(),
            signal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Payload, Source, Topic};
    use chrono::{DateTime, Utc};
    use serde_json::json;

    fn create_test_signal() -> Signal {
        Signal {
            id: "sig_1704067200_abc123def456".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: Source::new("email", "adapter-1", "msg-1"),
            topic: Topic::new_unchecked("signal.email.received"),
            payload: Payload::new(json!({"text": "hello"}), "application/json"),
            metadata: None,
            encrypted: None,
        }
    }

    #[test]
    fn test_signal_delivery_new() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.email.received", signal.clone());

        assert_eq!(delivery.topic, "signal.email.received");
        assert_eq!(delivery.signal.id, signal.id);
    }

    #[test]
    fn test_signal_delivery_serialization() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.email", signal);
        let json = serde_json::to_string(&delivery).unwrap();

        assert!(json.contains("\"topic\":\"signal.email\""));
        assert!(json.contains("\"signal\":{"));
        assert!(json.contains("\"id\":\"sig_1704067200_abc123def456\""));
    }

    #[test]
    fn test_signal_delivery_deserialization() {
        let json = r#"{
            "topic": "signal.slack.message",
            "signal": {
                "id": "sig_1704067200_xyz789ghi012",
                "version": "1.0",
                "timestamp": "2024-01-01T00:00:00Z",
                "source": {"type": "slack", "adapter_id": "slack-1", "native_id": "msg-1"},
                "topic": "signal.slack.message",
                "payload": {"raw": {"text": "hi"}, "content_type": "application/json", "size_bytes": 14}
            }
        }"#;

        let delivery: SignalDelivery = serde_json::from_str(json).unwrap();
        assert_eq!(delivery.topic, "signal.slack.message");
        assert_eq!(delivery.signal.id, "sig_1704067200_xyz789ghi012");
    }

    #[test]
    fn test_signal_delivery_roundtrip() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test", signal);

        let json = serde_json::to_string(&delivery).unwrap();
        let restored: SignalDelivery = serde_json::from_str(&json).unwrap();
        assert_eq!(delivery, restored);
    }
}

//! Publish method types for the Cauce Protocol.
//!
//! Used to publish signals or actions to topics.

use serde::{Deserialize, Serialize};

use crate::types::{Action, Signal};

/// A message that can be published (either Signal or Action).
///
/// Uses untagged serialization to accept either type based on content.
///
/// # Example
///
/// ```ignore
/// use cauce_core::methods::PublishMessage;
/// use cauce_core::types::Signal;
///
/// // Wrap a signal
/// let message = PublishMessage::Signal(signal);
///
/// // Or wrap an action
/// let message = PublishMessage::Action(action);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PublishMessage {
    /// A signal to be published
    Signal(Signal),
    /// An action to be published
    Action(Action),
}

impl From<Signal> for PublishMessage {
    fn from(signal: Signal) -> Self {
        Self::Signal(signal)
    }
}

impl From<Action> for PublishMessage {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

/// Request parameters for the `cauce.publish` method.
///
/// # Example
///
/// ```ignore
/// use cauce_core::methods::{PublishRequest, PublishMessage};
///
/// let request = PublishRequest {
///     topic: "signal.email.received".to_string(),
///     message: PublishMessage::Signal(signal),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishRequest {
    /// The topic to publish to
    pub topic: String,

    /// The message to publish (Signal or Action)
    pub message: PublishMessage,
}

impl PublishRequest {
    /// Creates a new PublishRequest with a signal.
    pub fn signal(topic: impl Into<String>, signal: Signal) -> Self {
        Self {
            topic: topic.into(),
            message: PublishMessage::Signal(signal),
        }
    }

    /// Creates a new PublishRequest with an action.
    pub fn action(topic: impl Into<String>, action: Action) -> Self {
        Self {
            topic: topic.into(),
            message: PublishMessage::Action(action),
        }
    }
}

/// Response from the `cauce.publish` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::PublishResponse;
///
/// let response = PublishResponse {
///     message_id: "msg_abc123".to_string(),
///     delivered_to: 5,
///     queued_for: 2,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishResponse {
    /// Unique identifier for the published message
    pub message_id: String,

    /// Number of subscribers the message was immediately delivered to
    pub delivered_to: u32,

    /// Number of subscribers the message was queued for (offline/webhook)
    pub queued_for: u32,
}

impl PublishResponse {
    /// Creates a new PublishResponse.
    pub fn new(message_id: impl Into<String>, delivered_to: u32, queued_for: u32) -> Self {
        Self {
            message_id: message_id.into(),
            delivered_to,
            queued_for,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ActionBody, ActionType, Payload, Source, Topic};
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

    fn create_test_action() -> Action {
        Action {
            id: "act_1704067200_abc123def456".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            topic: Topic::new_unchecked("action.email.send"),
            action: ActionBody::new(ActionType::Send, json!({"text": "Hello!"})),
            context: None,
            encrypted: None,
        }
    }

    // ===== PublishMessage Tests =====

    #[test]
    fn test_publish_message_from_signal() {
        let signal = create_test_signal();
        let message: PublishMessage = signal.clone().into();

        match message {
            PublishMessage::Signal(s) => assert_eq!(s.id, signal.id),
            PublishMessage::Action(_) => panic!("Expected Signal"),
        }
    }

    #[test]
    fn test_publish_message_from_action() {
        let action = create_test_action();
        let message: PublishMessage = action.clone().into();

        match message {
            PublishMessage::Action(a) => assert_eq!(a.id, action.id),
            PublishMessage::Signal(_) => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_publish_message_signal_serialization() {
        let signal = create_test_signal();
        let message = PublishMessage::Signal(signal.clone());
        let json = serde_json::to_string(&message).unwrap();

        // Should contain signal fields
        assert!(json.contains("\"id\":\"sig_1704067200_abc123def456\""));
        assert!(json.contains("\"source\""));
        assert!(json.contains("\"payload\""));
    }

    #[test]
    fn test_publish_message_action_serialization() {
        let action = create_test_action();
        let message = PublishMessage::Action(action.clone());
        let json = serde_json::to_string(&message).unwrap();

        // Should contain action fields
        assert!(json.contains("\"id\":\"act_1704067200_abc123def456\""));
        assert!(json.contains("\"action\""));
        assert!(json.contains("\"type\":\"send\""));
    }

    // ===== PublishRequest Tests =====

    #[test]
    fn test_publish_request_signal() {
        let signal = create_test_signal();
        let request = PublishRequest::signal("signal.email.received", signal.clone());

        assert_eq!(request.topic, "signal.email.received");
        match request.message {
            PublishMessage::Signal(s) => assert_eq!(s.id, signal.id),
            _ => panic!("Expected Signal"),
        }
    }

    #[test]
    fn test_publish_request_action() {
        let action = create_test_action();
        let request = PublishRequest::action("action.email.send", action.clone());

        assert_eq!(request.topic, "action.email.send");
        match request.message {
            PublishMessage::Action(a) => assert_eq!(a.id, action.id),
            _ => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_publish_request_serialization() {
        let signal = create_test_signal();
        let request = PublishRequest::signal("test.topic", signal);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"topic\":\"test.topic\""));
        assert!(json.contains("\"message\":"));
    }

    #[test]
    fn test_publish_request_roundtrip_signal() {
        let signal = create_test_signal();
        let request = PublishRequest::signal("signal.email", signal);

        let json = serde_json::to_string(&request).unwrap();
        let restored: PublishRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    #[test]
    fn test_publish_request_roundtrip_action() {
        let action = create_test_action();
        let request = PublishRequest::action("action.email", action);

        let json = serde_json::to_string(&request).unwrap();
        let restored: PublishRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== PublishResponse Tests =====

    #[test]
    fn test_publish_response_new() {
        let response = PublishResponse::new("msg_123", 5, 2);
        assert_eq!(response.message_id, "msg_123");
        assert_eq!(response.delivered_to, 5);
        assert_eq!(response.queued_for, 2);
    }

    #[test]
    fn test_publish_response_serialization() {
        let response = PublishResponse::new("msg_abc", 10, 3);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"message_id\":\"msg_abc\""));
        assert!(json.contains("\"delivered_to\":10"));
        assert!(json.contains("\"queued_for\":3"));
    }

    #[test]
    fn test_publish_response_deserialization() {
        let json = r#"{"message_id":"msg_test","delivered_to":8,"queued_for":1}"#;
        let response: PublishResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.message_id, "msg_test");
        assert_eq!(response.delivered_to, 8);
        assert_eq!(response.queued_for, 1);
    }

    #[test]
    fn test_publish_response_roundtrip() {
        let response = PublishResponse::new("msg_roundtrip", 100, 50);
        let json = serde_json::to_string(&response).unwrap();
        let restored: PublishResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, restored);
    }
}

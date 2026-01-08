//! Integration tests for cauce-core re-exports.

use cauce_core::{
    Action, ActionBody, ActionContext, ActionType, Encrypted, EncryptionAlgorithm, Metadata,
    Payload, Priority, Signal, Source, Topic,
};
use chrono::Utc;
use serde_json::json;

/// Verify Signal type is re-exported at crate root.
#[test]
fn signal_reexported() {
    let signal = Signal {
        id: "sig_1704067200_abc123def456".to_string(),
        version: "1.0".to_string(),
        timestamp: Utc::now(),
        source: Source::new("email", "email-1", "msg-1"),
        topic: Topic::new_unchecked("signal.email.received"),
        payload: Payload::new(json!({}), "application/json"),
        metadata: None,
        encrypted: None,
    };
    assert_eq!(signal.id(), "sig_1704067200_abc123def456");
}

/// Verify Action type is re-exported at crate root.
#[test]
fn action_reexported() {
    let action = Action {
        id: "act_1704067200_xyz789abc123".to_string(),
        version: "1.0".to_string(),
        timestamp: Utc::now(),
        topic: Topic::new_unchecked("action.slack.send"),
        action: ActionBody::new(ActionType::Send, json!({"text": "hello"})),
        context: None,
        encrypted: None,
    };
    assert_eq!(action.id, "act_1704067200_xyz789abc123");
}

/// Verify Topic type is re-exported at crate root.
#[test]
fn topic_reexported() {
    let topic = Topic::new("signal.email.received").unwrap();
    assert_eq!(topic.as_str(), "signal.email.received");
}

/// Verify all types can be imported and used together.
#[test]
fn all_types_usable() {
    // Enums
    let _priority = Priority::High;
    let _action_type = ActionType::Send;
    let _algo = EncryptionAlgorithm::A256Gcm;

    // Supporting types
    let source = Source::new("slack", "slack-1", "msg-1");
    let payload = Payload::new(json!({"key": "value"}), "application/json");
    let metadata = Metadata::new().priority(Priority::Normal);
    let encrypted = Encrypted::new(EncryptionAlgorithm::A256Gcm, "pk", "nonce", "ct");
    let topic = Topic::new("test.topic").unwrap();
    let action_body = ActionBody::new(ActionType::Reply, json!({}));
    let context = ActionContext::new().with_correlation_id("corr-1");

    // Full Signal
    let _signal = Signal {
        id: "sig_1_abc".to_string(),
        version: "1.0".to_string(),
        timestamp: Utc::now(),
        source,
        topic: topic.clone(),
        payload,
        metadata: Some(metadata),
        encrypted: Some(encrypted.clone()),
    };

    // Full Action
    let _action = Action {
        id: "act_1_xyz".to_string(),
        version: "1.0".to_string(),
        timestamp: Utc::now(),
        topic,
        action: action_body,
        context: Some(context),
        encrypted: Some(encrypted),
    };

    assert!(true, "All types can be used together");
}

/// Verify error types are re-exported.
#[test]
fn errors_reexported() {
    use cauce_core::{BuilderError, ValidationError};

    let validation_err = ValidationError::InvalidTopic {
        reason: "test".to_string(),
    };
    assert!(validation_err.to_string().contains("invalid topic"));

    let builder_err = BuilderError::MissingField {
        field: "id".to_string(),
    };
    assert!(builder_err.to_string().contains("missing required field"));
}

/// Verify validation functions are re-exported.
#[test]
fn validation_reexported() {
    use cauce_core::{is_valid_action_id, is_valid_signal_id, is_valid_topic};

    assert!(is_valid_signal_id("sig_1704067200_abc123def456").is_ok());
    assert!(is_valid_action_id("act_1704067200_xyz789abc123").is_ok());
    assert!(is_valid_topic("valid.topic").is_ok());

    assert!(is_valid_signal_id("invalid").is_err());
    assert!(is_valid_action_id("invalid").is_err());
    assert!(is_valid_topic("..invalid").is_err());
}

/// Verify constants are re-exported.
#[test]
fn constants_reexported() {
    use cauce_core::{
        ACTION_ID_PATTERN, ACTION_ID_PREFIX, ID_RANDOM_LENGTH, PROTOCOL_VERSION, SIGNAL_ID_PATTERN,
        SIGNAL_ID_PREFIX, TOPIC_ALLOWED_CHARS, TOPIC_MAX_LENGTH, TOPIC_MIN_LENGTH,
    };

    assert_eq!(SIGNAL_ID_PREFIX, "sig_");
    assert_eq!(ACTION_ID_PREFIX, "act_");
    assert_eq!(ID_RANDOM_LENGTH, 12);
    assert_eq!(TOPIC_MIN_LENGTH, 1);
    assert_eq!(TOPIC_MAX_LENGTH, 255);
    assert_eq!(PROTOCOL_VERSION, "1.0");
    assert!(!SIGNAL_ID_PATTERN.is_empty());
    assert!(!ACTION_ID_PATTERN.is_empty());
    assert!(!TOPIC_ALLOWED_CHARS.is_empty());
}

/// Verify builders are re-exported.
#[test]
fn builders_reexported() {
    use cauce_core::{ActionBuilder, SignalBuilder};

    // Build a signal using the builder
    let signal = SignalBuilder::new()
        .source(Source::new("email", "email-1", "msg-1"))
        .topic(Topic::new_unchecked("signal.email.received"))
        .payload(Payload::new(json!({}), "application/json"))
        .build()
        .unwrap();

    assert!(signal.id.starts_with("sig_"));

    // Build an action using the builder
    let action = ActionBuilder::new()
        .topic(Topic::new_unchecked("action.email.send"))
        .action(ActionBody::new(ActionType::Send, json!({})))
        .build()
        .unwrap();

    assert!(action.id.starts_with("act_"));
}

/// Verify Signal::builder() convenience method works
#[test]
fn signal_builder_method() {
    let signal = Signal::builder()
        .source(Source::new("email", "email-1", "msg-1"))
        .topic(Topic::new_unchecked("signal.email.received"))
        .payload(Payload::new(json!({}), "application/json"))
        .build()
        .unwrap();

    assert!(signal.id.starts_with("sig_"));
}

/// Verify Action::builder() convenience method works
#[test]
fn action_builder_method() {
    let action = Action::builder()
        .topic(Topic::new_unchecked("action.email.send"))
        .action(ActionBody::new(ActionType::Send, json!({})))
        .build()
        .unwrap();

    assert!(action.id.starts_with("act_"));
}

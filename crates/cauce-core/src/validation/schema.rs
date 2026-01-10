//! JSON Schema validation for the Cauce Protocol.
//!
//! This module provides schema validation functions for protocol messages:
//!
//! - [`validate_signal`] - Validate JSON against Signal schema
//! - [`validate_action`] - Validate JSON against Action schema

use crate::errors::ValidationError;
use crate::types::{Action, Signal};
use serde_json::Value;

/// Validates a JSON value as a Signal.
///
/// This function first validates the JSON structure, then attempts to
/// deserialize it into a [`Signal`] type.
///
/// # Arguments
///
/// * `value` - The JSON value to validate
///
/// # Returns
///
/// A validated `Signal` or a `ValidationError` if validation fails.
///
/// # Example
///
/// ```
/// use cauce_core::validation::validate_signal;
/// use serde_json::json;
///
/// let signal_json = json!({
///     "id": "sig_1704067200_abc123def456",
///     "version": "1.0",
///     "timestamp": "2024-01-01T00:00:00Z",
///     "source": { "type": "email", "adapter_id": "adapter-1", "native_id": "msg-1" },
///     "topic": "signal.email.received",
///     "payload": { "raw": {"text": "hello"}, "content_type": "application/json", "size_bytes": 18 }
/// });
///
/// let signal = validate_signal(&signal_json);
/// assert!(signal.is_ok());
/// ```
pub fn validate_signal(value: &Value) -> Result<Signal, ValidationError> {
    // Validate required fields exist
    validate_required_field(value, "id")?;
    validate_required_field(value, "version")?;
    validate_required_field(value, "timestamp")?;
    validate_required_field(value, "source")?;
    validate_required_field(value, "topic")?;
    validate_required_field(value, "payload")?;

    // Validate source structure
    if let Some(source) = value.get("source") {
        validate_required_field(source, "type")?;
        validate_required_field(source, "adapter_id")?;
        validate_required_field(source, "native_id")?;
    }

    // Validate payload structure
    if let Some(payload) = value.get("payload") {
        validate_required_field(payload, "raw")?;
        validate_required_field(payload, "content_type")?;
        validate_required_field(payload, "size_bytes")?;
    }

    // Deserialize and return
    serde_json::from_value(value.clone()).map_err(|e| ValidationError::InvalidField {
        field: "signal".to_string(),
        reason: format!("deserialization failed: {}", e),
    })
}

/// Validates a JSON value as an Action.
///
/// This function first validates the JSON structure, then attempts to
/// deserialize it into an [`Action`] type.
///
/// # Arguments
///
/// * `value` - The JSON value to validate
///
/// # Returns
///
/// A validated `Action` or a `ValidationError` if validation fails.
///
/// # Example
///
/// ```
/// use cauce_core::validation::validate_action;
/// use serde_json::json;
///
/// let action_json = json!({
///     "id": "act_1704067200_abc123def456",
///     "version": "1.0",
///     "timestamp": "2024-01-01T00:00:00Z",
///     "topic": "action.email.send",
///     "action": { "type": "send", "payload": { "text": "Hello!" } }
/// });
///
/// let action = validate_action(&action_json);
/// assert!(action.is_ok());
/// ```
pub fn validate_action(value: &Value) -> Result<Action, ValidationError> {
    // Validate required fields exist
    validate_required_field(value, "id")?;
    validate_required_field(value, "version")?;
    validate_required_field(value, "timestamp")?;
    validate_required_field(value, "topic")?;
    validate_required_field(value, "action")?;

    // Validate action body structure
    if let Some(action_body) = value.get("action") {
        validate_required_field(action_body, "type")?;
        validate_required_field(action_body, "payload")?;
    }

    // Deserialize and return
    serde_json::from_value(value.clone()).map_err(|e| ValidationError::InvalidField {
        field: "action".to_string(),
        reason: format!("deserialization failed: {}", e),
    })
}

/// Validates that a required field exists in a JSON object.
fn validate_required_field(value: &Value, field: &str) -> Result<(), ValidationError> {
    if value.get(field).is_none() {
        return Err(ValidationError::InvalidField {
            field: field.to_string(),
            reason: "required field is missing".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ===== Signal Validation Tests =====

    #[test]
    fn test_validate_signal_valid() {
        let signal_json = json!({
            "id": "sig_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "source": {
                "type": "email",
                "adapter_id": "email-adapter-1",
                "native_id": "msg-12345"
            },
            "topic": "signal.email.received",
            "payload": {
                "raw": {"from": "alice@example.com", "subject": "Hello"},
                "content_type": "application/json",
                "size_bytes": 42
            }
        });

        let result = validate_signal(&signal_json);
        assert!(result.is_ok());
        let signal = result.unwrap();
        assert_eq!(signal.id, "sig_1704067200_abc123def456");
    }

    #[test]
    fn test_validate_signal_missing_id() {
        let signal_json = json!({
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "source": { "type": "email", "adapter_id": "a", "native_id": "n" },
            "topic": "signal.email",
            "payload": { "raw": {}, "content_type": "application/json", "size_bytes": 0 }
        });

        let result = validate_signal(&signal_json);
        assert!(result.is_err());
        if let Err(ValidationError::InvalidField { field, .. }) = result {
            assert_eq!(field, "id");
        } else {
            panic!("Expected InvalidField error for 'id'");
        }
    }

    #[test]
    fn test_validate_signal_missing_source_type() {
        let signal_json = json!({
            "id": "sig_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "source": { "adapter_id": "a", "native_id": "n" },
            "topic": "signal.email",
            "payload": { "raw": {}, "content_type": "application/json", "size_bytes": 0 }
        });

        let result = validate_signal(&signal_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_signal_with_metadata() {
        let signal_json = json!({
            "id": "sig_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "source": { "type": "slack", "adapter_id": "slack-1", "native_id": "msg-1" },
            "topic": "signal.slack.message",
            "payload": { "raw": {"text": "hello"}, "content_type": "application/json", "size_bytes": 18 },
            "metadata": { "thread_id": "thread-123", "priority": "high" }
        });

        let result = validate_signal(&signal_json);
        assert!(result.is_ok());
        let signal = result.unwrap();
        assert!(signal.metadata.is_some());
    }

    // ===== Action Validation Tests =====

    #[test]
    fn test_validate_action_valid() {
        let action_json = json!({
            "id": "act_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "topic": "action.email.send",
            "action": {
                "type": "send",
                "payload": { "to": "bob@example.com", "subject": "Hi", "body": "Hello!" }
            }
        });

        let result = validate_action(&action_json);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action.id, "act_1704067200_abc123def456");
    }

    #[test]
    fn test_validate_action_missing_action() {
        let action_json = json!({
            "id": "act_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "topic": "action.email.send"
        });

        let result = validate_action(&action_json);
        assert!(result.is_err());
        if let Err(ValidationError::InvalidField { field, .. }) = result {
            assert_eq!(field, "action");
        } else {
            panic!("Expected InvalidField error for 'action'");
        }
    }

    #[test]
    fn test_validate_action_missing_action_type() {
        let action_json = json!({
            "id": "act_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "topic": "action.email.send",
            "action": { "payload": {} }
        });

        let result = validate_action(&action_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_action_with_context() {
        let action_json = json!({
            "id": "act_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "topic": "action.slack.send",
            "action": { "type": "send", "payload": {"text": "Hello!"} },
            "context": { "in_reply_to": "sig_123_abc123def456" }
        });

        let result = validate_action(&action_json);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert!(action.context.is_some());
    }

    // ===== Edge Cases =====

    #[test]
    fn test_validate_signal_empty_object() {
        let signal_json = json!({});
        let result = validate_signal(&signal_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_action_empty_object() {
        let action_json = json!({});
        let result = validate_action(&action_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_signal_invalid_timestamp_format() {
        let signal_json = json!({
            "id": "sig_1704067200_abc123def456",
            "version": "1.0",
            "timestamp": "not-a-valid-timestamp",
            "source": { "type": "email", "adapter_id": "a", "native_id": "n" },
            "topic": "signal.email",
            "payload": { "raw": {}, "content_type": "application/json", "size_bytes": 0 }
        });

        let result = validate_signal(&signal_json);
        assert!(result.is_err());
    }
}

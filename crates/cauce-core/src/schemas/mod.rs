//! Embedded JSON schemas for the Cauce Protocol.
//!
//! This module provides access to embedded JSON schemas for protocol validation.
//! Schemas are compiled at build time using `include_str!`.
//!
//! ## Available Schemas
//!
//! - Signal schema
//! - Action schema
//! - JSON-RPC schema
//! - Error schemas

/// Embedded Signal JSON schema (placeholder).
///
/// In a full implementation, this would be loaded from a schema file.
pub const SIGNAL_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cauce.ai/schemas/signal.json",
  "title": "Signal",
  "type": "object",
  "required": ["id", "version", "timestamp", "source", "topic", "payload"],
  "properties": {
    "id": { "type": "string", "pattern": "^sig_\\d+_[a-zA-Z0-9]{12}$" },
    "version": { "type": "string" },
    "timestamp": { "type": "string", "format": "date-time" },
    "source": { "$ref": "#/$defs/source" },
    "topic": { "type": "string" },
    "payload": { "$ref": "#/$defs/payload" },
    "metadata": { "$ref": "#/$defs/metadata" },
    "encrypted": { "$ref": "#/$defs/encrypted" }
  },
  "$defs": {
    "source": {
      "type": "object",
      "required": ["type", "adapter_id", "native_id"],
      "properties": {
        "type": { "type": "string" },
        "adapter_id": { "type": "string" },
        "native_id": { "type": "string" }
      }
    },
    "payload": {
      "type": "object",
      "required": ["raw", "content_type", "size_bytes"],
      "properties": {
        "raw": {},
        "content_type": { "type": "string" },
        "size_bytes": { "type": "integer", "minimum": 0 }
      }
    },
    "metadata": {
      "type": "object",
      "properties": {
        "thread_id": { "type": "string" },
        "priority": { "enum": ["low", "normal", "high", "urgent"] },
        "tags": { "type": "array", "items": { "type": "string" } }
      }
    },
    "encrypted": {
      "type": "object",
      "required": ["algorithm", "recipient_public_key", "nonce", "ciphertext"],
      "properties": {
        "algorithm": { "type": "string" },
        "recipient_public_key": { "type": "string" },
        "nonce": { "type": "string" },
        "ciphertext": { "type": "string" }
      }
    }
  }
}"##;

/// Embedded Action JSON schema (placeholder).
pub const ACTION_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cauce.ai/schemas/action.json",
  "title": "Action",
  "type": "object",
  "required": ["id", "version", "timestamp", "target", "topic", "body"],
  "properties": {
    "id": { "type": "string", "pattern": "^act_\\d+_[a-zA-Z0-9]{12}$" },
    "version": { "type": "string" },
    "timestamp": { "type": "string", "format": "date-time" },
    "target": { "$ref": "#/$defs/target" },
    "topic": { "type": "string" },
    "body": { "$ref": "#/$defs/body" },
    "context": { "$ref": "#/$defs/context" }
  },
  "$defs": {
    "target": {
      "type": "object",
      "required": ["type", "adapter_id", "native_id"],
      "properties": {
        "type": { "type": "string" },
        "adapter_id": { "type": "string" },
        "native_id": { "type": "string" }
      }
    },
    "body": {
      "type": "object",
      "required": ["type", "content"],
      "properties": {
        "type": { "enum": ["send", "reply", "forward", "react", "update", "delete"] },
        "content": {}
      }
    },
    "context": {
      "type": "object",
      "properties": {
        "reply_to_signal_id": { "type": "string" },
        "correlation_id": { "type": "string" }
      }
    }
  }
}"##;

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "schemas: Embedded JSON schemas for protocol validation"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_schema_is_valid_json() {
        let result: Result<serde_json::Value, _> = serde_json::from_str(SIGNAL_SCHEMA);
        assert!(result.is_ok(), "Signal schema should be valid JSON");
    }

    #[test]
    fn test_action_schema_is_valid_json() {
        let result: Result<serde_json::Value, _> = serde_json::from_str(ACTION_SCHEMA);
        assert!(result.is_ok(), "Action schema should be valid JSON");
    }

    #[test]
    fn test_signal_schema_has_required_fields() {
        let schema: serde_json::Value = serde_json::from_str(SIGNAL_SCHEMA).unwrap();
        let required = schema["required"].as_array().unwrap();

        assert!(required.contains(&serde_json::json!("id")));
        assert!(required.contains(&serde_json::json!("version")));
        assert!(required.contains(&serde_json::json!("timestamp")));
        assert!(required.contains(&serde_json::json!("source")));
        assert!(required.contains(&serde_json::json!("topic")));
        assert!(required.contains(&serde_json::json!("payload")));
    }

    #[test]
    fn test_action_schema_has_required_fields() {
        let schema: serde_json::Value = serde_json::from_str(ACTION_SCHEMA).unwrap();
        let required = schema["required"].as_array().unwrap();

        assert!(required.contains(&serde_json::json!("id")));
        assert!(required.contains(&serde_json::json!("version")));
        assert!(required.contains(&serde_json::json!("timestamp")));
        assert!(required.contains(&serde_json::json!("target")));
        assert!(required.contains(&serde_json::json!("topic")));
        assert!(required.contains(&serde_json::json!("body")));
    }
}

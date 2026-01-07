//! Payload type for the Cauce Protocol.
//!
//! The [`Payload`] struct contains the actual message content.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The actual message content with type information.
///
/// Payload wraps the raw message content along with metadata
/// about the content type and size.
///
/// # Fields
///
/// - `raw` - Arbitrary JSON content (can be any valid JSON value)
/// - `content_type` - MIME type (e.g., "application/json", "text/plain")
/// - `size_bytes` - Size of the raw content in bytes
///
/// # Example
///
/// ```
/// use cauce_core::types::Payload;
/// use serde_json::json;
///
/// let payload = Payload {
///     raw: json!({"from": "alice@example.com", "subject": "Hello"}),
///     content_type: "application/json".to_string(),
///     size_bytes: 50,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    /// Arbitrary JSON content
    pub raw: Value,

    /// MIME type (e.g., "application/json", "text/plain")
    pub content_type: String,

    /// Size of raw content in bytes
    pub size_bytes: u64,
}

impl Payload {
    /// Creates a new Payload with the given content.
    ///
    /// The size_bytes is automatically calculated from the serialized JSON.
    ///
    /// # Arguments
    ///
    /// * `raw` - The JSON content
    /// * `content_type` - The MIME type of the content
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Payload;
    /// use serde_json::json;
    ///
    /// let payload = Payload::new(json!({"key": "value"}), "application/json");
    /// assert!(payload.size_bytes > 0);
    /// ```
    pub fn new(raw: Value, content_type: impl Into<String>) -> Self {
        let size_bytes = serde_json::to_string(&raw)
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        Self {
            raw,
            content_type: content_type.into(),
            size_bytes,
        }
    }

    /// Creates a Payload with explicit size.
    ///
    /// Use this when you have a pre-computed size or need exact control.
    ///
    /// # Arguments
    ///
    /// * `raw` - The JSON content
    /// * `content_type` - The MIME type of the content
    /// * `size_bytes` - The size in bytes
    pub fn with_size(raw: Value, content_type: impl Into<String>, size_bytes: u64) -> Self {
        Self {
            raw,
            content_type: content_type.into(),
            size_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_payload_new() {
        let payload = Payload::new(json!({"key": "value"}), "application/json");
        assert_eq!(payload.content_type, "application/json");
        assert!(payload.size_bytes > 0);
    }

    #[test]
    fn test_payload_with_size() {
        let payload = Payload::with_size(json!(null), "text/plain", 100);
        assert_eq!(payload.size_bytes, 100);
    }

    #[test]
    fn test_payload_serialization() {
        let payload = Payload {
            raw: json!({"from": "alice"}),
            content_type: "application/json".to_string(),
            size_bytes: 17,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"raw\":{\"from\":\"alice\"}"));
        assert!(json.contains("\"content_type\":\"application/json\""));
        assert!(json.contains("\"size_bytes\":17"));
    }

    #[test]
    fn test_payload_deserialization() {
        let json = r#"{"raw":{"msg":"hello"},"content_type":"text/plain","size_bytes":15}"#;
        let payload: Payload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.raw, json!({"msg": "hello"}));
        assert_eq!(payload.content_type, "text/plain");
        assert_eq!(payload.size_bytes, 15);
    }

    #[test]
    fn test_payload_round_trip() {
        let payload = Payload::new(
            json!({"complex": [1, 2, 3], "nested": {"a": "b"}}),
            "application/json",
        );
        let serialized = serde_json::to_string(&payload).unwrap();
        let restored: Payload = serde_json::from_str(&serialized).unwrap();
        assert_eq!(payload, restored);
    }
}

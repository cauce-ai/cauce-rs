//! Source type for the Cauce Protocol.
//!
//! The [`Source`] struct identifies where a signal originated.

use serde::{Deserialize, Serialize};

/// Identifies where a signal originated.
///
/// Source contains information about the adapter that generated
/// a signal and the native identifier from the source platform.
///
/// # Fields
///
/// - `type_` - The adapter type (e.g., "email", "slack", "telegram")
/// - `adapter_id` - Unique identifier for the adapter instance
/// - `native_id` - Platform-specific message ID
///
/// # JSON Serialization
///
/// The `type_` field is serialized as `"type"` to match the JSON schema
/// (avoiding Rust's reserved keyword).
///
/// # Example
///
/// ```
/// use cauce_core::types::Source;
///
/// let source = Source {
///     type_: "email".to_string(),
///     adapter_id: "email-adapter-1".to_string(),
///     native_id: "msg-12345".to_string(),
/// };
///
/// let json = serde_json::to_string(&source).unwrap();
/// assert!(json.contains("\"type\":\"email\""));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Adapter type (e.g., "email", "slack", "telegram")
    #[serde(rename = "type")]
    pub type_: String,

    /// Unique adapter instance identifier
    pub adapter_id: String,

    /// Platform-specific message ID
    pub native_id: String,
}

impl Source {
    /// Creates a new Source.
    ///
    /// # Arguments
    ///
    /// * `type_` - The adapter type
    /// * `adapter_id` - The adapter instance identifier
    /// * `native_id` - The platform-specific message ID
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Source;
    ///
    /// let source = Source::new("email", "email-adapter-1", "msg-12345");
    /// assert_eq!(source.type_, "email");
    /// ```
    pub fn new(
        type_: impl Into<String>,
        adapter_id: impl Into<String>,
        native_id: impl Into<String>,
    ) -> Self {
        Self {
            type_: type_.into(),
            adapter_id: adapter_id.into(),
            native_id: native_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_new() {
        let source = Source::new("email", "adapter-1", "native-123");
        assert_eq!(source.type_, "email");
        assert_eq!(source.adapter_id, "adapter-1");
        assert_eq!(source.native_id, "native-123");
    }

    #[test]
    fn test_source_serialization() {
        let source = Source {
            type_: "slack".to_string(),
            adapter_id: "slack-1".to_string(),
            native_id: "12345".to_string(),
        };

        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"type\":\"slack\""));
        assert!(json.contains("\"adapter_id\":\"slack-1\""));
        assert!(json.contains("\"native_id\":\"12345\""));
    }

    #[test]
    fn test_source_deserialization() {
        let json = r#"{"type":"email","adapter_id":"email-1","native_id":"msg-1"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        assert_eq!(source.type_, "email");
        assert_eq!(source.adapter_id, "email-1");
        assert_eq!(source.native_id, "msg-1");
    }

    #[test]
    fn test_source_round_trip() {
        let source = Source::new("telegram", "tg-adapter", "tg-msg-789");
        let json = serde_json::to_string(&source).unwrap();
        let restored: Source = serde_json::from_str(&json).unwrap();
        assert_eq!(source, restored);
    }
}

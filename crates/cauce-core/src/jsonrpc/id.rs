//! RequestId type for JSON-RPC 2.0.
//!
//! This module provides the [`RequestId`] type which can be either a string or an integer,
//! as specified in the JSON-RPC 2.0 specification.

use serde::{Deserialize, Serialize};

/// A JSON-RPC 2.0 request identifier.
///
/// Per the JSON-RPC 2.0 specification, the id can be either a string or an integer.
/// The id is used to correlate requests with responses.
///
/// # Examples
///
/// ```
/// use cauce_core::jsonrpc::RequestId;
///
/// // Create from string
/// let id = RequestId::String("request-123".to_string());
///
/// // Create from integer
/// let id = RequestId::Number(42);
///
/// // Use helper constructors
/// let id = RequestId::from_string("request-456");
/// let id = RequestId::from_number(123);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// String identifier
    String(String),
    /// Numeric identifier
    Number(i64),
}

impl RequestId {
    /// Creates a new RequestId from a string.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::RequestId;
    ///
    /// let id = RequestId::from_string("my-request");
    /// assert!(matches!(id, RequestId::String(_)));
    /// ```
    pub fn from_string(s: impl Into<String>) -> Self {
        RequestId::String(s.into())
    }

    /// Creates a new RequestId from a number.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::RequestId;
    ///
    /// let id = RequestId::from_number(42);
    /// assert!(matches!(id, RequestId::Number(42)));
    /// ```
    pub fn from_number(n: i64) -> Self {
        RequestId::Number(n)
    }

    /// Returns true if this is a string id.
    pub fn is_string(&self) -> bool {
        matches!(self, RequestId::String(_))
    }

    /// Returns true if this is a numeric id.
    pub fn is_number(&self) -> bool {
        matches!(self, RequestId::Number(_))
    }

    /// Returns the string value if this is a string id.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            RequestId::String(s) => Some(s),
            RequestId::Number(_) => None,
        }
    }

    /// Returns the numeric value if this is a numeric id.
    pub fn as_number(&self) -> Option<i64> {
        match self {
            RequestId::String(_) => None,
            RequestId::Number(n) => Some(*n),
        }
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<i32> for RequestId {
    fn from(n: i32) -> Self {
        RequestId::Number(n as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T007: Write unit tests for RequestId string variant serialization
    #[test]
    fn test_string_id_serialization() {
        let id = RequestId::String("test-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"test-123\"");
    }

    #[test]
    fn test_string_id_deserialization() {
        let json = "\"request-abc\"";
        let id: RequestId = serde_json::from_str(json).unwrap();
        assert_eq!(id, RequestId::String("request-abc".to_string()));
    }

    // T008: Write unit tests for RequestId integer variant serialization
    #[test]
    fn test_number_id_serialization() {
        let id = RequestId::Number(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_number_id_deserialization() {
        let json = "123";
        let id: RequestId = serde_json::from_str(json).unwrap();
        assert_eq!(id, RequestId::Number(123));
    }

    #[test]
    fn test_negative_number_id() {
        let id = RequestId::Number(-1);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "-1");

        let restored: RequestId = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, id);
    }

    // T009: Write unit tests for RequestId roundtrip (type preservation)
    #[test]
    fn test_string_id_roundtrip() {
        let original = RequestId::String("roundtrip-test".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let restored: RequestId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
        assert!(restored.is_string());
    }

    #[test]
    fn test_number_id_roundtrip() {
        let original = RequestId::Number(999);
        let json = serde_json::to_string(&original).unwrap();
        let restored: RequestId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
        assert!(restored.is_number());
    }

    #[test]
    fn test_from_string_helper() {
        let id = RequestId::from_string("helper-test");
        assert!(matches!(id, RequestId::String(s) if s == "helper-test"));
    }

    #[test]
    fn test_from_number_helper() {
        let id = RequestId::from_number(77);
        assert!(matches!(id, RequestId::Number(77)));
    }

    #[test]
    fn test_from_impl() {
        let id: RequestId = "from-impl".into();
        assert!(id.is_string());

        let id: RequestId = 42i64.into();
        assert!(id.is_number());

        let id: RequestId = 42i32.into();
        assert!(id.is_number());
    }

    #[test]
    fn test_accessor_methods() {
        let string_id = RequestId::String("test".to_string());
        assert_eq!(string_id.as_string(), Some("test"));
        assert_eq!(string_id.as_number(), None);

        let number_id = RequestId::Number(42);
        assert_eq!(number_id.as_string(), None);
        assert_eq!(number_id.as_number(), Some(42));
    }
}

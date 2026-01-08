//! JsonRpcError type for JSON-RPC 2.0.
//!
//! This module provides the [`JsonRpcError`] type for structured error responses
//! as specified in the JSON-RPC 2.0 specification.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 error object.
///
/// Per the JSON-RPC 2.0 specification, error objects contain:
/// - `code`: An integer error code
/// - `message`: A human-readable error message
/// - `data`: Optional additional data about the error
///
/// # Standard Error Codes
///
/// | Code   | Message          | Meaning                              |
/// |--------|------------------|--------------------------------------|
/// | -32700 | Parse error      | Invalid JSON was received            |
/// | -32600 | Invalid Request  | Not a valid JSON-RPC request object  |
/// | -32601 | Method not found | The method does not exist            |
/// | -32602 | Invalid params   | Invalid method parameter(s)          |
/// | -32603 | Internal error   | Internal JSON-RPC error              |
///
/// # Examples
///
/// ```
/// use cauce_core::jsonrpc::JsonRpcError;
/// use serde_json::json;
///
/// // Create a simple error
/// let error = JsonRpcError::new(-32600, "Invalid Request");
///
/// // Create an error with additional data
/// let error = JsonRpcError::with_data(
///     -32602,
///     "Invalid params",
///     json!({"field": "topics", "reason": "must be non-empty"}),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code (integer)
    pub code: i32,

    /// Human-readable error message
    pub message: String,

    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Creates a new error with code and message.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::JsonRpcError;
    ///
    /// let error = JsonRpcError::new(-32601, "Method not found");
    /// assert_eq!(error.code, -32601);
    /// assert_eq!(error.message, "Method not found");
    /// assert!(error.data.is_none());
    /// ```
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Creates a new error with code, message, and additional data.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::JsonRpcError;
    /// use serde_json::json;
    ///
    /// let error = JsonRpcError::with_data(
    ///     -32602,
    ///     "Invalid params",
    ///     json!({"missing": ["topic"]}),
    /// );
    /// assert!(error.data.is_some());
    /// ```
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Creates a parse error (-32700).
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    /// Creates an invalid request error (-32600).
    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request")
    }

    /// Creates a method not found error (-32601).
    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found")
    }

    /// Creates an invalid params error (-32602).
    pub fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params")
    }

    /// Creates an internal error (-32603).
    pub fn internal_error() -> Self {
        Self::new(-32603, "Internal error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T045: Write test: error with code, message, data serializes all fields
    #[test]
    fn test_error_with_all_fields_serialization() {
        let error = JsonRpcError::with_data(-32602, "Invalid params", json!({"field": "topics"}));

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"code\":-32602"));
        assert!(json.contains("\"message\":\"Invalid params\""));
        assert!(json.contains("\"data\":{\"field\":\"topics\"}"));
    }

    // T046: Write test: error without data omits data field
    #[test]
    fn test_error_without_data_omits_field() {
        let error = JsonRpcError::new(-32601, "Method not found");

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"code\":-32601"));
        assert!(json.contains("\"message\":\"Method not found\""));
        assert!(!json.contains("\"data\""));
    }

    // T047: Write test: standard error codes (-32700, -32600, etc.) are representable
    #[test]
    fn test_standard_error_codes() {
        let parse_error = JsonRpcError::parse_error();
        assert_eq!(parse_error.code, -32700);

        let invalid_request = JsonRpcError::invalid_request();
        assert_eq!(invalid_request.code, -32600);

        let method_not_found = JsonRpcError::method_not_found();
        assert_eq!(method_not_found.code, -32601);

        let invalid_params = JsonRpcError::invalid_params();
        assert_eq!(invalid_params.code, -32602);

        let internal_error = JsonRpcError::internal_error();
        assert_eq!(internal_error.code, -32603);
    }

    // T048: Write test: error deserialization works correctly
    #[test]
    fn test_error_deserialization() {
        let json = r#"{"code":-32600,"message":"Invalid Request"}"#;
        let error: JsonRpcError = serde_json::from_str(json).unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_error_deserialization_with_data() {
        let json =
            r#"{"code":-32602,"message":"Invalid params","data":{"reason":"missing field"}}"#;
        let error: JsonRpcError = serde_json::from_str(json).unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.data.is_some());
        assert_eq!(error.data.unwrap()["reason"], "missing field");
    }

    // T049: Write test: error roundtrip preserves all fields
    #[test]
    fn test_error_roundtrip_simple() {
        let original = JsonRpcError::new(-32603, "Internal error");
        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_error_roundtrip_with_data() {
        let original = JsonRpcError::with_data(
            -32602,
            "Invalid params",
            json!({"field": "topics", "reason": "must be array"}),
        );
        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_error_new_constructor() {
        let error = JsonRpcError::new(-1, "Custom error");
        assert_eq!(error.code, -1);
        assert_eq!(error.message, "Custom error");
        assert!(error.data.is_none());
    }
}

//! JsonRpcRequest type for JSON-RPC 2.0.
//!
//! This module provides the [`JsonRpcRequest`] type for constructing
//! JSON-RPC 2.0 request messages.

use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::RequestId;

/// The JSON-RPC version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// A JSON-RPC 2.0 request message.
///
/// Per the JSON-RPC 2.0 specification, a request contains:
/// - `jsonrpc`: Must be exactly "2.0"
/// - `id`: An identifier for correlating requests with responses
/// - `method`: The name of the method to invoke
/// - `params`: Optional parameters for the method
///
/// # Examples
///
/// ```
/// use cauce_core::jsonrpc::{JsonRpcRequest, RequestId};
/// use serde_json::json;
///
/// // Create a request with params
/// let request = JsonRpcRequest::new(
///     RequestId::from_number(1),
///     "cauce.subscribe",
///     Some(json!({"topics": ["signal.email.*"]})),
/// );
///
/// // Create a request without params
/// let request = JsonRpcRequest::new(
///     RequestId::from_string("req-1"),
///     "cauce.ping",
///     None,
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request identifier
    pub id: RequestId,

    /// Method name
    pub method: String,

    /// Optional method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Creates a new JSON-RPC request.
    ///
    /// The `jsonrpc` field is automatically set to "2.0".
    ///
    /// # Arguments
    ///
    /// * `id` - The request identifier
    /// * `method` - The method name to invoke
    /// * `params` - Optional parameters for the method
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::{JsonRpcRequest, RequestId};
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest::new(
    ///     RequestId::from_number(1),
    ///     "cauce.publish",
    ///     Some(json!({"topic": "signal.test"})),
    /// );
    ///
    /// assert_eq!(request.jsonrpc, "2.0");
    /// assert_eq!(request.method, "cauce.publish");
    /// ```
    pub fn new(id: RequestId, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }

    /// Returns the request id.
    pub fn id(&self) -> &RequestId {
        &self.id
    }

    /// Returns the method name.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns a reference to the params if present.
    pub fn params(&self) -> Option<&Value> {
        self.params.as_ref()
    }
}

impl<'de> Deserialize<'de> for JsonRpcRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRequest {
            jsonrpc: String,
            id: RequestId,
            method: String,
            params: Option<Value>,
        }

        let raw = RawRequest::deserialize(deserializer)?;

        // Validate jsonrpc version
        if raw.jsonrpc != JSONRPC_VERSION {
            return Err(de::Error::custom(format!(
                "invalid jsonrpc version: expected '{}', got '{}'",
                JSONRPC_VERSION, raw.jsonrpc
            )));
        }

        Ok(JsonRpcRequest {
            jsonrpc: raw.jsonrpc,
            id: raw.id,
            method: raw.method,
            params: raw.params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T013: Write test: request serialization includes jsonrpc "2.0"
    #[test]
    fn test_request_serialization_includes_jsonrpc() {
        let request = JsonRpcRequest::new(RequestId::from_number(1), "test.method", None);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
    }

    // T014: Write test: request with string id serializes correctly
    #[test]
    fn test_request_with_string_id() {
        let request = JsonRpcRequest::new(RequestId::from_string("req-123"), "test.method", None);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"id\":\"req-123\""));
    }

    // T015: Write test: request with integer id serializes correctly
    #[test]
    fn test_request_with_integer_id() {
        let request = JsonRpcRequest::new(RequestId::from_number(42), "test.method", None);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"id\":42"));
    }

    // T016: Write test: request with params serializes correctly
    #[test]
    fn test_request_with_params() {
        let request = JsonRpcRequest::new(
            RequestId::from_number(1),
            "cauce.subscribe",
            Some(json!({"topics": ["signal.*"]})),
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"params\":{\"topics\":[\"signal.*\"]}"));
    }

    // T017: Write test: request without params omits params field
    #[test]
    fn test_request_without_params_omits_field() {
        let request = JsonRpcRequest::new(RequestId::from_number(1), "cauce.ping", None);

        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("\"params\""));
    }

    // T018: Write test: request deserialization validates jsonrpc == "2.0"
    #[test]
    fn test_request_deserialization_validates_version() {
        // Valid version
        let valid_json = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let result: Result<JsonRpcRequest, _> = serde_json::from_str(valid_json);
        assert!(result.is_ok());

        // Invalid version
        let invalid_json = r#"{"jsonrpc":"1.0","id":1,"method":"test"}"#;
        let result: Result<JsonRpcRequest, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid jsonrpc version"));
    }

    #[test]
    fn test_request_deserialization_with_params() {
        let json = r#"{"jsonrpc":"2.0","id":"req-1","method":"test","params":{"key":"value"}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, RequestId::from_string("req-1"));
        assert_eq!(request.method, "test");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_request_roundtrip() {
        let original = JsonRpcRequest::new(
            RequestId::from_number(42),
            "cauce.subscribe",
            Some(json!({"topics": ["signal.email.*"]})),
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_accessor_methods() {
        let request = JsonRpcRequest::new(
            RequestId::from_number(1),
            "test.method",
            Some(json!({"key": "value"})),
        );

        assert_eq!(request.id(), &RequestId::from_number(1));
        assert_eq!(request.method(), "test.method");
        assert!(request.params().is_some());
    }
}

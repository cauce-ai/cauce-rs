//! JsonRpcResponse type for JSON-RPC 2.0.
//!
//! This module provides the [`JsonRpcResponse`] type for parsing and constructing
//! JSON-RPC 2.0 response messages.

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::request::JSONRPC_VERSION;
use super::{JsonRpcError, RequestId};

/// A JSON-RPC 2.0 response message.
///
/// Per the JSON-RPC 2.0 specification, a response contains:
/// - `jsonrpc`: Must be exactly "2.0"
/// - `id`: The identifier from the request (or null for errors to unidentifiable requests)
/// - Either `result` (for success) OR `error` (for failure), never both
///
/// # Examples
///
/// ```
/// use cauce_core::jsonrpc::{JsonRpcResponse, JsonRpcError, RequestId};
/// use serde_json::json;
///
/// // Parse a success response
/// let json = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
/// let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
/// assert!(response.is_success());
///
/// // Create a success response
/// let response = JsonRpcResponse::success(
///     RequestId::from_number(1),
///     json!({"status": "ok"}),
/// );
///
/// // Create an error response
/// let response = JsonRpcResponse::error(
///     Some(RequestId::from_number(1)),
///     JsonRpcError::new(-32600, "Invalid Request"),
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum JsonRpcResponse {
    /// A successful response with a result value
    Success {
        /// JSON-RPC version (always "2.0")
        jsonrpc: String,
        /// Request identifier
        id: RequestId,
        /// Result value
        result: Value,
    },
    /// An error response
    Error {
        /// JSON-RPC version (always "2.0")
        jsonrpc: String,
        /// Request identifier (may be null for parse errors)
        id: Option<RequestId>,
        /// Error object
        error: JsonRpcError,
    },
}

impl JsonRpcResponse {
    /// Creates a success response.
    ///
    /// # Arguments
    ///
    /// * `id` - The request identifier
    /// * `result` - The result value
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::{JsonRpcResponse, RequestId};
    /// use serde_json::json;
    ///
    /// let response = JsonRpcResponse::success(
    ///     RequestId::from_number(1),
    ///     json!({"status": "ok"}),
    /// );
    /// assert!(response.is_success());
    /// ```
    pub fn success(id: RequestId, result: Value) -> Self {
        JsonRpcResponse::Success {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result,
        }
    }

    /// Creates an error response.
    ///
    /// # Arguments
    ///
    /// * `id` - The request identifier (None for parse errors where id is unknown)
    /// * `error` - The error object
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::{JsonRpcResponse, JsonRpcError, RequestId};
    ///
    /// let response = JsonRpcResponse::error(
    ///     Some(RequestId::from_number(1)),
    ///     JsonRpcError::new(-32600, "Invalid Request"),
    /// );
    /// assert!(response.is_error());
    /// ```
    pub fn error(id: Option<RequestId>, error: JsonRpcError) -> Self {
        JsonRpcResponse::Error {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            error,
        }
    }

    /// Returns true if this is a success response.
    pub fn is_success(&self) -> bool {
        matches!(self, JsonRpcResponse::Success { .. })
    }

    /// Returns true if this is an error response.
    pub fn is_error(&self) -> bool {
        matches!(self, JsonRpcResponse::Error { .. })
    }

    /// Returns the response id.
    ///
    /// For success responses, this is always Some.
    /// For error responses, this may be None for parse errors.
    pub fn id(&self) -> Option<&RequestId> {
        match self {
            JsonRpcResponse::Success { id, .. } => Some(id),
            JsonRpcResponse::Error { id, .. } => id.as_ref(),
        }
    }

    /// Returns the result if this is a success response.
    pub fn result(&self) -> Option<&Value> {
        match self {
            JsonRpcResponse::Success { result, .. } => Some(result),
            JsonRpcResponse::Error { .. } => None,
        }
    }

    /// Returns the error if this is an error response.
    pub fn error_obj(&self) -> Option<&JsonRpcError> {
        match self {
            JsonRpcResponse::Success { .. } => None,
            JsonRpcResponse::Error { error, .. } => Some(error),
        }
    }

    /// Converts the response into a Result.
    ///
    /// Returns `Ok(value)` for success responses, `Err(error)` for error responses.
    pub fn into_result(self) -> Result<Value, JsonRpcError> {
        match self {
            JsonRpcResponse::Success { result, .. } => Ok(result),
            JsonRpcResponse::Error { error, .. } => Err(error),
        }
    }
}

impl Serialize for JsonRpcResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            JsonRpcResponse::Success {
                jsonrpc,
                id,
                result,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("jsonrpc", jsonrpc)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("result", result)?;
                map.end()
            }
            JsonRpcResponse::Error { jsonrpc, id, error } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("jsonrpc", jsonrpc)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("error", error)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for JsonRpcResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into raw Value first to check for key presence
        // (Option<Value> can't distinguish missing field from null value)
        let raw: Value = Value::deserialize(deserializer)?;

        let obj = raw
            .as_object()
            .ok_or_else(|| de::Error::custom("expected JSON object"))?;

        // Validate jsonrpc version
        let jsonrpc = obj
            .get("jsonrpc")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::custom("missing or invalid 'jsonrpc' field"))?;

        if jsonrpc != JSONRPC_VERSION {
            return Err(de::Error::custom(format!(
                "invalid jsonrpc version: expected '{}', got '{}'",
                JSONRPC_VERSION, jsonrpc
            )));
        }

        // Check for presence of result and error keys (not just their values)
        // This correctly handles result: null as a valid success response
        let has_result = obj.contains_key("result");
        let has_error = obj.contains_key("error");

        match (has_result, has_error) {
            (true, false) => {
                // Success response - id must be present
                let id: RequestId = obj
                    .get("id")
                    .ok_or_else(|| de::Error::custom("success response must have an id"))
                    .and_then(|v| serde_json::from_value(v.clone()).map_err(de::Error::custom))?;
                let result = obj.get("result").cloned().unwrap_or(Value::Null);
                Ok(JsonRpcResponse::Success {
                    jsonrpc: jsonrpc.to_string(),
                    id,
                    result,
                })
            }
            (false, true) => {
                // Error response - id may be null
                let id: Option<RequestId> = obj.get("id").and_then(|v| {
                    if v.is_null() {
                        None
                    } else {
                        serde_json::from_value(v.clone()).ok()
                    }
                });
                let error: JsonRpcError = obj
                    .get("error")
                    .ok_or_else(|| de::Error::custom("missing 'error' field"))
                    .and_then(|v| serde_json::from_value(v.clone()).map_err(de::Error::custom))?;
                Ok(JsonRpcResponse::Error {
                    jsonrpc: jsonrpc.to_string(),
                    id,
                    error,
                })
            }
            (true, true) => Err(de::Error::custom(
                "response cannot contain both 'result' and 'error' fields",
            )),
            (false, false) => Err(de::Error::custom(
                "response must contain either 'result' or 'error' field",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T024: Write test: success response deserializes with result field
    #[test]
    fn test_success_response_deserializes() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert!(response.is_success());
        assert_eq!(response.result().unwrap()["status"], "ok");
    }

    // T025: Write test: error response deserializes with error field
    #[test]
    fn test_error_response_deserializes() {
        let json =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert!(response.is_error());
        assert_eq!(response.error_obj().unwrap().code, -32600);
    }

    // T026: Write test: response id is correctly extracted for correlation
    #[test]
    fn test_response_id_extraction() {
        let success_json = r#"{"jsonrpc":"2.0","id":"req-123","result":{}}"#;
        let success: JsonRpcResponse = serde_json::from_str(success_json).unwrap();
        assert_eq!(success.id(), Some(&RequestId::from_string("req-123")));

        let error_json = r#"{"jsonrpc":"2.0","id":42,"error":{"code":-1,"message":"test"}}"#;
        let error: JsonRpcResponse = serde_json::from_str(error_json).unwrap();
        assert_eq!(error.id(), Some(&RequestId::from_number(42)));
    }

    // T027: Write test: response with both result and error is rejected (FR-011)
    #[test]
    fn test_response_with_both_result_and_error_rejected() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-1,"message":"test"}}"#;
        let result: Result<JsonRpcResponse, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot contain both"));
    }

    // T028: Write test: null id in error response is accepted (FR-012)
    #[test]
    fn test_null_id_in_error_response_accepted() {
        let json = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert!(response.is_error());
        assert!(response.id().is_none());
    }

    // T029: Write test: success response serializes correctly
    #[test]
    fn test_success_response_serializes() {
        let response = JsonRpcResponse::success(
            RequestId::from_number(1),
            json!({"subscription_id": "sub_123"}),
        );

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"result\":{\"subscription_id\":\"sub_123\"}"));
        assert!(!json.contains("\"error\""));
    }

    // T030: Write test: error response serializes correctly
    #[test]
    fn test_error_response_serializes() {
        let response = JsonRpcResponse::error(
            Some(RequestId::from_number(1)),
            JsonRpcError::new(-32600, "Invalid Request"),
        );

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"error\":{"));
        assert!(json.contains("\"code\":-32600"));
        assert!(!json.contains("\"result\""));
    }

    // T054: Write test: success() helper creates valid success response
    #[test]
    fn test_success_helper() {
        let response = JsonRpcResponse::success(RequestId::from_number(1), json!({"status": "ok"}));

        assert!(response.is_success());
        assert_eq!(response.id(), Some(&RequestId::from_number(1)));
        assert_eq!(response.result().unwrap()["status"], "ok");
    }

    // T055: Write test: error() helper creates valid error response
    #[test]
    fn test_error_helper() {
        let response = JsonRpcResponse::error(
            Some(RequestId::from_number(1)),
            JsonRpcError::new(-32600, "Invalid Request"),
        );

        assert!(response.is_error());
        assert_eq!(response.id(), Some(&RequestId::from_number(1)));
        assert_eq!(response.error_obj().unwrap().code, -32600);
    }

    // T056: Write test: error() helper with null id creates valid response
    #[test]
    fn test_error_helper_with_null_id() {
        let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());

        assert!(response.is_error());
        assert!(response.id().is_none());
    }

    // T057: Write test: helper-created responses serialize to valid JSON-RPC 2.0
    #[test]
    fn test_helper_responses_serialize_correctly() {
        // Success helper
        let success =
            JsonRpcResponse::success(RequestId::from_string("req-1"), json!({"data": "test"}));
        let json = serde_json::to_string(&success).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));

        // Error helper
        let error = JsonRpcResponse::error(
            Some(RequestId::from_number(42)),
            JsonRpcError::method_not_found(),
        );
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"error\""));
    }

    #[test]
    fn test_into_result() {
        let success = JsonRpcResponse::success(RequestId::from_number(1), json!({"data": "test"}));
        assert!(success.into_result().is_ok());

        let error = JsonRpcResponse::error(
            Some(RequestId::from_number(1)),
            JsonRpcError::new(-1, "test"),
        );
        assert!(error.into_result().is_err());
    }

    #[test]
    fn test_response_roundtrip_success() {
        let original = JsonRpcResponse::success(
            RequestId::from_string("roundtrip"),
            json!({"nested": {"key": "value"}}),
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_response_roundtrip_error() {
        let original = JsonRpcResponse::error(
            Some(RequestId::from_number(99)),
            JsonRpcError::with_data(-32602, "Invalid params", json!({"field": "topics"})),
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_response_missing_both_result_and_error() {
        let json = r#"{"jsonrpc":"2.0","id":1}"#;
        let result: Result<JsonRpcResponse, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("must contain either"));
    }

    // Test that result: null is a valid success response per JSON-RPC 2.0 spec
    #[test]
    fn test_success_response_with_null_result() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":null}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert!(response.is_success());
        assert_eq!(response.id(), Some(&RequestId::from_number(1)));
        assert_eq!(response.result(), Some(&Value::Null));
    }

    // Test roundtrip with null result
    #[test]
    fn test_response_roundtrip_null_result() {
        let original = JsonRpcResponse::success(RequestId::from_number(1), Value::Null);

        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(restored.result(), Some(&Value::Null));
    }
}

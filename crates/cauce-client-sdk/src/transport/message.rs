//! JSON-RPC message types for transport layer.
//!
//! This module provides the [`JsonRpcMessage`] enum which represents
//! any JSON-RPC 2.0 message that can be sent or received over a transport.

use cauce_core::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId};
use serde_json::Value;

/// A JSON-RPC 2.0 message.
///
/// This enum represents any valid JSON-RPC message that can be sent
/// or received over a transport connection.
///
/// # Parsing Messages
///
/// Use [`JsonRpcMessage::parse`] to parse a JSON string into a message:
///
/// ```rust
/// use cauce_client_sdk::JsonRpcMessage;
///
/// let json = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
/// let message = JsonRpcMessage::parse(json).expect("valid JSON-RPC");
///
/// match message {
///     JsonRpcMessage::Response(resp) => {
///         println!("Got response for request {:?}", resp.id());
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum JsonRpcMessage {
    /// A request expecting a response.
    Request(JsonRpcRequest),

    /// A response to a previous request.
    Response(JsonRpcResponse),

    /// A notification (no response expected).
    Notification(JsonRpcNotification),
}

impl JsonRpcMessage {
    /// Parse a JSON string into a [`JsonRpcMessage`].
    ///
    /// This function determines the message type by examining the JSON structure:
    /// - Has `id` and `method` → Request
    /// - Has `id` and `result` or `error` → Response
    /// - Has `method` but no `id` → Notification
    ///
    /// # Example
    ///
    /// ```rust
    /// use cauce_client_sdk::JsonRpcMessage;
    ///
    /// // Parse a request
    /// let request = r#"{"jsonrpc":"2.0","id":1,"method":"cauce.hello","params":{}}"#;
    /// let msg = JsonRpcMessage::parse(request).unwrap();
    /// assert!(matches!(msg, JsonRpcMessage::Request(_)));
    ///
    /// // Parse a response
    /// let response = r#"{"jsonrpc":"2.0","id":1,"result":{"session_id":"sess_123"}}"#;
    /// let msg = JsonRpcMessage::parse(response).unwrap();
    /// assert!(matches!(msg, JsonRpcMessage::Response(_)));
    ///
    /// // Parse a notification
    /// let notification = r#"{"jsonrpc":"2.0","method":"cauce.signal","params":{}}"#;
    /// let msg = JsonRpcMessage::parse(notification).unwrap();
    /// assert!(matches!(msg, JsonRpcMessage::Notification(_)));
    /// ```
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        // First, parse as a generic Value to inspect the structure
        let value: Value = serde_json::from_str(json)?;

        let has_id = value.get("id").is_some() && !value.get("id").unwrap().is_null();
        let has_method = value.get("method").is_some();
        let has_result = value.get("result").is_some();
        let has_error = value.get("error").is_some();

        if has_id && (has_result || has_error) {
            // Response: has id and either result or error
            let response: JsonRpcResponse = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Response(response))
        } else if has_id && has_method {
            // Request: has id and method
            let request: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Request(request))
        } else if has_method && !has_id {
            // Notification: has method but no id
            let notification: JsonRpcNotification = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Notification(notification))
        } else {
            // Invalid message structure - try to parse as request which will give
            // a more meaningful error message
            let request: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Request(request))
        }
    }

    /// Serialize this message to a JSON string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cauce_client_sdk::{JsonRpcMessage, JsonRpcRequest, RequestId};
    ///
    /// let request = JsonRpcRequest::new(
    ///     RequestId::from_number(1),
    ///     "cauce.hello".to_string(),
    ///     None,
    /// );
    /// let message = JsonRpcMessage::Request(request);
    /// let json = message.to_json().unwrap();
    /// assert!(json.contains("cauce.hello"));
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            JsonRpcMessage::Request(r) => serde_json::to_string(r),
            JsonRpcMessage::Response(r) => serde_json::to_string(r),
            JsonRpcMessage::Notification(n) => serde_json::to_string(n),
        }
    }

    /// Returns the request ID if this is a Request or Response.
    pub fn id(&self) -> Option<&RequestId> {
        match self {
            JsonRpcMessage::Request(r) => Some(r.id()),
            JsonRpcMessage::Response(r) => r.id(),
            JsonRpcMessage::Notification(_) => None,
        }
    }

    /// Returns the method name if this is a Request or Notification.
    pub fn method(&self) -> Option<&str> {
        match self {
            JsonRpcMessage::Request(r) => Some(r.method()),
            JsonRpcMessage::Response(_) => None,
            JsonRpcMessage::Notification(n) => Some(n.method()),
        }
    }

    /// Returns true if this is a request.
    pub fn is_request(&self) -> bool {
        matches!(self, JsonRpcMessage::Request(_))
    }

    /// Returns true if this is a response.
    pub fn is_response(&self) -> bool {
        matches!(self, JsonRpcMessage::Response(_))
    }

    /// Returns true if this is a notification.
    pub fn is_notification(&self) -> bool {
        matches!(self, JsonRpcMessage::Notification(_))
    }

    /// Returns true if this is an error response.
    pub fn is_error(&self) -> bool {
        matches!(self, JsonRpcMessage::Response(r) if r.is_error())
    }

    /// Returns true if this is a success response.
    pub fn is_success(&self) -> bool {
        matches!(self, JsonRpcMessage::Response(r) if r.is_success())
    }

    /// Extract the request if this is a Request.
    pub fn into_request(self) -> Option<JsonRpcRequest> {
        match self {
            JsonRpcMessage::Request(r) => Some(r),
            _ => None,
        }
    }

    /// Extract the response if this is a Response.
    pub fn into_response(self) -> Option<JsonRpcResponse> {
        match self {
            JsonRpcMessage::Response(r) => Some(r),
            _ => None,
        }
    }

    /// Extract the notification if this is a Notification.
    pub fn into_notification(self) -> Option<JsonRpcNotification> {
        match self {
            JsonRpcMessage::Notification(n) => Some(n),
            _ => None,
        }
    }
}

impl From<JsonRpcRequest> for JsonRpcMessage {
    fn from(request: JsonRpcRequest) -> Self {
        JsonRpcMessage::Request(request)
    }
}

impl From<JsonRpcResponse> for JsonRpcMessage {
    fn from(response: JsonRpcResponse) -> Self {
        JsonRpcMessage::Response(response)
    }
}

impl From<JsonRpcNotification> for JsonRpcMessage {
    fn from(notification: JsonRpcNotification) -> Self {
        JsonRpcMessage::Notification(notification)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"cauce.hello","params":{}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_request());
        assert!(!msg.is_response());
        assert!(!msg.is_notification());
        assert_eq!(msg.method(), Some("cauce.hello"));
        assert!(msg.id().is_some());
    }

    #[test]
    fn test_parse_request_string_id() {
        let json = r#"{"jsonrpc":"2.0","id":"abc123","method":"cauce.subscribe","params":{}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_request());
        match msg.id() {
            Some(RequestId::String(s)) => assert_eq!(s, "abc123"),
            _ => panic!("Expected string ID"),
        }
    }

    #[test]
    fn test_parse_success_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"session_id":"sess_123"}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_response());
        assert!(msg.is_success());
        assert!(!msg.is_error());
        assert!(msg.id().is_some());
        assert!(msg.method().is_none());
    }

    #[test]
    fn test_parse_error_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid request"}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_response());
        assert!(msg.is_error());
        assert!(!msg.is_success());
    }

    #[test]
    fn test_parse_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"cauce.signal","params":{"topic":"signal.email"}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_notification());
        assert!(!msg.is_request());
        assert!(!msg.is_response());
        assert_eq!(msg.method(), Some("cauce.signal"));
        assert!(msg.id().is_none());
    }

    #[test]
    fn test_parse_response_null_id() {
        // JSON-RPC spec allows null id for error responses when the request id couldn't be determined
        // However, cauce-core's RequestId doesn't support null, so this is parsed as a notification
        // since our logic treats null id as no id
        let json = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#;

        // Since the RequestId doesn't support null values, this should fail to parse
        // as a response but we treat it as having no id (notification-like)
        let result = JsonRpcMessage::parse(json);

        // The error variant should contain the parse error details, but we can't
        // represent null id in our type system. This is a limitation.
        // The JSON-RPC 2.0 spec says error responses MAY have a null id when the
        // request was not parseable, but we don't support this edge case.
        assert!(result.is_err());
    }

    #[test]
    fn test_to_json_request() {
        let request = JsonRpcRequest::new(
            RequestId::from_number(42),
            "cauce.ping".to_string(),
            None,
        );
        let msg = JsonRpcMessage::Request(request);
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"method\":\"cauce.ping\""));
    }

    #[test]
    fn test_to_json_notification() {
        let notification = JsonRpcNotification::new(
            "cauce.signal".to_string(),
            Some(serde_json::json!({"topic": "signal.email"})),
        );
        let msg = JsonRpcMessage::Notification(notification);
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"method\":\"cauce.signal\""));
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_into_methods() {
        let request = JsonRpcRequest::new(
            RequestId::from_number(1),
            "test".to_string(),
            None,
        );
        let msg = JsonRpcMessage::Request(request);

        assert!(msg.clone().into_request().is_some());
        assert!(msg.clone().into_response().is_none());
        assert!(msg.into_notification().is_none());
    }

    #[test]
    fn test_from_implementations() {
        let request = JsonRpcRequest::new(
            RequestId::from_number(1),
            "test".to_string(),
            None,
        );
        let msg: JsonRpcMessage = request.into();
        assert!(msg.is_request());

        let notification = JsonRpcNotification::new("test".to_string(), None);
        let msg: JsonRpcMessage = notification.into();
        assert!(msg.is_notification());
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = JsonRpcMessage::parse("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip() {
        let original = r#"{"jsonrpc":"2.0","id":123,"method":"cauce.hello","params":{"client_id":"test"}}"#;
        let msg = JsonRpcMessage::parse(original).unwrap();
        let serialized = msg.to_json().unwrap();
        let reparsed = JsonRpcMessage::parse(&serialized).unwrap();

        assert!(reparsed.is_request());
        assert_eq!(reparsed.method(), Some("cauce.hello"));
    }
}

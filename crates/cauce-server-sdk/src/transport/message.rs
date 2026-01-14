//! JSON-RPC message types for server transport layer.
//!
//! This module provides the [`JsonRpcMessage`] enum which represents
//! any JSON-RPC 2.0 message that can be sent or received over a transport.

use cauce_core::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId};
use serde_json::Value;

/// A JSON-RPC 2.0 message.
///
/// This enum represents any valid JSON-RPC message that can be sent
/// or received over a transport connection.
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
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        let value: Value = serde_json::from_str(json)?;

        let has_id = value.get("id").is_some() && !value.get("id").unwrap().is_null();
        let has_method = value.get("method").is_some();
        let has_result = value.get("result").is_some();
        let has_error = value.get("error").is_some();

        if has_id && (has_result || has_error) {
            let response: JsonRpcResponse = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Response(response))
        } else if has_id && has_method {
            let request: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Request(request))
        } else if has_method && !has_id {
            let notification: JsonRpcNotification = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Notification(notification))
        } else {
            let request: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Request(request))
        }
    }

    /// Serialize this message to a JSON string.
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
    fn test_parse_success_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"session_id":"sess_123"}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_response());
        assert!(msg.id().is_some());
        assert!(msg.method().is_none());
    }

    #[test]
    fn test_parse_error_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid request"}}"#;
        let msg = JsonRpcMessage::parse(json).unwrap();

        assert!(msg.is_response());
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
    fn test_to_json_request() {
        let request = JsonRpcRequest::new(RequestId::from_number(42), "cauce.ping".to_string(), None);
        let msg = JsonRpcMessage::Request(request);
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"method\":\"cauce.ping\""));
    }

    #[test]
    fn test_into_methods() {
        let request = JsonRpcRequest::new(RequestId::from_number(1), "test".to_string(), None);
        let msg = JsonRpcMessage::Request(request);

        assert!(msg.clone().into_request().is_some());
        assert!(msg.clone().into_response().is_none());
        assert!(msg.into_notification().is_none());
    }

    #[test]
    fn test_from_implementations() {
        let request = JsonRpcRequest::new(RequestId::from_number(1), "test".to_string(), None);
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
}

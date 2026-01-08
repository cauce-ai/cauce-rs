//! JsonRpcNotification type for JSON-RPC 2.0.
//!
//! This module provides the [`JsonRpcNotification`] type for constructing
//! JSON-RPC 2.0 notification messages (requests without an id).

use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::request::JSONRPC_VERSION;

/// A JSON-RPC 2.0 notification message.
///
/// Notifications are requests that do not expect a response.
/// They do not have an `id` field.
///
/// Per the JSON-RPC 2.0 specification, a notification contains:
/// - `jsonrpc`: Must be exactly "2.0"
/// - `method`: The name of the method to invoke
/// - `params`: Optional parameters for the method
///
/// # Examples
///
/// ```
/// use cauce_core::jsonrpc::JsonRpcNotification;
/// use serde_json::json;
///
/// // Create a notification with params
/// let notification = JsonRpcNotification::new(
///     "cauce.signal",
///     Some(json!({"topic": "signal.email.received", "signal": {}})),
/// );
///
/// // Create a notification without params
/// let notification = JsonRpcNotification::new("cauce.ping", None);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Method name
    pub method: String,

    /// Optional method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Creates a new JSON-RPC notification.
    ///
    /// The `jsonrpc` field is automatically set to "2.0".
    ///
    /// # Arguments
    ///
    /// * `method` - The method name to invoke
    /// * `params` - Optional parameters for the method
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::jsonrpc::JsonRpcNotification;
    /// use serde_json::json;
    ///
    /// let notification = JsonRpcNotification::new(
    ///     "cauce.signal",
    ///     Some(json!({"topic": "signal.test"})),
    /// );
    ///
    /// assert_eq!(notification.jsonrpc, "2.0");
    /// assert_eq!(notification.method, "cauce.signal");
    /// ```
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
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

impl<'de> Deserialize<'de> for JsonRpcNotification {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawNotification {
            jsonrpc: String,
            method: String,
            params: Option<Value>,
        }

        let raw = RawNotification::deserialize(deserializer)?;

        // Validate jsonrpc version
        if raw.jsonrpc != JSONRPC_VERSION {
            return Err(de::Error::custom(format!(
                "invalid jsonrpc version: expected '{}', got '{}'",
                JSONRPC_VERSION, raw.jsonrpc
            )));
        }

        Ok(JsonRpcNotification {
            jsonrpc: raw.jsonrpc,
            method: raw.method,
            params: raw.params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T036: Write test: notification serialization has no id field
    #[test]
    fn test_notification_has_no_id_field() {
        let notification = JsonRpcNotification::new("test.method", None);

        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"id\""));
    }

    // T037: Write test: notification includes jsonrpc "2.0"
    #[test]
    fn test_notification_includes_jsonrpc() {
        let notification = JsonRpcNotification::new("test.method", None);

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
    }

    // T038: Write test: notification with params serializes correctly
    #[test]
    fn test_notification_with_params() {
        let notification = JsonRpcNotification::new(
            "cauce.signal",
            Some(json!({"topic": "signal.email.received"})),
        );

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("\"params\":{\"topic\":\"signal.email.received\"}"));
    }

    // T039: Write test: notification without params omits params field
    #[test]
    fn test_notification_without_params_omits_field() {
        let notification = JsonRpcNotification::new("cauce.ping", None);

        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"params\""));
    }

    // T040: Write test: notification deserialization validates jsonrpc == "2.0"
    #[test]
    fn test_notification_deserialization_validates_version() {
        // Valid version
        let valid_json = r#"{"jsonrpc":"2.0","method":"test"}"#;
        let result: Result<JsonRpcNotification, _> = serde_json::from_str(valid_json);
        assert!(result.is_ok());

        // Invalid version
        let invalid_json = r#"{"jsonrpc":"1.0","method":"test"}"#;
        let result: Result<JsonRpcNotification, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid jsonrpc version"));
    }

    #[test]
    fn test_notification_deserialization_with_params() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{"key":"value"}}"#;
        let notification: JsonRpcNotification = serde_json::from_str(json).unwrap();

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "test");
        assert!(notification.params.is_some());
    }

    #[test]
    fn test_notification_roundtrip() {
        let original = JsonRpcNotification::new(
            "cauce.signal",
            Some(json!({"topic": "signal.test", "data": 42})),
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: JsonRpcNotification = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_accessor_methods() {
        let notification = JsonRpcNotification::new("test.method", Some(json!({"key": "value"})));

        assert_eq!(notification.method(), "test.method");
        assert!(notification.params().is_some());
    }
}

//! Protocol error types for the Cauce Protocol.
//!
//! This module provides the [`CauceError`] enum which represents all possible
//! protocol errors with their corresponding JSON-RPC error codes.

use crate::jsonrpc::JsonRpcError;
use serde_json::json;
use thiserror::Error;

/// Protocol errors for the Cauce Protocol.
///
/// Each variant maps to a specific JSON-RPC error code and standard message.
/// Use the [`code`](CauceError::code) and [`message`](CauceError::message) methods
/// to get the error code and message, or convert to [`JsonRpcError`] using `Into`.
///
/// # Error Code Ranges
///
/// - `-32700` to `-32600`: JSON-RPC standard errors
/// - `-32001` to `-32015`: Cauce protocol-specific errors
///
/// # Example
///
/// ```
/// use cauce_core::errors::CauceError;
/// use cauce_core::JsonRpcError;
///
/// let error = CauceError::RateLimited { retry_after_ms: 5000 };
/// assert_eq!(error.code(), -32006);
/// assert_eq!(error.message(), "Rate limited");
///
/// let rpc_error: JsonRpcError = error.into();
/// assert_eq!(rpc_error.code, -32006);
/// ```
#[derive(Debug, Clone, Error, PartialEq)]
pub enum CauceError {
    // ===== JSON-RPC Standard Errors =====
    /// Parse error - Invalid JSON was received (-32700)
    #[error("Parse error: {message}")]
    ParseError {
        /// Details about the parse error
        message: String,
    },

    /// Invalid Request - The JSON sent is not a valid Request object (-32600)
    #[error("Invalid request: {message}")]
    InvalidRequest {
        /// Details about what makes the request invalid
        message: String,
    },

    /// Method not found - The method does not exist (-32601)
    #[error("Method not found: {method}")]
    MethodNotFound {
        /// The method name that was not found
        method: String,
    },

    /// Invalid params - Invalid method parameter(s) (-32602)
    #[error("Invalid params: {message}")]
    InvalidParams {
        /// Details about the invalid parameters
        message: String,
    },

    /// Internal error - Internal JSON-RPC error (-32603)
    #[error("Internal error: {message}")]
    InternalError {
        /// Details about the internal error
        message: String,
    },

    // ===== Cauce Protocol Errors =====
    /// Subscription not found (-32001)
    #[error("Subscription not found: {id}")]
    SubscriptionNotFound {
        /// The subscription ID that was not found
        id: String,
    },

    /// Topic not found (-32002)
    #[error("Topic not found: {topic}")]
    TopicNotFound {
        /// The topic that was not found
        topic: String,
    },

    /// Not authorized (-32003)
    #[error("Not authorized: {reason}")]
    NotAuthorized {
        /// The reason for the authorization failure
        reason: String,
    },

    /// Subscription pending approval (-32004)
    #[error("Subscription pending approval: {id}")]
    SubscriptionPending {
        /// The subscription ID that is pending
        id: String,
    },

    /// Subscription denied (-32005)
    #[error("Subscription denied: {id}")]
    SubscriptionDenied {
        /// The subscription ID that was denied
        id: String,
        /// Optional reason for denial
        reason: Option<String>,
    },

    /// Rate limited (-32006)
    #[error("Rate limited, retry after {retry_after_ms}ms")]
    RateLimited {
        /// Time in milliseconds to wait before retrying
        retry_after_ms: u64,
    },

    /// Signal too large (-32007)
    #[error("Signal too large: {size} bytes exceeds maximum of {max} bytes")]
    SignalTooLarge {
        /// The actual size of the signal
        size: usize,
        /// The maximum allowed size
        max: usize,
    },

    /// Encryption required (-32008)
    #[error("Encryption required for topic: {topic}")]
    EncryptionRequired {
        /// The topic that requires encryption
        topic: String,
    },

    /// Invalid encryption (-32009)
    #[error("Invalid encryption: {reason}")]
    InvalidEncryption {
        /// The reason the encryption is invalid
        reason: String,
    },

    /// Adapter unavailable (-32010)
    #[error("Adapter unavailable: {adapter}")]
    AdapterUnavailable {
        /// The adapter that is unavailable
        adapter: String,
    },

    /// Delivery failed (-32011)
    #[error("Delivery failed for signal {signal_id}: {reason}")]
    DeliveryFailed {
        /// The signal ID that failed to deliver
        signal_id: String,
        /// The reason for the delivery failure
        reason: String,
    },

    /// Queue full (-32012)
    #[error("Queue full, capacity: {capacity}")]
    QueueFull {
        /// The capacity of the queue
        capacity: usize,
    },

    /// Session expired (-32013)
    #[error("Session expired: {session_id}")]
    SessionExpired {
        /// The session ID that expired
        session_id: String,
    },

    /// Unsupported transport (-32014)
    #[error("Unsupported transport: {transport}")]
    UnsupportedTransport {
        /// The transport that is not supported
        transport: String,
    },

    /// Invalid topic (-32015)
    #[error("Invalid topic '{topic}': {reason}")]
    InvalidTopic {
        /// The invalid topic
        topic: String,
        /// The reason the topic is invalid
        reason: String,
    },
}

impl CauceError {
    /// Returns the JSON-RPC error code for this error.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::errors::CauceError;
    ///
    /// let error = CauceError::ParseError { message: "unexpected EOF".to_string() };
    /// assert_eq!(error.code(), -32700);
    /// ```
    pub fn code(&self) -> i32 {
        match self {
            // JSON-RPC standard errors
            CauceError::ParseError { .. } => -32700,
            CauceError::InvalidRequest { .. } => -32600,
            CauceError::MethodNotFound { .. } => -32601,
            CauceError::InvalidParams { .. } => -32602,
            CauceError::InternalError { .. } => -32603,

            // Cauce protocol errors
            CauceError::SubscriptionNotFound { .. } => -32001,
            CauceError::TopicNotFound { .. } => -32002,
            CauceError::NotAuthorized { .. } => -32003,
            CauceError::SubscriptionPending { .. } => -32004,
            CauceError::SubscriptionDenied { .. } => -32005,
            CauceError::RateLimited { .. } => -32006,
            CauceError::SignalTooLarge { .. } => -32007,
            CauceError::EncryptionRequired { .. } => -32008,
            CauceError::InvalidEncryption { .. } => -32009,
            CauceError::AdapterUnavailable { .. } => -32010,
            CauceError::DeliveryFailed { .. } => -32011,
            CauceError::QueueFull { .. } => -32012,
            CauceError::SessionExpired { .. } => -32013,
            CauceError::UnsupportedTransport { .. } => -32014,
            CauceError::InvalidTopic { .. } => -32015,
        }
    }

    /// Returns the standard JSON-RPC message for this error.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::errors::CauceError;
    ///
    /// let error = CauceError::RateLimited { retry_after_ms: 1000 };
    /// assert_eq!(error.message(), "Rate limited");
    /// ```
    pub fn message(&self) -> &'static str {
        match self {
            // JSON-RPC standard errors
            CauceError::ParseError { .. } => "Parse error",
            CauceError::InvalidRequest { .. } => "Invalid request",
            CauceError::MethodNotFound { .. } => "Method not found",
            CauceError::InvalidParams { .. } => "Invalid params",
            CauceError::InternalError { .. } => "Internal error",

            // Cauce protocol errors
            CauceError::SubscriptionNotFound { .. } => "Subscription not found",
            CauceError::TopicNotFound { .. } => "Topic not found",
            CauceError::NotAuthorized { .. } => "Not authorized",
            CauceError::SubscriptionPending { .. } => "Subscription pending approval",
            CauceError::SubscriptionDenied { .. } => "Subscription denied",
            CauceError::RateLimited { .. } => "Rate limited",
            CauceError::SignalTooLarge { .. } => "Signal too large",
            CauceError::EncryptionRequired { .. } => "Encryption required",
            CauceError::InvalidEncryption { .. } => "Invalid encryption",
            CauceError::AdapterUnavailable { .. } => "Adapter unavailable",
            CauceError::DeliveryFailed { .. } => "Delivery failed",
            CauceError::QueueFull { .. } => "Queue full",
            CauceError::SessionExpired { .. } => "Session expired",
            CauceError::UnsupportedTransport { .. } => "Unsupported transport",
            CauceError::InvalidTopic { .. } => "Invalid topic",
        }
    }

    /// Converts this error to a [`JsonRpcError`].
    ///
    /// This is equivalent to using `Into<JsonRpcError>`.
    pub fn to_json_rpc_error(&self) -> JsonRpcError {
        self.clone().into()
    }
}

impl From<CauceError> for JsonRpcError {
    fn from(err: CauceError) -> Self {
        let code = err.code();
        let message = err.message().to_string();

        let data = match &err {
            CauceError::ParseError { message: msg } => Some(json!({ "message": msg })),
            CauceError::InvalidRequest { message: msg } => Some(json!({ "message": msg })),
            CauceError::MethodNotFound { method } => Some(json!({ "method": method })),
            CauceError::InvalidParams { message: msg } => Some(json!({ "message": msg })),
            CauceError::InternalError { message: msg } => Some(json!({ "message": msg })),

            CauceError::SubscriptionNotFound { id } => Some(json!({ "id": id })),
            CauceError::TopicNotFound { topic } => Some(json!({ "topic": topic })),
            CauceError::NotAuthorized { reason } => Some(json!({ "reason": reason })),
            CauceError::SubscriptionPending { id } => Some(json!({ "id": id })),
            CauceError::SubscriptionDenied { id, reason } => {
                Some(json!({ "id": id, "reason": reason }))
            }
            CauceError::RateLimited { retry_after_ms } => {
                Some(json!({ "retry_after_ms": retry_after_ms }))
            }
            CauceError::SignalTooLarge { size, max } => Some(json!({ "size": size, "max": max })),
            CauceError::EncryptionRequired { topic } => Some(json!({ "topic": topic })),
            CauceError::InvalidEncryption { reason } => Some(json!({ "reason": reason })),
            CauceError::AdapterUnavailable { adapter } => Some(json!({ "adapter": adapter })),
            CauceError::DeliveryFailed { signal_id, reason } => {
                Some(json!({ "signal_id": signal_id, "reason": reason }))
            }
            CauceError::QueueFull { capacity } => Some(json!({ "capacity": capacity })),
            CauceError::SessionExpired { session_id } => Some(json!({ "session_id": session_id })),
            CauceError::UnsupportedTransport { transport } => {
                Some(json!({ "transport": transport }))
            }
            CauceError::InvalidTopic { topic, reason } => {
                Some(json!({ "topic": topic, "reason": reason }))
            }
        };

        match data {
            Some(d) => JsonRpcError::with_data(code, message, d),
            None => JsonRpcError::new(code, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== JSON-RPC Standard Error Tests =====

    #[test]
    fn test_parse_error_code_and_message() {
        let err = CauceError::ParseError {
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.code(), -32700);
        assert_eq!(err.message(), "Parse error");
    }

    #[test]
    fn test_invalid_request_code_and_message() {
        let err = CauceError::InvalidRequest {
            message: "missing jsonrpc field".to_string(),
        };
        assert_eq!(err.code(), -32600);
        assert_eq!(err.message(), "Invalid request");
    }

    #[test]
    fn test_method_not_found_code_and_message() {
        let err = CauceError::MethodNotFound {
            method: "cauce.unknown".to_string(),
        };
        assert_eq!(err.code(), -32601);
        assert_eq!(err.message(), "Method not found");
    }

    #[test]
    fn test_invalid_params_code_and_message() {
        let err = CauceError::InvalidParams {
            message: "topics must be an array".to_string(),
        };
        assert_eq!(err.code(), -32602);
        assert_eq!(err.message(), "Invalid params");
    }

    #[test]
    fn test_internal_error_code_and_message() {
        let err = CauceError::InternalError {
            message: "database connection failed".to_string(),
        };
        assert_eq!(err.code(), -32603);
        assert_eq!(err.message(), "Internal error");
    }

    // ===== Cauce Protocol Error Tests =====

    #[test]
    fn test_subscription_not_found_code_and_message() {
        let err = CauceError::SubscriptionNotFound {
            id: "sub_123".to_string(),
        };
        assert_eq!(err.code(), -32001);
        assert_eq!(err.message(), "Subscription not found");
    }

    #[test]
    fn test_topic_not_found_code_and_message() {
        let err = CauceError::TopicNotFound {
            topic: "signal.unknown".to_string(),
        };
        assert_eq!(err.code(), -32002);
        assert_eq!(err.message(), "Topic not found");
    }

    #[test]
    fn test_not_authorized_code_and_message() {
        let err = CauceError::NotAuthorized {
            reason: "invalid token".to_string(),
        };
        assert_eq!(err.code(), -32003);
        assert_eq!(err.message(), "Not authorized");
    }

    #[test]
    fn test_subscription_pending_code_and_message() {
        let err = CauceError::SubscriptionPending {
            id: "sub_pending".to_string(),
        };
        assert_eq!(err.code(), -32004);
        assert_eq!(err.message(), "Subscription pending approval");
    }

    #[test]
    fn test_subscription_denied_code_and_message() {
        let err = CauceError::SubscriptionDenied {
            id: "sub_denied".to_string(),
            reason: Some("user rejected".to_string()),
        };
        assert_eq!(err.code(), -32005);
        assert_eq!(err.message(), "Subscription denied");
    }

    #[test]
    fn test_rate_limited_code_and_message() {
        let err = CauceError::RateLimited {
            retry_after_ms: 5000,
        };
        assert_eq!(err.code(), -32006);
        assert_eq!(err.message(), "Rate limited");
    }

    #[test]
    fn test_signal_too_large_code_and_message() {
        let err = CauceError::SignalTooLarge {
            size: 20_000_000,
            max: 10_485_760,
        };
        assert_eq!(err.code(), -32007);
        assert_eq!(err.message(), "Signal too large");
    }

    #[test]
    fn test_encryption_required_code_and_message() {
        let err = CauceError::EncryptionRequired {
            topic: "signal.secure".to_string(),
        };
        assert_eq!(err.code(), -32008);
        assert_eq!(err.message(), "Encryption required");
    }

    #[test]
    fn test_invalid_encryption_code_and_message() {
        let err = CauceError::InvalidEncryption {
            reason: "unsupported algorithm".to_string(),
        };
        assert_eq!(err.code(), -32009);
        assert_eq!(err.message(), "Invalid encryption");
    }

    #[test]
    fn test_adapter_unavailable_code_and_message() {
        let err = CauceError::AdapterUnavailable {
            adapter: "email-adapter".to_string(),
        };
        assert_eq!(err.code(), -32010);
        assert_eq!(err.message(), "Adapter unavailable");
    }

    #[test]
    fn test_delivery_failed_code_and_message() {
        let err = CauceError::DeliveryFailed {
            signal_id: "sig_123".to_string(),
            reason: "timeout".to_string(),
        };
        assert_eq!(err.code(), -32011);
        assert_eq!(err.message(), "Delivery failed");
    }

    #[test]
    fn test_queue_full_code_and_message() {
        let err = CauceError::QueueFull { capacity: 10000 };
        assert_eq!(err.code(), -32012);
        assert_eq!(err.message(), "Queue full");
    }

    #[test]
    fn test_session_expired_code_and_message() {
        let err = CauceError::SessionExpired {
            session_id: "sess_abc".to_string(),
        };
        assert_eq!(err.code(), -32013);
        assert_eq!(err.message(), "Session expired");
    }

    #[test]
    fn test_unsupported_transport_code_and_message() {
        let err = CauceError::UnsupportedTransport {
            transport: "grpc".to_string(),
        };
        assert_eq!(err.code(), -32014);
        assert_eq!(err.message(), "Unsupported transport");
    }

    #[test]
    fn test_invalid_topic_code_and_message() {
        let err = CauceError::InvalidTopic {
            topic: "..invalid".to_string(),
            reason: "starts with dot".to_string(),
        };
        assert_eq!(err.code(), -32015);
        assert_eq!(err.message(), "Invalid topic");
    }

    // ===== JsonRpcError Conversion Tests =====

    #[test]
    fn test_from_cauce_error_parse_error() {
        let err = CauceError::ParseError {
            message: "unexpected EOF".to_string(),
        };
        let rpc_error: JsonRpcError = err.into();

        assert_eq!(rpc_error.code, -32700);
        assert_eq!(rpc_error.message, "Parse error");
        assert!(rpc_error.data.is_some());
        assert_eq!(rpc_error.data.unwrap()["message"], "unexpected EOF");
    }

    #[test]
    fn test_from_cauce_error_rate_limited() {
        let err = CauceError::RateLimited {
            retry_after_ms: 3000,
        };
        let rpc_error: JsonRpcError = err.into();

        assert_eq!(rpc_error.code, -32006);
        assert_eq!(rpc_error.message, "Rate limited");
        assert!(rpc_error.data.is_some());
        assert_eq!(rpc_error.data.unwrap()["retry_after_ms"], 3000);
    }

    #[test]
    fn test_from_cauce_error_subscription_denied_with_reason() {
        let err = CauceError::SubscriptionDenied {
            id: "sub_test".to_string(),
            reason: Some("not allowed".to_string()),
        };
        let rpc_error: JsonRpcError = err.into();

        assert_eq!(rpc_error.code, -32005);
        let data = rpc_error.data.unwrap();
        assert_eq!(data["id"], "sub_test");
        assert_eq!(data["reason"], "not allowed");
    }

    #[test]
    fn test_from_cauce_error_subscription_denied_without_reason() {
        let err = CauceError::SubscriptionDenied {
            id: "sub_test".to_string(),
            reason: None,
        };
        let rpc_error: JsonRpcError = err.into();

        assert_eq!(rpc_error.code, -32005);
        let data = rpc_error.data.unwrap();
        assert_eq!(data["id"], "sub_test");
        assert!(data["reason"].is_null());
    }

    #[test]
    fn test_from_cauce_error_signal_too_large() {
        let err = CauceError::SignalTooLarge {
            size: 15_000_000,
            max: 10_485_760,
        };
        let rpc_error: JsonRpcError = err.into();

        assert_eq!(rpc_error.code, -32007);
        let data = rpc_error.data.unwrap();
        assert_eq!(data["size"], 15_000_000);
        assert_eq!(data["max"], 10_485_760);
    }

    #[test]
    fn test_to_json_rpc_error() {
        let err = CauceError::NotAuthorized {
            reason: "expired token".to_string(),
        };
        let rpc_error = err.to_json_rpc_error();

        assert_eq!(rpc_error.code, -32003);
        assert_eq!(rpc_error.message, "Not authorized");
    }

    // ===== Display Tests =====

    #[test]
    fn test_display_parse_error() {
        let err = CauceError::ParseError {
            message: "invalid JSON".to_string(),
        };
        assert_eq!(err.to_string(), "Parse error: invalid JSON");
    }

    #[test]
    fn test_display_rate_limited() {
        let err = CauceError::RateLimited {
            retry_after_ms: 1000,
        };
        assert_eq!(err.to_string(), "Rate limited, retry after 1000ms");
    }

    #[test]
    fn test_display_signal_too_large() {
        let err = CauceError::SignalTooLarge {
            size: 20_000_000,
            max: 10_000_000,
        };
        assert_eq!(
            err.to_string(),
            "Signal too large: 20000000 bytes exceeds maximum of 10000000 bytes"
        );
    }

    // ===== Error Roundtrip Test =====

    #[test]
    fn test_error_roundtrip_through_json() {
        let err = CauceError::DeliveryFailed {
            signal_id: "sig_test_123".to_string(),
            reason: "network error".to_string(),
        };
        let rpc_error: JsonRpcError = err.into();
        let json = serde_json::to_string(&rpc_error).unwrap();
        let restored: JsonRpcError = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.code, -32011);
        assert_eq!(restored.message, "Delivery failed");
        assert_eq!(restored.data.unwrap()["signal_id"], "sig_test_123");
    }
}

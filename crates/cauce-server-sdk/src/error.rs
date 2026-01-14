//! Server error types for the Cauce Server SDK.
//!
//! This module defines the error types used throughout the server SDK,
//! following the same patterns as `cauce-client-sdk`.

use cauce_core::{CauceError, JsonRpcError};
use thiserror::Error;

/// Result type alias for server operations.
pub type ServerResult<T> = Result<T, ServerError>;

/// Errors that can occur during server operations.
#[derive(Debug, Error)]
pub enum ServerError {
    // =========================================================================
    // Configuration Errors
    // =========================================================================
    /// Configuration validation failed.
    #[error("configuration error: {message}")]
    ConfigError {
        /// Description of the configuration issue.
        message: String,
    },

    /// Invalid address format.
    #[error("invalid address: {address}")]
    InvalidAddress {
        /// The invalid address.
        address: String,
    },

    // =========================================================================
    // Session Errors
    // =========================================================================
    /// Session not found.
    #[error("session not found: {id}")]
    SessionNotFound {
        /// The session ID that was not found.
        id: String,
    },

    /// Session has expired.
    #[error("session expired: {id}")]
    SessionExpired {
        /// The expired session ID.
        id: String,
    },

    /// Invalid session state for the requested operation.
    #[error("invalid session state: {message}")]
    InvalidSessionState {
        /// Description of the state issue.
        message: String,
    },

    // =========================================================================
    // Subscription Errors
    // =========================================================================
    /// Subscription not found.
    #[error("subscription not found: {id}")]
    SubscriptionNotFound {
        /// The subscription ID that was not found.
        id: String,
    },

    /// Subscription is pending approval.
    #[error("subscription pending approval: {id}")]
    SubscriptionPending {
        /// The pending subscription ID.
        id: String,
    },

    /// Subscription was denied.
    #[error("subscription denied: {id}")]
    SubscriptionDenied {
        /// The denied subscription ID.
        id: String,
        /// Optional reason for denial.
        reason: Option<String>,
    },

    /// Maximum subscriptions limit reached.
    #[error("subscription limit exceeded: max {max} subscriptions per client")]
    SubscriptionLimitExceeded {
        /// Maximum allowed subscriptions.
        max: usize,
    },

    /// Too many topics in a subscription request.
    #[error("too many topics: max {max} topics per subscription")]
    TooManyTopics {
        /// Maximum allowed topics per subscription.
        max: usize,
    },

    // =========================================================================
    // Authentication Errors
    // =========================================================================
    /// Authentication failed.
    #[error("authentication failed: {reason}")]
    AuthenticationFailed {
        /// Reason for the authentication failure.
        reason: String,
    },

    /// Authorization failed (authenticated but not permitted).
    #[error("not authorized: {reason}")]
    NotAuthorized {
        /// Reason for the authorization failure.
        reason: String,
    },

    /// Missing authentication credentials.
    #[error("authentication required")]
    AuthenticationRequired,

    // =========================================================================
    // Rate Limiting Errors
    // =========================================================================
    /// Rate limit exceeded.
    #[error("rate limited: retry after {retry_after_ms}ms")]
    RateLimited {
        /// Milliseconds to wait before retrying.
        retry_after_ms: u64,
    },

    // =========================================================================
    // Delivery Errors
    // =========================================================================
    /// Signal delivery failed.
    #[error("delivery failed: {reason}")]
    DeliveryFailed {
        /// Reason for the delivery failure.
        reason: String,
    },

    /// Webhook delivery failed.
    #[error("webhook delivery failed: {url} - {reason}")]
    WebhookFailed {
        /// The webhook URL that failed.
        url: String,
        /// Reason for the failure.
        reason: String,
    },

    /// Signal not found (e.g., during ack).
    #[error("signal not found: {id}")]
    SignalNotFound {
        /// The signal ID that was not found.
        id: String,
    },

    // =========================================================================
    // Transport Errors
    // =========================================================================
    /// Transport-level error.
    #[error("transport error: {message}")]
    TransportError {
        /// Description of the transport error.
        message: String,
    },

    /// WebSocket error.
    #[error("websocket error: {message}")]
    WebSocketError {
        /// Description of the WebSocket error.
        message: String,
    },

    /// Connection closed unexpectedly.
    #[error("connection closed: {reason}")]
    ConnectionClosed {
        /// Reason for the closure.
        reason: String,
    },

    /// Transport has been closed.
    #[error("transport closed")]
    TransportClosed,

    /// Serialization/deserialization error.
    #[error("serialization error: {message}")]
    Serialization {
        /// Description of the serialization error.
        message: String,
    },

    // =========================================================================
    // Protocol Errors
    // =========================================================================
    /// Protocol error from cauce-core.
    #[error("protocol error: {0}")]
    ProtocolError(#[from] CauceError),

    /// JSON serialization/deserialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid JSON-RPC message.
    #[error("invalid message: {message}")]
    InvalidMessage {
        /// Description of what was invalid.
        message: String,
    },

    /// Method not found.
    #[error("method not found: {method}")]
    MethodNotFound {
        /// The method that was not found.
        method: String,
    },

    /// Invalid parameters for a method.
    #[error("invalid params: {message}")]
    InvalidParams {
        /// Description of the parameter issue.
        message: String,
    },

    // =========================================================================
    // Internal Errors
    // =========================================================================
    /// Internal server error.
    #[error("internal error: {message}")]
    InternalError {
        /// Description of the internal error.
        message: String,
    },

    /// IO error.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ServerError {
    // =========================================================================
    // Constructor Helpers
    // =========================================================================

    /// Create a configuration error.
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create an authentication failed error.
    pub fn auth_failed(reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            reason: reason.into(),
        }
    }

    /// Create a not authorized error.
    pub fn not_authorized(reason: impl Into<String>) -> Self {
        Self::NotAuthorized {
            reason: reason.into(),
        }
    }

    /// Create a transport error.
    pub fn transport_error(message: impl Into<String>) -> Self {
        Self::TransportError {
            message: message.into(),
        }
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Create an invalid message error.
    pub fn invalid_message(message: impl Into<String>) -> Self {
        Self::InvalidMessage {
            message: message.into(),
        }
    }

    /// Create an invalid params error.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::InvalidParams {
            message: message.into(),
        }
    }

    // =========================================================================
    // Error Classification
    // =========================================================================

    /// Returns true if this error indicates the client should retry.
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. }
                | Self::DeliveryFailed { .. }
                | Self::WebhookFailed { .. }
                | Self::TransportError { .. }
        )
    }

    /// Returns true if this error is due to authentication/authorization.
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            Self::AuthenticationFailed { .. }
                | Self::NotAuthorized { .. }
                | Self::AuthenticationRequired
        )
    }

    /// Returns true if this error is a client error (4xx equivalent).
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::ConfigError { .. }
                | Self::InvalidAddress { .. }
                | Self::SessionNotFound { .. }
                | Self::SubscriptionNotFound { .. }
                | Self::SubscriptionPending { .. }
                | Self::SubscriptionDenied { .. }
                | Self::SubscriptionLimitExceeded { .. }
                | Self::TooManyTopics { .. }
                | Self::AuthenticationFailed { .. }
                | Self::NotAuthorized { .. }
                | Self::AuthenticationRequired
                | Self::InvalidMessage { .. }
                | Self::MethodNotFound { .. }
                | Self::InvalidParams { .. }
                | Self::SignalNotFound { .. }
        )
    }

    /// Returns true if this error is a server error (5xx equivalent).
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::InternalError { .. }
                | Self::DeliveryFailed { .. }
                | Self::WebhookFailed { .. }
                | Self::TransportError { .. }
                | Self::WebSocketError { .. }
                | Self::TransportClosed
                | Self::Serialization { .. }
                | Self::IoError(_)
        )
    }
}

impl From<ServerError> for JsonRpcError {
    fn from(err: ServerError) -> Self {
        match err {
            ServerError::SubscriptionNotFound { id } => {
                CauceError::SubscriptionNotFound { id }.into()
            }
            ServerError::NotAuthorized { reason } => {
                CauceError::NotAuthorized { reason }.into()
            }
            ServerError::AuthenticationRequired | ServerError::AuthenticationFailed { .. } => {
                CauceError::NotAuthorized {
                    reason: err.to_string(),
                }
                .into()
            }
            ServerError::RateLimited { retry_after_ms } => {
                CauceError::RateLimited { retry_after_ms }.into()
            }
            ServerError::SubscriptionPending { id } => {
                CauceError::SubscriptionPending { id }.into()
            }
            ServerError::SubscriptionDenied { id, reason } => {
                CauceError::SubscriptionDenied { id, reason }.into()
            }
            ServerError::SessionExpired { id } => CauceError::SessionExpired { session_id: id }.into(),
            ServerError::MethodNotFound { method } => CauceError::MethodNotFound { method }.into(),
            ServerError::InvalidParams { message } => {
                CauceError::InvalidParams { message }.into()
            }
            ServerError::InvalidMessage { message } => {
                CauceError::InvalidRequest { message }.into()
            }
            ServerError::ProtocolError(e) => e.into(),
            ServerError::SerializationError(e) => CauceError::ParseError {
                message: e.to_string(),
            }
            .into(),
            _ => CauceError::InternalError {
                message: err.to_string(),
            }
            .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let err = ServerError::config_error("invalid port");
        assert!(matches!(err, ServerError::ConfigError { .. }));
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_auth_errors() {
        let err = ServerError::auth_failed("invalid token");
        assert!(err.is_auth_error());
        assert!(err.is_client_error());

        let err = ServerError::not_authorized("missing permission");
        assert!(err.is_auth_error());

        let err = ServerError::AuthenticationRequired;
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_rate_limited() {
        let err = ServerError::RateLimited {
            retry_after_ms: 1000,
        };
        assert!(err.should_retry());
        assert!(!err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_subscription_errors() {
        let err = ServerError::SubscriptionNotFound {
            id: "sub_123".to_string(),
        };
        assert!(err.is_client_error());

        let err = ServerError::SubscriptionDenied {
            id: "sub_123".to_string(),
            reason: Some("not allowed".to_string()),
        };
        assert!(err.is_client_error());
    }

    #[test]
    fn test_server_errors() {
        let err = ServerError::internal("something went wrong");
        assert!(err.is_server_error());
        assert!(!err.is_client_error());

        let err = ServerError::transport_error("connection failed");
        assert!(err.is_server_error());
        assert!(err.should_retry());
    }

    #[test]
    fn test_json_rpc_conversion() {
        let err = ServerError::SubscriptionNotFound {
            id: "sub_123".to_string(),
        };
        let rpc_err: JsonRpcError = err.into();
        assert_eq!(rpc_err.code, -32001); // SubscriptionNotFound code

        let err = ServerError::RateLimited {
            retry_after_ms: 5000,
        };
        let rpc_err: JsonRpcError = err.into();
        assert_eq!(rpc_err.code, -32006); // RateLimited code
    }

    #[test]
    fn test_error_display() {
        let err = ServerError::SessionNotFound {
            id: "sess_abc".to_string(),
        };
        assert_eq!(err.to_string(), "session not found: sess_abc");

        let err = ServerError::RateLimited {
            retry_after_ms: 1000,
        };
        assert_eq!(err.to_string(), "rate limited: retry after 1000ms");
    }

    #[test]
    fn test_invalid_address() {
        let err = ServerError::InvalidAddress {
            address: "not_an_address".to_string(),
        };
        assert!(err.is_client_error());
        assert!(err.to_string().contains("not_an_address"));
    }

    #[test]
    fn test_session_expired() {
        let err = ServerError::SessionExpired {
            id: "sess_123".to_string(),
        };
        assert!(!err.is_client_error()); // SessionExpired is not in the client error list
        assert!(err.to_string().contains("sess_123"));
    }

    #[test]
    fn test_invalid_session_state() {
        let err = ServerError::InvalidSessionState {
            message: "wrong state".to_string(),
        };
        assert!(err.to_string().contains("wrong state"));
    }

    #[test]
    fn test_subscription_pending() {
        let err = ServerError::SubscriptionPending {
            id: "sub_pending".to_string(),
        };
        assert!(err.is_client_error());
        assert!(err.to_string().contains("sub_pending"));
    }

    #[test]
    fn test_subscription_limit_exceeded() {
        let err = ServerError::SubscriptionLimitExceeded { max: 10 };
        assert!(err.is_client_error());
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_too_many_topics() {
        let err = ServerError::TooManyTopics { max: 5 };
        assert!(err.is_client_error());
        assert!(err.to_string().contains("5"));
    }

    #[test]
    fn test_delivery_failed() {
        let err = ServerError::DeliveryFailed {
            reason: "timeout".to_string(),
        };
        assert!(err.is_server_error());
        assert!(err.should_retry());
    }

    #[test]
    fn test_webhook_failed() {
        let err = ServerError::WebhookFailed {
            url: "https://example.com/webhook".to_string(),
            reason: "connection refused".to_string(),
        };
        assert!(err.is_server_error());
        assert!(err.should_retry());
        assert!(err.to_string().contains("example.com"));
    }

    #[test]
    fn test_signal_not_found() {
        let err = ServerError::SignalNotFound {
            id: "sig_123".to_string(),
        };
        assert!(err.is_client_error());
    }

    #[test]
    fn test_websocket_error() {
        let err = ServerError::WebSocketError {
            message: "upgrade failed".to_string(),
        };
        assert!(err.is_server_error());
    }

    #[test]
    fn test_connection_closed() {
        let err = ServerError::ConnectionClosed {
            reason: "client disconnect".to_string(),
        };
        assert!(!err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_transport_closed() {
        let err = ServerError::TransportClosed;
        assert!(err.is_server_error());
    }

    #[test]
    fn test_serialization_error() {
        let err = ServerError::Serialization {
            message: "invalid utf-8".to_string(),
        };
        assert!(err.is_server_error());
    }

    #[test]
    fn test_method_not_found() {
        let err = ServerError::MethodNotFound {
            method: "unknown.method".to_string(),
        };
        assert!(err.is_client_error());
    }

    #[test]
    fn test_invalid_message() {
        let err = ServerError::invalid_message("missing id");
        assert!(err.is_client_error());
    }

    #[test]
    fn test_invalid_params() {
        let err = ServerError::invalid_params("required field missing");
        assert!(err.is_client_error());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ServerError = io_err.into();
        assert!(err.is_server_error());
    }

    #[test]
    fn test_serde_json_error_conversion() {
        let json_str = "invalid json {{{";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let serde_err = result.unwrap_err();
        let err: ServerError = serde_err.into();
        assert!(matches!(err, ServerError::SerializationError(_)));
    }

    #[test]
    fn test_json_rpc_conversion_all_variants() {
        // NotAuthorized
        let err = ServerError::NotAuthorized {
            reason: "forbidden".to_string(),
        };
        let _: JsonRpcError = err.into();

        // AuthenticationRequired
        let err = ServerError::AuthenticationRequired;
        let _: JsonRpcError = err.into();

        // AuthenticationFailed
        let err = ServerError::AuthenticationFailed {
            reason: "bad token".to_string(),
        };
        let _: JsonRpcError = err.into();

        // SubscriptionPending
        let err = ServerError::SubscriptionPending {
            id: "sub_1".to_string(),
        };
        let _: JsonRpcError = err.into();

        // SubscriptionDenied
        let err = ServerError::SubscriptionDenied {
            id: "sub_2".to_string(),
            reason: Some("denied".to_string()),
        };
        let _: JsonRpcError = err.into();

        // SessionExpired
        let err = ServerError::SessionExpired {
            id: "sess_1".to_string(),
        };
        let _: JsonRpcError = err.into();

        // MethodNotFound
        let err = ServerError::MethodNotFound {
            method: "unknown".to_string(),
        };
        let _: JsonRpcError = err.into();

        // InvalidParams
        let err = ServerError::InvalidParams {
            message: "bad params".to_string(),
        };
        let _: JsonRpcError = err.into();

        // InvalidMessage
        let err = ServerError::InvalidMessage {
            message: "bad msg".to_string(),
        };
        let _: JsonRpcError = err.into();

        // SerializationError (from serde_json::Error)
        let json_str = "invalid";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let serde_err = result.unwrap_err();
        let err: ServerError = serde_err.into();
        let _: JsonRpcError = err.into();

        // InternalError (catch-all)
        let err = ServerError::InternalError {
            message: "internal".to_string(),
        };
        let _: JsonRpcError = err.into();

        // TransportError (catch-all path)
        let err = ServerError::TransportError {
            message: "transport issue".to_string(),
        };
        let _: JsonRpcError = err.into();
    }

    #[test]
    fn test_protocol_error_conversion() {
        let cauce_err = CauceError::InvalidTopic {
            topic: "bad topic".to_string(),
            reason: "spaces not allowed".to_string(),
        };
        let err: ServerError = cauce_err.into();
        assert!(matches!(err, ServerError::ProtocolError(_)));
        let _: JsonRpcError = err.into();
    }
}

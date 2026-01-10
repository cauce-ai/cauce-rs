//! Client error types for the Cauce Client SDK.
//!
//! This module provides the [`ClientError`] enum which represents all possible
//! errors that can occur during client operations.

use cauce_core::{CauceError, JsonRpcError};
use thiserror::Error;

/// Errors that can occur in the Cauce Client SDK.
///
/// This enum covers all error conditions from connection failures to protocol
/// errors, providing detailed information for debugging and error handling.
///
/// # Example
///
/// ```rust
/// use cauce_client_sdk::ClientError;
///
/// fn handle_error(err: ClientError) {
///     match err {
///         ClientError::ConnectionFailed { message } => {
///             eprintln!("Failed to connect: {}", message);
///         }
///         ClientError::NotConnected => {
///             eprintln!("Not connected to Hub");
///         }
///         ClientError::ProtocolError(cauce_err) => {
///             eprintln!("Protocol error: {}", cauce_err);
///         }
///         _ => eprintln!("Error: {}", err),
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum ClientError {
    // =========================================================================
    // Connection Errors
    // =========================================================================
    /// Connection to Hub failed.
    #[error("connection failed: {message}")]
    ConnectionFailed {
        /// Details about why the connection failed.
        message: String,
    },

    /// Connection attempt timed out.
    #[error("connection timeout after {timeout_ms}ms")]
    ConnectionTimeout {
        /// The timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Client is not connected to Hub.
    #[error("not connected to Hub")]
    NotConnected,

    /// Connection was closed unexpectedly.
    #[error("connection closed: {reason}")]
    ConnectionClosed {
        /// The reason the connection was closed.
        reason: String,
    },

    // =========================================================================
    // Handshake Errors
    // =========================================================================
    /// Hello handshake failed.
    #[error("handshake failed: {message}")]
    HandshakeFailed {
        /// Details about why the handshake failed.
        message: String,
    },

    /// Protocol version mismatch.
    #[error("version mismatch: client={client_version}, server={server_version}")]
    VersionMismatch {
        /// The client's protocol version.
        client_version: String,
        /// The server's protocol version.
        server_version: String,
    },

    // =========================================================================
    // Request Errors
    // =========================================================================
    /// Request timed out waiting for response.
    #[error("request timeout after {timeout_ms}ms")]
    RequestTimeout {
        /// The timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Request was cancelled.
    #[error("request cancelled")]
    RequestCancelled,

    // =========================================================================
    // Transport Errors
    // =========================================================================
    /// Transport-level error occurred.
    #[error("transport error: {message}")]
    TransportError {
        /// Details about the transport error.
        message: String,
    },

    /// WebSocket-specific error.
    #[error("websocket error: {message}")]
    WebSocketError {
        /// Details about the WebSocket error.
        message: String,
    },

    /// SSE-specific error.
    #[error("sse error: {message}")]
    SseError {
        /// Details about the SSE error.
        message: String,
    },

    /// HTTP polling error.
    #[error("polling error: {message}")]
    PollingError {
        /// Details about the polling error.
        message: String,
    },

    // =========================================================================
    // Protocol Errors
    // =========================================================================
    /// Protocol error from Hub (wraps CauceError).
    #[error("protocol error: {0}")]
    ProtocolError(#[from] CauceError),

    /// JSON-RPC error from Hub.
    #[error("rpc error [{code}]: {message}")]
    RpcError {
        /// The JSON-RPC error code.
        code: i32,
        /// The error message.
        message: String,
        /// Optional additional data.
        data: Option<serde_json::Value>,
    },

    // =========================================================================
    // Serialization Errors
    // =========================================================================
    /// JSON serialization/deserialization failed.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Failed to parse JSON-RPC message.
    #[error("invalid message: {message}")]
    InvalidMessage {
        /// Details about why the message is invalid.
        message: String,
    },

    // =========================================================================
    // Configuration Errors
    // =========================================================================
    /// Invalid configuration provided.
    #[error("configuration error: {message}")]
    ConfigError {
        /// Details about the configuration error.
        message: String,
    },

    /// Invalid URL format.
    #[error("invalid URL: {url}")]
    InvalidUrl {
        /// The invalid URL.
        url: String,
    },

    // =========================================================================
    // Queue Errors
    // =========================================================================
    /// Local queue error.
    #[error("queue error: {message}")]
    QueueError {
        /// Details about the queue error.
        message: String,
    },

    /// Queue is full, cannot enqueue more messages.
    #[error("queue full: capacity={capacity}")]
    QueueFull {
        /// The maximum queue capacity.
        capacity: usize,
    },

    // =========================================================================
    // Reconnection Errors
    // =========================================================================
    /// Reconnection failed after maximum attempts.
    #[error("reconnection failed after {attempts} attempts")]
    ReconnectionFailed {
        /// The number of reconnection attempts made.
        attempts: u32,
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

    /// Subscription is not active.
    #[error("subscription not active: {id} (status={status})")]
    SubscriptionNotActive {
        /// The subscription ID.
        id: String,
        /// The current status.
        status: String,
    },
}

impl From<JsonRpcError> for ClientError {
    fn from(err: JsonRpcError) -> Self {
        ClientError::RpcError {
            code: err.code,
            message: err.message,
            data: err.data,
        }
    }
}

impl ClientError {
    /// Creates a new connection failed error.
    pub fn connection_failed(message: impl Into<String>) -> Self {
        ClientError::ConnectionFailed {
            message: message.into(),
        }
    }

    /// Creates a new transport error.
    pub fn transport_error(message: impl Into<String>) -> Self {
        ClientError::TransportError {
            message: message.into(),
        }
    }

    /// Creates a new configuration error.
    pub fn config_error(message: impl Into<String>) -> Self {
        ClientError::ConfigError {
            message: message.into(),
        }
    }

    /// Creates a new invalid message error.
    pub fn invalid_message(message: impl Into<String>) -> Self {
        ClientError::InvalidMessage {
            message: message.into(),
        }
    }

    /// Returns true if this error indicates the client should reconnect.
    pub fn should_reconnect(&self) -> bool {
        matches!(
            self,
            ClientError::ConnectionClosed { .. }
                | ClientError::TransportError { .. }
                | ClientError::WebSocketError { .. }
                | ClientError::ConnectionTimeout { .. }
        )
    }

    /// Returns true if this error indicates the client should retry the request.
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            ClientError::RequestTimeout { .. } | ClientError::RequestCancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_failed_display() {
        let err = ClientError::connection_failed("connection refused");
        assert_eq!(err.to_string(), "connection failed: connection refused");
    }

    #[test]
    fn test_not_connected_display() {
        let err = ClientError::NotConnected;
        assert_eq!(err.to_string(), "not connected to Hub");
    }

    #[test]
    fn test_connection_timeout_display() {
        let err = ClientError::ConnectionTimeout { timeout_ms: 5000 };
        assert_eq!(err.to_string(), "connection timeout after 5000ms");
    }

    #[test]
    fn test_request_timeout_display() {
        let err = ClientError::RequestTimeout { timeout_ms: 30000 };
        assert_eq!(err.to_string(), "request timeout after 30000ms");
    }

    #[test]
    fn test_rpc_error_display() {
        let err = ClientError::RpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };
        assert_eq!(err.to_string(), "rpc error [-32601]: Method not found");
    }

    #[test]
    fn test_from_json_rpc_error() {
        let rpc_err = JsonRpcError::new(-32600, "Invalid request".to_string());
        let err: ClientError = rpc_err.into();

        match err {
            ClientError::RpcError { code, message, .. } => {
                assert_eq!(code, -32600);
                assert_eq!(message, "Invalid request");
            }
            _ => panic!("Expected RpcError"),
        }
    }

    #[test]
    fn test_should_reconnect() {
        assert!(ClientError::ConnectionClosed {
            reason: "server closed".to_string()
        }
        .should_reconnect());
        assert!(ClientError::transport_error("network error").should_reconnect());
        assert!(ClientError::WebSocketError {
            message: "closed".to_string()
        }
        .should_reconnect());

        assert!(!ClientError::NotConnected.should_reconnect());
        assert!(!ClientError::config_error("bad config").should_reconnect());
    }

    #[test]
    fn test_should_retry() {
        assert!(ClientError::RequestTimeout { timeout_ms: 1000 }.should_retry());
        assert!(ClientError::RequestCancelled.should_retry());

        assert!(!ClientError::NotConnected.should_retry());
        assert!(!ClientError::connection_failed("refused").should_retry());
    }

    #[test]
    fn test_config_error() {
        let err = ClientError::config_error("missing hub_url");
        assert_eq!(err.to_string(), "configuration error: missing hub_url");
    }

    #[test]
    fn test_queue_full() {
        let err = ClientError::QueueFull { capacity: 1000 };
        assert_eq!(err.to_string(), "queue full: capacity=1000");
    }

    #[test]
    fn test_reconnection_failed() {
        let err = ClientError::ReconnectionFailed { attempts: 10 };
        assert_eq!(err.to_string(), "reconnection failed after 10 attempts");
    }

    #[test]
    fn test_subscription_not_found() {
        let err = ClientError::SubscriptionNotFound {
            id: "sub_123".to_string(),
        };
        assert_eq!(err.to_string(), "subscription not found: sub_123");
    }
}

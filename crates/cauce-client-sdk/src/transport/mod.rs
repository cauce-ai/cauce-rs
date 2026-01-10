//! Transport layer for the Cauce Client SDK.
//!
//! This module provides the [`Transport`] trait which abstracts over different
//! transport mechanisms (WebSocket, SSE, HTTP Polling, etc.) and provides a
//! unified interface for sending and receiving JSON-RPC messages.
//!
//! ## Available Transports
//!
//! - **WebSocket**: Full-duplex, lowest latency (recommended)
//! - **SSE**: Server-Sent Events for streaming
//! - **Polling**: HTTP short polling
//! - **Long Polling**: HTTP long polling
//! - **Webhook**: Receives signals via HTTP callbacks
//!
//! ## Example
//!
//! ```rust,ignore
//! use cauce_client_sdk::transport::{Transport, ConnectionState};
//!
//! async fn example(transport: &mut dyn Transport) {
//!     // Connect
//!     transport.connect().await?;
//!     assert_eq!(transport.state(), ConnectionState::Connected);
//!
//!     // Send a message
//!     let request = JsonRpcMessage::Request(/* ... */);
//!     transport.send(request).await?;
//!
//!     // Receive a message
//!     if let Some(message) = transport.receive().await? {
//!         println!("Received: {:?}", message);
//!     }
//!
//!     // Disconnect
//!     transport.disconnect().await?;
//! }
//! ```

mod message;

pub use message::JsonRpcMessage;

use crate::error::ClientError;
use async_trait::async_trait;

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, ClientError>;

/// Connection state of a transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    /// Not connected to the Hub.
    Disconnected,

    /// Currently attempting to connect.
    Connecting,

    /// Successfully connected to the Hub.
    Connected,

    /// Attempting to reconnect after a connection loss.
    Reconnecting,
}

impl ConnectionState {
    /// Returns true if the transport is connected.
    pub fn is_connected(&self) -> bool {
        *self == ConnectionState::Connected
    }

    /// Returns true if the transport is in a connecting state.
    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionState::Connecting | ConnectionState::Reconnecting)
    }

    /// Returns true if the transport is disconnected.
    pub fn is_disconnected(&self) -> bool {
        *self == ConnectionState::Disconnected
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "disconnected"),
            ConnectionState::Connecting => write!(f, "connecting"),
            ConnectionState::Connected => write!(f, "connected"),
            ConnectionState::Reconnecting => write!(f, "reconnecting"),
        }
    }
}

/// Abstract interface for transport implementations.
///
/// Transports handle the low-level connection and message framing,
/// while the client handles protocol semantics (handshake, subscriptions, etc.).
///
/// ## Implementing a Transport
///
/// Custom transports must implement this trait. The transport is responsible for:
///
/// 1. Establishing and maintaining the connection
/// 2. Serializing outgoing messages to the wire format
/// 3. Deserializing incoming messages from the wire format
/// 4. Handling transport-level errors
///
/// The client layer handles:
///
/// 1. Protocol handshake (cauce.hello)
/// 2. Request/response correlation
/// 3. Subscription management
/// 4. Reconnection logic (calling connect again)
#[async_trait]
pub trait Transport: Send + Sync {
    /// Establish a connection to the Hub.
    ///
    /// This should perform any necessary handshaking at the transport level
    /// (e.g., WebSocket upgrade, HTTP connection setup) but NOT the Cauce
    /// protocol handshake (cauce.hello).
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established (network error,
    /// TLS error, etc.).
    async fn connect(&mut self) -> TransportResult<()>;

    /// Gracefully close the connection.
    ///
    /// This should perform a clean shutdown (e.g., WebSocket close frame).
    /// After calling this, the transport state should be `Disconnected`.
    ///
    /// # Errors
    ///
    /// Returns an error if the disconnect fails, but the connection should
    /// still be considered closed.
    async fn disconnect(&mut self) -> TransportResult<()>;

    /// Send a JSON-RPC message to the Hub.
    ///
    /// The message is serialized and sent over the transport connection.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transport is not connected
    /// - Serialization fails
    /// - The underlying send operation fails
    async fn send(&mut self, message: JsonRpcMessage) -> TransportResult<()>;

    /// Receive the next message from the Hub.
    ///
    /// This is an async operation that waits for the next incoming message.
    /// Returns `None` if the connection is closed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The underlying receive operation fails
    /// - Deserialization fails
    ///
    /// # Note
    ///
    /// This method may block indefinitely waiting for a message. Use
    /// tokio::time::timeout to add a timeout if needed.
    async fn receive(&mut self) -> TransportResult<Option<JsonRpcMessage>>;

    /// Returns the current connection state.
    fn state(&self) -> ConnectionState;

    /// Returns true if currently connected.
    ///
    /// This is a convenience method equivalent to `state().is_connected()`.
    fn is_connected(&self) -> bool {
        self.state().is_connected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_is_connected() {
        assert!(ConnectionState::Connected.is_connected());
        assert!(!ConnectionState::Disconnected.is_connected());
        assert!(!ConnectionState::Connecting.is_connected());
        assert!(!ConnectionState::Reconnecting.is_connected());
    }

    #[test]
    fn test_connection_state_is_connecting() {
        assert!(ConnectionState::Connecting.is_connecting());
        assert!(ConnectionState::Reconnecting.is_connecting());
        assert!(!ConnectionState::Connected.is_connecting());
        assert!(!ConnectionState::Disconnected.is_connecting());
    }

    #[test]
    fn test_connection_state_is_disconnected() {
        assert!(ConnectionState::Disconnected.is_disconnected());
        assert!(!ConnectionState::Connected.is_disconnected());
        assert!(!ConnectionState::Connecting.is_disconnected());
        assert!(!ConnectionState::Reconnecting.is_disconnected());
    }

    #[test]
    fn test_connection_state_display() {
        assert_eq!(ConnectionState::Disconnected.to_string(), "disconnected");
        assert_eq!(ConnectionState::Connecting.to_string(), "connecting");
        assert_eq!(ConnectionState::Connected.to_string(), "connected");
        assert_eq!(ConnectionState::Reconnecting.to_string(), "reconnecting");
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }
}

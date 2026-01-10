//! Mock transport implementation for testing.
//!
//! This module provides [`MockTransport`], a simple transport implementation
//! that can be used for unit testing without real network connections.

use crate::error::ClientError;
use crate::transport::{ConnectionState, JsonRpcMessage, Transport, TransportResult};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A mock transport for testing purposes.
///
/// This transport maintains queues for sent and received messages,
/// allowing tests to inspect what was sent and inject messages to be received.
///
/// # Example
///
/// ```rust
/// use cauce_client_sdk::transport::mock::MockTransport;
/// use cauce_client_sdk::transport::{Transport, JsonRpcMessage};
/// use cauce_core::JsonRpcNotification;
///
/// #[tokio::test]
/// async fn test_example() {
///     let mut transport = MockTransport::new();
///     transport.connect().await.unwrap();
///
///     // Inject a message to be received
///     let notification = JsonRpcNotification::new("test".to_string(), None);
///     transport.push_receive(JsonRpcMessage::Notification(notification));
///
///     // Receive returns the injected message
///     let msg = transport.receive().await.unwrap();
///     assert!(msg.is_some());
/// }
/// ```
pub struct MockTransport {
    /// Current connection state.
    state: ConnectionState,

    /// Queue of messages that have been sent.
    sent: Arc<Mutex<VecDeque<JsonRpcMessage>>>,

    /// Queue of messages to be received.
    receive_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,

    /// If true, receive() will return a connection closed error.
    should_fail_receive: bool,
}

impl MockTransport {
    /// Create a new mock transport in the disconnected state.
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            sent: Arc::new(Mutex::new(VecDeque::new())),
            receive_queue: Arc::new(Mutex::new(VecDeque::new())),
            should_fail_receive: false,
        }
    }

    /// Push a message onto the receive queue.
    ///
    /// The next call to `receive()` will return this message.
    pub fn push_receive(&self, message: JsonRpcMessage) {
        // Use blocking lock since this is typically called from sync test setup
        let mut queue = futures::executor::block_on(self.receive_queue.lock());
        queue.push_back(message);
    }

    /// Push a message onto the receive queue (async version).
    pub async fn push_receive_async(&self, message: JsonRpcMessage) {
        self.receive_queue.lock().await.push_back(message);
    }

    /// Pop the oldest sent message.
    ///
    /// Returns `None` if no messages have been sent.
    pub fn pop_sent(&self) -> Option<JsonRpcMessage> {
        futures::executor::block_on(self.sent.lock()).pop_front()
    }

    /// Pop the oldest sent message (async version).
    pub async fn pop_sent_async(&self) -> Option<JsonRpcMessage> {
        self.sent.lock().await.pop_front()
    }

    /// Get the number of messages in the sent queue.
    pub async fn sent_count(&self) -> usize {
        self.sent.lock().await.len()
    }

    /// Get the number of messages in the receive queue.
    pub async fn receive_queue_len(&self) -> usize {
        self.receive_queue.lock().await.len()
    }

    /// Clear all sent messages.
    pub async fn clear_sent(&self) {
        self.sent.lock().await.clear();
    }

    /// Clear the receive queue.
    pub async fn clear_receive_queue(&self) {
        self.receive_queue.lock().await.clear();
    }

    /// Set whether receive() should fail with a connection error.
    pub fn set_fail_receive(&mut self, should_fail: bool) {
        self.should_fail_receive = should_fail;
    }

    /// Manually set the connection state.
    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> TransportResult<()> {
        self.state = ConnectionState::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        self.state = ConnectionState::Disconnected;
        Ok(())
    }

    async fn send(&mut self, message: JsonRpcMessage) -> TransportResult<()> {
        if self.state != ConnectionState::Connected {
            return Err(ClientError::NotConnected);
        }
        self.sent.lock().await.push_back(message);
        Ok(())
    }

    async fn receive(&mut self) -> TransportResult<Option<JsonRpcMessage>> {
        if self.state != ConnectionState::Connected {
            return Err(ClientError::NotConnected);
        }

        if self.should_fail_receive {
            return Err(ClientError::ConnectionClosed {
                reason: "Mock failure".to_string(),
            });
        }

        Ok(self.receive_queue.lock().await.pop_front())
    }

    fn state(&self) -> ConnectionState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cauce_core::{JsonRpcNotification, JsonRpcRequest};

    #[tokio::test]
    async fn test_mock_transport_new() {
        let transport = MockTransport::new();
        assert_eq!(transport.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_mock_transport_connect_disconnect() {
        let mut transport = MockTransport::new();

        transport.connect().await.unwrap();
        assert_eq!(transport.state(), ConnectionState::Connected);

        transport.disconnect().await.unwrap();
        assert_eq!(transport.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_mock_transport_send() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        let request = JsonRpcRequest::new(1.into(), "test".to_string(), None);
        let message = JsonRpcMessage::Request(request);

        transport.send(message).await.unwrap();

        assert_eq!(transport.sent_count().await, 1);

        let sent = transport.pop_sent_async().await.unwrap();
        assert!(sent.is_request());
    }

    #[tokio::test]
    async fn test_mock_transport_send_not_connected() {
        let mut transport = MockTransport::new();

        let request = JsonRpcRequest::new(1.into(), "test".to_string(), None);
        let message = JsonRpcMessage::Request(request);

        let result = transport.send(message).await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_mock_transport_receive() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        // Push a message to receive
        let notification = JsonRpcNotification::new("test".to_string(), None);
        transport.push_receive_async(JsonRpcMessage::Notification(notification)).await;

        // Receive it
        let received = transport.receive().await.unwrap();
        assert!(received.is_some());
        assert!(received.unwrap().is_notification());
    }

    #[tokio::test]
    async fn test_mock_transport_receive_empty() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        let received = transport.receive().await.unwrap();
        assert!(received.is_none());
    }

    #[tokio::test]
    async fn test_mock_transport_receive_not_connected() {
        let mut transport = MockTransport::new();

        let result = transport.receive().await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_mock_transport_fail_receive() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        transport.set_fail_receive(true);

        let result = transport.receive().await;
        assert!(matches!(result, Err(ClientError::ConnectionClosed { .. })));
    }
}

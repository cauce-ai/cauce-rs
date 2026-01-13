//! Message router for request-response correlation and notification routing.
//!
//! The [`MessageRouter`] bridges the low-level [`Transport`](crate::transport::Transport)
//! layer with the high-level client API, handling:
//!
//! - **Request-response correlation**: Match response IDs to pending requests
//! - **Notification routing**: Broadcast incoming notifications to subscribers
//! - **Timeout management**: Cancel requests that exceed their timeout
//!
//! ## Example
//!
//! ```rust,ignore
//! use cauce_client_sdk::{ClientConfig, WebSocketTransport, MessageRouter, RouterConfig};
//! use cauce_client_sdk::Transport;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and connect transport
//!     let config = ClientConfig::builder("ws://localhost:8080", "my-agent").build()?;
//!     let mut transport = WebSocketTransport::new(config);
//!     transport.connect().await?;
//!
//!     // Create and start router
//!     let mut router = MessageRouter::new(Box::new(transport), RouterConfig::default());
//!     router.start()?;
//!
//!     // Send a request
//!     let response = router.send_request("cauce.hello", None).await?;
//!
//!     // Clean shutdown
//!     router.stop().await;
//!     Ok(())
//! }
//! ```

mod tracker;

use crate::error::ClientError;
use crate::transport::{ConnectionState, JsonRpcMessage, Transport};

use cauce_core::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::task::JoinHandle;

use tracker::RequestTracker;

/// Result type for router operations.
pub type RouterResult<T> = Result<T, ClientError>;

/// Configuration for the message router.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Default timeout for requests.
    ///
    /// Individual requests can override this with
    /// [`MessageRouter::send_request_with_timeout`].
    pub request_timeout: Duration,

    /// Capacity for the notification broadcast channel.
    ///
    /// When this capacity is exceeded, the oldest notifications
    /// are dropped for lagging subscribers.
    pub notification_channel_capacity: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            notification_channel_capacity: 1000,
        }
    }
}

impl RouterConfig {
    /// Create a new configuration with custom request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Create a new configuration with custom notification channel capacity.
    pub fn with_notification_capacity(mut self, capacity: usize) -> Self {
        self.notification_channel_capacity = capacity;
        self
    }
}

/// Message router for request-response correlation and notification routing.
///
/// The router owns a transport and spawns a background task to continuously
/// receive messages, routing them appropriately:
///
/// - **Responses** are matched to pending requests by their ID
/// - **Notifications** are broadcast to all subscribers
///
/// ## Lifecycle
///
/// 1. Create router with [`MessageRouter::new`]
/// 2. The transport should already be connected
/// 3. Call [`MessageRouter::start`] to begin the background receive task
/// 4. Use [`MessageRouter::send_request`] and [`MessageRouter::subscribe_notifications`]
/// 5. Call [`MessageRouter::stop`] when done
pub struct MessageRouter {
    /// Shared state for tracking pending requests.
    tracker: Arc<RequestTracker>,

    /// Transport wrapped in Arc<Mutex> for shared access.
    transport: Arc<Mutex<Box<dyn Transport>>>,

    /// Channel for broadcasting notifications to subscribers.
    notification_tx: broadcast::Sender<JsonRpcNotification>,

    /// Handle to the background receive task.
    receive_task: Option<JoinHandle<()>>,

    /// Shutdown signal sender.
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Router configuration.
    config: RouterConfig,
}

impl MessageRouter {
    /// Create a new message router with the given transport.
    ///
    /// The router takes ownership of the transport. The transport should
    /// already be connected before starting the router.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let transport = Box::new(WebSocketTransport::new(config));
    /// let router = MessageRouter::new(transport, RouterConfig::default());
    /// ```
    pub fn new(transport: Box<dyn Transport>, config: RouterConfig) -> Self {
        let (notification_tx, _) = broadcast::channel(config.notification_channel_capacity);

        Self {
            tracker: Arc::new(RequestTracker::new()),
            transport: Arc::new(Mutex::new(transport)),
            notification_tx,
            receive_task: None,
            shutdown_tx: None,
            config,
        }
    }

    /// Start the router's background receive task.
    ///
    /// This begins continuously receiving messages from the transport and
    /// routing them to pending requests or notification subscribers.
    ///
    /// # Errors
    ///
    /// Returns an error if the router is already started.
    pub fn start(&mut self) -> RouterResult<()> {
        if self.receive_task.is_some() {
            return Err(ClientError::config_error("Router already started"));
        }

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let task_handle = self.spawn_receive_task(shutdown_rx);
        self.receive_task = Some(task_handle);

        tracing::info!("Message router started");
        Ok(())
    }

    /// Stop the router and clean up resources.
    ///
    /// This stops the background receive task and clears all pending requests.
    /// Pending requests will have their receivers dropped, causing them to
    /// return [`ClientError::RequestCancelled`].
    pub async fn stop(&mut self) {
        tracing::info!("Stopping message router");

        // Signal shutdown to background task
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Wait for background task to complete
        if let Some(handle) = self.receive_task.take() {
            handle.abort();
            let _ = handle.await;
        }

        // Clear all pending requests
        self.tracker.clear().await;

        tracing::info!("Message router stopped");
    }

    /// Check if the router is currently running.
    pub fn is_running(&self) -> bool {
        self.receive_task.is_some()
    }

    /// Send a request and wait for a response.
    ///
    /// This method:
    /// 1. Generates a unique request ID
    /// 2. Sends the request via the transport
    /// 3. Waits for the response (with the configured timeout)
    /// 4. Returns the response or an error
    ///
    /// # Arguments
    ///
    /// * `method` - The JSON-RPC method name (e.g., "cauce.hello")
    /// * `params` - Optional parameters as a JSON value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transport fails to send the request
    /// - The request times out waiting for a response
    /// - The request is cancelled
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let response = router.send_request(
    ///     "cauce.subscribe",
    ///     Some(serde_json::json!({
    ///         "topics": ["signal.email.*"]
    ///     }))
    /// ).await?;
    /// ```
    pub async fn send_request(
        &self,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> RouterResult<JsonRpcResponse> {
        self.send_request_with_timeout(method, params, self.config.request_timeout)
            .await
    }

    /// Send a request with a custom timeout.
    ///
    /// Similar to [`send_request`](Self::send_request), but allows specifying
    /// a custom timeout for this specific request.
    pub async fn send_request_with_timeout(
        &self,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
        timeout: Duration,
    ) -> RouterResult<JsonRpcResponse> {
        let method = method.into();

        // Generate unique request ID
        let request_id = self.tracker.next_id();

        // Create the request
        let request = JsonRpcRequest::new(request_id.clone(), method.clone(), params);
        let message = JsonRpcMessage::Request(request);

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Register as pending before sending
        self.tracker.register(request_id.clone(), tx).await;

        tracing::debug!("Sending request: method={}, id={:?}", method, request_id);

        // Send via transport
        let send_result = {
            let mut transport = self.transport.lock().await;
            transport.send(message).await
        };

        if let Err(e) = send_result {
            // Remove pending request on send failure
            self.tracker.cancel(&request_id).await;
            tracing::error!("Failed to send request: {}", e);
            return Err(e);
        }

        // Wait for response with timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => {
                tracing::debug!("Received response for id={:?}", request_id);
                Ok(response)
            }
            Ok(Err(_)) => {
                // Channel closed (request was cancelled during shutdown)
                tracing::debug!("Request cancelled: id={:?}", request_id);
                Err(ClientError::RequestCancelled)
            }
            Err(_) => {
                // Timeout - remove from pending
                self.tracker.cancel(&request_id).await;
                tracing::warn!(
                    "Request timeout after {:?}: method={}, id={:?}",
                    timeout,
                    method,
                    request_id
                );
                Err(ClientError::RequestTimeout {
                    timeout_ms: timeout.as_millis() as u64,
                })
            }
        }
    }

    /// Send a notification (fire-and-forget).
    ///
    /// Notifications do not expect a response. This method returns
    /// once the message has been sent.
    ///
    /// # Arguments
    ///
    /// * `method` - The JSON-RPC method name
    /// * `params` - Optional parameters as a JSON value
    ///
    /// # Errors
    ///
    /// Returns an error if the transport fails to send.
    pub async fn send_notification(
        &self,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> RouterResult<()> {
        let method = method.into();
        let notification = JsonRpcNotification::new(method.clone(), params);
        let message = JsonRpcMessage::Notification(notification);

        tracing::debug!("Sending notification: method={}", method);

        let mut transport = self.transport.lock().await;
        transport.send(message).await
    }

    /// Subscribe to incoming notifications.
    ///
    /// Returns a receiver that will receive all incoming notifications
    /// (e.g., cauce.signal, cauce.subscription.status).
    ///
    /// Multiple subscribers are supported. Each subscriber receives
    /// all notifications independently.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut notifications = router.subscribe_notifications();
    ///
    /// tokio::spawn(async move {
    ///     while let Ok(notification) = notifications.recv().await {
    ///         match notification.method() {
    ///             "cauce.signal" => { /* handle signal */ }
    ///             _ => {}
    ///         }
    ///     }
    /// });
    /// ```
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<JsonRpcNotification> {
        self.notification_tx.subscribe()
    }

    /// Get the current connection state from the transport.
    pub async fn connection_state(&self) -> ConnectionState {
        self.transport.lock().await.state()
    }

    /// Get the number of pending requests.
    ///
    /// Useful for debugging and metrics.
    pub async fn pending_requests(&self) -> usize {
        self.tracker.pending_count().await
    }

    /// Get mutable access to the underlying transport.
    ///
    /// This can be used to connect/disconnect the transport.
    /// Note: The background task will exit if the transport disconnects.
    pub async fn transport(&self) -> tokio::sync::MutexGuard<'_, Box<dyn Transport>> {
        self.transport.lock().await
    }

    /// Spawn the background task that receives messages from the transport.
    fn spawn_receive_task(&self, mut shutdown_rx: broadcast::Receiver<()>) -> JoinHandle<()> {
        let transport = Arc::clone(&self.transport);
        let tracker = Arc::clone(&self.tracker);
        let notification_tx = self.notification_tx.clone();

        tokio::spawn(async move {
            tracing::debug!("Message router receive task started");

            loop {
                // Check for shutdown signal (non-blocking)
                if shutdown_rx.try_recv().is_ok() {
                    tracing::debug!("Receive task shutting down (shutdown signal)");
                    break;
                }

                // Receive next message from transport
                let message_result = {
                    let mut transport = transport.lock().await;

                    // Use a short timeout to allow checking shutdown signal periodically
                    tokio::time::timeout(Duration::from_millis(100), transport.receive()).await
                };

                match message_result {
                    Ok(Ok(Some(message))) => {
                        // Route the message
                        Self::route_message(message, &tracker, &notification_tx).await;
                    }
                    Ok(Ok(None)) => {
                        // Connection closed
                        tracing::info!("Transport connection closed, stopping receive task");
                        break;
                    }
                    Ok(Err(e)) => {
                        // Transport error
                        tracing::error!("Transport receive error: {}", e);

                        // On connection errors, break the loop
                        if e.should_reconnect() {
                            tracing::info!(
                                "Stopping receive task due to transport error (reconnectable)"
                            );
                            break;
                        }
                    }
                    Err(_) => {
                        // Timeout - continue loop to check shutdown
                        continue;
                    }
                }
            }

            tracing::debug!("Message router receive task stopped");
        })
    }

    /// Route an incoming message to the appropriate handler.
    async fn route_message(
        message: JsonRpcMessage,
        tracker: &Arc<RequestTracker>,
        notification_tx: &broadcast::Sender<JsonRpcNotification>,
    ) {
        match message {
            JsonRpcMessage::Response(response) => {
                // Match response to pending request
                if let Some(id) = response.id() {
                    if !tracker.complete(id, response.clone()).await {
                        tracing::warn!("Received response for unknown request ID: {:?}", id);
                    }
                } else {
                    tracing::warn!("Received response without ID");
                }
            }

            JsonRpcMessage::Notification(notification) => {
                // Broadcast notification to all subscribers
                tracing::debug!("Routing notification: method={}", notification.method());

                // Ignore error if no subscribers (or all have lagged too far behind)
                let _ = notification_tx.send(notification);
            }

            JsonRpcMessage::Request(request) => {
                // Unexpected: clients don't normally receive requests from the hub
                tracing::warn!(
                    "Received unexpected request from hub: method={}, id={:?}",
                    request.method(),
                    request.id()
                );
            }
        }
    }
}

impl Drop for MessageRouter {
    fn drop(&mut self) {
        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Abort receive task
        if let Some(handle) = &self.receive_task {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;

    fn make_router() -> MessageRouter {
        let transport = Box::new(MockTransport::new());
        MessageRouter::new(transport, RouterConfig::default())
    }

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert_eq!(config.request_timeout, Duration::from_secs(60));
        assert_eq!(config.notification_channel_capacity, 1000);
    }

    #[test]
    fn test_router_config_builder() {
        let config = RouterConfig::default()
            .with_request_timeout(Duration::from_secs(30))
            .with_notification_capacity(500);

        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.notification_channel_capacity, 500);
    }

    #[test]
    fn test_router_new() {
        let router = make_router();
        assert!(!router.is_running());
    }

    #[tokio::test]
    async fn test_router_start() {
        let mut router = make_router();
        router.start().expect("should start");
        assert!(router.is_running());
    }

    #[tokio::test]
    async fn test_router_start_twice_fails() {
        let mut router = make_router();
        router.start().expect("first start should succeed");

        let result = router.start();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_router_stop() {
        let mut router = make_router();
        router.start().expect("should start");
        assert!(router.is_running());

        router.stop().await;
        assert!(!router.is_running());
    }

    #[tokio::test]
    async fn test_send_request_timeout() {
        // Create transport and connect it first
        let mut transport = MockTransport::new();
        transport.connect().await.expect("connect");
        let mut router = MessageRouter::new(Box::new(transport), RouterConfig::default());
        router.start().expect("should start");

        // Send request with very short timeout - will timeout since mock doesn't respond
        let result = router
            .send_request_with_timeout("test.method", None, Duration::from_millis(10))
            .await;

        assert!(matches!(result, Err(ClientError::RequestTimeout { .. })));
        assert_eq!(router.pending_requests().await, 0);
    }

    #[tokio::test]
    async fn test_send_notification() {
        // Create transport and connect it first
        let mut transport = MockTransport::new();
        transport.connect().await.expect("connect");
        let mut router = MessageRouter::new(Box::new(transport), RouterConfig::default());
        router.start().expect("should start");

        // Should succeed (fire and forget)
        let result = router.send_notification("test.ping", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_subscribe_notifications() {
        let router = make_router();

        let mut rx1 = router.subscribe_notifications();
        let mut rx2 = router.subscribe_notifications();

        // Send a notification via the internal channel
        let notification = JsonRpcNotification::new("test.event".to_string(), None);
        router.notification_tx.send(notification).unwrap();

        // Both subscribers should receive it
        let recv1 = rx1.recv().await.unwrap();
        let recv2 = rx2.recv().await.unwrap();

        assert_eq!(recv1.method(), "test.event");
        assert_eq!(recv2.method(), "test.event");
    }

    #[tokio::test]
    async fn test_connection_state() {
        let router = make_router();
        let state = router.connection_state().await;
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_pending_requests() {
        let router = make_router();
        assert_eq!(router.pending_requests().await, 0);
    }

    #[tokio::test]
    async fn test_route_response_to_pending() {
        let tracker = Arc::new(RequestTracker::new());
        let (notification_tx, _) = broadcast::channel(10);

        let request_id = tracker.next_id();
        let (tx, rx) = oneshot::channel();
        tracker.register(request_id.clone(), tx).await;

        // Create response with matching ID
        let response = JsonRpcResponse::success(request_id.clone(), serde_json::json!({"ok": true}));
        let message = JsonRpcMessage::Response(response);

        // Route the message
        MessageRouter::route_message(message, &tracker, &notification_tx).await;

        // Should have completed the pending request
        let received = rx.await.expect("should receive response");
        assert!(received.is_success());
    }

    #[tokio::test]
    async fn test_route_notification_to_subscribers() {
        let tracker = Arc::new(RequestTracker::new());
        let (notification_tx, mut rx) = broadcast::channel(10);

        let notification = JsonRpcNotification::new("cauce.signal".to_string(), None);
        let message = JsonRpcMessage::Notification(notification);

        // Route the message
        MessageRouter::route_message(message, &tracker, &notification_tx).await;

        // Subscriber should receive it
        let received = rx.recv().await.expect("should receive notification");
        assert_eq!(received.method(), "cauce.signal");
    }
}

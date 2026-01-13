//! WebSocket transport implementation for the Cauce Client SDK.
//!
//! This module provides [`WebSocketTransport`], a full-duplex, low-latency
//! transport implementation using the WebSocket protocol (RFC 6455).
//!
//! ## Features
//!
//! - **TLS Support**: Both `ws://` and `wss://` URLs
//! - **Authentication**: API key and Bearer token via headers
//! - **Keepalive**: Automatic ping/pong to maintain connection
//!
//! ## Example
//!
//! ```rust,ignore
//! use cauce_client_sdk::{ClientConfig, AuthConfig};
//! use cauce_client_sdk::transport::{Transport, WebSocketTransport};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
//!         .auth(AuthConfig::bearer("token"))
//!         .build()?;
//!
//!     let mut transport = WebSocketTransport::new(config);
//!     transport.connect().await?;
//!
//!     // Use transport...
//!
//!     transport.disconnect().await?;
//!     Ok(())
//! }
//! ```

use crate::config::ClientConfig;
use crate::error::ClientError;
use crate::transport::{ConnectionState, JsonRpcMessage, TransportResult};

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::Transport;

/// Type alias for the WebSocket stream.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket transport for connecting to a Cauce Hub.
///
/// This transport uses the WebSocket protocol for full-duplex communication
/// with the Hub. It supports both secure (`wss://`) and insecure (`ws://`)
/// connections.
///
/// ## Connection Lifecycle
///
/// 1. Create transport with [`WebSocketTransport::new`]
/// 2. Call [`Transport::connect`] to establish the connection
/// 3. Use [`Transport::send`] and [`Transport::receive`] for messaging
/// 4. Call [`Transport::disconnect`] when done
///
/// ## Reconnection
///
/// The transport itself does not handle reconnection. When a connection is
/// lost, the state will change to `Disconnected` and the client layer should
/// call `connect()` again to reconnect.
pub struct WebSocketTransport {
    /// Client configuration.
    config: ClientConfig,

    /// Current connection state.
    state: ConnectionState,

    /// WebSocket stream (when connected).
    ws_stream: Option<Arc<Mutex<WsStream>>>,

    /// Shutdown signal sender for keepalive task.
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Keepalive task handle.
    keepalive_handle: Option<JoinHandle<()>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport with the given configuration.
    ///
    /// The transport is created in the `Disconnected` state. Call
    /// [`Transport::connect`] to establish the connection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cauce_client_sdk::{ClientConfig, Transport, WebSocketTransport};
    ///
    /// let config = ClientConfig::builder("ws://localhost:8080", "my-agent")
    ///     .build()
    ///     .expect("valid config");
    ///
    /// let transport = WebSocketTransport::new(config);
    /// assert!(!transport.is_connected());
    /// ```
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            state: ConnectionState::Disconnected,
            ws_stream: None,
            shutdown_tx: None,
            keepalive_handle: None,
        }
    }

    /// Build a WebSocket request with authentication headers.
    fn build_request(
        &self,
    ) -> TransportResult<tokio_tungstenite::tungstenite::handshake::client::Request> {
        let url = self.config.websocket_url();

        let mut request = url
            .into_client_request()
            .map_err(|e| ClientError::connection_failed(format!("invalid URL: {}", e)))?;

        // Add authentication header if configured
        if let Some(ref auth) = self.config.auth {
            let header_name = auth.header_name();
            let header_value = auth.header_value();

            let value = HeaderValue::from_str(&header_value)
                .map_err(|e| ClientError::config_error(format!("invalid auth header: {}", e)))?;

            request.headers_mut().insert(header_name, value);

            tracing::debug!("Added {} authentication header", auth.auth_type());
        }

        Ok(request)
    }

    /// Spawn the keepalive background task.
    fn spawn_keepalive_task(&mut self) {
        let interval = self.config.keepalive_interval;
        let ws_stream = self.ws_stream.clone();
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);

        self.shutdown_tx = Some(shutdown_tx);

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if let Some(ref stream) = ws_stream {
                            let mut stream = stream.lock().await;
                            if let Err(e) = stream.send(Message::Ping(vec![])).await {
                                tracing::warn!("Keepalive ping failed: {}", e);
                                break;
                            }
                            tracing::trace!("Sent keepalive ping");
                        } else {
                            break;
                        }
                    }

                    _ = shutdown_rx.recv() => {
                        tracing::debug!("Keepalive task shutting down");
                        break;
                    }
                }
            }
        });

        self.keepalive_handle = Some(handle);
    }

    /// Stop the keepalive task if running.
    fn stop_keepalive_task(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()); // Ignore error if already stopped
        }

        if let Some(handle) = self.keepalive_handle.take() {
            handle.abort();
        }
    }

    /// Map a tungstenite error to a ClientError.
    fn map_ws_error(e: WsError) -> ClientError {
        match e {
            WsError::ConnectionClosed | WsError::AlreadyClosed => ClientError::ConnectionClosed {
                reason: e.to_string(),
            },
            WsError::Io(io_err) => ClientError::connection_failed(io_err.to_string()),
            WsError::Tls(tls_err) => {
                ClientError::connection_failed(format!("TLS error: {}", tls_err))
            }
            WsError::Protocol(proto_err) => ClientError::WebSocketError {
                message: format!("Protocol error: {}", proto_err),
            },
            _ => ClientError::WebSocketError {
                message: e.to_string(),
            },
        }
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    /// Establish a connection to the Hub.
    ///
    /// This performs the WebSocket handshake and starts the keepalive task.
    /// After this returns successfully, the transport is in the `Connected` state.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - The connection times out
    /// - TLS handshake fails
    /// - The server rejects the connection
    async fn connect(&mut self) -> TransportResult<()> {
        // Update state
        self.state = ConnectionState::Connecting;
        tracing::info!("Connecting to WebSocket at {}", self.config.websocket_url());

        // Build request with auth headers
        let request = self.build_request()?;

        // Connect with timeout
        let connect_future = tokio_tungstenite::connect_async(request);

        let result = tokio::time::timeout(self.config.connect_timeout, connect_future).await;

        let (ws_stream, response) = match result {
            Ok(Ok((stream, response))) => (stream, response),
            Ok(Err(e)) => {
                self.state = ConnectionState::Disconnected;
                tracing::error!("WebSocket connection failed: {}", e);
                return Err(Self::map_ws_error(e));
            }
            Err(_) => {
                self.state = ConnectionState::Disconnected;
                tracing::error!(
                    "WebSocket connection timeout after {:?}",
                    self.config.connect_timeout
                );
                return Err(ClientError::ConnectionTimeout {
                    timeout_ms: self.config.connect_timeout.as_millis() as u64,
                });
            }
        };

        tracing::debug!(
            "WebSocket handshake completed, status: {}",
            response.status()
        );

        // Store stream
        self.ws_stream = Some(Arc::new(Mutex::new(ws_stream)));

        // Spawn keepalive task
        self.spawn_keepalive_task();

        // Update state
        self.state = ConnectionState::Connected;
        tracing::info!("WebSocket connected to {}", self.config.hub_url);

        Ok(())
    }

    /// Gracefully close the connection.
    ///
    /// Sends a close frame to the server and cleans up resources.
    /// After this returns, the transport is in the `Disconnected` state.
    ///
    /// # Errors
    ///
    /// Returns an error if sending the close frame fails, but the connection
    /// is still considered closed afterward.
    async fn disconnect(&mut self) -> TransportResult<()> {
        tracing::info!("Disconnecting WebSocket");

        // Stop keepalive task first
        self.stop_keepalive_task();

        // Send close frame if connected
        if let Some(stream) = self.ws_stream.take() {
            let mut stream = stream.lock().await;

            let close_result = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                stream.close(None),
            )
            .await;

            match close_result {
                Ok(Ok(_)) => tracing::debug!("Close frame sent"),
                Ok(Err(e)) => tracing::debug!("Failed to send close frame: {}", e),
                Err(_) => tracing::debug!("Close frame send timed out"),
            }
        }

        // Update state
        self.state = ConnectionState::Disconnected;
        tracing::info!("WebSocket disconnected");

        Ok(())
    }

    /// Send a JSON-RPC message to the Hub.
    ///
    /// The message is serialized to JSON and sent as a WebSocket text frame.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transport is not connected
    /// - JSON serialization fails
    /// - The underlying send operation fails
    async fn send(&mut self, message: JsonRpcMessage) -> TransportResult<()> {
        // Check state
        if !self.is_connected() {
            return Err(ClientError::NotConnected);
        }

        // Get stream
        let stream = self
            .ws_stream
            .as_ref()
            .ok_or(ClientError::NotConnected)?;

        // Serialize message
        let json = message
            .to_json()
            .map_err(|e| ClientError::invalid_message(e.to_string()))?;

        tracing::trace!("Sending WebSocket message: {}", json);

        // Send as text frame
        let mut stream = stream.lock().await;
        stream
            .send(Message::Text(json))
            .await
            .map_err(Self::map_ws_error)?;

        Ok(())
    }

    /// Receive the next message from the Hub.
    ///
    /// This waits for the next incoming message and parses it as JSON-RPC.
    /// Returns `None` if the connection is closed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transport is not connected
    /// - The underlying receive operation fails
    /// - JSON deserialization fails
    async fn receive(&mut self) -> TransportResult<Option<JsonRpcMessage>> {
        // Check state
        if !self.is_connected() {
            return Err(ClientError::NotConnected);
        }

        // Get stream
        let stream = self
            .ws_stream
            .as_ref()
            .ok_or(ClientError::NotConnected)?;

        let mut stream = stream.lock().await;

        loop {
            match stream.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => {
                        tracing::trace!("Received WebSocket message: {}", text);
                        let message = JsonRpcMessage::parse(&text)
                            .map_err(|e| ClientError::invalid_message(e.to_string()))?;
                        return Ok(Some(message));
                    }

                    Message::Close(frame) => {
                        tracing::info!("WebSocket closed by server: {:?}", frame);
                        drop(stream); // Release lock before updating state
                        self.state = ConnectionState::Disconnected;
                        self.stop_keepalive_task();
                        return Ok(None);
                    }

                    Message::Ping(_) => {
                        // tungstenite auto-responds to ping
                        tracing::trace!("Received ping");
                        continue;
                    }

                    Message::Pong(_) => {
                        tracing::trace!("Received pong");
                        continue;
                    }

                    Message::Binary(data) => {
                        tracing::warn!(
                            "Received unexpected binary frame ({} bytes), ignoring",
                            data.len()
                        );
                        continue;
                    }

                    Message::Frame(_) => {
                        // Raw frames shouldn't appear here
                        continue;
                    }
                },

                Some(Err(e)) => {
                    tracing::error!("WebSocket receive error: {}", e);
                    drop(stream); // Release lock before updating state
                    self.state = ConnectionState::Disconnected;
                    self.stop_keepalive_task();
                    return Err(Self::map_ws_error(e));
                }

                None => {
                    // Stream ended
                    tracing::info!("WebSocket stream ended");
                    drop(stream); // Release lock before updating state
                    self.state = ConnectionState::Disconnected;
                    self.stop_keepalive_task();
                    return Ok(None);
                }
            }
        }
    }

    /// Returns the current connection state.
    fn state(&self) -> ConnectionState {
        self.state
    }
}

impl Drop for WebSocketTransport {
    fn drop(&mut self) {
        // Stop keepalive task on drop
        self.stop_keepalive_task();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config(url: &str) -> ClientConfig {
        ClientConfig::builder(url, "test-client")
            .build()
            .expect("valid config")
    }

    #[test]
    fn test_new_transport_starts_disconnected() {
        let config = make_test_config("ws://localhost:8080");
        let transport = WebSocketTransport::new(config);

        assert_eq!(transport.state(), ConnectionState::Disconnected);
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_build_request_without_auth() {
        let config = make_test_config("ws://localhost:8080");
        let transport = WebSocketTransport::new(config);

        let request = transport.build_request().expect("should build request");
        assert_eq!(request.uri().to_string(), "ws://localhost:8080/");
    }

    #[test]
    fn test_build_request_with_api_key() {
        use crate::config::AuthConfig;

        let config = ClientConfig::builder("ws://localhost:8080", "test-client")
            .auth(AuthConfig::api_key("test-key"))
            .build()
            .expect("valid config");

        let transport = WebSocketTransport::new(config);
        let request = transport.build_request().expect("should build request");

        let headers = request.headers();
        assert_eq!(
            headers.get("X-Cauce-API-Key").map(|v| v.to_str().unwrap()),
            Some("test-key")
        );
    }

    #[test]
    fn test_build_request_with_bearer() {
        use crate::config::AuthConfig;

        let config = ClientConfig::builder("ws://localhost:8080", "test-client")
            .auth(AuthConfig::bearer("my-token"))
            .build()
            .expect("valid config");

        let transport = WebSocketTransport::new(config);
        let request = transport.build_request().expect("should build request");

        let headers = request.headers();
        assert_eq!(
            headers.get("Authorization").map(|v| v.to_str().unwrap()),
            Some("Bearer my-token")
        );
    }

    #[test]
    fn test_build_request_converts_http_to_ws() {
        let config = ClientConfig::builder("http://localhost:8080", "test-client")
            .build()
            .expect("valid config");

        let transport = WebSocketTransport::new(config);
        let request = transport.build_request().expect("should build request");

        assert!(request.uri().to_string().starts_with("ws://"));
    }

    #[test]
    fn test_build_request_converts_https_to_wss() {
        let config = ClientConfig::builder("https://localhost:8080", "test-client")
            .build()
            .expect("valid config");

        let transport = WebSocketTransport::new(config);
        let request = transport.build_request().expect("should build request");

        assert!(request.uri().to_string().starts_with("wss://"));
    }

    #[test]
    fn test_map_ws_error_connection_closed() {
        let err = WebSocketTransport::map_ws_error(WsError::ConnectionClosed);
        match err {
            ClientError::ConnectionClosed { .. } => {}
            _ => panic!("Expected ConnectionClosed, got {:?}", err),
        }
    }

    #[test]
    fn test_map_ws_error_already_closed() {
        let err = WebSocketTransport::map_ws_error(WsError::AlreadyClosed);
        match err {
            ClientError::ConnectionClosed { .. } => {}
            _ => panic!("Expected ConnectionClosed, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_send_when_disconnected() {
        use cauce_core::JsonRpcRequest;

        let config = make_test_config("ws://localhost:8080");
        let mut transport = WebSocketTransport::new(config);

        let request = JsonRpcRequest::new(1.into(), "test.method".to_string(), None);
        let message = JsonRpcMessage::Request(request);

        let result = transport.send(message).await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_receive_when_disconnected() {
        let config = make_test_config("ws://localhost:8080");
        let mut transport = WebSocketTransport::new(config);

        let result = transport.receive().await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }
}

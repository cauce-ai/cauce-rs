//! Server-Sent Events (SSE) transport implementation.
//!
//! SSE provides a unidirectional channel from server to client for receiving
//! signals, with a separate HTTP POST channel for sending messages.
//!
//! # Example
//!
//! ```ignore
//! use cauce_client_sdk::{ClientConfig, transport::SseTransport};
//!
//! let config = ClientConfig::builder("https://hub.example.com", "my-client")
//!     .build()?;
//!
//! let mut transport = SseTransport::new(config);
//! transport.connect().await?;
//! ```

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::ClientConfig;
use crate::error::ClientError;
use crate::transport::{ConnectionState, JsonRpcMessage, Transport, TransportResult};

/// SSE transport for receiving messages via Server-Sent Events.
///
/// This transport uses:
/// - GET with `Accept: text/event-stream` for receiving messages
/// - POST for sending messages to the hub
///
/// # Reconnection
///
/// The transport automatically tracks the last event ID and sends it
/// in the `Last-Event-ID` header on reconnection to resume from where
/// it left off.
pub struct SseTransport {
    /// Client configuration.
    config: ClientConfig,

    /// Current connection state.
    state: ConnectionState,

    /// HTTP client for requests.
    client: reqwest::Client,

    /// Queue of received messages.
    receive_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,

    /// Last event ID for reconnection.
    last_event_id: Arc<Mutex<Option<String>>>,

    /// Background task for receiving SSE events.
    receive_task: Option<JoinHandle<()>>,

    /// Shutdown signal sender.
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl SseTransport {
    /// Creates a new SSE transport with the given configuration.
    ///
    /// The transport starts in the `Disconnected` state.
    pub fn new(config: ClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // Long timeout for SSE
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            state: ConnectionState::Disconnected,
            client,
            receive_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_event_id: Arc::new(Mutex::new(None)),
            receive_task: None,
            shutdown_tx: None,
        }
    }

    /// Get the SSE endpoint URL.
    fn sse_url(&self) -> String {
        format!("{}/cauce/v1/events", self.config.http_url())
    }

    /// Get the send endpoint URL.
    fn send_url(&self) -> String {
        format!("{}/cauce/v1/messages", self.config.http_url())
    }

    /// Build request headers including authentication.
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(auth) = &self.config.auth {
            headers.insert(
                reqwest::header::HeaderName::from_static(auth.header_name()),
                auth.header_value()
                    .parse()
                    .expect("Invalid auth header value"),
            );
        }

        headers
    }

    /// Spawn the background task that receives SSE events.
    ///
    /// Returns the `JoinHandle` so the caller can store it for proper cleanup.
    fn spawn_receive_task(
        &self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> JoinHandle<()> {
        let client = self.client.clone();
        let url = self.sse_url();
        let headers = self.build_headers();
        let receive_queue = Arc::clone(&self.receive_queue);
        let last_event_id = Arc::clone(&self.last_event_id);

        tokio::spawn(async move {
            loop {
                // Check for shutdown before starting a new connection attempt
                if shutdown_rx.try_recv().is_ok() {
                    tracing::debug!("SSE receive task shutting down");
                    break;
                }

                // Build request with Last-Event-ID if available
                let mut request = client
                    .get(&url)
                    .headers(headers.clone())
                    .header("Accept", "text/event-stream");

                if let Some(id) = last_event_id.lock().await.as_ref() {
                    request = request.header("Last-Event-ID", id.clone());
                }

                // Send request with shutdown check using select
                let response = tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::debug!("SSE receive task shutting down during connect");
                        return;
                    }
                    result = request.send() => result,
                };

                match response {
                    Ok(response) => {
                        if !response.status().is_success() {
                            tracing::error!("SSE connection failed: {}", response.status());
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }

                        tracing::debug!("SSE connection established");

                        // Process the byte stream as SSE events
                        let mut stream = response.bytes_stream();
                        let mut buffer = String::new();

                        loop {
                            // Use select to check shutdown while waiting for stream data
                            let chunk_result = tokio::select! {
                                _ = shutdown_rx.recv() => {
                                    tracing::debug!("SSE receive task shutting down");
                                    return;
                                }
                                chunk = stream.next() => chunk,
                            };

                            let Some(chunk_result) = chunk_result else {
                                // Stream ended
                                break;
                            };

                            match chunk_result {
                                Ok(chunk) => {
                                    if let Ok(text) = std::str::from_utf8(&chunk) {
                                        buffer.push_str(text);

                                        // Process complete events
                                        while let Some(pos) = buffer.find("\n\n") {
                                            let event_text = buffer[..pos].to_string();
                                            buffer = buffer[pos + 2..].to_string();

                                            if let Some((event_id, message)) =
                                                Self::parse_sse_event(&event_text)
                                            {
                                                // Update last event ID
                                                if let Some(id) = event_id {
                                                    *last_event_id.lock().await = Some(id);
                                                }

                                                // Queue the message
                                                receive_queue.lock().await.push_back(message);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("SSE stream error: {}", e);
                                    break;
                                }
                            }
                        }

                        tracing::debug!("SSE connection closed, reconnecting...");
                    }
                    Err(e) => {
                        tracing::error!("SSE request failed: {}", e);
                    }
                }

                // Wait before reconnecting, but also check for shutdown
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::debug!("SSE receive task shutting down during reconnect wait");
                        return;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }
            }
        })
    }

    /// Parse an SSE event into an optional event ID and message.
    fn parse_sse_event(event_text: &str) -> Option<(Option<String>, JsonRpcMessage)> {
        let mut event_type = None;
        let mut event_id = None;
        let mut data_lines = Vec::new();

        for line in event_text.lines() {
            if let Some(stripped) = line.strip_prefix("event:") {
                event_type = Some(stripped.trim().to_string());
            } else if let Some(stripped) = line.strip_prefix("id:") {
                event_id = Some(stripped.trim().to_string());
            } else if let Some(stripped) = line.strip_prefix("data:") {
                data_lines.push(stripped.trim().to_string());
            } else if line.starts_with(':') {
                // Comment/keepalive, ignore
                continue;
            }
        }

        // Ignore keepalive events
        if event_type.as_deref() == Some("keepalive") {
            return None;
        }

        // Join data lines and parse as JSON-RPC message
        if !data_lines.is_empty() {
            let data = data_lines.join("\n");
            match JsonRpcMessage::parse(&data) {
                Ok(message) => Some((event_id, message)),
                Err(e) => {
                    tracing::warn!("Failed to parse SSE data as JSON-RPC: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl Transport for SseTransport {
    async fn connect(&mut self) -> TransportResult<()> {
        if self.state == ConnectionState::Connected {
            return Ok(());
        }

        self.state = ConnectionState::Connecting;
        tracing::debug!("Connecting SSE transport to {}", self.sse_url());

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn receive task and store the handle for proper cleanup
        self.receive_task = Some(self.spawn_receive_task(shutdown_rx));

        self.state = ConnectionState::Connected;
        tracing::info!("SSE transport connected");

        Ok(())
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        if self.state == ConnectionState::Disconnected {
            return Ok(());
        }

        tracing::debug!("Disconnecting SSE transport");

        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Abort receive task
        if let Some(handle) = self.receive_task.take() {
            handle.abort();
        }

        self.state = ConnectionState::Disconnected;
        self.shutdown_tx = None;

        tracing::info!("SSE transport disconnected");
        Ok(())
    }

    async fn send(&mut self, message: JsonRpcMessage) -> TransportResult<()> {
        if self.state != ConnectionState::Connected {
            return Err(ClientError::NotConnected);
        }

        let json = message
            .to_json()
            .map_err(ClientError::SerializationError)?;

        let response = self
            .client
            .post(self.send_url())
            .headers(self.build_headers())
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await
            .map_err(|e| ClientError::TransportError {
                message: format!("Failed to send message: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(ClientError::TransportError {
                message: format!("Send failed with status: {}", response.status()),
            });
        }

        Ok(())
    }

    async fn receive(&mut self) -> TransportResult<Option<JsonRpcMessage>> {
        if self.state != ConnectionState::Connected {
            return Err(ClientError::NotConnected);
        }

        // Try to get a message from the queue
        if let Some(message) = self.receive_queue.lock().await.pop_front() {
            return Ok(Some(message));
        }

        // No message available
        Ok(None)
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }
}

impl Drop for SseTransport {
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
    use crate::ClientType;

    fn make_config() -> ClientConfig {
        ClientConfig::builder("https://localhost:8080", "test-client")
            .client_type(ClientType::Agent)
            .build()
            .unwrap()
    }

    #[test]
    fn test_new_transport_starts_disconnected() {
        let transport = SseTransport::new(make_config());
        assert_eq!(transport.state(), ConnectionState::Disconnected);
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_sse_url() {
        let transport = SseTransport::new(make_config());
        assert_eq!(transport.sse_url(), "https://localhost:8080/cauce/v1/events");
    }

    #[test]
    fn test_send_url() {
        let transport = SseTransport::new(make_config());
        assert_eq!(
            transport.send_url(),
            "https://localhost:8080/cauce/v1/messages"
        );
    }

    #[test]
    fn test_parse_sse_event_simple() {
        let event = "data: {\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
        let result = SseTransport::parse_sse_event(event);
        assert!(result.is_some());
        let (id, message) = result.unwrap();
        assert!(id.is_none());
        assert!(message.is_notification());
    }

    #[test]
    fn test_parse_sse_event_with_id() {
        let event = "id: 123\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
        let result = SseTransport::parse_sse_event(event);
        assert!(result.is_some());
        let (id, _message) = result.unwrap();
        assert_eq!(id, Some("123".to_string()));
    }

    #[test]
    fn test_parse_sse_event_with_event_type() {
        let event = "event: message\nid: 456\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
        let result = SseTransport::parse_sse_event(event);
        assert!(result.is_some());
        let (id, _message) = result.unwrap();
        assert_eq!(id, Some("456".to_string()));
    }

    #[test]
    fn test_parse_sse_event_keepalive_ignored() {
        let event = "event: keepalive\ndata: {}";
        let result = SseTransport::parse_sse_event(event);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_event_comment_ignored() {
        let event = ": this is a comment\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
        let result = SseTransport::parse_sse_event(event);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_sse_event_multiline_data() {
        // SSE spec: multiple data lines are joined with newlines
        // This creates valid JSON when joined: {"jsonrpc":"2.0","method":"test"}
        let event = "data: {\"jsonrpc\":\"2.0\",\ndata: \"method\":\"test\"}";
        let result = SseTransport::parse_sse_event(event);
        // Data lines are joined with newline, creating valid JSON with whitespace
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_sse_event_empty() {
        let event = "";
        let result = SseTransport::parse_sse_event(event);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_send_when_disconnected() {
        let mut transport = SseTransport::new(make_config());
        let message = JsonRpcMessage::Notification(cauce_core::JsonRpcNotification::new(
            "test".to_string(),
            None,
        ));
        let result = transport.send(message).await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_receive_when_disconnected() {
        let mut transport = SseTransport::new(make_config());
        let result = transport.receive().await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }
}

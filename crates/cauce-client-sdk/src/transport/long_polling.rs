//! Long Polling transport implementation.
//!
//! Long polling keeps a request open on the server until data is available
//! or a timeout occurs, providing near-real-time message delivery without
//! the complexity of WebSocket or SSE.
//!
//! # Example
//!
//! ```ignore
//! use cauce_client_sdk::{ClientConfig, transport::LongPollingTransport};
//!
//! let config = ClientConfig::builder("https://hub.example.com", "my-client")
//!     .build()?;
//!
//! let mut transport = LongPollingTransport::new(config);
//! transport.connect().await?;
//! ```

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::ClientConfig;
use crate::error::ClientError;
use crate::transport::{ConnectionState, JsonRpcMessage, Transport, TransportResult};

/// Default server-side timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Long Polling transport for near-real-time message delivery.
///
/// This transport:
/// - Sends `GET /cauce/v1/poll?wait=true&timeout=...` requests
/// - Server holds the connection until data is available or timeout
/// - Immediately reconnects after receiving data or timeout
pub struct LongPollingTransport {
    /// Client configuration.
    config: ClientConfig,

    /// Current connection state.
    state: ConnectionState,

    /// HTTP client for requests.
    client: reqwest::Client,

    /// Queue of received messages.
    receive_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,

    /// Last message ID for pagination.
    last_id: Arc<Mutex<Option<String>>>,

    /// Server-side timeout in seconds.
    timeout_secs: u64,

    /// Subscription ID for polling (set after subscribe).
    subscription_id: Arc<Mutex<Option<String>>>,

    /// Background long polling task.
    poll_task: Option<JoinHandle<()>>,

    /// Shutdown signal sender.
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

/// Response from the long poll endpoint.
#[derive(Debug, serde::Deserialize)]
struct LongPollResponse {
    /// Messages received (empty array on timeout).
    messages: Vec<serde_json::Value>,
    /// Last message ID for pagination.
    #[serde(default)]
    last_id: Option<String>,
    /// Whether this was a timeout response.
    #[serde(default)]
    timeout: bool,
}

impl LongPollingTransport {
    /// Creates a new long polling transport with the given configuration.
    ///
    /// The transport starts in the `Disconnected` state.
    pub fn new(config: ClientConfig) -> Self {
        // Client timeout should be longer than server timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS + 10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            state: ConnectionState::Disconnected,
            client,
            receive_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_id: Arc::new(Mutex::new(None)),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            subscription_id: Arc::new(Mutex::new(None)),
            poll_task: None,
            shutdown_tx: None,
        }
    }

    /// Creates a new long polling transport with a custom timeout.
    pub fn with_timeout(config: ClientConfig, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs + 10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            state: ConnectionState::Disconnected,
            client,
            receive_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_id: Arc::new(Mutex::new(None)),
            timeout_secs,
            subscription_id: Arc::new(Mutex::new(None)),
            poll_task: None,
            shutdown_tx: None,
        }
    }

    /// Set the subscription ID for polling.
    ///
    /// This should be called after subscribing to topics.
    pub async fn set_subscription_id(&self, id: impl Into<String>) {
        *self.subscription_id.lock().await = Some(id.into());
    }

    /// Get the poll endpoint URL.
    fn poll_url(&self) -> String {
        format!("{}/cauce/v1/poll", self.config.http_url())
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

    /// Spawn the background long polling task.
    fn spawn_poll_task(&mut self, mut shutdown_rx: tokio::sync::broadcast::Receiver<()>) {
        let client = self.client.clone();
        let base_url = self.poll_url();
        let headers = self.build_headers();
        let receive_queue = Arc::clone(&self.receive_queue);
        let last_id = Arc::clone(&self.last_id);
        let timeout_secs = self.timeout_secs;
        let subscription_id = Arc::clone(&self.subscription_id);

        let task = tokio::spawn(async move {
            loop {
                // Check for shutdown
                if shutdown_rx.try_recv().is_ok() {
                    tracing::debug!("Long polling task shutting down");
                    break;
                }

                // Get current subscription ID
                let sub_id = subscription_id.lock().await.clone();

                // Only poll if we have a subscription
                if let Some(sub_id) = sub_id {
                    // Build poll URL with query parameters
                    let mut url = format!(
                        "{}?subscription_id={}&wait=true&timeout={}",
                        base_url, sub_id, timeout_secs
                    );
                    if let Some(id) = last_id.lock().await.as_ref() {
                        url.push_str(&format!("&last_id={}", id));
                    }

                    // Send long poll request
                    match client.get(&url).headers(headers.clone()).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                match response.json::<LongPollResponse>().await {
                                    Ok(poll_response) => {
                                        // Update last_id
                                        if let Some(id) = poll_response.last_id {
                                            *last_id.lock().await = Some(id);
                                        }

                                        // Parse and queue messages
                                        for msg_value in poll_response.messages {
                                            let json = serde_json::to_string(&msg_value)
                                                .unwrap_or_default();
                                            if let Ok(message) = JsonRpcMessage::parse(&json) {
                                                receive_queue.lock().await.push_back(message);
                                            }
                                        }

                                        // Log timeout vs data response
                                        if poll_response.timeout {
                                            tracing::trace!("Long poll timeout, reconnecting");
                                        } else {
                                            tracing::trace!("Long poll received data, reconnecting");
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse long poll response: {}", e);
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                    }
                                }
                            } else {
                                tracing::warn!("Long poll request failed: {}", response.status());
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Long poll request error: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                } else {
                    // No subscription yet, wait a bit before checking again
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }

                // Immediately reconnect (no delay between polls for long polling)
            }
        });

        self.poll_task = Some(task);
    }
}

#[async_trait]
impl Transport for LongPollingTransport {
    async fn connect(&mut self) -> TransportResult<()> {
        if self.state == ConnectionState::Connected {
            return Ok(());
        }

        self.state = ConnectionState::Connecting;
        tracing::debug!("Connecting long polling transport to {}", self.poll_url());

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn long polling task
        self.spawn_poll_task(shutdown_rx);

        self.state = ConnectionState::Connected;
        tracing::info!("Long polling transport connected");

        Ok(())
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        if self.state == ConnectionState::Disconnected {
            return Ok(());
        }

        tracing::debug!("Disconnecting long polling transport");

        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Abort poll task
        if let Some(handle) = self.poll_task.take() {
            handle.abort();
        }

        self.state = ConnectionState::Disconnected;
        self.shutdown_tx = None;

        tracing::info!("Long polling transport disconnected");
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

impl Drop for LongPollingTransport {
    fn drop(&mut self) {
        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Abort poll task
        if let Some(handle) = &self.poll_task {
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
        let transport = LongPollingTransport::new(make_config());
        assert_eq!(transport.state(), ConnectionState::Disconnected);
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_poll_url() {
        let transport = LongPollingTransport::new(make_config());
        assert_eq!(transport.poll_url(), "https://localhost:8080/cauce/v1/poll");
    }

    #[test]
    fn test_send_url() {
        let transport = LongPollingTransport::new(make_config());
        assert_eq!(
            transport.send_url(),
            "https://localhost:8080/cauce/v1/messages"
        );
    }

    #[test]
    fn test_custom_timeout() {
        let transport = LongPollingTransport::with_timeout(make_config(), 60);
        assert_eq!(transport.timeout_secs, 60);
    }

    #[tokio::test]
    async fn test_set_subscription_id() {
        let transport = LongPollingTransport::new(make_config());
        transport.set_subscription_id("sub_123").await;
        assert_eq!(
            *transport.subscription_id.lock().await,
            Some("sub_123".to_string())
        );
    }

    #[tokio::test]
    async fn test_send_when_disconnected() {
        let mut transport = LongPollingTransport::new(make_config());
        let message = JsonRpcMessage::Notification(cauce_core::JsonRpcNotification::new(
            "test".to_string(),
            None,
        ));
        let result = transport.send(message).await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_receive_when_disconnected() {
        let mut transport = LongPollingTransport::new(make_config());
        let result = transport.receive().await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut transport = LongPollingTransport::new(make_config());

        // Connect
        transport.connect().await.unwrap();
        assert_eq!(transport.state(), ConnectionState::Connected);
        assert!(transport.is_connected());

        // Disconnect
        transport.disconnect().await.unwrap();
        assert_eq!(transport.state(), ConnectionState::Disconnected);
        assert!(!transport.is_connected());
    }
}

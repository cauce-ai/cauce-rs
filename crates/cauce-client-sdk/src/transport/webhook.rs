//! Webhook transport implementation.
//!
//! Webhook transport receives messages via HTTP callbacks from the hub.
//! The client registers a callback URL, and the hub POSTs messages to it.
//!
//! # Example
//!
//! ```ignore
//! use cauce_client_sdk::{ClientConfig, transport::WebhookTransport};
//!
//! let config = ClientConfig::builder("https://hub.example.com", "my-client")
//!     .build()?;
//!
//! // Create webhook transport listening on port 8081
//! let mut transport = WebhookTransport::new(config, "http://localhost:8081/webhook");
//! transport.connect().await?;
//! ```

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::ClientConfig;
use crate::error::ClientError;
use crate::transport::{ConnectionState, JsonRpcMessage, Transport, TransportResult};

/// Default webhook server port.
const DEFAULT_WEBHOOK_PORT: u16 = 8081;

/// Webhook transport for receiving messages via HTTP callbacks.
///
/// This transport:
/// - Runs a local HTTP server to receive webhook callbacks
/// - Registers the callback URL with the hub
/// - Sends messages via POST to the hub's message endpoint
pub struct WebhookTransport {
    /// Client configuration.
    config: ClientConfig,

    /// Current connection state.
    state: ConnectionState,

    /// HTTP client for outgoing requests.
    client: reqwest::Client,

    /// Queue of received messages.
    receive_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,

    /// The callback URL for receiving webhooks.
    callback_url: String,

    /// Local address to bind the webhook server.
    bind_addr: SocketAddr,

    /// Background webhook server task.
    server_task: Option<JoinHandle<()>>,

    /// Shutdown signal sender.
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl WebhookTransport {
    /// Creates a new webhook transport with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    /// * `callback_url` - The public URL where the hub should send webhooks
    ///
    /// The transport starts in the `Disconnected` state.
    pub fn new(config: ClientConfig, callback_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            state: ConnectionState::Disconnected,
            client,
            receive_queue: Arc::new(Mutex::new(VecDeque::new())),
            callback_url: callback_url.into(),
            bind_addr: SocketAddr::from(([127, 0, 0, 1], DEFAULT_WEBHOOK_PORT)),
            server_task: None,
            shutdown_tx: None,
        }
    }

    /// Creates a new webhook transport with a custom bind address.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    /// * `callback_url` - The public URL where the hub should send webhooks
    /// * `bind_addr` - Local address to bind the webhook server
    pub fn with_bind_addr(
        config: ClientConfig,
        callback_url: impl Into<String>,
        bind_addr: SocketAddr,
    ) -> Self {
        let mut transport = Self::new(config, callback_url);
        transport.bind_addr = bind_addr;
        transport
    }

    /// Get the callback URL.
    pub fn callback_url(&self) -> &str {
        &self.callback_url
    }

    /// Get the send endpoint URL.
    fn send_url(&self) -> String {
        format!("{}/cauce/v1/messages", self.config.http_url())
    }

    /// Get the webhook registration endpoint URL.
    #[allow(dead_code)]
    fn register_url(&self) -> String {
        format!("{}/cauce/v1/webhooks", self.config.http_url())
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

    /// Spawn the webhook server task.
    fn spawn_server_task(
        &mut self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), ClientError> {
        let receive_queue = Arc::clone(&self.receive_queue);
        let bind_addr = self.bind_addr;

        // Create a simple webhook handler using hyper
        let task = tokio::spawn(async move {
            use hyper::service::{make_service_fn, service_fn};
            use hyper::{Body, Request, Response, Server, StatusCode};

            let queue = receive_queue;

            let make_svc = make_service_fn(move |_conn| {
                let queue = Arc::clone(&queue);
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let queue = Arc::clone(&queue);
                        async move {
                            // Only handle POST requests to the webhook path
                            if req.method() != hyper::Method::POST {
                                return Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(StatusCode::METHOD_NOT_ALLOWED)
                                        .body(Body::empty())
                                        .unwrap(),
                                );
                            }

                            // Read the body
                            let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
                                Ok(bytes) => bytes,
                                Err(_) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from("Failed to read body"))
                                        .unwrap());
                                }
                            };

                            let body_str = match std::str::from_utf8(&body_bytes) {
                                Ok(s) => s,
                                Err(_) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from("Invalid UTF-8"))
                                        .unwrap());
                                }
                            };

                            // Parse as JSON-RPC message
                            match JsonRpcMessage::parse(body_str) {
                                Ok(message) => {
                                    queue.lock().await.push_back(message);
                                    Ok(Response::builder()
                                        .status(StatusCode::OK)
                                        .body(Body::from("{\"ok\":true}"))
                                        .unwrap())
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse webhook message: {}", e);
                                    Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Parse error: {}", e)))
                                        .unwrap())
                                }
                            }
                        }
                    }))
                }
            });

            let server = Server::bind(&bind_addr).serve(make_svc);
            tracing::info!("Webhook server listening on {}", bind_addr);

            // Run server with graceful shutdown
            let graceful = server.with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
                tracing::debug!("Webhook server shutting down");
            });

            if let Err(e) = graceful.await {
                tracing::error!("Webhook server error: {}", e);
            }
        });

        self.server_task = Some(task);
        Ok(())
    }
}

#[async_trait]
impl Transport for WebhookTransport {
    async fn connect(&mut self) -> TransportResult<()> {
        if self.state == ConnectionState::Connected {
            return Ok(());
        }

        self.state = ConnectionState::Connecting;
        tracing::debug!(
            "Starting webhook transport, callback URL: {}",
            self.callback_url
        );

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn webhook server
        self.spawn_server_task(shutdown_rx)?;

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.state = ConnectionState::Connected;
        tracing::info!("Webhook transport connected");

        Ok(())
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        if self.state == ConnectionState::Disconnected {
            return Ok(());
        }

        tracing::debug!("Disconnecting webhook transport");

        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Abort server task
        if let Some(handle) = self.server_task.take() {
            handle.abort();
        }

        self.state = ConnectionState::Disconnected;
        self.shutdown_tx = None;

        tracing::info!("Webhook transport disconnected");
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

impl Drop for WebhookTransport {
    fn drop(&mut self) {
        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Abort server task
        if let Some(handle) = &self.server_task {
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
        let transport = WebhookTransport::new(make_config(), "http://localhost:8081/webhook");
        assert_eq!(transport.state(), ConnectionState::Disconnected);
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_callback_url() {
        let transport = WebhookTransport::new(make_config(), "http://example.com/webhook");
        assert_eq!(transport.callback_url(), "http://example.com/webhook");
    }

    #[test]
    fn test_send_url() {
        let transport = WebhookTransport::new(make_config(), "http://localhost:8081/webhook");
        assert_eq!(
            transport.send_url(),
            "https://localhost:8080/cauce/v1/messages"
        );
    }

    #[test]
    fn test_register_url() {
        let transport = WebhookTransport::new(make_config(), "http://localhost:8081/webhook");
        assert_eq!(
            transport.register_url(),
            "https://localhost:8080/cauce/v1/webhooks"
        );
    }

    #[test]
    fn test_custom_bind_addr() {
        let bind_addr: SocketAddr = "0.0.0.0:9000".parse().unwrap();
        let transport = WebhookTransport::with_bind_addr(
            make_config(),
            "http://example.com/webhook",
            bind_addr,
        );
        assert_eq!(transport.bind_addr, bind_addr);
    }

    #[tokio::test]
    async fn test_send_when_disconnected() {
        let mut transport = WebhookTransport::new(make_config(), "http://localhost:8081/webhook");
        let message = JsonRpcMessage::Notification(cauce_core::JsonRpcNotification::new(
            "test".to_string(),
            None,
        ));
        let result = transport.send(message).await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }

    #[tokio::test]
    async fn test_receive_when_disconnected() {
        let mut transport = WebhookTransport::new(make_config(), "http://localhost:8081/webhook");
        let result = transport.receive().await;
        assert!(matches!(result, Err(ClientError::NotConnected)));
    }
}

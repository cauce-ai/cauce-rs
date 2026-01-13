//! High-level Cauce client implementation.
//!
//! This module provides [`CauceClient`], the main entry point for connecting
//! to Cauce Protocol Hubs and interacting with signals and subscriptions.
//!
//! # Example
//!
//! ```ignore
//! use cauce_client_sdk::{CauceClient, ClientConfig, AuthConfig};
//! use cauce_core::ClientType;
//!
//! // Connect to the hub
//! let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
//!     .client_type(ClientType::Agent)
//!     .auth(AuthConfig::bearer("my-token"))
//!     .build()?;
//!
//! let client = CauceClient::connect(config).await?;
//!
//! // Subscribe to topics
//! let mut subscription = client.subscribe(&["signal.email.*"]).await?;
//!
//! // Receive signals
//! while let Some(signal) = subscription.next().await {
//!     println!("Received: {}", signal.id);
//!     client.ack(subscription.subscription_id(), &[&signal.id]).await?;
//! }
//!
//! // Disconnect
//! client.disconnect().await?;
//! ```

mod subscription;

pub use subscription::Subscription;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use cauce_core::{
    AckRequest, AckResponse, Auth, HelloRequest, HelloResponse, PublishMessage, PublishRequest,
    PublishResponse, SubscribeRequest, SubscribeResponse, SubscriptionStatus, UnsubscribeRequest,
    UnsubscribeResponse, METHOD_ACK, METHOD_GOODBYE, METHOD_HELLO, METHOD_PUBLISH,
    METHOD_SUBSCRIBE, METHOD_UNSUBSCRIBE,
};

use crate::config::{AuthConfig, ClientConfig};
use crate::error::ClientError;
use crate::router::{MessageRouter, RouterConfig};
use crate::transport::{Transport, WebSocketTransport};
use crate::ClientResult;

/// Internal subscription tracking information.
#[derive(Debug, Clone)]
struct SubscriptionInfo {
    /// The subscription ID.
    #[allow(dead_code)]
    id: String,
    /// Topics this subscription covers.
    #[allow(dead_code)]
    topics: Vec<String>,
    /// Current status of the subscription.
    #[allow(dead_code)]
    status: SubscriptionStatus,
}

/// High-level client for connecting to and interacting with a Cauce Hub.
///
/// `CauceClient` provides a convenient API for:
/// - Connecting to a Hub with automatic hello handshake
/// - Subscribing to topics and receiving signals
/// - Publishing signals and actions
/// - Acknowledging signal receipt
///
/// # Example
///
/// ```ignore
/// use cauce_client_sdk::{CauceClient, ClientConfig, AuthConfig};
/// use cauce_core::ClientType;
///
/// let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
///     .client_type(ClientType::Agent)
///     .auth(AuthConfig::bearer("token"))
///     .build()?;
///
/// let client = CauceClient::connect(config).await?;
/// println!("Connected with session: {}", client.session_id().unwrap());
///
/// client.disconnect().await?;
/// ```
pub struct CauceClient {
    /// The underlying message router.
    router: MessageRouter,

    /// Client configuration.
    config: ClientConfig,

    /// Session ID from hello response.
    session_id: Arc<RwLock<Option<String>>>,

    /// Server protocol version from hello response.
    server_version: Arc<RwLock<Option<String>>>,

    /// Active subscriptions: subscription_id -> SubscriptionInfo.
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionInfo>>>,
}

impl CauceClient {
    /// Connect to a Cauce Hub and perform the hello handshake.
    ///
    /// This method:
    /// 1. Validates the configuration
    /// 2. Creates and connects the transport
    /// 3. Sends the `cauce.hello` request
    /// 4. Validates the server's response
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration specifying hub URL, auth, etc.
    ///
    /// # Returns
    ///
    /// A connected `CauceClient` on success.
    ///
    /// # Errors
    ///
    /// - [`ClientError::ConfigError`] - Invalid configuration
    /// - [`ClientError::ConnectionFailed`] - Transport connection failed
    /// - [`ClientError::HandshakeFailed`] - Hello handshake failed
    /// - [`ClientError::VersionMismatch`] - Server version incompatible
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
    ///     .build()?;
    ///
    /// let client = CauceClient::connect(config).await?;
    /// ```
    pub async fn connect(config: ClientConfig) -> ClientResult<Self> {
        // Validate configuration
        config.validate()?;

        // Create transport
        let mut transport = WebSocketTransport::new(config.clone());

        // Connect transport
        transport
            .connect()
            .await
            .map_err(|e| ClientError::ConnectionFailed {
                message: e.to_string(),
            })?;

        // Create router config from client config
        let router_config = RouterConfig::default().with_request_timeout(config.request_timeout);

        // Create and start message router
        let mut router = MessageRouter::new(Box::new(transport), router_config);
        router.start().map_err(|e| ClientError::ConnectionFailed {
            message: format!("Failed to start message router: {}", e),
        })?;

        // Build hello request
        let hello_request = Self::build_hello_request(&config);

        // Send hello request
        let hello_params = serde_json::to_value(&hello_request).map_err(|e| {
            ClientError::HandshakeFailed {
                message: format!("Failed to serialize hello request: {}", e),
            }
        })?;

        let response = router
            .send_request(METHOD_HELLO, Some(hello_params))
            .await
            .map_err(|e| ClientError::HandshakeFailed {
                message: format!("Hello request failed: {}", e),
            })?;

        // Check for RPC error
        if let Some(error) = response.error_obj() {
            return Err(ClientError::HandshakeFailed {
                message: format!(
                    "Hub rejected hello: {} (code: {})",
                    &error.message,
                    error.code
                ),
            });
        }

        // Parse hello response
        let result = response.result().ok_or_else(|| ClientError::HandshakeFailed {
            message: "Hello response missing result".to_string(),
        })?;

        let hello_response: HelloResponse =
            serde_json::from_value(result.clone()).map_err(|e| ClientError::HandshakeFailed {
                message: format!("Failed to parse hello response: {}", e),
            })?;

        // Validate server version
        Self::validate_version(&config, &hello_response)?;

        tracing::info!(
            session_id = %hello_response.session_id,
            server_version = %hello_response.server_version,
            "Connected to Cauce Hub"
        );

        Ok(Self {
            router,
            config,
            session_id: Arc::new(RwLock::new(Some(hello_response.session_id))),
            server_version: Arc::new(RwLock::new(Some(hello_response.server_version))),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Disconnect gracefully from the Hub.
    ///
    /// Sends a `cauce.goodbye` notification and closes the transport.
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.disconnect().await?;
    /// ```
    pub async fn disconnect(&mut self) -> ClientResult<()> {
        tracing::info!("Disconnecting from Cauce Hub");

        // Send goodbye notification (fire-and-forget)
        let _ = self.router.send_notification(METHOD_GOODBYE, None).await;

        // Stop the router
        self.router.stop().await;

        // Clear session state
        *self.session_id.write().await = None;
        *self.server_version.write().await = None;
        self.subscriptions.write().await.clear();

        tracing::info!("Disconnected from Cauce Hub");
        Ok(())
    }

    /// Subscribe to the specified topics.
    ///
    /// Returns a [`Subscription`] handle that can be used to receive signals
    /// matching the subscribed topic patterns.
    ///
    /// # Arguments
    ///
    /// * `topics` - Topic patterns to subscribe to (supports wildcards)
    ///
    /// # Returns
    ///
    /// A [`Subscription`] handle on success.
    ///
    /// # Errors
    ///
    /// - [`ClientError::NotConnected`] - Not connected to hub
    /// - [`ClientError::RpcError`] - Hub rejected subscription
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Subscribe to email signals
    /// let mut subscription = client.subscribe(&["signal.email.*"]).await?;
    ///
    /// // Subscribe to multiple patterns
    /// let mut subscription = client
    ///     .subscribe(&["signal.email.*", "signal.slack.**"])
    ///     .await?;
    /// ```
    pub async fn subscribe(&self, topics: &[&str]) -> ClientResult<Subscription> {
        // Check connection
        if !self.is_connected().await {
            return Err(ClientError::NotConnected);
        }

        // Build subscribe request
        let request =
            SubscribeRequest::new(topics.iter().map(|t| t.to_string()).collect::<Vec<_>>());

        let params =
            serde_json::to_value(&request).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to serialize subscribe request: {}", e),
            })?;

        // Send request
        let response = self
            .router
            .send_request(METHOD_SUBSCRIBE, Some(params))
            .await?;

        // Check for RPC error
        if let Some(error) = response.error_obj() {
            return Err(ClientError::RpcError {
                code: error.code,
                message: error.message.to_string(),
                data: error.data.clone(),
            });
        }

        // Parse response
        let result = response.result().ok_or_else(|| ClientError::InvalidMessage {
            message: "Subscribe response missing result".to_string(),
        })?;

        let subscribe_response: SubscribeResponse =
            serde_json::from_value(result.clone()).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to parse subscribe response: {}", e),
            })?;

        // Check subscription status
        match subscribe_response.status {
            SubscriptionStatus::Active | SubscriptionStatus::Pending => {
                // OK - continue
            }
            SubscriptionStatus::Denied => {
                return Err(ClientError::RpcError {
                    code: -1,
                    message: "Subscription denied".to_string(),
                    data: None,
                });
            }
            status => {
                return Err(ClientError::RpcError {
                    code: -1,
                    message: format!("Unexpected subscription status: {:?}", status),
                    data: None,
                });
            }
        }

        // Store subscription info
        let info = SubscriptionInfo {
            id: subscribe_response.subscription_id.clone(),
            topics: subscribe_response.topics.clone(),
            status: subscribe_response.status,
        };
        self.subscriptions
            .write()
            .await
            .insert(subscribe_response.subscription_id.clone(), info);

        tracing::info!(
            subscription_id = %subscribe_response.subscription_id,
            topics = ?subscribe_response.topics,
            "Subscribed to topics"
        );

        // Create subscription handle
        Ok(Subscription::new(
            subscribe_response.subscription_id,
            subscribe_response.topics,
            self.router.subscribe_notifications(),
        ))
    }

    /// Unsubscribe from a subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID to unsubscribe from
    ///
    /// # Errors
    ///
    /// - [`ClientError::NotConnected`] - Not connected to hub
    /// - [`ClientError::SubscriptionNotFound`] - Unknown subscription ID
    /// - [`ClientError::RpcError`] - Hub rejected unsubscription
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.unsubscribe(subscription.subscription_id()).await?;
    /// ```
    pub async fn unsubscribe(&self, subscription_id: &str) -> ClientResult<()> {
        // Check connection
        if !self.is_connected().await {
            return Err(ClientError::NotConnected);
        }

        // Check subscription exists
        if !self.subscriptions.read().await.contains_key(subscription_id) {
            return Err(ClientError::SubscriptionNotFound {
                id: subscription_id.to_string(),
            });
        }

        // Build unsubscribe request
        let request = UnsubscribeRequest::new(subscription_id);
        let params =
            serde_json::to_value(&request).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to serialize unsubscribe request: {}", e),
            })?;

        // Send request
        let response = self
            .router
            .send_request(METHOD_UNSUBSCRIBE, Some(params))
            .await?;

        // Check for RPC error
        if let Some(error) = response.error_obj() {
            return Err(ClientError::RpcError {
                code: error.code,
                message: error.message.to_string(),
                data: error.data.clone(),
            });
        }

        // Parse response
        let result = response.result().ok_or_else(|| ClientError::InvalidMessage {
            message: "Unsubscribe response missing result".to_string(),
        })?;

        let unsubscribe_response: UnsubscribeResponse =
            serde_json::from_value(result.clone()).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to parse unsubscribe response: {}", e),
            })?;

        if !unsubscribe_response.success {
            return Err(ClientError::RpcError {
                code: -1,
                message: "Unsubscribe failed".to_string(),
                data: None,
            });
        }

        // Remove from tracking
        self.subscriptions.write().await.remove(subscription_id);

        tracing::info!(subscription_id = %subscription_id, "Unsubscribed");
        Ok(())
    }

    /// Publish a signal or action to a topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to publish to
    /// * `message` - The message to publish (Signal or Action)
    ///
    /// # Returns
    ///
    /// A [`PublishResponse`] containing delivery information.
    ///
    /// # Errors
    ///
    /// - [`ClientError::NotConnected`] - Not connected to hub
    /// - [`ClientError::RpcError`] - Hub rejected publish
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.publish("signal.email.received", signal.into()).await?;
    /// println!("Delivered to {} subscribers", response.delivered_to);
    /// ```
    pub async fn publish(
        &self,
        topic: &str,
        message: PublishMessage,
    ) -> ClientResult<PublishResponse> {
        // Check connection
        if !self.is_connected().await {
            return Err(ClientError::NotConnected);
        }

        // Build publish request
        let request = PublishRequest {
            topic: topic.to_string(),
            message,
        };

        let params =
            serde_json::to_value(&request).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to serialize publish request: {}", e),
            })?;

        // Send request
        let response = self
            .router
            .send_request(METHOD_PUBLISH, Some(params))
            .await?;

        // Check for RPC error
        if let Some(error) = response.error_obj() {
            return Err(ClientError::RpcError {
                code: error.code,
                message: error.message.to_string(),
                data: error.data.clone(),
            });
        }

        // Parse response
        let result = response.result().ok_or_else(|| ClientError::InvalidMessage {
            message: "Publish response missing result".to_string(),
        })?;

        let publish_response: PublishResponse =
            serde_json::from_value(result.clone()).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to parse publish response: {}", e),
            })?;

        tracing::debug!(
            topic = %topic,
            message_id = %publish_response.message_id,
            delivered_to = publish_response.delivered_to,
            "Published message"
        );

        Ok(publish_response)
    }

    /// Acknowledge receipt of signals.
    ///
    /// Acknowledging signals informs the hub that you have successfully
    /// processed them. This is important for reliable delivery guarantees.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription that received the signals
    /// * `signal_ids` - IDs of signals to acknowledge
    ///
    /// # Returns
    ///
    /// An [`AckResponse`] containing acknowledgment results.
    ///
    /// # Errors
    ///
    /// - [`ClientError::NotConnected`] - Not connected to hub
    /// - [`ClientError::SubscriptionNotFound`] - Unknown subscription ID
    /// - [`ClientError::RpcError`] - Hub rejected acknowledgment
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Acknowledge a single signal
    /// client.ack(subscription.subscription_id(), &[&signal.id]).await?;
    ///
    /// // Acknowledge multiple signals
    /// client.ack(subscription.subscription_id(), &["sig_1", "sig_2"]).await?;
    /// ```
    pub async fn ack(
        &self,
        subscription_id: &str,
        signal_ids: &[&str],
    ) -> ClientResult<AckResponse> {
        // Check connection
        if !self.is_connected().await {
            return Err(ClientError::NotConnected);
        }

        // Check subscription exists
        if !self.subscriptions.read().await.contains_key(subscription_id) {
            return Err(ClientError::SubscriptionNotFound {
                id: subscription_id.to_string(),
            });
        }

        // Build ack request
        let request = AckRequest::new(
            subscription_id,
            signal_ids.iter().map(|s| s.to_string()).collect(),
        );

        let params = serde_json::to_value(&request).map_err(|e| ClientError::InvalidMessage {
            message: format!("Failed to serialize ack request: {}", e),
        })?;

        // Send request
        let response = self.router.send_request(METHOD_ACK, Some(params)).await?;

        // Check for RPC error
        if let Some(error) = response.error_obj() {
            return Err(ClientError::RpcError {
                code: error.code,
                message: error.message.to_string(),
                data: error.data.clone(),
            });
        }

        // Parse response
        let result = response.result().ok_or_else(|| ClientError::InvalidMessage {
            message: "Ack response missing result".to_string(),
        })?;

        let ack_response: AckResponse =
            serde_json::from_value(result.clone()).map_err(|e| ClientError::InvalidMessage {
                message: format!("Failed to parse ack response: {}", e),
            })?;

        tracing::debug!(
            subscription_id = %subscription_id,
            acknowledged = ack_response.acknowledged.len(),
            "Acknowledged signals"
        );

        Ok(ack_response)
    }

    /// Returns the session ID if connected.
    ///
    /// The session ID is assigned by the hub during the hello handshake.
    ///
    /// # Returns
    ///
    /// `Some(session_id)` if connected, `None` otherwise.
    pub async fn session_id(&self) -> Option<String> {
        self.session_id.read().await.clone()
    }

    /// Returns the server's protocol version if connected.
    ///
    /// # Returns
    ///
    /// `Some(version)` if connected, `None` otherwise.
    pub async fn server_version(&self) -> Option<String> {
        self.server_version.read().await.clone()
    }

    /// Checks if the client is currently connected.
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise.
    pub async fn is_connected(&self) -> bool {
        self.session_id.read().await.is_some() && self.router.connection_state().await.is_connected()
    }

    /// Returns a list of active subscription IDs.
    pub async fn active_subscriptions(&self) -> Vec<String> {
        self.subscriptions
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Returns the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    // =========================================================================
    // Private helpers
    // =========================================================================

    /// Build the hello request from client config.
    fn build_hello_request(config: &ClientConfig) -> HelloRequest {
        let mut request = HelloRequest::new(
            &config.protocol_version,
            &config.client_id,
            config.client_type,
        );

        // Add auth if configured
        if let Some(auth_config) = &config.auth {
            let auth = match auth_config {
                AuthConfig::ApiKey { key } => Auth::api_key(key),
                AuthConfig::Bearer { token } => Auth::bearer(token),
            };
            request = request.with_auth(auth);
        }

        // Add min protocol version if configured
        if config.min_protocol_version != config.protocol_version {
            request.min_protocol_version = Some(config.min_protocol_version.clone());
        }

        request
    }

    /// Validate server version compatibility.
    fn validate_version(config: &ClientConfig, response: &HelloResponse) -> ClientResult<()> {
        // Simple version comparison - server version should be >= min_protocol_version
        // For now, we just check they're in the same major version
        let server_major = response
            .server_version
            .split('.')
            .next()
            .unwrap_or("0");
        let min_major = config.min_protocol_version.split('.').next().unwrap_or("0");

        if server_major < min_major {
            return Err(ClientError::VersionMismatch {
                client_version: config.min_protocol_version.clone(),
                server_version: response.server_version.clone(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;
    use cauce_core::{ClientType, JsonRpcResponse, RequestId};

    fn make_config() -> ClientConfig {
        ClientConfig::builder("ws://localhost:8080", "test-client")
            .client_type(ClientType::Agent)
            .build()
            .unwrap()
    }

    fn make_hello_response(session_id: &str) -> serde_json::Value {
        serde_json::json!({
            "session_id": session_id,
            "server_version": "1.0",
            "capabilities": []
        })
    }

    fn make_subscribe_response(subscription_id: &str, topics: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "subscription_id": subscription_id,
            "status": "active",
            "topics": topics,
            "created_at": "2024-01-01T00:00:00Z"
        })
    }

    fn make_unsubscribe_response(success: bool) -> serde_json::Value {
        serde_json::json!({
            "success": success
        })
    }

    fn make_publish_response(message_id: &str) -> serde_json::Value {
        serde_json::json!({
            "message_id": message_id,
            "delivered_to": 5,
            "queued_for": 2
        })
    }

    fn make_ack_response(acknowledged: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "acknowledged": acknowledged
        })
    }

    #[test]
    fn test_build_hello_request_minimal() {
        let config = make_config();
        let request = CauceClient::build_hello_request(&config);

        assert_eq!(request.protocol_version, "1.0");
        assert_eq!(request.client_id, "test-client");
        assert_eq!(request.client_type, ClientType::Agent);
        assert!(request.auth.is_none());
    }

    #[test]
    fn test_build_hello_request_with_auth() {
        let config = ClientConfig::builder("ws://localhost:8080", "test-client")
            .auth(AuthConfig::bearer("my-token"))
            .build()
            .unwrap();

        let request = CauceClient::build_hello_request(&config);
        assert!(request.auth.is_some());
    }

    #[test]
    fn test_validate_version_compatible() {
        let config = make_config();
        let response = HelloResponse::new("sess_123", "1.0");
        assert!(CauceClient::validate_version(&config, &response).is_ok());
    }

    #[test]
    fn test_validate_version_newer_server() {
        let config = make_config();
        let response = HelloResponse::new("sess_123", "2.0");
        assert!(CauceClient::validate_version(&config, &response).is_ok());
    }

    #[test]
    fn test_validate_version_incompatible() {
        let mut config = make_config();
        config.min_protocol_version = "2.0".to_string();
        let response = HelloResponse::new("sess_123", "1.0");
        assert!(CauceClient::validate_version(&config, &response).is_err());
    }

    // Integration-style tests using MockTransport

    #[tokio::test]
    async fn test_connect_with_mock_transport() {
        // Create mock transport
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        // Queue hello response
        let response = JsonRpcResponse::success(RequestId::Number(1), make_hello_response("sess_test"));
        transport.push_receive(response.into());

        // Create router
        let config = make_config();
        let router_config = RouterConfig::default();
        let mut router = MessageRouter::new(Box::new(transport), router_config);
        router.start().unwrap();

        // Build and send hello
        let hello_request = CauceClient::build_hello_request(&config);
        let params = serde_json::to_value(&hello_request).unwrap();
        let response = router.send_request(METHOD_HELLO, Some(params)).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.is_success());

        let result = response.result().unwrap();
        let hello_response: HelloResponse = serde_json::from_value(result.clone()).unwrap();
        assert_eq!(hello_response.session_id, "sess_test");
    }

    #[tokio::test]
    async fn test_subscribe_flow() {
        // Create connected client state manually for testing
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        // Queue subscribe response
        let response = JsonRpcResponse::success(
            RequestId::Number(1),
            make_subscribe_response("sub_123", &["signal.email.*"]),
        );
        transport.push_receive(response.into());

        let router_config = RouterConfig::default();
        let mut router = MessageRouter::new(Box::new(transport), router_config);
        router.start().unwrap();

        // Build and send subscribe request
        let request = SubscribeRequest::new(vec!["signal.email.*".to_string()]);
        let params = serde_json::to_value(&request).unwrap();
        let response = router.send_request(METHOD_SUBSCRIBE, Some(params)).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.is_success());

        let result = response.result().unwrap();
        let sub_response: SubscribeResponse = serde_json::from_value(result.clone()).unwrap();
        assert_eq!(sub_response.subscription_id, "sub_123");
        assert_eq!(sub_response.topics, vec!["signal.email.*"]);
    }

    #[tokio::test]
    async fn test_unsubscribe_flow() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        // Queue unsubscribe response
        let response = JsonRpcResponse::success(RequestId::Number(1), make_unsubscribe_response(true));
        transport.push_receive(response.into());

        let router_config = RouterConfig::default();
        let mut router = MessageRouter::new(Box::new(transport), router_config);
        router.start().unwrap();

        let request = UnsubscribeRequest::new("sub_123");
        let params = serde_json::to_value(&request).unwrap();
        let response = router.send_request(METHOD_UNSUBSCRIBE, Some(params)).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_publish_flow() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        // Queue publish response
        let response = JsonRpcResponse::success(RequestId::Number(1), make_publish_response("msg_001"));
        transport.push_receive(response.into());

        let router_config = RouterConfig::default();
        let mut router = MessageRouter::new(Box::new(transport), router_config);
        router.start().unwrap();

        let request = PublishRequest {
            topic: "signal.email.received".to_string(),
            message: PublishMessage::Signal(cauce_core::Signal {
                id: "sig_001".to_string(),
                version: "1.0".to_string(),
                timestamp: chrono::Utc::now(),
                source: cauce_core::Source::new("email", "adapter-1", "native-1"),
                topic: cauce_core::Topic::new_unchecked("signal.email.received"),
                payload: cauce_core::Payload::new(serde_json::json!({}), "application/json"),
                metadata: None,
                encrypted: None,
            }),
        };
        let params = serde_json::to_value(&request).unwrap();
        let response = router.send_request(METHOD_PUBLISH, Some(params)).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_ack_flow() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();

        // Queue ack response
        let response = JsonRpcResponse::success(
            RequestId::Number(1),
            make_ack_response(&["sig_001", "sig_002"]),
        );
        transport.push_receive(response.into());

        let router_config = RouterConfig::default();
        let mut router = MessageRouter::new(Box::new(transport), router_config);
        router.start().unwrap();

        let request = AckRequest::new("sub_123", vec!["sig_001".to_string(), "sig_002".to_string()]);
        let params = serde_json::to_value(&request).unwrap();
        let response = router.send_request(METHOD_ACK, Some(params)).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.is_success());

        let result = response.result().unwrap();
        let ack_response: AckResponse = serde_json::from_value(result.clone()).unwrap();
        assert_eq!(ack_response.acknowledged.len(), 2);
    }
}

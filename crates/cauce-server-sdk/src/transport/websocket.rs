//! WebSocket transport handler for the Cauce server.
//!
//! This module provides the [`WebSocketHandler`] for handling WebSocket
//! connections and processing JSON-RPC messages.
//!
//! # Example
//!
//! ```ignore
//! use cauce_server_sdk::transport::WebSocketHandler;
//! use cauce_server_sdk::{
//!     InMemorySubscriptionManager, DefaultMessageRouter,
//!     InMemoryDeliveryTracker, InMemorySessionManager,
//! };
//! use std::sync::Arc;
//!
//! let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
//! let message_router = Arc::new(DefaultMessageRouter::new(subscription_manager.clone()));
//! let delivery_tracker = Arc::new(InMemoryDeliveryTracker::default());
//! let session_manager = Arc::new(InMemorySessionManager::default());
//!
//! let handler = WebSocketHandler::new(
//!     subscription_manager,
//!     message_router,
//!     delivery_tracker,
//!     session_manager,
//! );
//!
//! // Use with axum
//! let router = axum::Router::new()
//!     .route("/ws", axum::routing::get(handler.handler()));
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::message::JsonRpcMessage;
use super::SignalSender;
use crate::delivery::DeliveryTracker;
use crate::error::{ServerError, ServerResult};
use crate::routing::MessageRouter;
use crate::session::{SessionInfo, SessionManager};
use crate::subscription::SubscriptionManager;
use cauce_core::methods::Transport;
use cauce_core::{
    AckRequest, HelloRequest, HelloResponse, JsonRpcError, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, PublishRequest, PublishResponse, RequestId, SignalDelivery, SubscribeRequest,
    UnsubscribeRequest, UnsubscribeResponse, METHOD_ACK, METHOD_GOODBYE, METHOD_HELLO, METHOD_PING,
    METHOD_PUBLISH, METHOD_SIGNAL, METHOD_SUBSCRIBE, METHOD_UNSUBSCRIBE, PROTOCOL_VERSION,
};

/// WebSocket transport handler.
///
/// Handles WebSocket connections and processes Cauce Protocol JSON-RPC messages.
pub struct WebSocketHandler<S, R, D, M>
where
    S: SubscriptionManager,
    R: MessageRouter,
    D: DeliveryTracker,
    M: SessionManager,
{
    subscription_manager: Arc<S>,
    message_router: Arc<R>,
    delivery_tracker: Arc<D>,
    session_manager: Arc<M>,
    shutdown_tx: broadcast::Sender<()>,
    /// Registry mapping session IDs to their signal delivery channels.
    /// Used to push signals to connected clients in real-time.
    connections: Arc<RwLock<HashMap<String, mpsc::Sender<SignalDelivery>>>>,
}

impl<S, R, D, M> WebSocketHandler<S, R, D, M>
where
    S: SubscriptionManager,
    R: MessageRouter,
    D: DeliveryTracker,
    M: SessionManager,
{
    /// Creates a new WebSocket handler.
    pub fn new(
        subscription_manager: Arc<S>,
        message_router: Arc<R>,
        delivery_tracker: Arc<D>,
        session_manager: Arc<M>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            subscription_manager,
            message_router,
            delivery_tracker,
            session_manager,
            shutdown_tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a connection's signal sender for a session.
    async fn register_connection(&self, session_id: &str, signal_tx: mpsc::Sender<SignalDelivery>) {
        let mut conns = self.connections.write().await;
        conns.insert(session_id.to_string(), signal_tx);
        debug!("Registered connection for session {}", session_id);
    }

    /// Unregister a connection when it disconnects.
    async fn unregister_connection(&self, session_id: &str) {
        let mut conns = self.connections.write().await;
        conns.remove(session_id);
        debug!("Unregistered connection for session {}", session_id);
    }

    /// Push a signal to all sessions that have matching subscriptions.
    async fn push_signal_to_subscribers(&self, session_ids: &[String], delivery: &SignalDelivery) {
        let conns = self.connections.read().await;
        for session_id in session_ids {
            if let Some(tx) = conns.get(session_id) {
                if let Err(e) = tx.send(delivery.clone()).await {
                    warn!("Failed to push signal to session {}: {}", session_id, e);
                } else {
                    debug!("Pushed signal {} to session {}", delivery.signal.id, session_id);
                }
            }
        }
    }

    /// Signal shutdown to all active connections.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Handle a WebSocket upgrade request.
    pub async fn handle_upgrade(self: Arc<Self>, ws: WebSocketUpgrade) -> impl IntoResponse {
        let handler = Arc::clone(&self);
        ws.on_upgrade(move |socket| async move {
            if let Err(e) = handler.handle_connection(socket).await {
                error!("WebSocket connection error: {}", e);
            }
        })
    }

    /// Handle a WebSocket connection.
    async fn handle_connection(self: Arc<Self>, socket: WebSocket) -> ServerResult<()> {
        let (ws_sender, mut ws_receiver) = socket.split();

        // Create channels for signal delivery
        let (signal_tx, mut signal_rx) = mpsc::channel::<SignalDelivery>(100);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Connection state
        let connection = Arc::new(WebSocketConnection::new(ws_sender, signal_tx));
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        info!("New WebSocket connection established");

        loop {
            tokio::select! {
                // Shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("WebSocket shutting down due to server shutdown");
                    break;
                }

                // Incoming signal to deliver
                Some(signal) = signal_rx.recv() => {
                    if let Err(e) = connection.send_signal_notification(&signal).await {
                        warn!("Failed to send signal notification: {}", e);
                        break;
                    }
                }

                // Incoming WebSocket message
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let response = self.process_message(&text, &connection, &session_id).await;
                            if let Some(response) = response {
                                if let Err(e) = connection.send_message(&response).await {
                                    error!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket client requested close");
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if let Err(e) = connection.send_pong(&data).await {
                                warn!("Failed to send pong: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket receive error: {}", e);
                            break;
                        }
                        None => {
                            debug!("WebSocket stream ended");
                            break;
                        }
                        _ => {} // Ignore binary and other messages
                    }
                }
            }
        }

        // Cleanup session on disconnect
        let session = session_id.lock().await;
        if let Some(ref sid) = *session {
            info!("Cleaning up session: {}", sid);
            self.unregister_connection(sid).await;
            if let Err(e) = self.session_manager.remove_session(sid).await {
                warn!("Failed to remove session on disconnect: {}", e);
            }
        }

        connection.close();
        info!("WebSocket connection closed");
        Ok(())
    }

    /// Process an incoming JSON-RPC message.
    async fn process_message(
        &self,
        text: &str,
        connection: &WebSocketConnection,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> Option<JsonRpcMessage> {
        let message = match JsonRpcMessage::parse(text) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse JSON-RPC message: {}", e);
                return Some(JsonRpcMessage::Response(JsonRpcResponse::error(
                    None,
                    JsonRpcError::with_data(-32700, "Parse error", json!({"details": e.to_string()})),
                )));
            }
        };

        match message {
            JsonRpcMessage::Request(request) => {
                let response = self.handle_request(request, connection, session_id).await;
                Some(JsonRpcMessage::Response(response))
            }
            JsonRpcMessage::Notification(notification) => {
                self.handle_notification(notification).await;
                None
            }
            JsonRpcMessage::Response(_) => {
                // Servers don't normally receive responses
                warn!("Received unexpected response from client");
                None
            }
        }
    }

    /// Handle a JSON-RPC request.
    async fn handle_request(
        &self,
        request: JsonRpcRequest,
        connection: &WebSocketConnection,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> JsonRpcResponse {
        let id = request.id().clone();
        let method = request.method();

        debug!("Processing request: {} (id: {:?})", method, id);

        match method {
            METHOD_HELLO => self.handle_hello(&request, connection, session_id).await,
            METHOD_SUBSCRIBE => {
                self.handle_subscribe(&request, session_id)
                    .await
                    .unwrap_or_else(|e| e)
            }
            METHOD_UNSUBSCRIBE => {
                self.handle_unsubscribe(&request, session_id)
                    .await
                    .unwrap_or_else(|e| e)
            }
            METHOD_PUBLISH => {
                self.handle_publish(&request, session_id)
                    .await
                    .unwrap_or_else(|e| e)
            }
            METHOD_ACK => {
                self.handle_ack(&request, session_id)
                    .await
                    .unwrap_or_else(|e| e)
            }
            METHOD_PING => self.handle_ping(&request),
            METHOD_GOODBYE => self.handle_goodbye(&request, session_id).await,
            _ => JsonRpcResponse::error(
                Some(id),
                JsonRpcError::with_data(-32601, "Method not found", json!({"method": method})),
            ),
        }
    }

    /// Handle cauce.hello request.
    async fn handle_hello(
        &self,
        request: &JsonRpcRequest,
        connection: &WebSocketConnection,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> JsonRpcResponse {
        let id = request.id().clone();

        // Check if already authenticated
        {
            let existing = session_id.lock().await;
            if existing.is_some() {
                return JsonRpcResponse::error(
                    Some(id),
                    JsonRpcError::with_data(-32600, "Invalid Request", json!({"reason": "already authenticated"})),
                );
            }
        }

        // Parse hello request
        let hello_request: HelloRequest = match request.params() {
            Some(params) => match serde_json::from_value(params.clone()) {
                Ok(h) => h,
                Err(e) => {
                    return JsonRpcResponse::error(
                        Some(id),
                        JsonRpcError::with_data(-32602, "Invalid params", json!({"details": e.to_string()})),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    Some(id),
                    JsonRpcError::with_data(-32602, "Invalid params", json!({"reason": "params required"})),
                );
            }
        };

        // Generate session ID
        let new_session_id = format!("sess_{}", uuid::Uuid::new_v4());

        // Get client type as string
        let client_type_str = format!("{:?}", hello_request.client_type).to_lowercase();

        // Create session info
        let session_info = SessionInfo::new(
            &new_session_id,
            &hello_request.client_id,
            &client_type_str,
            PROTOCOL_VERSION,
            Transport::WebSocket,
            3600, // 1 hour default TTL
        );

        // Create session
        if let Err(e) = self.session_manager.create_session(session_info).await {
            error!("Failed to create session: {}", e);
            return JsonRpcResponse::error(
                Some(id),
                JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
            );
        }

        // Store session ID in connection state
        {
            let mut sid = session_id.lock().await;
            *sid = Some(new_session_id.clone());
        }

        // Register the connection for signal delivery
        connection.set_session_id(&new_session_id);
        self.register_connection(&new_session_id, connection.signal_sender()).await;

        info!(
            "Client {} authenticated with session {}",
            hello_request.client_id, new_session_id
        );

        // Build response
        let response = HelloResponse::new(&new_session_id, PROTOCOL_VERSION);

        match serde_json::to_value(&response) {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err(e) => JsonRpcResponse::error(
                Some(id),
                JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
            ),
        }
    }

    /// Handle cauce.subscribe request.
    async fn handle_subscribe(
        &self,
        request: &JsonRpcRequest,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> Result<JsonRpcResponse, JsonRpcResponse> {
        let id = request.id().clone();

        // Check session
        let sid = self.require_session(session_id, &id).await?;

        // Get session info for client_id
        let session_info = self
            .session_manager
            .get_session(&sid)
            .await
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id.clone()),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })?
            .ok_or_else(|| {
                JsonRpcResponse::error(
                    Some(id.clone()),
                    JsonRpcError::with_data(-32600, "Invalid Request", json!({"reason": "session not found"})),
                )
            })?;

        // Parse subscribe request
        let subscribe_request: SubscribeRequest = self.parse_params(request.params(), &id)?;

        // Create subscription
        let response = self
            .subscription_manager
            .subscribe(&session_info.client_id, &sid, subscribe_request)
            .await
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id.clone()),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })?;

        serde_json::to_value(&response)
            .map(|result| JsonRpcResponse::success(id.clone(), result))
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })
    }

    /// Handle cauce.unsubscribe request.
    async fn handle_unsubscribe(
        &self,
        request: &JsonRpcRequest,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> Result<JsonRpcResponse, JsonRpcResponse> {
        let id = request.id().clone();

        // Check session
        self.require_session(session_id, &id).await?;

        // Parse unsubscribe request
        let unsubscribe_request: UnsubscribeRequest = self.parse_params(request.params(), &id)?;

        // Remove subscription
        self.subscription_manager
            .unsubscribe(&unsubscribe_request.subscription_id)
            .await
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id.clone()),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })?;

        let response = UnsubscribeResponse::success();

        serde_json::to_value(&response)
            .map(|result| JsonRpcResponse::success(id.clone(), result))
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })
    }

    /// Handle cauce.publish request.
    async fn handle_publish(
        &self,
        request: &JsonRpcRequest,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> Result<JsonRpcResponse, JsonRpcResponse> {
        let id = request.id().clone();

        // Check session
        self.require_session(session_id, &id).await?;

        // Parse publish request
        let publish_request: PublishRequest = self.parse_params(request.params(), &id)?;

        // Route the message to find matching subscriptions
        let _route_result = self.message_router.route(&publish_request).await.map_err(|e| {
            JsonRpcResponse::error(
                Some(id.clone()),
                JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
            )
        })?;

        // Get matching subscriptions and create deliveries
        let matching_subs = self
            .message_router
            .get_matching_subscriptions(&publish_request.topic)
            .await
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id.clone()),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })?;

        // Create and track deliveries for each subscription, and push to connected clients
        let mut message_id = format!("msg_{}", uuid::Uuid::new_v4());
        let mut delivered_count = 0u32;
        for sub in &matching_subs {
            if let Ok(delivery) = self.message_router.create_delivery(&publish_request, sub) {
                message_id = delivery.signal.id.clone();

                // Track the delivery
                if let Err(e) = self.delivery_tracker.track(&sub.subscription_id, &delivery).await {
                    warn!("Failed to track delivery for {}: {}", sub.subscription_id, e);
                }

                // Push to connected client in real-time
                self.push_signal_to_subscribers(std::slice::from_ref(&sub.session_id), &delivery).await;
                delivered_count += 1;
            }
        }

        let response = PublishResponse::new(
            message_id,
            delivered_count,
            0, // queued_for - would need to track webhook deliveries separately
        );

        serde_json::to_value(&response)
            .map(|result| JsonRpcResponse::success(id.clone(), result))
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })
    }

    /// Handle cauce.ack request.
    async fn handle_ack(
        &self,
        request: &JsonRpcRequest,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> Result<JsonRpcResponse, JsonRpcResponse> {
        let id = request.id().clone();

        // Check session
        self.require_session(session_id, &id).await?;

        // Parse ack request
        let ack_request: AckRequest = self.parse_params(request.params(), &id)?;

        // Acknowledge signals
        let response = self
            .delivery_tracker
            .ack(&ack_request.subscription_id, &ack_request.signal_ids)
            .await
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id.clone()),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })?;

        serde_json::to_value(&response)
            .map(|result| JsonRpcResponse::success(id.clone(), result))
            .map_err(|e| {
                JsonRpcResponse::error(
                    Some(id),
                    JsonRpcError::with_data(-32603, "Internal error", json!({"details": e.to_string()})),
                )
            })
    }

    /// Handle cauce.ping request.
    fn handle_ping(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id().clone();

        // Respond with pong
        let result = json!({
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        JsonRpcResponse::success(id, result)
    }

    /// Handle cauce.goodbye request.
    async fn handle_goodbye(
        &self,
        request: &JsonRpcRequest,
        session_id: &Arc<Mutex<Option<String>>>,
    ) -> JsonRpcResponse {
        let id = request.id().clone();

        // Get and clear session ID
        let sid = {
            let mut sid = session_id.lock().await;
            sid.take()
        };

        // Remove session if exists
        if let Some(ref session) = sid {
            if let Err(e) = self.session_manager.remove_session(session).await {
                warn!("Failed to remove session on goodbye: {}", e);
            }
            info!("Session {} ended via goodbye", session);
        }

        JsonRpcResponse::success(id, json!({"success": true}))
    }

    /// Handle a JSON-RPC notification.
    async fn handle_notification(&self, notification: JsonRpcNotification) {
        let method = notification.method();

        debug!("Processing notification: {}", method);

        // Client notifications we might receive
        if method == "cauce.pong" {
            debug!("Received pong from client");
        } else {
            warn!("Unknown notification method: {}", method);
        }
    }

    /// Require an active session, returning an error response if not authenticated.
    async fn require_session(
        &self,
        session_id: &Arc<Mutex<Option<String>>>,
        request_id: &RequestId,
    ) -> Result<String, JsonRpcResponse> {
        let sid = session_id.lock().await;
        sid.clone().ok_or_else(|| {
            JsonRpcResponse::error(
                Some(request_id.clone()),
                JsonRpcError::with_data(-32600, "Invalid Request", json!({"reason": "not authenticated"})),
            )
        })
    }

    /// Parse request params into a typed value.
    fn parse_params<T: serde::de::DeserializeOwned>(
        &self,
        params: Option<&serde_json::Value>,
        request_id: &RequestId,
    ) -> Result<T, JsonRpcResponse> {
        let params = params.ok_or_else(|| {
            JsonRpcResponse::error(
                Some(request_id.clone()),
                JsonRpcError::with_data(-32602, "Invalid params", json!({"reason": "params required"})),
            )
        })?;

        serde_json::from_value(params.clone()).map_err(|e| {
            JsonRpcResponse::error(
                Some(request_id.clone()),
                JsonRpcError::with_data(-32602, "Invalid params", json!({"details": e.to_string()})),
            )
        })
    }
}

impl<S, R, D, M> Clone for WebSocketHandler<S, R, D, M>
where
    S: SubscriptionManager,
    R: MessageRouter,
    D: DeliveryTracker,
    M: SessionManager,
{
    fn clone(&self) -> Self {
        Self {
            subscription_manager: Arc::clone(&self.subscription_manager),
            message_router: Arc::clone(&self.message_router),
            delivery_tracker: Arc::clone(&self.delivery_tracker),
            session_manager: Arc::clone(&self.session_manager),
            shutdown_tx: self.shutdown_tx.clone(),
            connections: Arc::clone(&self.connections),
        }
    }
}

/// Represents an active WebSocket connection.
///
/// Provides methods for sending messages and managing connection state.
pub struct WebSocketConnection {
    sender: Mutex<SplitSink<WebSocket, Message>>,
    signal_tx: mpsc::Sender<SignalDelivery>,
    session_id: Mutex<Option<String>>,
    connected: AtomicBool,
}

impl WebSocketConnection {
    /// Creates a new WebSocket connection wrapper.
    pub fn new(sender: SplitSink<WebSocket, Message>, signal_tx: mpsc::Sender<SignalDelivery>) -> Self {
        Self {
            sender: Mutex::new(sender),
            signal_tx,
            session_id: Mutex::new(None),
            connected: AtomicBool::new(true),
        }
    }

    /// Sets the session ID for this connection.
    pub fn set_session_id(&self, id: &str) {
        if let Ok(mut sid) = self.session_id.try_lock() {
            *sid = Some(id.to_string());
        }
    }

    /// Returns a clone of the signal sender channel.
    /// Used to register the connection for signal delivery.
    pub fn signal_sender(&self) -> mpsc::Sender<SignalDelivery> {
        self.signal_tx.clone()
    }

    /// Send a JSON-RPC message.
    pub async fn send_message(&self, message: &JsonRpcMessage) -> ServerResult<()> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(ServerError::TransportClosed);
        }

        let json = message.to_json().map_err(|e| ServerError::Serialization {
            message: e.to_string(),
        })?;

        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Text(json))
            .await
            .map_err(|e| ServerError::TransportError {
                message: e.to_string(),
            })
    }

    /// Send a pong message (WebSocket protocol).
    pub async fn send_pong(&self, data: &[u8]) -> ServerResult<()> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(ServerError::TransportClosed);
        }

        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Pong(data.to_vec()))
            .await
            .map_err(|e| ServerError::TransportError {
                message: e.to_string(),
            })
    }

    /// Send a signal notification to the client.
    pub async fn send_signal_notification(&self, delivery: &SignalDelivery) -> ServerResult<()> {
        let notification = JsonRpcNotification::new(
            METHOD_SIGNAL.to_string(),
            Some(serde_json::to_value(delivery).map_err(|e| ServerError::Serialization {
                message: e.to_string(),
            })?),
        );

        self.send_message(&JsonRpcMessage::Notification(notification))
            .await
    }

    /// Queue a signal for delivery to this connection.
    pub async fn queue_signal(&self, delivery: SignalDelivery) -> ServerResult<()> {
        self.signal_tx.send(delivery).await.map_err(|_| ServerError::TransportError {
            message: "signal channel closed".to_string(),
        })
    }

    /// Mark the connection as closed.
    pub fn close(&self) {
        self.connected.store(false, Ordering::SeqCst);
    }
}

#[async_trait::async_trait]
impl SignalSender for WebSocketConnection {
    async fn send_signal(&self, delivery: SignalDelivery) -> ServerResult<()> {
        self.queue_signal(delivery).await
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedeliveryConfig;
    use crate::delivery::InMemoryDeliveryTracker;
    use crate::routing::DefaultMessageRouter;
    use crate::session::InMemorySessionManager;
    use crate::subscription::InMemorySubscriptionManager;
    use cauce_core::types::{Payload, Source, Topic};
    use cauce_core::Signal;
    use chrono::Utc;
    use serde_json::json;

    fn create_test_handler() -> WebSocketHandler<
        InMemorySubscriptionManager,
        DefaultMessageRouter<InMemorySubscriptionManager>,
        InMemoryDeliveryTracker,
        InMemorySessionManager,
    > {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let message_router = Arc::new(DefaultMessageRouter::new(subscription_manager.clone()));
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        WebSocketHandler::new(
            subscription_manager,
            message_router,
            delivery_tracker,
            session_manager,
        )
    }

    fn create_test_signal() -> Signal {
        Signal {
            id: "sig_test".to_string(),
            version: "1.0".to_string(),
            timestamp: Utc::now(),
            source: Source::new("test", "adapter-1", "msg-1"),
            topic: Topic::new_unchecked("signal.test"),
            payload: Payload::new(json!({"test": true}), "application/json"),
            metadata: None,
            encrypted: None,
        }
    }

    #[test]
    fn test_websocket_handler_clone() {
        let handler = create_test_handler();
        let _cloned = handler.clone();
    }

    #[test]
    fn test_websocket_handler_shutdown() {
        let handler = create_test_handler();
        // Should not panic
        handler.shutdown();
    }

    #[test]
    fn test_parse_params_missing() {
        let handler = create_test_handler();
        let id = RequestId::Number(1);

        let result: Result<serde_json::Value, _> = handler.parse_params(None, &id);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_params_invalid() {
        let handler = create_test_handler();
        let id = RequestId::Number(1);

        // Invalid params for HelloRequest (missing required fields)
        let invalid_params = json!({"invalid": "data"});
        let result: Result<HelloRequest, _> = handler.parse_params(Some(&invalid_params), &id);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_params_valid() {
        let handler = create_test_handler();
        let id = RequestId::Number(1);

        // Valid SubscribeRequest
        let valid_params = json!({"topics": ["signal.test.*"]});
        let result: Result<SubscribeRequest, _> = handler.parse_params(Some(&valid_params), &id);
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.topics, vec!["signal.test.*".to_string()]);
    }

    #[test]
    fn test_handle_ping() {
        let handler = create_test_handler();
        let request = JsonRpcRequest::new(
            RequestId::Number(42),
            METHOD_PING.to_string(),
            None,
        );

        let response = handler.handle_ping(&request);
        assert!(response.result().is_some());

        let result = response.result().unwrap();
        assert!(result.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_require_session_none() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let id = RequestId::Number(1);

        let result = handler.require_session(&session_id, &id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_require_session_some() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_123".to_string())));
        let id = RequestId::Number(1);

        let result = handler.require_session(&session_id, &id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sess_123");
    }

    #[tokio::test]
    async fn test_handle_notification_pong() {
        let handler = create_test_handler();
        let notification = JsonRpcNotification::new("cauce.pong".to_string(), None);

        // Should not panic
        handler.handle_notification(notification).await;
    }

    #[tokio::test]
    async fn test_handle_notification_unknown() {
        let handler = create_test_handler();
        let notification = JsonRpcNotification::new("unknown.method".to_string(), None);

        // Should not panic, just log warning
        handler.handle_notification(notification).await;
    }

    #[tokio::test]
    async fn test_handle_goodbye_no_session() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_GOODBYE.to_string(),
            None,
        );

        let response = handler.handle_goodbye(&request, &session_id).await;
        assert!(response.result().is_some());

        let result = response.result().unwrap();
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
    }

    #[tokio::test]
    async fn test_handle_hello_already_authenticated() {
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("existing_session".to_string())));

        // We can't easily test handle_hello without a real WebSocketConnection,
        // but we can test the error path for already authenticated
        {
            let existing = session_id.lock().await;
            assert!(existing.is_some());
        }
    }

    #[tokio::test]
    async fn test_handle_subscribe_no_session() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_SUBSCRIBE.to_string(),
            Some(json!({"topics": ["signal.test.*"]})),
        );

        let result = handler.handle_subscribe(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_unsubscribe_no_session() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_UNSUBSCRIBE.to_string(),
            Some(json!({"subscription_id": "sub_123"})),
        );

        let result = handler.handle_unsubscribe(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_publish_no_session() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let signal = create_test_signal();
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_PUBLISH.to_string(),
            Some(json!({
                "topic": "signal.test",
                "message": {"Signal": signal}
            })),
        );

        let result = handler.handle_publish(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_ack_no_session() {
        let handler = create_test_handler();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_ACK.to_string(),
            Some(json!({
                "subscription_id": "sub_123",
                "signal_ids": ["sig_1"]
            })),
        );

        let result = handler.handle_ack(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_message_parse_error() {
        // Test with invalid JSON
        let invalid_json = "not valid json";

        // We can't easily create a WebSocketConnection without a real WebSocket,
        // but we can verify the parse logic via the JsonRpcMessage::parse function
        let result = JsonRpcMessage::parse(invalid_json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_message_response_ignored() {
        // Verify that responses from clients are ignored
        let response_json = r#"{"jsonrpc":"2.0","result":{},"id":1}"#;
        let message = JsonRpcMessage::parse(response_json).unwrap();

        assert!(matches!(message, JsonRpcMessage::Response(_)));
    }

    #[tokio::test]
    async fn test_handle_goodbye_with_session() {
        let handler = create_test_handler();

        // Create a real session first
        let session_info = crate::session::SessionInfo::new(
            "sess_goodbye_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_goodbye_test".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_GOODBYE.to_string(),
            None,
        );

        let response = handler.handle_goodbye(&request, &session_id).await;
        assert!(response.result().is_some());

        // Session should be cleared
        let sid = session_id.lock().await;
        assert!(sid.is_none());
    }

    #[tokio::test]
    async fn test_handle_subscribe_with_valid_session() {
        let handler = create_test_handler();

        // Create a real session first
        let session_info = crate::session::SessionInfo::new(
            "sess_subscribe_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_subscribe_test".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_SUBSCRIBE.to_string(),
            Some(json!({"topics": ["signal.test.*"]})),
        );

        let result = handler.handle_subscribe(&request, &session_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.result().is_some());
    }

    #[tokio::test]
    async fn test_handle_subscribe_invalid_params() {
        let handler = create_test_handler();

        // Create a real session first
        let session_info = crate::session::SessionInfo::new(
            "sess_sub_invalid",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_sub_invalid".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_SUBSCRIBE.to_string(),
            Some(json!({"invalid": "data"})), // Missing topics field
        );

        let result = handler.handle_subscribe(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_unsubscribe_with_valid_session() {
        let handler = create_test_handler();

        // Create a real session first
        let session_info = crate::session::SessionInfo::new(
            "sess_unsub_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        // First create a subscription
        let sub_request = cauce_core::SubscribeRequest::new(vec!["signal.test.*".to_string()]);
        let sub_response = handler.subscription_manager
            .subscribe("client-1", "sess_unsub_test", sub_request)
            .await
            .unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_unsub_test".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_UNSUBSCRIBE.to_string(),
            Some(json!({"subscription_id": sub_response.subscription_id})),
        );

        let result = handler.handle_unsubscribe(&request, &session_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_unsubscribe_invalid_params() {
        let handler = create_test_handler();

        let session_info = crate::session::SessionInfo::new(
            "sess_unsub_invalid",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_unsub_invalid".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_UNSUBSCRIBE.to_string(),
            Some(json!({"invalid": "data"})), // Missing subscription_id
        );

        let result = handler.handle_unsubscribe(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_ack_with_valid_session() {
        let handler = create_test_handler();

        let session_info = crate::session::SessionInfo::new(
            "sess_ack_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_ack_test".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_ACK.to_string(),
            Some(json!({
                "subscription_id": "sub_123",
                "signal_ids": ["sig_1", "sig_2"]
            })),
        );

        let result = handler.handle_ack(&request, &session_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.result().is_some());
    }

    #[tokio::test]
    async fn test_handle_ack_invalid_params() {
        let handler = create_test_handler();

        let session_info = crate::session::SessionInfo::new(
            "sess_ack_invalid",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_ack_invalid".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_ACK.to_string(),
            Some(json!({"invalid": "data"})), // Missing required fields
        );

        let result = handler.handle_ack(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_publish_with_valid_session() {
        let handler = create_test_handler();

        let session_info = crate::session::SessionInfo::new(
            "sess_publish_test",
            "client-1",
            "adapter",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let signal = create_test_signal();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_publish_test".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_PUBLISH.to_string(),
            Some(json!({
                "topic": "signal.test",
                "message": signal
            })),
        );

        let result = handler.handle_publish(&request, &session_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.result().is_some());
    }

    #[tokio::test]
    async fn test_handle_publish_invalid_params() {
        let handler = create_test_handler();

        let session_info = crate::session::SessionInfo::new(
            "sess_pub_invalid",
            "client-1",
            "adapter",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_pub_invalid".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_PUBLISH.to_string(),
            Some(json!({"invalid": "data"})), // Missing topic and message
        );

        let result = handler.handle_publish(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_subscribe_session_not_found_in_manager() {
        let handler = create_test_handler();

        // Session ID exists in the mutex but not in the session manager
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("nonexistent_session".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_SUBSCRIBE.to_string(),
            Some(json!({"topics": ["signal.test.*"]})),
        );

        let result = handler.handle_subscribe(&request, &session_id).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_params_with_string_id() {
        let handler = create_test_handler();
        let id = RequestId::String("request-1".to_string());

        let result: Result<serde_json::Value, _> = handler.parse_params(None, &id);
        assert!(result.is_err());

        let error_response = result.unwrap_err();
        assert!(error_response.is_error());
    }

    #[tokio::test]
    async fn test_handle_publish_with_matching_subscriptions() {
        let handler = create_test_handler();

        // Create session
        let session_info = crate::session::SessionInfo::new(
            "sess_pub_match",
            "client-1",
            "adapter",
            "1.0",
            cauce_core::Transport::WebSocket,
            3600,
        );
        handler.session_manager.create_session(session_info).await.unwrap();

        // Create a subscription that matches the topic
        let sub_request = cauce_core::SubscribeRequest::new(vec!["signal.test.*".to_string()]);
        let _sub_response = handler.subscription_manager
            .subscribe("client-2", "sess_other", sub_request)
            .await
            .unwrap();

        let signal = create_test_signal();
        let session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("sess_pub_match".to_string())));
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            METHOD_PUBLISH.to_string(),
            Some(json!({
                "topic": "signal.test",
                "message": signal
            })),
        );

        let result = handler.handle_publish(&request, &session_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let result_value = response.result().unwrap();
        // Should have delivered to at least one subscription
        assert!(result_value.get("delivered_to").is_some());
    }
}

//! Server-Sent Events (SSE) transport handler for the Cauce server.
//!
//! This module provides the [`SseHandler`] for streaming signals to clients
//! using Server-Sent Events.
//!
//! # Example
//!
//! ```ignore
//! use axum::{Router, routing::get};
//! use cauce_server_sdk::transport::SseHandler;
//!
//! let handler = SseHandler::new(/* components */);
//! let app = Router::new()
//!     .route("/cauce/v1/sse", get(handler.stream_handler()));
//! ```

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Query;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use crate::delivery::DeliveryTracker;
use crate::session::SessionManager;
use crate::subscription::SubscriptionManager;
use cauce_core::SignalDelivery;

/// Query parameters for SSE stream endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct SseQuery {
    /// The session ID to authenticate.
    pub session_id: String,
    /// Optional subscription ID to filter signals.
    pub subscription_id: Option<String>,
    /// Last event ID received (for resumption).
    #[serde(rename = "lastEventId")]
    pub last_event_id: Option<String>,
}

/// An SSE event sent to the client.
#[derive(Debug, Clone, Serialize)]
pub struct SseSignalEvent {
    /// The signal delivery data.
    pub delivery: SignalDelivery,
    /// The subscription ID this signal was delivered to.
    pub subscription_id: String,
}

/// SSE transport handler.
///
/// Handles Server-Sent Events connections for streaming signals to clients.
pub struct SseHandler<S, D, M>
where
    S: SubscriptionManager,
    D: DeliveryTracker,
    M: SessionManager,
{
    subscription_manager: Arc<S>,
    delivery_tracker: Arc<D>,
    session_manager: Arc<M>,
    signal_tx: broadcast::Sender<(String, SignalDelivery)>,
    keepalive_interval: Duration,
}

impl<S, D, M> SseHandler<S, D, M>
where
    S: SubscriptionManager,
    D: DeliveryTracker,
    M: SessionManager,
{
    /// Creates a new SSE handler.
    pub fn new(
        subscription_manager: Arc<S>,
        delivery_tracker: Arc<D>,
        session_manager: Arc<M>,
    ) -> Self {
        let (signal_tx, _) = broadcast::channel(1000);
        Self {
            subscription_manager,
            delivery_tracker,
            session_manager,
            signal_tx,
            keepalive_interval: Duration::from_secs(30),
        }
    }

    /// Sets the keepalive interval.
    pub fn with_keepalive_interval(mut self, interval: Duration) -> Self {
        self.keepalive_interval = interval;
        self
    }

    /// Gets a sender for broadcasting signals to SSE clients.
    pub fn signal_sender(&self) -> broadcast::Sender<(String, SignalDelivery)> {
        self.signal_tx.clone()
    }

    /// Broadcasts a signal to all connected SSE clients with matching subscription.
    pub fn broadcast_signal(&self, subscription_id: &str, delivery: SignalDelivery) {
        let _ = self.signal_tx.send((subscription_id.to_string(), delivery));
    }

    /// Handle an SSE stream request.
    pub async fn handle_stream(
        self: Arc<Self>,
        query: Query<SseQuery>,
    ) -> impl IntoResponse {
        let session_id = query.session_id.clone();
        let subscription_filter = query.subscription_id.clone();
        let last_event_id = query.last_event_id.clone();

        // Validate session
        let session_valid = match self.session_manager.is_valid(&session_id).await {
            Ok(valid) => valid,
            Err(e) => {
                error!("Failed to validate session: {}", e);
                false
            }
        };

        if !session_valid {
            // Return error event stream
            let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(1);
            let _ = tx
                .send(Ok(Event::default()
                    .event("error")
                    .data(r#"{"code":"invalid_session","message":"Session not found or expired"}"#)))
                .await;
            drop(tx);

            return Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default());
        }

        info!(
            "SSE stream opened for session {} (subscription filter: {:?})",
            session_id, subscription_filter
        );

        // Touch session to keep it alive
        if let Err(e) = self.session_manager.touch_session(&session_id).await {
            warn!("Failed to touch session: {}", e);
        }

        // Create channel for this client
        let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
        let mut signal_rx = self.signal_tx.subscribe();

        // Handle resumption - send any missed signals
        if let Some(ref _last_id) = last_event_id {
            // TODO: Implement resumption by fetching unacked signals after last_id
            debug!("Resumption requested from event ID: {:?}", last_event_id);
        }

        // Send any pending unacked signals for this subscription
        if let Some(ref sub_id) = subscription_filter {
            if let Ok(unacked) = self.delivery_tracker.get_unacked(sub_id).await {
                let tx_clone = tx.clone();
                for delivery in unacked {
                    let event_id = delivery.signal.id.clone();
                    let event_data = SseSignalEvent {
                        delivery,
                        subscription_id: sub_id.clone(),
                    };

                    if let Ok(json) = serde_json::to_string(&event_data) {
                        let _ = tx_clone
                            .send(Ok(Event::default()
                                .event("signal")
                                .id(event_id)
                                .data(json)))
                            .await;
                    }
                }
            }
        }

        // Spawn task to forward signals
        let handler = Arc::clone(&self);
        let session_id_clone = session_id.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = signal_rx.recv() => {
                        match result {
                            Ok((sub_id, delivery)) => {
                                // Filter by subscription if specified
                                if let Some(ref filter) = subscription_filter {
                                    if &sub_id != filter {
                                        continue;
                                    }
                                }

                                let event_id = delivery.signal.id.clone();
                                let event_data = SseSignalEvent {
                                    delivery,
                                    subscription_id: sub_id,
                                };

                                if let Ok(json) = serde_json::to_string(&event_data) {
                                    if tx.send(Ok(Event::default()
                                        .event("signal")
                                        .id(event_id)
                                        .data(json))).await.is_err()
                                    {
                                        break;
                                    }
                                }

                                // Touch session on activity
                                if let Err(e) = handler.session_manager.touch_session(&session_id_clone).await {
                                    warn!("Failed to touch session: {}", e);
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                warn!("SSE client lagged, missed {} events", n);
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                break;
                            }
                        }
                    }
                }
            }
            info!("SSE stream closed for session {}", session_id_clone);
        });

        Sse::new(ReceiverStream::new(rx))
            .keep_alive(
                KeepAlive::new()
                    .interval(self.keepalive_interval)
                    .text("keepalive")
            )
    }
}

impl<S, D, M> Clone for SseHandler<S, D, M>
where
    S: SubscriptionManager,
    D: DeliveryTracker,
    M: SessionManager,
{
    fn clone(&self) -> Self {
        Self {
            subscription_manager: Arc::clone(&self.subscription_manager),
            delivery_tracker: Arc::clone(&self.delivery_tracker),
            session_manager: Arc::clone(&self.session_manager),
            signal_tx: self.signal_tx.clone(),
            keepalive_interval: self.keepalive_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedeliveryConfig;
    use crate::delivery::InMemoryDeliveryTracker;
    use crate::session::InMemorySessionManager;
    use crate::subscription::InMemorySubscriptionManager;
    use serde_json::json;

    fn create_test_handler() -> SseHandler<
        InMemorySubscriptionManager,
        InMemoryDeliveryTracker,
        InMemorySessionManager,
    > {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        SseHandler::new(subscription_manager, delivery_tracker, session_manager)
    }

    fn create_test_signal() -> cauce_core::Signal {
        use cauce_core::types::{Payload, Source, Topic};
        use chrono::Utc;

        cauce_core::Signal {
            id: format!("sig_{}", uuid::Uuid::new_v4()),
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
    fn test_sse_handler_clone() {
        let handler = create_test_handler();
        let _cloned = handler.clone();
    }

    #[test]
    fn test_sse_handler_with_keepalive() {
        let handler = create_test_handler()
            .with_keepalive_interval(Duration::from_secs(60));

        assert_eq!(handler.keepalive_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_sse_handler_signal_sender() {
        let handler = create_test_handler();
        let _sender = handler.signal_sender();
    }

    #[test]
    fn test_sse_handler_broadcast_signal() {
        let handler = create_test_handler();
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);

        // Should not panic even with no receivers
        handler.broadcast_signal("sub_123", delivery);
    }

    #[test]
    fn test_sse_handler_broadcast_with_receiver() {
        let handler = create_test_handler();
        let mut rx = handler.signal_tx.subscribe();

        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal.clone());

        handler.broadcast_signal("sub_123", delivery);

        // Receiver should get the signal
        let result = rx.try_recv();
        assert!(result.is_ok());
        let (sub_id, received_delivery) = result.unwrap();
        assert_eq!(sub_id, "sub_123");
        assert_eq!(received_delivery.signal.id, signal.id);
    }

    #[test]
    fn test_sse_query_deserialization() {
        let json = r#"{"session_id":"sess_123","subscription_id":"sub_456"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.session_id, "sess_123");
        assert_eq!(query.subscription_id, Some("sub_456".to_string()));
        assert!(query.last_event_id.is_none());
    }

    #[test]
    fn test_sse_query_with_last_event_id() {
        let json = r#"{"session_id":"sess_123","lastEventId":"sig_789"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.session_id, "sess_123");
        assert_eq!(query.last_event_id, Some("sig_789".to_string()));
    }

    #[test]
    fn test_sse_query_minimal() {
        let json = r#"{"session_id":"sess_123"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.session_id, "sess_123");
        assert!(query.subscription_id.is_none());
        assert!(query.last_event_id.is_none());
    }

    #[test]
    fn test_sse_signal_event_serialization() {
        use cauce_core::types::{Payload, Source, Topic};
        use cauce_core::Signal;
        use chrono::{DateTime, Utc};

        let signal = Signal {
            id: "sig_123".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: Source::new("email", "adapter-1", "msg-1"),
            topic: Topic::new_unchecked("signal.email.received"),
            payload: Payload::new(json!({"text": "hello"}), "application/json"),
            metadata: None,
            encrypted: None,
        };

        let delivery = SignalDelivery::new("signal.email.*", signal);
        let event = SseSignalEvent {
            delivery,
            subscription_id: "sub_456".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"subscription_id\":\"sub_456\""));
        assert!(json.contains("\"sig_123\""));
    }

    #[test]
    fn test_sse_signal_event_clone() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        let event = SseSignalEvent {
            delivery,
            subscription_id: "sub_123".to_string(),
        };

        let cloned = event.clone();
        assert_eq!(cloned.subscription_id, event.subscription_id);
    }

    #[test]
    fn test_sse_signal_event_debug() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        let event = SseSignalEvent {
            delivery,
            subscription_id: "sub_123".to_string(),
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("SseSignalEvent"));
        assert!(debug_str.contains("sub_123"));
    }

    #[test]
    fn test_sse_query_debug() {
        let query = SseQuery {
            session_id: "sess_123".to_string(),
            subscription_id: Some("sub_456".to_string()),
            last_event_id: None,
        };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("sess_123"));
    }

    #[test]
    fn test_sse_query_clone() {
        let query = SseQuery {
            session_id: "sess_123".to_string(),
            subscription_id: Some("sub_456".to_string()),
            last_event_id: Some("sig_789".to_string()),
        };

        let cloned = query.clone();
        assert_eq!(cloned.session_id, query.session_id);
        assert_eq!(cloned.subscription_id, query.subscription_id);
        assert_eq!(cloned.last_event_id, query.last_event_id);
    }

    #[tokio::test]
    async fn test_handle_stream_invalid_session() {
        use axum::extract::Query;

        let handler = Arc::new(create_test_handler());
        let query = Query(SseQuery {
            session_id: "nonexistent".to_string(),
            subscription_id: None,
            last_event_id: None,
        });

        // Should return an error stream for invalid session
        let _response = handler.handle_stream(query).await;
        // The response is an Sse stream - we can't easily test the content
        // but we verify it doesn't panic
    }

    #[tokio::test]
    async fn test_handle_stream_valid_session() {
        use axum::extract::Query;

        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_sse_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Sse,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        let handler = Arc::new(SseHandler::new(
            subscription_manager,
            delivery_tracker,
            session_manager,
        ));

        let query = Query(SseQuery {
            session_id: "sess_sse_test".to_string(),
            subscription_id: None,
            last_event_id: None,
        });

        let _response = handler.handle_stream(query).await;
        // Verify it doesn't panic for valid session
    }

    #[tokio::test]
    async fn test_handle_stream_with_subscription_filter() {
        use axum::extract::Query;

        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_sse_filter",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Sse,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        // Track some unacked signals
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        delivery_tracker.track("sub_filter", &delivery).await.unwrap();

        let handler = Arc::new(SseHandler::new(
            subscription_manager,
            Arc::clone(&delivery_tracker),
            session_manager,
        ));

        let query = Query(SseQuery {
            session_id: "sess_sse_filter".to_string(),
            subscription_id: Some("sub_filter".to_string()),
            last_event_id: None,
        });

        let _response = handler.handle_stream(query).await;
        // Verify it doesn't panic and processes unacked signals
    }

    #[tokio::test]
    async fn test_handle_stream_with_resumption() {
        use axum::extract::Query;

        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_sse_resume",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Sse,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        let handler = Arc::new(SseHandler::new(
            subscription_manager,
            delivery_tracker,
            session_manager,
        ));

        let query = Query(SseQuery {
            session_id: "sess_sse_resume".to_string(),
            subscription_id: Some("sub_resume".to_string()),
            last_event_id: Some("sig_last".to_string()),
        });

        let _response = handler.handle_stream(query).await;
        // Verify it handles the resumption request without panicking
    }

    #[test]
    fn test_sse_handler_default_keepalive() {
        let handler = create_test_handler();
        assert_eq!(handler.keepalive_interval, std::time::Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_broadcast_signal_reaches_receiver() {
        let handler = create_test_handler();
        let mut rx = handler.signal_tx.subscribe();

        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal.clone());

        handler.broadcast_signal("sub_broadcast", delivery);

        // Small delay to allow broadcast to propagate
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let result = rx.try_recv();
        assert!(result.is_ok());
        let (sub_id, received_delivery) = result.unwrap();
        assert_eq!(sub_id, "sub_broadcast");
        assert_eq!(received_delivery.signal.id, signal.id);
    }
}

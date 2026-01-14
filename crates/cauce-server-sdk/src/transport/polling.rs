//! HTTP Polling transport handler for the Cauce server.
//!
//! This module provides the [`PollingHandler`] for clients that cannot
//! use WebSocket or SSE connections.
//!
//! # Example
//!
//! ```ignore
//! use axum::{Router, routing::{get, post}};
//! use cauce_server_sdk::transport::PollingHandler;
//!
//! let handler = PollingHandler::new(/* components */);
//! let app = Router::new()
//!     .route("/cauce/v1/poll", get(handler.poll_handler()))
//!     .route("/cauce/v1/ack", post(handler.ack_handler()));
//! ```

use std::sync::Arc;
use std::time::Duration;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{debug, error, warn};

use crate::delivery::DeliveryTracker;
use crate::error::ServerResult;
use crate::session::SessionManager;
use crate::subscription::SubscriptionManager;
use cauce_core::{AckRequest, SignalDelivery};

/// Query parameters for poll endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct PollQuery {
    /// The session ID to authenticate.
    pub session_id: String,
    /// Optional subscription ID to filter signals.
    pub subscription_id: Option<String>,
    /// Timeout for long polling in seconds (0 for short polling).
    #[serde(default)]
    pub timeout_secs: u64,
    /// Maximum number of signals to return.
    #[serde(default = "default_max_signals")]
    pub max_signals: usize,
}

fn default_max_signals() -> usize {
    100
}

/// Response from the poll endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct PollResponse {
    /// List of pending signals.
    pub signals: Vec<PollSignal>,
    /// Whether there are more signals available.
    pub has_more: bool,
}

/// A signal in the poll response.
#[derive(Debug, Clone, Serialize)]
pub struct PollSignal {
    /// The subscription ID this signal was delivered to.
    pub subscription_id: String,
    /// The signal delivery data.
    pub delivery: SignalDelivery,
}

/// Error response.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
}

impl ErrorResponse {
    /// Creates a new error response.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Invalid session error.
    pub fn invalid_session() -> Self {
        Self::new("invalid_session", "Session not found or expired")
    }

    /// Internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal_error", message)
    }
}

/// HTTP Polling transport handler.
///
/// Supports both short and long polling for environments where
/// WebSocket or SSE are not available.
pub struct PollingHandler<S, D, M>
where
    S: SubscriptionManager,
    D: DeliveryTracker,
    M: SessionManager,
{
    subscription_manager: Arc<S>,
    delivery_tracker: Arc<D>,
    session_manager: Arc<M>,
    max_long_poll_timeout: Duration,
}

impl<S, D, M> PollingHandler<S, D, M>
where
    S: SubscriptionManager,
    D: DeliveryTracker,
    M: SessionManager,
{
    /// Creates a new polling handler.
    pub fn new(
        subscription_manager: Arc<S>,
        delivery_tracker: Arc<D>,
        session_manager: Arc<M>,
    ) -> Self {
        Self {
            subscription_manager,
            delivery_tracker,
            session_manager,
            max_long_poll_timeout: Duration::from_secs(60),
        }
    }

    /// Sets the maximum long poll timeout.
    pub fn with_max_timeout(mut self, timeout: Duration) -> Self {
        self.max_long_poll_timeout = timeout;
        self
    }

    /// Handle a poll request.
    pub async fn handle_poll(self: Arc<Self>, query: Query<PollQuery>) -> impl IntoResponse {
        // Validate session
        let session_valid = match self.session_manager.is_valid(&query.session_id).await {
            Ok(valid) => valid,
            Err(e) => {
                error!("Failed to validate session: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::internal(e.to_string())),
                )
                    .into_response();
            }
        };

        if !session_valid {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::invalid_session()),
            )
                .into_response();
        }

        // Touch session
        if let Err(e) = self.session_manager.touch_session(&query.session_id).await {
            warn!("Failed to touch session: {}", e);
        }

        // Determine timeout
        let poll_timeout = if query.timeout_secs > 0 {
            Duration::from_secs(query.timeout_secs.min(self.max_long_poll_timeout.as_secs()))
        } else {
            Duration::ZERO
        };

        // Get pending signals
        let get_signals = || async {
            self.get_pending_signals(&query.subscription_id, query.max_signals)
                .await
        };

        let signals = if poll_timeout.is_zero() {
            // Short poll - return immediately
            match get_signals().await {
                Ok(signals) => signals,
                Err(e) => {
                    error!("Failed to get signals: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::internal(e.to_string())),
                    )
                        .into_response();
                }
            }
        } else {
            // Long poll - wait for signals or timeout
            let poll_interval = Duration::from_millis(500);
            let result = timeout(poll_timeout, async {
                loop {
                    match get_signals().await {
                        Ok(signals) if !signals.is_empty() => return Ok(signals),
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                    tokio::time::sleep(poll_interval).await;
                }
            })
            .await;

            match result {
                Ok(Ok(signals)) => signals,
                Ok(Err(e)) => {
                    error!("Failed to get signals: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::internal(e.to_string())),
                    )
                        .into_response();
                }
                Err(_) => {
                    // Timeout - return empty response
                    vec![]
                }
            }
        };

        let has_more = signals.len() >= query.max_signals;

        debug!(
            "Poll returning {} signals for session {}",
            signals.len(),
            query.session_id
        );

        let response = PollResponse { signals, has_more };

        (StatusCode::OK, Json(response)).into_response()
    }

    /// Handle an acknowledgment request.
    pub async fn handle_ack(
        self: Arc<Self>,
        query: Query<AckQuery>,
        Json(request): Json<AckRequest>,
    ) -> impl IntoResponse {
        // Validate session
        let session_valid = match self.session_manager.is_valid(&query.session_id).await {
            Ok(valid) => valid,
            Err(e) => {
                error!("Failed to validate session: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::internal(e.to_string())),
                )
                    .into_response();
            }
        };

        if !session_valid {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::invalid_session()),
            )
                .into_response();
        }

        // Touch session
        if let Err(e) = self.session_manager.touch_session(&query.session_id).await {
            warn!("Failed to touch session: {}", e);
        }

        // Acknowledge signals
        let response = match self
            .delivery_tracker
            .ack(&request.subscription_id, &request.signal_ids)
            .await
        {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to ack signals: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::internal(e.to_string())),
                )
                    .into_response();
            }
        };

        debug!(
            "Acknowledged {} signals for subscription {}",
            response.acknowledged.len(),
            request.subscription_id
        );

        (StatusCode::OK, Json(response)).into_response()
    }

    /// Get pending signals for a session.
    async fn get_pending_signals(
        &self,
        subscription_filter: &Option<String>,
        max_signals: usize,
    ) -> ServerResult<Vec<PollSignal>> {
        // If a specific subscription is requested, get signals for that
        if let Some(ref sub_id) = subscription_filter {
            let unacked = self.delivery_tracker.get_unacked(sub_id).await?;
            let signals: Vec<PollSignal> = unacked
                .into_iter()
                .take(max_signals)
                .map(|delivery| PollSignal {
                    subscription_id: sub_id.clone(),
                    delivery,
                })
                .collect();
            return Ok(signals);
        }

        // Otherwise, we'd need to enumerate all subscriptions for the session
        // For now, return empty - this would need session->subscription mapping
        Ok(vec![])
    }
}

/// Query parameters for ack endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct AckQuery {
    /// The session ID to authenticate.
    pub session_id: String,
}

impl<S, D, M> Clone for PollingHandler<S, D, M>
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
            max_long_poll_timeout: self.max_long_poll_timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedeliveryConfig;
    use crate::delivery::{DeliveryTracker, InMemoryDeliveryTracker};
    use crate::session::InMemorySessionManager;
    use crate::subscription::InMemorySubscriptionManager;

    fn create_test_handler() -> PollingHandler<
        InMemorySubscriptionManager,
        InMemoryDeliveryTracker,
        InMemorySessionManager,
    > {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        PollingHandler::new(subscription_manager, delivery_tracker, session_manager)
    }

    fn create_test_signal() -> cauce_core::Signal {
        use cauce_core::types::{Payload, Source, Topic};
        use chrono::Utc;
        use serde_json::json;

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
    fn test_polling_handler_clone() {
        let handler = create_test_handler();
        let _cloned = handler.clone();
    }

    #[test]
    fn test_polling_handler_with_timeout() {
        let handler = create_test_handler()
            .with_max_timeout(Duration::from_secs(120));

        assert_eq!(handler.max_long_poll_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_poll_query_deserialization() {
        let json = r#"{"session_id":"sess_123","subscription_id":"sub_456","timeout_secs":30}"#;
        let query: PollQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.session_id, "sess_123");
        assert_eq!(query.subscription_id, Some("sub_456".to_string()));
        assert_eq!(query.timeout_secs, 30);
        assert_eq!(query.max_signals, 100); // default
    }

    #[test]
    fn test_poll_query_defaults() {
        let json = r#"{"session_id":"sess_123"}"#;
        let query: PollQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.session_id, "sess_123");
        assert!(query.subscription_id.is_none());
        assert_eq!(query.timeout_secs, 0);
        assert_eq!(query.max_signals, 100);
    }

    #[test]
    fn test_poll_query_custom_max_signals() {
        let json = r#"{"session_id":"sess_123","max_signals":50}"#;
        let query: PollQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.max_signals, 50);
    }

    #[test]
    fn test_poll_query_clone() {
        let query = PollQuery {
            session_id: "sess_123".to_string(),
            subscription_id: Some("sub_456".to_string()),
            timeout_secs: 30,
            max_signals: 50,
        };

        let cloned = query.clone();
        assert_eq!(cloned.session_id, query.session_id);
        assert_eq!(cloned.subscription_id, query.subscription_id);
        assert_eq!(cloned.timeout_secs, query.timeout_secs);
        assert_eq!(cloned.max_signals, query.max_signals);
    }

    #[test]
    fn test_poll_query_debug() {
        let query = PollQuery {
            session_id: "sess_123".to_string(),
            subscription_id: None,
            timeout_secs: 0,
            max_signals: 100,
        };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("sess_123"));
    }

    #[test]
    fn test_poll_response_serialization() {
        use cauce_core::types::{Payload, Source, Topic};
        use cauce_core::Signal;
        use chrono::{DateTime, Utc};
        use serde_json::json;

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
        let poll_signal = PollSignal {
            subscription_id: "sub_456".to_string(),
            delivery,
        };

        let response = PollResponse {
            signals: vec![poll_signal],
            has_more: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"signals\""));
        assert!(json.contains("\"has_more\":false"));
        assert!(json.contains("\"subscription_id\":\"sub_456\""));
    }

    #[test]
    fn test_poll_response_clone() {
        let response = PollResponse {
            signals: vec![],
            has_more: true,
        };

        let cloned = response.clone();
        assert_eq!(cloned.signals.len(), response.signals.len());
        assert_eq!(cloned.has_more, response.has_more);
    }

    #[test]
    fn test_poll_response_debug() {
        let response = PollResponse {
            signals: vec![],
            has_more: false,
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("PollResponse"));
    }

    #[test]
    fn test_poll_signal_clone() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        let poll_signal = PollSignal {
            subscription_id: "sub_123".to_string(),
            delivery,
        };

        let cloned = poll_signal.clone();
        assert_eq!(cloned.subscription_id, poll_signal.subscription_id);
    }

    #[test]
    fn test_poll_signal_debug() {
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        let poll_signal = PollSignal {
            subscription_id: "sub_123".to_string(),
            delivery,
        };

        let debug_str = format!("{:?}", poll_signal);
        assert!(debug_str.contains("PollSignal"));
        assert!(debug_str.contains("sub_123"));
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse::invalid_session();
        assert_eq!(error.code, "invalid_session");

        let error = ErrorResponse::internal("something went wrong");
        assert_eq!(error.code, "internal_error");
        assert_eq!(error.message, "something went wrong");
    }

    #[test]
    fn test_error_response_new() {
        let error = ErrorResponse::new("custom_code", "custom message");
        assert_eq!(error.code, "custom_code");
        assert_eq!(error.message, "custom message");
    }

    #[test]
    fn test_error_response_clone() {
        let error = ErrorResponse::new("test", "test message");
        let cloned = error.clone();
        assert_eq!(cloned.code, error.code);
        assert_eq!(cloned.message, error.message);
    }

    #[test]
    fn test_error_response_debug() {
        let error = ErrorResponse::new("test_code", "test message");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("test_code"));
        assert!(debug_str.contains("test message"));
    }

    #[test]
    fn test_ack_query_deserialization() {
        let json = r#"{"session_id":"sess_123"}"#;
        let query: AckQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.session_id, "sess_123");
    }

    #[test]
    fn test_ack_query_clone() {
        let query = AckQuery {
            session_id: "sess_123".to_string(),
        };

        let cloned = query.clone();
        assert_eq!(cloned.session_id, query.session_id);
    }

    #[test]
    fn test_ack_query_debug() {
        let query = AckQuery {
            session_id: "sess_123".to_string(),
        };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("sess_123"));
    }

    #[tokio::test]
    async fn test_get_pending_signals_no_subscription() {
        let handler = create_test_handler();

        // Without a subscription filter, should return empty
        let signals = handler.get_pending_signals(&None, 100).await.unwrap();
        assert!(signals.is_empty());
    }

    #[tokio::test]
    async fn test_get_pending_signals_with_subscription() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        let handler = PollingHandler::new(
            subscription_manager,
            Arc::clone(&delivery_tracker),
            session_manager,
        );

        // Track a delivery
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        delivery_tracker.track("sub_123", &delivery).await.unwrap();

        // Get pending signals
        let signals = handler
            .get_pending_signals(&Some("sub_123".to_string()), 100)
            .await
            .unwrap();

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].subscription_id, "sub_123");
    }

    #[tokio::test]
    async fn test_get_pending_signals_with_max_limit() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        let handler = PollingHandler::new(
            subscription_manager,
            Arc::clone(&delivery_tracker),
            session_manager,
        );

        // Track multiple deliveries
        for _ in 0..5 {
            let signal = create_test_signal();
            let delivery = SignalDelivery::new("signal.test.*", signal);
            delivery_tracker.track("sub_123", &delivery).await.unwrap();
        }

        // Get only 2 signals
        let signals = handler
            .get_pending_signals(&Some("sub_123".to_string()), 2)
            .await
            .unwrap();

        assert_eq!(signals.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_poll_invalid_session() {
        let handler = Arc::new(create_test_handler());
        let query = Query(PollQuery {
            session_id: "nonexistent".to_string(),
            subscription_id: None,
            timeout_secs: 0,
            max_signals: 100,
        });

        let response = handler.handle_poll(query).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_poll_short_poll_valid_session() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_poll_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Polling,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        let handler = Arc::new(PollingHandler::new(
            subscription_manager,
            delivery_tracker,
            session_manager,
        ));

        let query = Query(PollQuery {
            session_id: "sess_poll_test".to_string(),
            subscription_id: None,
            timeout_secs: 0, // Short polling
            max_signals: 100,
        });

        let response = handler.handle_poll(query).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_poll_with_signals() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_poll_signals",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Polling,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        // Track a delivery for the subscription
        let signal = create_test_signal();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        delivery_tracker.track("sub_poll", &delivery).await.unwrap();

        let handler = Arc::new(PollingHandler::new(
            subscription_manager,
            Arc::clone(&delivery_tracker),
            session_manager,
        ));

        let query = Query(PollQuery {
            session_id: "sess_poll_signals".to_string(),
            subscription_id: Some("sub_poll".to_string()),
            timeout_secs: 0,
            max_signals: 100,
        });

        let response = handler.handle_poll(query).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_poll_long_poll_timeout() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_long_poll",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Polling,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        let handler = Arc::new(PollingHandler::new(
            subscription_manager,
            delivery_tracker,
            session_manager,
        ).with_max_timeout(Duration::from_secs(60)));

        let query = Query(PollQuery {
            session_id: "sess_long_poll".to_string(),
            subscription_id: Some("sub_none".to_string()),
            timeout_secs: 1, // 1 second timeout
            max_signals: 100,
        });

        // This should timeout and return empty
        let response = handler.handle_poll(query).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_ack_invalid_session() {
        let handler = Arc::new(create_test_handler());
        let query = Query(AckQuery {
            session_id: "nonexistent".to_string(),
        });
        let request = Json(cauce_core::AckRequest {
            subscription_id: "sub_123".to_string(),
            signal_ids: vec!["sig_1".to_string()],
        });

        let response = handler.handle_ack(query, request).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_ack_valid_session() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_ack_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Polling,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        // Track a delivery that we'll ack
        let signal = create_test_signal();
        let signal_id = signal.id.clone();
        let delivery = SignalDelivery::new("signal.test.*", signal);
        delivery_tracker.track("sub_ack", &delivery).await.unwrap();

        let handler = Arc::new(PollingHandler::new(
            subscription_manager,
            delivery_tracker,
            session_manager,
        ));

        let query = Query(AckQuery {
            session_id: "sess_ack_test".to_string(),
        });
        let request = Json(cauce_core::AckRequest {
            subscription_id: "sub_ack".to_string(),
            signal_ids: vec![signal_id],
        });

        let response = handler.handle_ack(query, request).await;
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_poll_timeout_capped() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let config = RedeliveryConfig::default();
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Create a valid session
        let session_info = crate::session::SessionInfo::new(
            "sess_cap_test",
            "client-1",
            "agent",
            "1.0",
            cauce_core::Transport::Polling,
            3600,
        );
        session_manager.create_session(session_info).await.unwrap();

        // Set max timeout to 1 second
        let handler = Arc::new(PollingHandler::new(
            subscription_manager,
            delivery_tracker,
            session_manager,
        ).with_max_timeout(Duration::from_secs(1)));

        // Request 60 second timeout, should be capped to 1 second
        let query = Query(PollQuery {
            session_id: "sess_cap_test".to_string(),
            subscription_id: Some("sub_none".to_string()),
            timeout_secs: 60,
            max_signals: 100,
        });

        let start = std::time::Instant::now();
        let response = handler.handle_poll(query).await;
        let elapsed = start.elapsed();

        // Should complete in about 1 second (capped), not 60 seconds
        assert!(elapsed < Duration::from_secs(5));

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_poll_response_has_more_true() {
        let signals: Vec<PollSignal> = (0..100)
            .map(|i| {
                let signal = create_test_signal();
                let delivery = SignalDelivery::new("signal.test.*", signal);
                PollSignal {
                    subscription_id: format!("sub_{}", i),
                    delivery,
                }
            })
            .collect();

        let response = PollResponse {
            signals,
            has_more: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"has_more\":true"));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::new("test_error", "Test error message");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"code\":\"test_error\""));
        assert!(json.contains("\"message\":\"Test error message\""));
    }
}

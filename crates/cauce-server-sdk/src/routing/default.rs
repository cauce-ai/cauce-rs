//! Default message router implementation.

use async_trait::async_trait;
use cauce_core::methods::{PublishMessage, PublishRequest, SignalDelivery, SubscriptionInfo};
use cauce_core::Signal;
use std::sync::Arc;

use super::{MessageRouter, RouteResult};
use crate::error::{ServerError, ServerResult};
use crate::subscription::SubscriptionManager;

/// Default implementation of [`MessageRouter`].
///
/// Uses a [`SubscriptionManager`] to find matching subscriptions
/// and creates signal deliveries from published messages.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::routing::DefaultMessageRouter;
/// use cauce_server_sdk::subscription::InMemorySubscriptionManager;
/// use std::sync::Arc;
///
/// let subscription_manager = Arc::new(InMemorySubscriptionManager::new());
/// let router = DefaultMessageRouter::new(subscription_manager);
/// ```
pub struct DefaultMessageRouter<S: SubscriptionManager> {
    subscription_manager: Arc<S>,
}

impl<S: SubscriptionManager> DefaultMessageRouter<S> {
    /// Creates a new DefaultMessageRouter with the given subscription manager.
    pub fn new(subscription_manager: Arc<S>) -> Self {
        Self {
            subscription_manager,
        }
    }

    /// Extracts the signal from a publish message.
    fn extract_signal(message: &PublishMessage) -> ServerResult<Signal> {
        match message {
            PublishMessage::Signal(signal) => Ok(signal.clone()),
            PublishMessage::Action(_) => Err(ServerError::InvalidParams {
                message: "actions cannot be delivered as signals".to_string(),
            }),
        }
    }
}

#[async_trait]
impl<S: SubscriptionManager> MessageRouter for DefaultMessageRouter<S> {
    async fn route(&self, request: &PublishRequest) -> ServerResult<RouteResult> {
        // Find matching subscriptions
        let subscriptions = self
            .subscription_manager
            .get_subscriptions_for_topic(&request.topic)
            .await?;

        if subscriptions.is_empty() {
            return Ok(RouteResult::empty());
        }

        // Collect subscription IDs
        let subscription_ids: Vec<String> = subscriptions
            .iter()
            .map(|s| s.subscription_id.clone())
            .collect();

        Ok(RouteResult::new(subscription_ids))
    }

    async fn get_matching_subscriptions(&self, topic: &str) -> ServerResult<Vec<SubscriptionInfo>> {
        self.subscription_manager
            .get_subscriptions_for_topic(topic)
            .await
    }

    fn create_delivery(
        &self,
        request: &PublishRequest,
        _subscription: &SubscriptionInfo,
    ) -> ServerResult<SignalDelivery> {
        let signal = Self::extract_signal(&request.message)?;
        Ok(SignalDelivery::new(&request.topic, signal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscription::InMemorySubscriptionManager;
    use cauce_core::methods::SubscribeRequest;
    use cauce_core::types::{ActionBody, ActionType, Payload, Source, Topic};
    use chrono::{DateTime, Utc};
    use serde_json::json;

    fn create_test_signal() -> Signal {
        Signal {
            id: "sig_test_123".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: Source::new("email", "adapter-1", "msg-1"),
            topic: Topic::new_unchecked("signal.email.received"),
            payload: Payload::new(json!({"text": "hello"}), "application/json"),
            metadata: None,
            encrypted: None,
        }
    }

    fn create_test_action() -> cauce_core::Action {
        cauce_core::Action {
            id: "act_test_123".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            topic: Topic::new_unchecked("action.email.send"),
            action: ActionBody::new(ActionType::Send, json!({"to": "test@example.com"})),
            context: None,
            encrypted: None,
        }
    }

    async fn setup_router() -> (
        DefaultMessageRouter<InMemorySubscriptionManager>,
        Arc<InMemorySubscriptionManager>,
    ) {
        let manager = Arc::new(InMemorySubscriptionManager::new());
        let router = DefaultMessageRouter::new(Arc::clone(&manager));
        (router, manager)
    }

    #[tokio::test]
    async fn test_route_no_subscriptions() {
        let (router, _) = setup_router().await;

        let signal = create_test_signal();
        let request = PublishRequest::signal("signal.email.received", signal);

        let result = router.route(&request).await.unwrap();
        assert_eq!(result.subscription_count, 0);
        assert!(result.subscription_ids.is_empty());
    }

    #[tokio::test]
    async fn test_route_with_matching_subscriptions() {
        let (router, manager) = setup_router().await;

        // Create subscriptions
        let sub1 = manager
            .subscribe(
                "client_1",
                "session_1",
                SubscribeRequest::single("signal.email.*"),
            )
            .await
            .unwrap();
        let sub2 = manager
            .subscribe(
                "client_2",
                "session_2",
                SubscribeRequest::single("signal.**"),
            )
            .await
            .unwrap();

        let signal = create_test_signal();
        let request = PublishRequest::signal("signal.email.received", signal);

        let result = router.route(&request).await.unwrap();
        assert_eq!(result.subscription_count, 2);
        assert!(result.subscription_ids.contains(&sub1.subscription_id));
        assert!(result.subscription_ids.contains(&sub2.subscription_id));
    }

    #[tokio::test]
    async fn test_route_partial_match() {
        let (router, manager) = setup_router().await;

        // Create subscriptions with different patterns
        let sub1 = manager
            .subscribe(
                "client_1",
                "session_1",
                SubscribeRequest::single("signal.email.*"),
            )
            .await
            .unwrap();
        let _sub2 = manager
            .subscribe(
                "client_2",
                "session_2",
                SubscribeRequest::single("signal.slack.*"),
            )
            .await
            .unwrap();

        let signal = create_test_signal();
        let request = PublishRequest::signal("signal.email.received", signal);

        let result = router.route(&request).await.unwrap();
        assert_eq!(result.subscription_count, 1);
        assert!(result.subscription_ids.contains(&sub1.subscription_id));
    }

    #[tokio::test]
    async fn test_get_matching_subscriptions() {
        let (router, manager) = setup_router().await;

        manager
            .subscribe(
                "client_1",
                "session_1",
                SubscribeRequest::single("signal.email.*"),
            )
            .await
            .unwrap();

        let subscriptions = router
            .get_matching_subscriptions("signal.email.received")
            .await
            .unwrap();

        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].client_id, "client_1");
    }

    #[tokio::test]
    async fn test_create_delivery() {
        let (router, manager) = setup_router().await;

        let sub = manager
            .subscribe(
                "client_1",
                "session_1",
                SubscribeRequest::single("signal.email.*"),
            )
            .await
            .unwrap();

        let subscription = manager
            .get_subscription(&sub.subscription_id)
            .await
            .unwrap()
            .unwrap();

        let signal = create_test_signal();
        let request = PublishRequest::signal("signal.email.received", signal.clone());

        let delivery = router.create_delivery(&request, &subscription).unwrap();
        assert_eq!(delivery.topic, "signal.email.received");
        assert_eq!(delivery.signal.id, signal.id);
    }

    #[tokio::test]
    async fn test_create_delivery_action_fails() {
        let (router, manager) = setup_router().await;

        let sub = manager
            .subscribe(
                "client_1",
                "session_1",
                SubscribeRequest::single("action.email.*"),
            )
            .await
            .unwrap();

        let subscription = manager
            .get_subscription(&sub.subscription_id)
            .await
            .unwrap()
            .unwrap();

        let action = create_test_action();
        let request = PublishRequest::action("action.email.send", action);

        let result = router.create_delivery(&request, &subscription);
        assert!(result.is_err());
    }

    #[test]
    fn test_route_result_new() {
        let result = RouteResult::new(vec!["sub_1".to_string(), "sub_2".to_string()]);
        assert_eq!(result.subscription_count, 2);
        assert_eq!(result.subscription_ids.len(), 2);
    }

    #[test]
    fn test_route_result_empty() {
        let result = RouteResult::empty();
        assert_eq!(result.subscription_count, 0);
        assert!(result.subscription_ids.is_empty());
    }
}

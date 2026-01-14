//! Message routing for the Cauce server.
//!
//! This module provides the [`MessageRouter`] trait and implementations
//! for routing published messages to matching subscriptions.

mod default;

pub use default::DefaultMessageRouter;

use async_trait::async_trait;
use cauce_core::methods::{PublishRequest, SignalDelivery, SubscriptionInfo};

use crate::error::ServerResult;

/// Result of routing a message.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// Number of subscriptions the message was routed to.
    pub subscription_count: usize,
    /// IDs of subscriptions that received the message.
    pub subscription_ids: Vec<String>,
}

impl RouteResult {
    /// Creates a new RouteResult.
    pub fn new(subscription_ids: Vec<String>) -> Self {
        Self {
            subscription_count: subscription_ids.len(),
            subscription_ids,
        }
    }

    /// Creates an empty result (no subscriptions matched).
    pub fn empty() -> Self {
        Self {
            subscription_count: 0,
            subscription_ids: vec![],
        }
    }
}

/// Result of delivering a signal to a subscription.
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    /// Whether the delivery was successful.
    pub success: bool,
    /// The signal ID that was delivered.
    pub signal_id: String,
    /// Error message if delivery failed.
    pub error: Option<String>,
}

impl DeliveryResult {
    /// Creates a successful delivery result.
    pub fn success(signal_id: impl Into<String>) -> Self {
        Self {
            success: true,
            signal_id: signal_id.into(),
            error: None,
        }
    }

    /// Creates a failed delivery result.
    pub fn failure(signal_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            signal_id: signal_id.into(),
            error: Some(error.into()),
        }
    }
}

/// Trait for routing messages to subscriptions.
///
/// The message router is responsible for:
/// - Finding subscriptions that match a published topic
/// - Creating signal deliveries from published messages
/// - Coordinating delivery to all matching subscriptions
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::routing::{MessageRouter, DefaultMessageRouter};
/// use cauce_server_sdk::subscription::InMemorySubscriptionManager;
/// use std::sync::Arc;
///
/// let subscription_manager = Arc::new(InMemorySubscriptionManager::new());
/// let router = DefaultMessageRouter::new(subscription_manager);
///
/// // Route a published message
/// let result = router.route(&publish_request).await?;
/// println!("Routed to {} subscriptions", result.subscription_count);
/// ```
#[async_trait]
pub trait MessageRouter: Send + Sync + 'static {
    /// Routes a published message to matching subscriptions.
    ///
    /// This finds all subscriptions that match the topic and creates
    /// signal deliveries for each. It does NOT perform the actual
    /// delivery - that's handled by the delivery tracker and transports.
    ///
    /// # Arguments
    ///
    /// * `request` - The publish request containing topic and message
    ///
    /// # Returns
    ///
    /// A RouteResult with the list of subscription IDs that matched.
    async fn route(&self, request: &PublishRequest) -> ServerResult<RouteResult>;

    /// Gets matching subscriptions for a topic without creating deliveries.
    ///
    /// Useful for checking what would be affected by a publish.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to match against
    ///
    /// # Returns
    ///
    /// List of subscription info for matching subscriptions.
    async fn get_matching_subscriptions(&self, topic: &str) -> ServerResult<Vec<SubscriptionInfo>>;

    /// Creates a signal delivery from a publish request for a specific subscription.
    ///
    /// # Arguments
    ///
    /// * `request` - The publish request
    /// * `subscription` - The target subscription
    ///
    /// # Returns
    ///
    /// The signal delivery ready for transmission.
    fn create_delivery(&self, request: &PublishRequest, subscription: &SubscriptionInfo) -> ServerResult<SignalDelivery>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_result_success() {
        let result = DeliveryResult::success("sig_123");
        assert!(result.success);
        assert_eq!(result.signal_id, "sig_123");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_delivery_result_failure() {
        let result = DeliveryResult::failure("sig_123", "connection timeout");
        assert!(!result.success);
        assert_eq!(result.signal_id, "sig_123");
        assert_eq!(result.error, Some("connection timeout".to_string()));
    }

    #[test]
    fn test_delivery_result_clone() {
        let result = DeliveryResult::success("sig_123");
        let cloned = result.clone();
        assert_eq!(cloned.signal_id, result.signal_id);
        assert_eq!(cloned.success, result.success);
    }

    #[test]
    fn test_delivery_result_debug() {
        let result = DeliveryResult::success("sig_123");
        let debug = format!("{:?}", result);
        assert!(debug.contains("DeliveryResult"));
        assert!(debug.contains("sig_123"));
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

    #[test]
    fn test_route_result_clone() {
        let result = RouteResult::new(vec!["sub_1".to_string()]);
        let cloned = result.clone();
        assert_eq!(cloned.subscription_count, result.subscription_count);
        assert_eq!(cloned.subscription_ids, result.subscription_ids);
    }

    #[test]
    fn test_route_result_debug() {
        let result = RouteResult::new(vec!["sub_1".to_string()]);
        let debug = format!("{:?}", result);
        assert!(debug.contains("RouteResult"));
    }
}

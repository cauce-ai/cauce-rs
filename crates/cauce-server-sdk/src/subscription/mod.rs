//! Subscription management for the Cauce server.
//!
//! This module provides the [`SubscriptionManager`] trait for managing
//! client subscriptions and the [`InMemorySubscriptionManager`] implementation.

mod memory;
mod trie;

pub use memory::InMemorySubscriptionManager;
pub use trie::TopicTrie;

use async_trait::async_trait;
use cauce_core::methods::{
    SubscribeRequest, SubscribeResponse, SubscriptionInfo, SubscriptionRestrictions,
};

use crate::error::ServerResult;

/// Trait for managing subscriptions.
///
/// Implementations of this trait handle the lifecycle of subscriptions:
/// creation, approval/denial, listing, and removal.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::subscription::{SubscriptionManager, InMemorySubscriptionManager};
/// use cauce_core::methods::SubscribeRequest;
///
/// let manager = InMemorySubscriptionManager::new();
///
/// // Create a subscription
/// let request = SubscribeRequest::single("signal.email.*");
/// let response = manager.subscribe("client_1", "session_1", request).await?;
///
/// // Approve it
/// manager.approve(&response.subscription_id, None).await?;
/// ```
#[async_trait]
pub trait SubscriptionManager: Send + Sync + 'static {
    /// Creates a new subscription.
    ///
    /// The subscription may start in a pending state depending on the
    /// configured approval type.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The ID of the client creating the subscription
    /// * `session_id` - The session ID for this connection
    /// * `request` - The subscription request details
    ///
    /// # Returns
    ///
    /// The subscription response with ID and initial status.
    async fn subscribe(
        &self,
        client_id: &str,
        session_id: &str,
        request: SubscribeRequest,
    ) -> ServerResult<SubscribeResponse>;

    /// Removes a subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The ID of the subscription to remove
    async fn unsubscribe(&self, subscription_id: &str) -> ServerResult<()>;

    /// Gets information about a subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The ID of the subscription
    ///
    /// # Returns
    ///
    /// The subscription info if found, or None.
    async fn get_subscription(&self, subscription_id: &str) -> ServerResult<Option<SubscriptionInfo>>;

    /// Gets all subscriptions that match a topic.
    ///
    /// This is used when routing messages - find all subscriptions
    /// that should receive a message published to the given topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The concrete topic being published to
    ///
    /// # Returns
    ///
    /// List of subscription info for all matching subscriptions.
    async fn get_subscriptions_for_topic(&self, topic: &str) -> ServerResult<Vec<SubscriptionInfo>>;

    /// Gets all subscriptions for a client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID to look up
    ///
    /// # Returns
    ///
    /// List of all subscriptions owned by this client.
    async fn get_subscriptions_for_client(&self, client_id: &str) -> ServerResult<Vec<SubscriptionInfo>>;

    /// Approves a pending subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The ID of the subscription to approve
    /// * `restrictions` - Optional restrictions to apply
    async fn approve(
        &self,
        subscription_id: &str,
        restrictions: Option<SubscriptionRestrictions>,
    ) -> ServerResult<()>;

    /// Denies a pending subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The ID of the subscription to deny
    /// * `reason` - Optional reason for the denial
    async fn deny(&self, subscription_id: &str, reason: Option<String>) -> ServerResult<()>;

    /// Revokes an active subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The ID of the subscription to revoke
    /// * `reason` - Optional reason for the revocation
    async fn revoke(&self, subscription_id: &str, reason: Option<String>) -> ServerResult<()>;

    /// Cleans up expired subscriptions.
    ///
    /// Called periodically to remove subscriptions that have passed
    /// their expiration time.
    ///
    /// # Returns
    ///
    /// The number of subscriptions removed.
    async fn cleanup_expired(&self) -> ServerResult<usize>;
}

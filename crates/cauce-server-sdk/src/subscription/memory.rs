//! In-memory subscription manager implementation.

use async_trait::async_trait;
use cauce_core::methods::{
    ApprovalType, SubscribeRequest, SubscribeResponse, SubscriptionInfo, SubscriptionRestrictions,
    SubscriptionStatus, Transport,
};
use chrono::Utc;
use dashmap::DashMap;
use std::sync::RwLock;
use uuid::Uuid;

use super::{SubscriptionManager, TopicTrie};
use crate::config::LimitsConfig;
use crate::error::{ServerError, ServerResult};

/// Internal subscription data stored by the manager.
#[derive(Debug, Clone)]
struct StoredSubscription {
    info: SubscriptionInfo,
    #[allow(dead_code)] // Will be used for session-scoped operations
    session_id: String,
    restrictions: Option<SubscriptionRestrictions>,
    denial_reason: Option<String>,
    revocation_reason: Option<String>,
}

/// In-memory implementation of [`SubscriptionManager`].
///
/// Uses DashMap for concurrent access and a TopicTrie for
/// efficient topic pattern matching.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::subscription::InMemorySubscriptionManager;
/// use cauce_server_sdk::config::LimitsConfig;
///
/// let manager = InMemorySubscriptionManager::new();
///
/// // Or with custom limits
/// let limits = LimitsConfig::default()
///     .with_max_subscriptions_per_client(100);
/// let manager = InMemorySubscriptionManager::with_limits(limits);
/// ```
pub struct InMemorySubscriptionManager {
    /// Subscriptions indexed by ID
    subscriptions: DashMap<String, StoredSubscription>,
    /// Subscriptions indexed by client ID
    client_subscriptions: DashMap<String, Vec<String>>,
    /// Topic trie for pattern matching
    topic_trie: RwLock<TopicTrie>,
    /// Configuration limits
    limits: LimitsConfig,
    /// Default approval type when not specified
    default_approval: ApprovalType,
}

impl InMemorySubscriptionManager {
    /// Creates a new in-memory subscription manager with default settings.
    pub fn new() -> Self {
        Self {
            subscriptions: DashMap::new(),
            client_subscriptions: DashMap::new(),
            topic_trie: RwLock::new(TopicTrie::new()),
            limits: LimitsConfig::default(),
            default_approval: ApprovalType::Automatic,
        }
    }

    /// Creates a new manager with custom limits.
    pub fn with_limits(limits: LimitsConfig) -> Self {
        Self {
            limits,
            ..Self::new()
        }
    }

    /// Sets the default approval type for subscriptions.
    pub fn with_default_approval(mut self, approval: ApprovalType) -> Self {
        self.default_approval = approval;
        self
    }

    /// Generates a new subscription ID.
    fn generate_subscription_id() -> String {
        format!("sub_{}", Uuid::new_v4().as_simple())
    }

    /// Determines the initial status based on approval type.
    fn initial_status(&self, request: &SubscribeRequest) -> SubscriptionStatus {
        let approval = request.approval_type.unwrap_or(self.default_approval);
        match approval {
            ApprovalType::Automatic => SubscriptionStatus::Active,
            ApprovalType::UserApproved => SubscriptionStatus::Pending,
        }
    }

    /// Determines the transport from the request.
    fn get_transport(&self, request: &SubscribeRequest) -> Transport {
        request.transport.unwrap_or(Transport::WebSocket)
    }

    /// Validates the subscription request.
    fn validate_request(&self, client_id: &str, request: &SubscribeRequest) -> ServerResult<()> {
        // Check topic count
        if request.topics.len() > self.limits.max_topics_per_subscription {
            return Err(ServerError::TooManyTopics {
                max: self.limits.max_topics_per_subscription,
            });
        }

        // Validate each topic pattern
        for topic in &request.topics {
            TopicTrie::validate_pattern(topic).map_err(|msg| ServerError::InvalidParams {
                message: format!("invalid topic pattern '{}': {}", topic, msg),
            })?;
        }

        // Check subscription limit per client
        if let Some(subs) = self.client_subscriptions.get(client_id) {
            if subs.len() >= self.limits.max_subscriptions_per_client {
                return Err(ServerError::SubscriptionLimitExceeded {
                    max: self.limits.max_subscriptions_per_client,
                });
            }
        }

        // Validate webhook config if transport is Webhook
        if request.transport == Some(Transport::Webhook) && request.webhook.is_none() {
            return Err(ServerError::InvalidParams {
                message: "webhook configuration required for webhook transport".to_string(),
            });
        }

        Ok(())
    }

    /// Adds a subscription to the topic trie.
    fn add_to_trie(&self, subscription_id: &str, topics: &[String]) {
        let mut trie = self.topic_trie.write().expect("trie lock poisoned");
        for topic in topics {
            trie.insert(topic, subscription_id);
        }
    }

    /// Removes a subscription from the topic trie.
    fn remove_from_trie(&self, subscription_id: &str, topics: &[String]) {
        let mut trie = self.topic_trie.write().expect("trie lock poisoned");
        for topic in topics {
            trie.remove(topic, subscription_id);
        }
    }
}

impl Default for InMemorySubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SubscriptionManager for InMemorySubscriptionManager {
    async fn subscribe(
        &self,
        client_id: &str,
        session_id: &str,
        request: SubscribeRequest,
    ) -> ServerResult<SubscribeResponse> {
        // Validate the request
        self.validate_request(client_id, &request)?;

        let subscription_id = Self::generate_subscription_id();
        let status = self.initial_status(&request);
        let transport = self.get_transport(&request);
        let topics = request.topics.clone();

        // Create subscription info
        let info = SubscriptionInfo::new(
            subscription_id.clone(),
            client_id,
            topics.clone(),
            status,
            transport,
        );

        // Store the subscription
        let stored = StoredSubscription {
            info: info.clone(),
            session_id: session_id.to_string(),
            restrictions: None,
            denial_reason: None,
            revocation_reason: None,
        };
        self.subscriptions.insert(subscription_id.clone(), stored);

        // Add to client's subscription list
        self.client_subscriptions
            .entry(client_id.to_string())
            .or_default()
            .push(subscription_id.clone());

        // Add to topic trie if active
        if status == SubscriptionStatus::Active {
            self.add_to_trie(&subscription_id, &topics);
        }

        Ok(SubscribeResponse::new(subscription_id, status, topics))
    }

    async fn unsubscribe(&self, subscription_id: &str) -> ServerResult<()> {
        // Get and remove the subscription
        let (_, stored) = self
            .subscriptions
            .remove(subscription_id)
            .ok_or_else(|| ServerError::SubscriptionNotFound {
                id: subscription_id.to_string(),
            })?;

        // Remove from topic trie
        self.remove_from_trie(subscription_id, &stored.info.topics);

        // Remove from client's subscription list
        if let Some(mut subs) = self.client_subscriptions.get_mut(&stored.info.client_id) {
            subs.retain(|id| id != subscription_id);
        }

        Ok(())
    }

    async fn get_subscription(&self, subscription_id: &str) -> ServerResult<Option<SubscriptionInfo>> {
        Ok(self
            .subscriptions
            .get(subscription_id)
            .map(|s| s.info.clone()))
    }

    async fn get_subscriptions_for_topic(&self, topic: &str) -> ServerResult<Vec<SubscriptionInfo>> {
        let trie = self.topic_trie.read().expect("trie lock poisoned");
        let subscription_ids = trie.get_matches(topic);

        let mut result = Vec::new();
        for id in subscription_ids {
            if let Some(stored) = self.subscriptions.get(&id) {
                // Only return active subscriptions
                if stored.info.status == SubscriptionStatus::Active {
                    // Check if topic is allowed by restrictions
                    if let Some(ref restrictions) = stored.restrictions {
                        if let Some(ref allowed) = restrictions.allowed_topics {
                            // Check if any allowed pattern matches
                            let matches = allowed.iter().any(|pattern| {
                                TopicTrie::pattern_matches(pattern, topic)
                            });
                            if !matches {
                                continue;
                            }
                        }
                    }
                    result.push(stored.info.clone());
                }
            }
        }

        Ok(result)
    }

    async fn get_subscriptions_for_client(&self, client_id: &str) -> ServerResult<Vec<SubscriptionInfo>> {
        let subscription_ids = self
            .client_subscriptions
            .get(client_id)
            .map(|s| s.clone())
            .unwrap_or_default();

        let mut result = Vec::new();
        for id in subscription_ids {
            if let Some(stored) = self.subscriptions.get(&id) {
                result.push(stored.info.clone());
            }
        }

        Ok(result)
    }

    async fn approve(
        &self,
        subscription_id: &str,
        restrictions: Option<SubscriptionRestrictions>,
    ) -> ServerResult<()> {
        let mut stored = self
            .subscriptions
            .get_mut(subscription_id)
            .ok_or_else(|| ServerError::SubscriptionNotFound {
                id: subscription_id.to_string(),
            })?;

        if stored.info.status != SubscriptionStatus::Pending {
            return Err(ServerError::InvalidSessionState {
                message: format!(
                    "cannot approve subscription in {} state",
                    status_str(&stored.info.status)
                ),
            });
        }

        // Update status and restrictions
        stored.info.status = SubscriptionStatus::Active;
        stored.restrictions = restrictions.clone();

        // Update expiry if specified in restrictions
        if let Some(ref r) = restrictions {
            if let Some(expires) = r.expires_at {
                stored.info.expires_at = Some(expires);
            }
        }

        // Add to topic trie now that it's active
        let topics = stored.info.topics.clone();
        drop(stored); // Release the lock before modifying trie
        self.add_to_trie(subscription_id, &topics);

        Ok(())
    }

    async fn deny(&self, subscription_id: &str, reason: Option<String>) -> ServerResult<()> {
        let mut stored = self
            .subscriptions
            .get_mut(subscription_id)
            .ok_or_else(|| ServerError::SubscriptionNotFound {
                id: subscription_id.to_string(),
            })?;

        if stored.info.status != SubscriptionStatus::Pending {
            return Err(ServerError::InvalidSessionState {
                message: format!(
                    "cannot deny subscription in {} state",
                    status_str(&stored.info.status)
                ),
            });
        }

        stored.info.status = SubscriptionStatus::Denied;
        stored.denial_reason = reason;

        Ok(())
    }

    async fn revoke(&self, subscription_id: &str, reason: Option<String>) -> ServerResult<()> {
        let mut stored = self
            .subscriptions
            .get_mut(subscription_id)
            .ok_or_else(|| ServerError::SubscriptionNotFound {
                id: subscription_id.to_string(),
            })?;

        if stored.info.status != SubscriptionStatus::Active {
            return Err(ServerError::InvalidSessionState {
                message: format!(
                    "cannot revoke subscription in {} state",
                    status_str(&stored.info.status)
                ),
            });
        }

        // Remove from topic trie first
        let topics = stored.info.topics.clone();
        stored.info.status = SubscriptionStatus::Revoked;
        stored.revocation_reason = reason;
        drop(stored); // Release lock before modifying trie

        self.remove_from_trie(subscription_id, &topics);

        Ok(())
    }

    async fn cleanup_expired(&self) -> ServerResult<usize> {
        let now = Utc::now();
        let mut expired_ids = Vec::new();

        // Find expired subscriptions
        for entry in self.subscriptions.iter() {
            if let Some(expires) = entry.info.expires_at {
                if expires < now {
                    expired_ids.push(entry.key().clone());
                }
            }
        }

        // Remove them
        for id in &expired_ids {
            let _ = self.unsubscribe(id).await;
        }

        Ok(expired_ids.len())
    }
}

/// Helper to convert status to string for error messages.
fn status_str(status: &SubscriptionStatus) -> &'static str {
    match status {
        SubscriptionStatus::Active => "active",
        SubscriptionStatus::Pending => "pending",
        SubscriptionStatus::Denied => "denied",
        SubscriptionStatus::Revoked => "revoked",
        SubscriptionStatus::Expired => "expired",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[tokio::test]
    async fn test_subscribe_basic() {
        let manager = InMemorySubscriptionManager::new();
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        assert!(response.subscription_id.starts_with("sub_"));
        assert_eq!(response.status, SubscriptionStatus::Active);
        assert_eq!(response.topics.len(), 1);
    }

    #[tokio::test]
    async fn test_subscribe_pending_approval() {
        let manager = InMemorySubscriptionManager::new()
            .with_default_approval(ApprovalType::UserApproved);
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        assert_eq!(response.status, SubscriptionStatus::Pending);
    }

    #[tokio::test]
    async fn test_subscribe_override_approval() {
        let manager = InMemorySubscriptionManager::new()
            .with_default_approval(ApprovalType::UserApproved);
        let request = SubscribeRequest::single("signal.email.*")
            .with_approval(ApprovalType::Automatic);

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        assert_eq!(response.status, SubscriptionStatus::Active);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let manager = InMemorySubscriptionManager::new();
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        manager.unsubscribe(&response.subscription_id).await.unwrap();

        let result = manager.get_subscription(&response.subscription_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_unsubscribe_not_found() {
        let manager = InMemorySubscriptionManager::new();

        let result = manager.unsubscribe("sub_nonexistent").await;
        assert!(matches!(result, Err(ServerError::SubscriptionNotFound { .. })));
    }

    #[tokio::test]
    async fn test_get_subscriptions_for_topic() {
        let manager = InMemorySubscriptionManager::new();

        // Create subscriptions with different patterns
        let _ = manager
            .subscribe("client_1", "session_1", SubscribeRequest::single("signal.email.*"))
            .await
            .unwrap();
        let _ = manager
            .subscribe("client_2", "session_2", SubscribeRequest::single("signal.**"))
            .await
            .unwrap();
        let _ = manager
            .subscribe("client_3", "session_3", SubscribeRequest::single("other.topic"))
            .await
            .unwrap();

        // Check matches
        let matches = manager
            .get_subscriptions_for_topic("signal.email.received")
            .await
            .unwrap();

        assert_eq!(matches.len(), 2);
        let client_ids: Vec<_> = matches.iter().map(|m| m.client_id.as_str()).collect();
        assert!(client_ids.contains(&"client_1"));
        assert!(client_ids.contains(&"client_2"));
    }

    #[tokio::test]
    async fn test_get_subscriptions_for_client() {
        let manager = InMemorySubscriptionManager::new();

        // Create multiple subscriptions for one client
        let _ = manager
            .subscribe("client_1", "session_1", SubscribeRequest::single("topic.a"))
            .await
            .unwrap();
        let _ = manager
            .subscribe("client_1", "session_1", SubscribeRequest::single("topic.b"))
            .await
            .unwrap();
        let _ = manager
            .subscribe("client_2", "session_2", SubscribeRequest::single("topic.c"))
            .await
            .unwrap();

        let subs = manager.get_subscriptions_for_client("client_1").await.unwrap();
        assert_eq!(subs.len(), 2);
    }

    #[tokio::test]
    async fn test_approve_subscription() {
        let manager = InMemorySubscriptionManager::new()
            .with_default_approval(ApprovalType::UserApproved);
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();
        assert_eq!(response.status, SubscriptionStatus::Pending);

        // Approve
        manager.approve(&response.subscription_id, None).await.unwrap();

        let info = manager
            .get_subscription(&response.subscription_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(info.status, SubscriptionStatus::Active);

        // Should now appear in topic matches
        let matches = manager
            .get_subscriptions_for_topic("signal.email.received")
            .await
            .unwrap();
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_approve_with_restrictions() {
        let manager = InMemorySubscriptionManager::new()
            .with_default_approval(ApprovalType::UserApproved);
        let request = SubscribeRequest::single("signal.**");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        // Approve with topic restrictions
        let restrictions = SubscriptionRestrictions::new()
            .with_topics(vec!["signal.email.*".to_string()]);
        manager
            .approve(&response.subscription_id, Some(restrictions))
            .await
            .unwrap();

        // Should match email topics
        let matches = manager
            .get_subscriptions_for_topic("signal.email.received")
            .await
            .unwrap();
        assert_eq!(matches.len(), 1);

        // Should NOT match slack topics (restricted)
        let matches = manager
            .get_subscriptions_for_topic("signal.slack.received")
            .await
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[tokio::test]
    async fn test_deny_subscription() {
        let manager = InMemorySubscriptionManager::new()
            .with_default_approval(ApprovalType::UserApproved);
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        manager
            .deny(&response.subscription_id, Some("Not authorized".to_string()))
            .await
            .unwrap();

        let info = manager
            .get_subscription(&response.subscription_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(info.status, SubscriptionStatus::Denied);
    }

    #[tokio::test]
    async fn test_revoke_subscription() {
        let manager = InMemorySubscriptionManager::new();
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        // Should match before revocation
        let matches = manager
            .get_subscriptions_for_topic("signal.email.received")
            .await
            .unwrap();
        assert_eq!(matches.len(), 1);

        // Revoke
        manager
            .revoke(&response.subscription_id, Some("Access removed".to_string()))
            .await
            .unwrap();

        let info = manager
            .get_subscription(&response.subscription_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(info.status, SubscriptionStatus::Revoked);

        // Should NOT match after revocation
        let matches = manager
            .get_subscriptions_for_topic("signal.email.received")
            .await
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[tokio::test]
    async fn test_subscription_limit() {
        let limits = LimitsConfig::default();
        let mut limits_clone = limits.clone();
        limits_clone.max_subscriptions_per_client = 2;
        let manager = InMemorySubscriptionManager::with_limits(limits_clone);

        // Create 2 subscriptions - should succeed
        let _ = manager
            .subscribe("client_1", "session_1", SubscribeRequest::single("topic.a"))
            .await
            .unwrap();
        let _ = manager
            .subscribe("client_1", "session_1", SubscribeRequest::single("topic.b"))
            .await
            .unwrap();

        // Third should fail
        let result = manager
            .subscribe("client_1", "session_1", SubscribeRequest::single("topic.c"))
            .await;
        assert!(matches!(
            result,
            Err(ServerError::SubscriptionLimitExceeded { .. })
        ));
    }

    #[tokio::test]
    async fn test_too_many_topics() {
        let limits = LimitsConfig::default().with_max_topics_per_subscription(2);
        let manager = InMemorySubscriptionManager::with_limits(limits);

        let request = SubscribeRequest::new(vec![
            "topic.a".to_string(),
            "topic.b".to_string(),
            "topic.c".to_string(),
        ]);

        let result = manager.subscribe("client_1", "session_1", request).await;
        assert!(matches!(result, Err(ServerError::TooManyTopics { .. })));
    }

    #[tokio::test]
    async fn test_invalid_topic_pattern() {
        let manager = InMemorySubscriptionManager::new();
        let request = SubscribeRequest::single("signal.**.invalid"); // ** must be last

        let result = manager.subscribe("client_1", "session_1", request).await;
        assert!(matches!(result, Err(ServerError::InvalidParams { .. })));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = InMemorySubscriptionManager::new()
            .with_default_approval(ApprovalType::UserApproved);
        let request = SubscribeRequest::single("signal.email.*");

        let response = manager
            .subscribe("client_1", "session_1", request)
            .await
            .unwrap();

        // Approve with past expiry
        let past = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let restrictions = SubscriptionRestrictions::new().with_expiry(past);
        manager
            .approve(&response.subscription_id, Some(restrictions))
            .await
            .unwrap();

        // Cleanup should remove it
        let removed = manager.cleanup_expired().await.unwrap();
        assert_eq!(removed, 1);

        // Should be gone
        let info = manager.get_subscription(&response.subscription_id).await.unwrap();
        assert!(info.is_none());
    }
}

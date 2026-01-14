//! In-memory delivery tracker implementation.

use async_trait::async_trait;
use cauce_core::methods::{AckResponse, SignalDelivery};
use chrono::{Duration, Utc};
use dashmap::DashMap;

use super::{DeliveryStatus, DeliveryTracker, PendingDelivery};
use crate::config::RedeliveryConfig;
use crate::error::{ServerError, ServerResult};

/// Key for indexing deliveries.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct DeliveryKey {
    subscription_id: String,
    signal_id: String,
}

impl DeliveryKey {
    fn new(subscription_id: &str, signal_id: &str) -> Self {
        Self {
            subscription_id: subscription_id.to_string(),
            signal_id: signal_id.to_string(),
        }
    }
}

/// Stored delivery record.
#[derive(Debug, Clone)]
struct StoredDelivery {
    pending: PendingDelivery,
    status: DeliveryStatus,
}

/// In-memory implementation of [`DeliveryTracker`].
///
/// Uses DashMap for concurrent access and implements
/// exponential backoff for redelivery scheduling.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::delivery::InMemoryDeliveryTracker;
/// use cauce_server_sdk::config::RedeliveryConfig;
///
/// let config = RedeliveryConfig::default();
/// let tracker = InMemoryDeliveryTracker::new(config);
/// ```
pub struct InMemoryDeliveryTracker {
    /// All deliveries indexed by (subscription_id, signal_id)
    deliveries: DashMap<DeliveryKey, StoredDelivery>,
    /// Redelivery configuration
    config: RedeliveryConfig,
}

impl InMemoryDeliveryTracker {
    /// Creates a new in-memory delivery tracker.
    pub fn new(config: RedeliveryConfig) -> Self {
        Self {
            deliveries: DashMap::new(),
            config,
        }
    }

    /// Calculates the next attempt time using exponential backoff.
    fn calculate_next_attempt(&self, attempt_count: u32) -> chrono::DateTime<Utc> {
        let delay = self.config.delay_for_attempt(attempt_count);
        Utc::now() + Duration::from_std(delay).unwrap_or(Duration::seconds(5))
    }
}

impl Default for InMemoryDeliveryTracker {
    fn default() -> Self {
        Self::new(RedeliveryConfig::default())
    }
}

#[async_trait]
impl DeliveryTracker for InMemoryDeliveryTracker {
    async fn track(&self, subscription_id: &str, signal: &SignalDelivery) -> ServerResult<()> {
        let key = DeliveryKey::new(subscription_id, &signal.signal.id);

        // Check if already tracking
        if self.deliveries.contains_key(&key) {
            return Ok(()); // Already tracking, idempotent
        }

        let pending = PendingDelivery::new(subscription_id, signal.clone());
        let stored = StoredDelivery {
            pending,
            status: DeliveryStatus::Pending,
        };

        self.deliveries.insert(key, stored);
        Ok(())
    }

    async fn ack(&self, subscription_id: &str, signal_ids: &[String]) -> ServerResult<AckResponse> {
        let mut acknowledged = Vec::new();
        let mut failed = Vec::new();

        for signal_id in signal_ids {
            let key = DeliveryKey::new(subscription_id, signal_id);

            if let Some(mut entry) = self.deliveries.get_mut(&key) {
                if entry.status == DeliveryStatus::Pending {
                    entry.status = DeliveryStatus::Acknowledged;
                    acknowledged.push(signal_id.clone());
                }
            } else {
                failed.push(cauce_core::methods::AckFailure {
                    signal_id: signal_id.clone(),
                    reason: "unknown signal".to_string(),
                });
            }
        }

        Ok(AckResponse { acknowledged, failed })
    }

    async fn get_unacked(&self, subscription_id: &str) -> ServerResult<Vec<SignalDelivery>> {
        let mut result = Vec::new();

        for entry in self.deliveries.iter() {
            if entry.key().subscription_id == subscription_id
                && entry.status == DeliveryStatus::Pending
            {
                result.push(entry.pending.signal.clone());
            }
        }

        Ok(result)
    }

    async fn get_for_redelivery(&self) -> ServerResult<Vec<PendingDelivery>> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let now = Utc::now();
        let mut result = Vec::new();

        for entry in self.deliveries.iter() {
            if entry.status == DeliveryStatus::Pending
                && entry.pending.next_attempt <= now
                && self.config.should_attempt(entry.pending.attempt_count)
            {
                result.push(entry.pending.clone());
            }
        }

        Ok(result)
    }

    async fn record_redelivery(&self, subscription_id: &str, signal_id: &str) -> ServerResult<()> {
        let key = DeliveryKey::new(subscription_id, signal_id);

        if let Some(mut entry) = self.deliveries.get_mut(&key) {
            entry.pending.attempt_count += 1;
            entry.pending.last_attempt = Utc::now();
            entry.pending.next_attempt = self.calculate_next_attempt(entry.pending.attempt_count);

            // Check if we've exceeded max attempts
            if !self.config.should_attempt(entry.pending.attempt_count) {
                entry.status = DeliveryStatus::DeadLetter;
            }
        } else {
            return Err(ServerError::SignalNotFound {
                id: signal_id.to_string(),
            });
        }

        Ok(())
    }

    async fn move_to_dead_letter(&self, subscription_id: &str, signal_id: &str) -> ServerResult<()> {
        let key = DeliveryKey::new(subscription_id, signal_id);

        if let Some(mut entry) = self.deliveries.get_mut(&key) {
            entry.status = DeliveryStatus::DeadLetter;
            Ok(())
        } else {
            Err(ServerError::SignalNotFound {
                id: signal_id.to_string(),
            })
        }
    }

    async fn get_dead_letters(&self, subscription_id: &str) -> ServerResult<Vec<SignalDelivery>> {
        let mut result = Vec::new();

        for entry in self.deliveries.iter() {
            if entry.key().subscription_id == subscription_id
                && entry.status == DeliveryStatus::DeadLetter
            {
                result.push(entry.pending.signal.clone());
            }
        }

        Ok(result)
    }

    async fn cleanup(&self) -> ServerResult<usize> {
        // Remove acknowledged deliveries older than 1 hour
        let cutoff = Utc::now() - Duration::hours(1);
        let mut removed = 0;

        self.deliveries.retain(|_, entry| {
            let should_remove = entry.status == DeliveryStatus::Acknowledged
                && entry.pending.last_attempt < cutoff;
            if should_remove {
                removed += 1;
            }
            !should_remove
        });

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cauce_core::types::{Payload, Source, Topic};
    use cauce_core::Signal;
    use chrono::DateTime;
    use serde_json::json;

    fn create_test_signal(id: &str) -> Signal {
        Signal {
            id: id.to_string(),
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

    fn create_test_delivery(id: &str) -> SignalDelivery {
        SignalDelivery::new("signal.email.received", create_test_signal(id))
    }

    #[tokio::test]
    async fn test_track_delivery() {
        let tracker = InMemoryDeliveryTracker::default();
        let delivery = create_test_delivery("sig_1");

        tracker.track("sub_1", &delivery).await.unwrap();

        let unacked = tracker.get_unacked("sub_1").await.unwrap();
        assert_eq!(unacked.len(), 1);
        assert_eq!(unacked[0].signal.id, "sig_1");
    }

    #[tokio::test]
    async fn test_track_idempotent() {
        let tracker = InMemoryDeliveryTracker::default();
        let delivery = create_test_delivery("sig_1");

        tracker.track("sub_1", &delivery).await.unwrap();
        tracker.track("sub_1", &delivery).await.unwrap(); // Should be idempotent

        let unacked = tracker.get_unacked("sub_1").await.unwrap();
        assert_eq!(unacked.len(), 1);
    }

    #[tokio::test]
    async fn test_ack_signals() {
        let tracker = InMemoryDeliveryTracker::default();

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();
        tracker
            .track("sub_1", &create_test_delivery("sig_2"))
            .await
            .unwrap();

        let response = tracker
            .ack("sub_1", &["sig_1".to_string()])
            .await
            .unwrap();
        assert_eq!(response.acknowledged.len(), 1);
        assert!(response.failed.is_empty());

        let unacked = tracker.get_unacked("sub_1").await.unwrap();
        assert_eq!(unacked.len(), 1);
        assert_eq!(unacked[0].signal.id, "sig_2");
    }

    #[tokio::test]
    async fn test_ack_unknown_signals() {
        let tracker = InMemoryDeliveryTracker::default();

        let response = tracker
            .ack("sub_1", &["sig_unknown".to_string()])
            .await
            .unwrap();
        assert!(response.acknowledged.is_empty());
        assert_eq!(response.failed.len(), 1);
    }

    #[tokio::test]
    async fn test_get_for_redelivery() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(std::time::Duration::from_millis(0)); // Immediate retry for testing
        let tracker = InMemoryDeliveryTracker::new(config);

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();

        // Should be ready for redelivery immediately
        let pending = tracker.get_for_redelivery().await.unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_redelivery_disabled() {
        let config = RedeliveryConfig::default().with_enabled(false);
        let tracker = InMemoryDeliveryTracker::new(config);

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();

        let pending = tracker.get_for_redelivery().await.unwrap();
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_record_redelivery() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(std::time::Duration::from_millis(0));
        let tracker = InMemoryDeliveryTracker::new(config);

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();

        tracker.record_redelivery("sub_1", "sig_1").await.unwrap();

        // Check attempt count increased
        let pending = tracker.get_for_redelivery().await.unwrap();
        assert_eq!(pending[0].attempt_count, 2);
    }

    #[tokio::test]
    async fn test_move_to_dead_letter() {
        let tracker = InMemoryDeliveryTracker::default();

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();

        tracker.move_to_dead_letter("sub_1", "sig_1").await.unwrap();

        // Should not be in unacked
        let unacked = tracker.get_unacked("sub_1").await.unwrap();
        assert!(unacked.is_empty());

        // Should be in dead letters
        let dead = tracker.get_dead_letters("sub_1").await.unwrap();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].signal.id, "sig_1");
    }

    #[tokio::test]
    async fn test_dead_letter_not_found() {
        let tracker = InMemoryDeliveryTracker::default();

        let result = tracker.move_to_dead_letter("sub_1", "sig_unknown").await;
        assert!(matches!(result, Err(ServerError::SignalNotFound { .. })));
    }

    #[tokio::test]
    async fn test_max_attempts_dead_letter() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(std::time::Duration::from_millis(0))
            .with_max_attempts(2);
        let tracker = InMemoryDeliveryTracker::new(config);

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();

        // First redelivery (attempt 2)
        tracker.record_redelivery("sub_1", "sig_1").await.unwrap();

        // Second redelivery (attempt 3) - should dead letter
        tracker.record_redelivery("sub_1", "sig_1").await.unwrap();

        // Should be in dead letters
        let dead = tracker.get_dead_letters("sub_1").await.unwrap();
        assert_eq!(dead.len(), 1);

        // Should not be pending anymore
        let pending = tracker.get_for_redelivery().await.unwrap();
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let tracker = InMemoryDeliveryTracker::default();

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();
        tracker
            .ack("sub_1", &["sig_1".to_string()])
            .await
            .unwrap();

        // Won't be cleaned up immediately (needs to be > 1 hour old)
        let removed = tracker.cleanup().await.unwrap();
        assert_eq!(removed, 0);
    }
}

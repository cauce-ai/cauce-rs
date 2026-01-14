//! Delivery tracking for the Cauce server.
//!
//! This module provides the [`DeliveryTracker`] trait and implementations
//! for tracking signal delivery and handling redelivery of unacknowledged signals.

mod memory;
mod redelivery;

pub use memory::InMemoryDeliveryTracker;
pub use redelivery::RedeliveryScheduler;

use async_trait::async_trait;
use cauce_core::methods::{AckResponse, SignalDelivery};
use chrono::{DateTime, Utc};

use crate::error::ServerResult;

/// Information about a pending delivery.
#[derive(Debug, Clone)]
pub struct PendingDelivery {
    /// The subscription ID this delivery is for.
    pub subscription_id: String,
    /// The signal being delivered.
    pub signal: SignalDelivery,
    /// When the delivery was first attempted.
    pub first_attempt: DateTime<Utc>,
    /// When the delivery was last attempted.
    pub last_attempt: DateTime<Utc>,
    /// Number of delivery attempts.
    pub attempt_count: u32,
    /// When the next delivery should be attempted.
    pub next_attempt: DateTime<Utc>,
}

impl PendingDelivery {
    /// Creates a new pending delivery.
    pub fn new(subscription_id: impl Into<String>, signal: SignalDelivery) -> Self {
        let now = Utc::now();
        Self {
            subscription_id: subscription_id.into(),
            signal,
            first_attempt: now,
            last_attempt: now,
            attempt_count: 1,
            next_attempt: now,
        }
    }

    /// Returns the signal ID.
    pub fn signal_id(&self) -> &str {
        &self.signal.signal.id
    }
}

/// Status of a delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Delivery is pending acknowledgment.
    Pending,
    /// Delivery was acknowledged.
    Acknowledged,
    /// Delivery was moved to dead letter queue.
    DeadLetter,
}

/// Trait for tracking signal delivery.
///
/// The delivery tracker manages the lifecycle of signal deliveries:
/// - Tracking which signals have been delivered
/// - Handling acknowledgments
/// - Scheduling redelivery of unacknowledged signals
/// - Moving failed deliveries to dead letter queue
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::delivery::{DeliveryTracker, InMemoryDeliveryTracker};
/// use cauce_server_sdk::config::RedeliveryConfig;
///
/// let config = RedeliveryConfig::default();
/// let tracker = InMemoryDeliveryTracker::new(config);
///
/// // Track a delivery
/// tracker.track("sub_123", &signal_delivery).await?;
///
/// // Acknowledge receipt
/// tracker.ack("sub_123", &["sig_abc"]).await?;
/// ```
#[async_trait]
pub trait DeliveryTracker: Send + Sync + 'static {
    /// Tracks a signal delivery.
    ///
    /// Call this when a signal is delivered to a subscription.
    /// The tracker will monitor it for acknowledgment.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription receiving the signal
    /// * `signal` - The signal being delivered
    async fn track(&self, subscription_id: &str, signal: &SignalDelivery) -> ServerResult<()>;

    /// Acknowledges signal receipt.
    ///
    /// Marks one or more signals as successfully delivered.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription acknowledging
    /// * `signal_ids` - IDs of signals to acknowledge
    ///
    /// # Returns
    ///
    /// Response with counts of acknowledged signals.
    async fn ack(&self, subscription_id: &str, signal_ids: &[String]) -> ServerResult<AckResponse>;

    /// Gets all unacknowledged signals for a subscription.
    ///
    /// Returns signals that have been delivered but not acknowledged.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription to check
    async fn get_unacked(&self, subscription_id: &str) -> ServerResult<Vec<SignalDelivery>>;

    /// Gets deliveries that are due for redelivery.
    ///
    /// Returns pending deliveries where the next_attempt time
    /// has passed and they haven't exceeded max attempts.
    async fn get_for_redelivery(&self) -> ServerResult<Vec<PendingDelivery>>;

    /// Records a redelivery attempt.
    ///
    /// Updates the attempt count and schedules the next attempt.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription
    /// * `signal_id` - The signal being redelivered
    async fn record_redelivery(&self, subscription_id: &str, signal_id: &str) -> ServerResult<()>;

    /// Moves a signal to the dead letter queue.
    ///
    /// Called when a signal has exceeded max delivery attempts.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription
    /// * `signal_id` - The signal to dead-letter
    async fn move_to_dead_letter(&self, subscription_id: &str, signal_id: &str) -> ServerResult<()>;

    /// Gets all dead-lettered signals for a subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription to check
    async fn get_dead_letters(&self, subscription_id: &str) -> ServerResult<Vec<SignalDelivery>>;

    /// Cleans up old acknowledged deliveries.
    ///
    /// Removes delivery records older than the retention period.
    ///
    /// # Returns
    ///
    /// Number of records cleaned up.
    async fn cleanup(&self) -> ServerResult<usize>;
}

//! Redelivery scheduler for unacknowledged signals.

use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::DeliveryTracker;
use crate::config::RedeliveryConfig;

/// Callback for attempting redelivery of a signal.
pub type RedeliveryCallback = Arc<
    dyn Fn(
            String,
            String,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
        > + Send
        + Sync,
>;

/// Background scheduler for redelivering unacknowledged signals.
///
/// The scheduler periodically checks for signals that need redelivery
/// and attempts to redeliver them using exponential backoff.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::delivery::{RedeliveryScheduler, InMemoryDeliveryTracker};
/// use cauce_server_sdk::config::RedeliveryConfig;
/// use std::sync::Arc;
///
/// let config = RedeliveryConfig::default();
/// let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));
///
/// let callback = Arc::new(|sub_id, sig_id| {
///     Box::pin(async move {
///         // Attempt redelivery
///         Ok(())
///     })
/// });
///
/// let scheduler = RedeliveryScheduler::new(tracker, config, callback);
/// scheduler.start();
/// ```
pub struct RedeliveryScheduler<T: DeliveryTracker> {
    tracker: Arc<T>,
    config: RedeliveryConfig,
    callback: Option<RedeliveryCallback>,
    shutdown_tx: broadcast::Sender<()>,
    task_handle: Option<JoinHandle<()>>,
}

impl<T: DeliveryTracker> RedeliveryScheduler<T> {
    /// Creates a new redelivery scheduler.
    pub fn new(tracker: Arc<T>, config: RedeliveryConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            tracker,
            config,
            callback: None,
            shutdown_tx,
            task_handle: None,
        }
    }

    /// Sets the callback for redelivery attempts.
    pub fn with_callback(mut self, callback: RedeliveryCallback) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Starts the redelivery scheduler.
    ///
    /// Spawns a background task that periodically checks for
    /// signals needing redelivery.
    pub fn start(&mut self) {
        if !self.config.enabled {
            info!("Redelivery scheduler disabled");
            return;
        }

        if self.callback.is_none() {
            warn!("Redelivery scheduler started without callback - redeliveries will be recorded but not attempted");
        }

        let tracker = Arc::clone(&self.tracker);
        let config = self.config.clone();
        let callback = self.callback.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            let check_interval = config.initial_delay / 2;

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Redelivery scheduler shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(check_interval) => {
                        if let Err(e) = process_redeliveries(&tracker, &callback, &config).await {
                            error!("Error processing redeliveries: {}", e);
                        }
                    }
                }
            }
        });

        self.task_handle = Some(handle);
        info!("Redelivery scheduler started");
    }

    /// Stops the redelivery scheduler.
    pub fn stop(&mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
        info!("Redelivery scheduler stopped");
    }
}

impl<T: DeliveryTracker> Drop for RedeliveryScheduler<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Process pending redeliveries.
async fn process_redeliveries<T: DeliveryTracker>(
    tracker: &Arc<T>,
    callback: &Option<RedeliveryCallback>,
    config: &RedeliveryConfig,
) -> Result<(), String> {
    let pending = tracker
        .get_for_redelivery()
        .await
        .map_err(|e| e.to_string())?;

    if pending.is_empty() {
        return Ok(());
    }

    debug!("Processing {} pending redeliveries", pending.len());

    for delivery in pending {
        let sub_id = delivery.subscription_id.clone();
        let sig_id = delivery.signal_id().to_string();

        // Check if we've exceeded max attempts
        if !config.should_attempt(delivery.attempt_count) {
            debug!(
                "Signal {} exceeded max attempts, moving to dead letter",
                sig_id
            );
            if let Err(e) = tracker.move_to_dead_letter(&sub_id, &sig_id).await {
                warn!("Failed to dead-letter signal {}: {}", sig_id, e);
            }
            continue;
        }

        // Attempt redelivery if callback is set
        if let Some(ref cb) = callback {
            let result = cb(sub_id.clone(), sig_id.clone()).await;
            if let Err(e) = result {
                debug!("Redelivery attempt failed for {}: {}", sig_id, e);
            }
        }

        // Record the attempt regardless of callback
        if let Err(e) = tracker.record_redelivery(&sub_id, &sig_id).await {
            warn!("Failed to record redelivery for {}: {}", sig_id, e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::InMemoryDeliveryTracker;
    use cauce_core::methods::SignalDelivery;
    use cauce_core::types::{Payload, Source, Topic};
    use cauce_core::Signal;
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

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
    async fn test_scheduler_creation() {
        let config = RedeliveryConfig::default();
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));
        let scheduler = RedeliveryScheduler::new(tracker, config);
        assert!(scheduler.callback.is_none());
    }

    #[tokio::test]
    async fn test_scheduler_with_callback() {
        let config = RedeliveryConfig::default();
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let callback: RedeliveryCallback = Arc::new(move |_, _| {
            let c = Arc::clone(&counter_clone);
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });

        let scheduler = RedeliveryScheduler::new(tracker, config).with_callback(callback);
        assert!(scheduler.callback.is_some());
    }

    #[tokio::test]
    async fn test_process_redeliveries_empty() {
        let config = RedeliveryConfig::default();
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        let result = process_redeliveries(&tracker, &None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_redeliveries_with_pending() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_millis(0));
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        tracker
            .track("sub_1", &create_test_delivery("sig_1"))
            .await
            .unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let callback: RedeliveryCallback = Arc::new(move |_, _| {
            let c = Arc::clone(&counter_clone);
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });

        let result = process_redeliveries(&tracker, &Some(callback), &config).await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_scheduler_disabled() {
        let config = RedeliveryConfig::default().with_enabled(false);
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        let mut scheduler = RedeliveryScheduler::new(tracker, config);
        scheduler.start();

        assert!(scheduler.task_handle.is_none());
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_millis(100));
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        let mut scheduler = RedeliveryScheduler::new(tracker, config);
        scheduler.start();

        assert!(scheduler.task_handle.is_some());

        scheduler.stop();
        // After stop, handle should be taken
        assert!(scheduler.task_handle.is_none());
    }

    #[tokio::test]
    async fn test_process_redeliveries_without_callback() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_millis(0));
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        tracker
            .track("sub_1", &create_test_delivery("sig_no_cb"))
            .await
            .unwrap();

        // Process without callback - should still record the attempt
        let result = process_redeliveries(&tracker, &None, &config).await;
        assert!(result.is_ok());

        // The signal should still be tracked (but with incremented attempt count)
        let unacked = tracker.get_unacked("sub_1").await.unwrap();
        assert!(!unacked.is_empty());
    }

    #[tokio::test]
    async fn test_process_redeliveries_callback_failure() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_millis(0));
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        tracker
            .track("sub_fail", &create_test_delivery("sig_fail"))
            .await
            .unwrap();

        // Callback that always fails
        let callback: RedeliveryCallback = Arc::new(|_, _| {
            Box::pin(async move {
                Err("Delivery failed".to_string())
            })
        });

        let result = process_redeliveries(&tracker, &Some(callback), &config).await;
        assert!(result.is_ok()); // Processing succeeds even if callback fails
    }

    #[tokio::test]
    async fn test_process_redeliveries_max_attempts_exceeded() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_millis(0))
            .with_max_attempts(2);
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        // Track and simulate multiple redeliveries
        tracker
            .track("sub_max", &create_test_delivery("sig_max"))
            .await
            .unwrap();

        // Record multiple redelivery attempts to exceed max
        tracker.record_redelivery("sub_max", "sig_max").await.unwrap();
        tracker.record_redelivery("sub_max", "sig_max").await.unwrap();
        tracker.record_redelivery("sub_max", "sig_max").await.unwrap();

        // Now process - signal should be moved to dead letter
        let result = process_redeliveries(&tracker, &None, &config).await;
        assert!(result.is_ok());

        // Signal should be gone from unacked
        let unacked = tracker.get_unacked("sub_max").await.unwrap();
        assert!(unacked.is_empty());
    }

    #[tokio::test]
    async fn test_scheduler_stop_without_start() {
        let config = RedeliveryConfig::default();
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        let mut scheduler = RedeliveryScheduler::new(tracker, config);
        // Stop without starting - should not panic
        scheduler.stop();
        assert!(scheduler.task_handle.is_none());
    }

    #[tokio::test]
    async fn test_scheduler_start_without_callback() {
        let config = RedeliveryConfig::default()
            .with_initial_delay(Duration::from_millis(50));
        let tracker = Arc::new(InMemoryDeliveryTracker::new(config.clone()));

        let mut scheduler = RedeliveryScheduler::new(tracker, config);
        scheduler.start(); // Should warn but not panic

        assert!(scheduler.task_handle.is_some());
        scheduler.stop();
    }
}

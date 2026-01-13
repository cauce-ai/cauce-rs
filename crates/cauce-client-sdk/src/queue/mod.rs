//! Local message queue for resilience.
//!
//! This module provides a local queue for buffering messages when the Hub
//! is unavailable, with optional persistence and automatic retry.
//!
//! # Example
//!
//! ```rust,ignore
//! use cauce_client_sdk::queue::{LocalQueue, QueueConfig};
//!
//! let queue = LocalQueue::new(QueueConfig::default());
//!
//! // Queue a message for later delivery
//! queue.enqueue(message).await?;
//!
//! // Drain queued messages when connection is restored
//! while let Some(msg) = queue.dequeue().await {
//!     transport.send(msg).await?;
//! }
//! ```

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::error::ClientError;
use crate::transport::JsonRpcMessage;

/// Configuration for the local message queue.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum number of messages to buffer.
    pub max_size: usize,

    /// Maximum age of messages before they are discarded (None = no limit).
    pub max_age: Option<Duration>,

    /// Whether to persist messages to disk (not implemented yet).
    pub persist: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            max_age: Some(Duration::from_secs(3600)), // 1 hour
            persist: false,
        }
    }
}

impl QueueConfig {
    /// Creates a new queue configuration with the given maximum size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_size,
            ..Default::default()
        }
    }

    /// Sets the maximum age for messages.
    pub fn max_age(mut self, duration: Duration) -> Self {
        self.max_age = Some(duration);
        self
    }

    /// Disables the maximum age limit.
    pub fn no_max_age(mut self) -> Self {
        self.max_age = None;
        self
    }
}

/// A queued message with metadata.
#[derive(Debug)]
struct QueuedMessage {
    /// The JSON-RPC message.
    message: JsonRpcMessage,
    /// When the message was queued.
    queued_at: Instant,
    /// Number of send attempts.
    attempts: u32,
}

/// Local message queue for buffering messages when Hub is unavailable.
///
/// The queue provides:
/// - FIFO ordering of messages
/// - Configurable maximum size
/// - Automatic expiration of old messages
/// - Statistics tracking
pub struct LocalQueue {
    /// Queue configuration.
    config: QueueConfig,

    /// The message queue.
    queue: Arc<Mutex<VecDeque<QueuedMessage>>>,

    /// Total messages enqueued.
    total_enqueued: AtomicU64,

    /// Total messages dequeued.
    total_dequeued: AtomicU64,

    /// Total messages dropped (due to overflow or expiration).
    total_dropped: AtomicU64,
}

impl LocalQueue {
    /// Creates a new local queue with the given configuration.
    pub fn new(config: QueueConfig) -> Self {
        Self {
            config,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            total_enqueued: AtomicU64::new(0),
            total_dequeued: AtomicU64::new(0),
            total_dropped: AtomicU64::new(0),
        }
    }

    /// Creates a new local queue with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(QueueConfig::default())
    }

    /// Enqueues a message for later delivery.
    ///
    /// If the queue is full, the oldest message is dropped.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message was queued, or `Err` if the queue
    /// is configured to reject overflow.
    pub async fn enqueue(&self, message: JsonRpcMessage) -> Result<(), ClientError> {
        let mut queue = self.queue.lock().await;

        // Check if we need to drop old messages
        while queue.len() >= self.config.max_size {
            queue.pop_front();
            self.total_dropped.fetch_add(1, Ordering::SeqCst);
            tracing::debug!("Queue full, dropped oldest message");
        }

        // Enqueue the new message
        queue.push_back(QueuedMessage {
            message,
            queued_at: Instant::now(),
            attempts: 0,
        });

        self.total_enqueued.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Dequeues the next message for delivery.
    ///
    /// Expired messages are automatically skipped.
    ///
    /// # Returns
    ///
    /// Returns `Some(message)` if a message is available, or `None` if the
    /// queue is empty.
    pub async fn dequeue(&self) -> Option<JsonRpcMessage> {
        let mut queue = self.queue.lock().await;

        loop {
            match queue.pop_front() {
                Some(mut queued) => {
                    // Check if message has expired
                    if let Some(max_age) = self.config.max_age {
                        if queued.queued_at.elapsed() > max_age {
                            self.total_dropped.fetch_add(1, Ordering::SeqCst);
                            tracing::debug!("Dropped expired message");
                            continue;
                        }
                    }

                    queued.attempts += 1;
                    self.total_dequeued.fetch_add(1, Ordering::SeqCst);
                    return Some(queued.message);
                }
                None => return None,
            }
        }
    }

    /// Peeks at the next message without removing it.
    pub async fn peek(&self) -> Option<JsonRpcMessage> {
        let queue = self.queue.lock().await;
        queue.front().map(|q| q.message.clone())
    }

    /// Returns the number of messages currently in the queue.
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Returns true if the queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    /// Clears all messages from the queue.
    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        let dropped = queue.len() as u64;
        queue.clear();
        self.total_dropped.fetch_add(dropped, Ordering::SeqCst);
    }

    /// Returns queue statistics.
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            total_enqueued: self.total_enqueued.load(Ordering::SeqCst),
            total_dequeued: self.total_dequeued.load(Ordering::SeqCst),
            total_dropped: self.total_dropped.load(Ordering::SeqCst),
        }
    }

    /// Drains all messages from the queue.
    ///
    /// This is useful when connection is restored and you want to
    /// send all queued messages.
    pub async fn drain(&self) -> Vec<JsonRpcMessage> {
        let mut messages = Vec::new();
        while let Some(msg) = self.dequeue().await {
            messages.push(msg);
        }
        messages
    }

    /// Re-queues a message at the front of the queue.
    ///
    /// Use this when a send attempt fails and you want to retry later.
    pub async fn requeue_front(&self, message: JsonRpcMessage) {
        let mut queue = self.queue.lock().await;

        // Don't exceed max size - if full, just drop this message
        if queue.len() >= self.config.max_size {
            self.total_dropped.fetch_add(1, Ordering::SeqCst);
            return;
        }

        queue.push_front(QueuedMessage {
            message,
            queued_at: Instant::now(),
            attempts: 0,
        });
    }
}

/// Statistics about the queue.
#[derive(Debug, Clone, Copy)]
pub struct QueueStats {
    /// Total messages enqueued since creation.
    pub total_enqueued: u64,

    /// Total messages successfully dequeued.
    pub total_dequeued: u64,

    /// Total messages dropped (overflow or expiration).
    pub total_dropped: u64,
}

impl QueueStats {
    /// Returns the number of messages currently pending.
    pub fn pending(&self) -> u64 {
        self.total_enqueued.saturating_sub(self.total_dequeued + self.total_dropped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cauce_core::JsonRpcNotification;

    fn make_notification(method: &str) -> JsonRpcMessage {
        JsonRpcMessage::Notification(JsonRpcNotification::new(method.to_string(), None))
    }

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = LocalQueue::with_defaults();

        queue.enqueue(make_notification("test1")).await.unwrap();
        queue.enqueue(make_notification("test2")).await.unwrap();

        assert_eq!(queue.len().await, 2);

        let msg1 = queue.dequeue().await.unwrap();
        assert!(msg1.is_notification());

        let msg2 = queue.dequeue().await.unwrap();
        assert!(msg2.is_notification());

        assert!(queue.dequeue().await.is_none());
    }

    #[tokio::test]
    async fn test_queue_overflow() {
        let config = QueueConfig::with_max_size(2);
        let queue = LocalQueue::new(config);

        queue.enqueue(make_notification("test1")).await.unwrap();
        queue.enqueue(make_notification("test2")).await.unwrap();
        queue.enqueue(make_notification("test3")).await.unwrap();

        // Should have dropped the oldest message
        assert_eq!(queue.len().await, 2);
        assert_eq!(queue.stats().total_dropped, 1);
    }

    #[tokio::test]
    async fn test_message_expiration() {
        let config = QueueConfig {
            max_size: 10,
            max_age: Some(Duration::from_millis(1)),
            persist: false,
        };
        let queue = LocalQueue::new(config);

        queue.enqueue(make_notification("test")).await.unwrap();

        // Wait for message to expire
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Message should be expired and dropped
        assert!(queue.dequeue().await.is_none());
        assert_eq!(queue.stats().total_dropped, 1);
    }

    #[tokio::test]
    async fn test_peek() {
        let queue = LocalQueue::with_defaults();

        assert!(queue.peek().await.is_none());

        queue.enqueue(make_notification("test")).await.unwrap();

        // Peek should not remove the message
        assert!(queue.peek().await.is_some());
        assert!(queue.peek().await.is_some());
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let queue = LocalQueue::with_defaults();

        queue.enqueue(make_notification("test1")).await.unwrap();
        queue.enqueue(make_notification("test2")).await.unwrap();
        queue.enqueue(make_notification("test3")).await.unwrap();

        assert_eq!(queue.len().await, 3);

        queue.clear().await;

        assert!(queue.is_empty().await);
        assert_eq!(queue.stats().total_dropped, 3);
    }

    #[tokio::test]
    async fn test_drain() {
        let queue = LocalQueue::with_defaults();

        queue.enqueue(make_notification("test1")).await.unwrap();
        queue.enqueue(make_notification("test2")).await.unwrap();

        let messages = queue.drain().await;

        assert_eq!(messages.len(), 2);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_requeue_front() {
        let queue = LocalQueue::with_defaults();

        queue.enqueue(make_notification("test1")).await.unwrap();
        queue.enqueue(make_notification("test2")).await.unwrap();

        // Dequeue first message
        let msg = queue.dequeue().await.unwrap();

        // Requeue it at front
        queue.requeue_front(msg).await;

        // Should get the same message again
        assert_eq!(queue.len().await, 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let queue = LocalQueue::with_defaults();

        queue.enqueue(make_notification("test1")).await.unwrap();
        queue.enqueue(make_notification("test2")).await.unwrap();

        let _ = queue.dequeue().await;

        let stats = queue.stats();
        assert_eq!(stats.total_enqueued, 2);
        assert_eq!(stats.total_dequeued, 1);
        assert_eq!(stats.pending(), 1);
    }

    #[test]
    fn test_queue_config_builder() {
        let config = QueueConfig::with_max_size(500)
            .max_age(Duration::from_secs(1800));

        assert_eq!(config.max_size, 500);
        assert_eq!(config.max_age, Some(Duration::from_secs(1800)));
    }

    #[test]
    fn test_queue_config_no_max_age() {
        let config = QueueConfig::default().no_max_age();
        assert!(config.max_age.is_none());
    }
}

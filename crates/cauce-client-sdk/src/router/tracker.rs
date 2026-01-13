//! Request tracking for the message router.
//!
//! This module provides the internal [`RequestTracker`] that manages
//! pending request-response correlation.

use cauce_core::{JsonRpcResponse, RequestId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{oneshot, Mutex};

/// Tracks pending requests awaiting responses.
///
/// This is an internal component used by [`MessageRouter`](super::MessageRouter)
/// to correlate responses with their originating requests.
pub(crate) struct RequestTracker {
    /// Map of request IDs to response channels.
    pending: Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>,

    /// Counter for generating sequential request IDs.
    next_id: AtomicU64,
}

impl RequestTracker {
    /// Create a new request tracker.
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Generate the next sequential request ID.
    ///
    /// IDs are numeric and strictly increasing within a session.
    pub fn next_id(&self) -> RequestId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        RequestId::from_number(id as i64)
    }

    /// Register a pending request.
    ///
    /// The sender will be used to deliver the response when it arrives.
    pub async fn register(&self, id: RequestId, tx: oneshot::Sender<JsonRpcResponse>) {
        self.pending.lock().await.insert(id, tx);
    }

    /// Complete a pending request with a response.
    ///
    /// Returns `true` if a matching pending request was found and completed,
    /// `false` if no request with that ID was pending.
    pub async fn complete(&self, id: &RequestId, response: JsonRpcResponse) -> bool {
        if let Some(tx) = self.pending.lock().await.remove(id) {
            // Ignore error if receiver was dropped (request was cancelled/timed out)
            let _ = tx.send(response);
            true
        } else {
            false
        }
    }

    /// Cancel a pending request.
    ///
    /// This removes the request from tracking without sending a response.
    /// Used when a request times out or is explicitly cancelled.
    ///
    /// Returns `true` if a matching pending request was found and cancelled.
    pub async fn cancel(&self, id: &RequestId) -> bool {
        self.pending.lock().await.remove(id).is_some()
    }

    /// Get the count of currently pending requests.
    ///
    /// Useful for debugging and metrics.
    pub async fn pending_count(&self) -> usize {
        self.pending.lock().await.len()
    }

    /// Clear all pending requests.
    ///
    /// Used during shutdown or disconnect cleanup.
    pub async fn clear(&self) {
        self.pending.lock().await.clear();
    }
}

impl Default for RequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_id_sequential() {
        let tracker = RequestTracker::new();

        let id1 = tracker.next_id();
        let id2 = tracker.next_id();
        let id3 = tracker.next_id();

        assert_eq!(id1.as_number(), Some(1));
        assert_eq!(id2.as_number(), Some(2));
        assert_eq!(id3.as_number(), Some(3));
    }

    #[tokio::test]
    async fn test_register_and_complete() {
        let tracker = RequestTracker::new();
        let id = tracker.next_id();

        let (tx, rx) = oneshot::channel();
        tracker.register(id.clone(), tx).await;

        assert_eq!(tracker.pending_count().await, 1);

        let response = JsonRpcResponse::success(id.clone(), serde_json::json!({"ok": true}));
        let completed = tracker.complete(&id, response).await;

        assert!(completed);
        assert_eq!(tracker.pending_count().await, 0);

        let received = rx.await.expect("should receive response");
        assert!(received.is_success());
    }

    #[tokio::test]
    async fn test_complete_unknown_id() {
        let tracker = RequestTracker::new();
        let unknown_id = RequestId::from_number(999);

        let response = JsonRpcResponse::success(unknown_id.clone(), serde_json::json!(null));
        let completed = tracker.complete(&unknown_id, response).await;

        assert!(!completed);
    }

    #[tokio::test]
    async fn test_cancel() {
        let tracker = RequestTracker::new();
        let id = tracker.next_id();

        let (tx, _rx) = oneshot::channel();
        tracker.register(id.clone(), tx).await;

        assert_eq!(tracker.pending_count().await, 1);

        let cancelled = tracker.cancel(&id).await;
        assert!(cancelled);
        assert_eq!(tracker.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_cancel_unknown_id() {
        let tracker = RequestTracker::new();
        let unknown_id = RequestId::from_number(999);

        let cancelled = tracker.cancel(&unknown_id).await;
        assert!(!cancelled);
    }

    #[tokio::test]
    async fn test_clear() {
        let tracker = RequestTracker::new();

        for _ in 0..5 {
            let id = tracker.next_id();
            let (tx, _rx) = oneshot::channel();
            tracker.register(id, tx).await;
        }

        assert_eq!(tracker.pending_count().await, 5);

        tracker.clear().await;
        assert_eq!(tracker.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_complete_after_receiver_dropped() {
        let tracker = RequestTracker::new();
        let id = tracker.next_id();

        let (tx, rx) = oneshot::channel();
        tracker.register(id.clone(), tx).await;

        // Drop the receiver before completing
        drop(rx);

        // Complete should still return true and not panic
        let response = JsonRpcResponse::success(id.clone(), serde_json::json!(null));
        let completed = tracker.complete(&id, response).await;

        assert!(completed);
        assert_eq!(tracker.pending_count().await, 0);
    }
}

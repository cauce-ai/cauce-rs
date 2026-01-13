//! Subscription handle for receiving signals.
//!
//! A [`Subscription`] is returned when you subscribe to topics via
//! [`CauceClient::subscribe`](super::CauceClient::subscribe).
//! It provides an async stream interface for receiving signals that match
//! the subscribed topic patterns.
//!
//! # Example
//!
//! ```ignore
//! let mut subscription = client.subscribe(&["signal.email.*"]).await?;
//!
//! while let Some(signal) = subscription.next().await {
//!     println!("Received: {}", signal.id);
//!     client.ack(subscription.subscription_id(), &[&signal.id]).await?;
//! }
//! ```

use cauce_core::{JsonRpcNotification, Signal, SignalDelivery, TopicMatcher, METHOD_SIGNAL};
use tokio::sync::broadcast;

/// A handle to an active subscription.
///
/// Provides an async stream of signals matching the subscription's topic patterns.
/// Use [`next()`](Self::next) to receive the next matching signal.
///
/// # Topic Matching
///
/// Signals are filtered based on the subscription's topic patterns:
/// - Exact match: `signal.email.received` matches `signal.email.received`
/// - Single wildcard: `signal.email.*` matches `signal.email.received`
/// - Multi wildcard: `signal.**` matches `signal.email.received.urgent`
///
/// # Example
///
/// ```ignore
/// let mut subscription = client.subscribe(&["signal.email.*", "signal.slack.**"]).await?;
///
/// // Receive next matching signal
/// if let Some(signal) = subscription.next().await {
///     println!("Signal ID: {}", signal.id);
///     println!("Topic: {:?}", signal.topic);
/// }
/// ```
pub struct Subscription {
    /// The subscription ID assigned by the hub.
    id: String,

    /// Topic patterns this subscription covers.
    topics: Vec<String>,

    /// Broadcast receiver for JSON-RPC notifications.
    notification_rx: broadcast::Receiver<JsonRpcNotification>,
}

impl Subscription {
    /// Creates a new Subscription handle.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription ID from the hub
    /// * `topics` - The topic patterns subscribed to
    /// * `notification_rx` - Receiver for incoming notifications
    pub(crate) fn new(
        id: String,
        topics: Vec<String>,
        notification_rx: broadcast::Receiver<JsonRpcNotification>,
    ) -> Self {
        Self {
            id,
            topics,
            notification_rx,
        }
    }

    /// Returns the subscription ID.
    ///
    /// This ID is used when acknowledging signals or unsubscribing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sub_id = subscription.subscription_id();
    /// client.ack(sub_id, &["sig_123"]).await?;
    /// ```
    pub fn subscription_id(&self) -> &str {
        &self.id
    }

    /// Returns the topic patterns this subscription covers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for topic in subscription.topics() {
    ///     println!("Subscribed to: {}", topic);
    /// }
    /// ```
    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    /// Checks if a topic matches this subscription's patterns.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to check
    ///
    /// # Returns
    ///
    /// `true` if the topic matches any of the subscription's patterns.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let subscription = client.subscribe(&["signal.email.*"]).await?;
    /// assert!(subscription.matches_topic("signal.email.received"));
    /// assert!(!subscription.matches_topic("signal.slack.message"));
    /// ```
    pub fn matches_topic(&self, topic: &str) -> bool {
        self.topics
            .iter()
            .any(|pattern| TopicMatcher::matches(topic, pattern))
    }

    /// Returns the next signal matching this subscription.
    ///
    /// This method will block until a matching signal is received or
    /// the subscription is closed.
    ///
    /// # Returns
    ///
    /// - `Some(Signal)` - The next matching signal
    /// - `None` - If the subscription is closed or the connection is lost
    ///
    /// # Example
    ///
    /// ```ignore
    /// while let Some(signal) = subscription.next().await {
    ///     println!("Received signal: {}", signal.id);
    ///
    ///     // Process the signal...
    ///
    ///     // Acknowledge receipt
    ///     client.ack(subscription.subscription_id(), &[&signal.id]).await?;
    /// }
    /// ```
    pub async fn next(&mut self) -> Option<Signal> {
        loop {
            match self.notification_rx.recv().await {
                Ok(notification) => {
                    // Check if this is a signal notification
                    if notification.method() != METHOD_SIGNAL {
                        continue;
                    }

                    // Parse the signal delivery
                    if let Some(params) = notification.params() {
                        match serde_json::from_value::<SignalDelivery>(params.clone()) {
                            Ok(delivery) => {
                                // Check if the topic matches our subscription
                                if self.matches_topic(&delivery.topic) {
                                    return Some(delivery.signal);
                                }
                                // Topic doesn't match, continue waiting
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse signal delivery: {}", e);
                                continue;
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // Channel closed, subscription is done
                    return None;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We missed some messages, log and continue
                    tracing::warn!("Subscription lagged behind by {} messages", n);
                    continue;
                }
            }
        }
    }

    /// Attempts to receive the next signal without blocking.
    ///
    /// # Returns
    ///
    /// - `Some(signal)` - A matching signal was available
    /// - `None` - No matching signal available or subscription is closed
    pub fn try_next(&mut self) -> Option<Signal> {
        loop {
            match self.notification_rx.try_recv() {
                Ok(notification) => {
                    if notification.method() != METHOD_SIGNAL {
                        continue;
                    }

                    if let Some(params) = notification.params() {
                        if let Ok(delivery) =
                            serde_json::from_value::<SignalDelivery>(params.clone())
                        {
                            if self.matches_topic(&delivery.topic) {
                                return Some(delivery.signal);
                            }
                        }
                    }
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    return None;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    return None;
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    continue;
                }
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use cauce_core::{Payload, Signal, Source, Topic};
    use chrono::Utc;
    use tokio::sync::broadcast;

    fn make_signal(id: &str, topic: &str) -> Signal {
        Signal {
            id: id.to_string(),
            version: "1.0".to_string(),
            timestamp: Utc::now(),
            source: Source::new("email", "adapter-1", "native-123"),
            topic: Topic::new_unchecked(topic),
            payload: Payload::new(serde_json::json!({}), "application/json"),
            metadata: None,
            encrypted: None,
        }
    }

    fn make_signal_notification(topic: &str, signal: Signal) -> JsonRpcNotification {
        let delivery = SignalDelivery::new(topic, signal);
        JsonRpcNotification::new(
            METHOD_SIGNAL.to_string(),
            Some(serde_json::to_value(&delivery).unwrap()),
        )
    }

    #[test]
    fn test_subscription_id() {
        let (tx, rx) = broadcast::channel(10);
        let sub = Subscription::new("sub_123".to_string(), vec!["signal.*".to_string()], rx);

        assert_eq!(sub.subscription_id(), "sub_123");
        drop(tx);
    }

    #[test]
    fn test_subscription_topics() {
        let (tx, rx) = broadcast::channel(10);
        let sub = Subscription::new(
            "sub_123".to_string(),
            vec!["signal.email.*".to_string(), "signal.slack.**".to_string()],
            rx,
        );

        assert_eq!(sub.topics().len(), 2);
        assert_eq!(sub.topics()[0], "signal.email.*");
        assert_eq!(sub.topics()[1], "signal.slack.**");
        drop(tx);
    }

    #[test]
    fn test_matches_topic_exact() {
        let (tx, rx) = broadcast::channel(10);
        let sub = Subscription::new(
            "sub_123".to_string(),
            vec!["signal.email.received".to_string()],
            rx,
        );

        assert!(sub.matches_topic("signal.email.received"));
        assert!(!sub.matches_topic("signal.email.sent"));
        drop(tx);
    }

    #[test]
    fn test_matches_topic_single_wildcard() {
        let (tx, rx) = broadcast::channel(10);
        let sub = Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        assert!(sub.matches_topic("signal.email.received"));
        assert!(sub.matches_topic("signal.email.sent"));
        assert!(!sub.matches_topic("signal.email.inbox.unread"));
        assert!(!sub.matches_topic("signal.slack.message"));
        drop(tx);
    }

    #[test]
    fn test_matches_topic_multi_wildcard() {
        let (tx, rx) = broadcast::channel(10);
        let sub = Subscription::new("sub_123".to_string(), vec!["signal.**".to_string()], rx);

        assert!(sub.matches_topic("signal.email"));
        assert!(sub.matches_topic("signal.email.received"));
        assert!(sub.matches_topic("signal.email.inbox.unread"));
        assert!(!sub.matches_topic("action.email.send"));
        drop(tx);
    }

    #[test]
    fn test_matches_topic_multiple_patterns() {
        let (tx, rx) = broadcast::channel(10);
        let sub = Subscription::new(
            "sub_123".to_string(),
            vec!["signal.email.*".to_string(), "signal.slack.**".to_string()],
            rx,
        );

        assert!(sub.matches_topic("signal.email.received"));
        assert!(sub.matches_topic("signal.slack.message"));
        assert!(sub.matches_topic("signal.slack.channel.join"));
        assert!(!sub.matches_topic("signal.teams.message"));
        drop(tx);
    }

    #[tokio::test]
    async fn test_next_receives_matching_signal() {
        let (tx, rx) = broadcast::channel(10);
        let mut sub =
            Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        // Send a matching signal
        let signal = make_signal("sig_001", "signal.email.received");
        let notification = make_signal_notification("signal.email.received", signal.clone());
        tx.send(notification).unwrap();

        // Should receive the signal
        let received = sub.next().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().id, "sig_001");
    }

    #[tokio::test]
    async fn test_next_filters_non_matching() {
        let (tx, rx) = broadcast::channel(10);
        let mut sub =
            Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        // Send non-matching signal first
        let non_matching = make_signal("sig_001", "signal.slack.message");
        tx.send(make_signal_notification("signal.slack.message", non_matching))
            .unwrap();

        // Send matching signal
        let matching = make_signal("sig_002", "signal.email.received");
        tx.send(make_signal_notification(
            "signal.email.received",
            matching.clone(),
        ))
        .unwrap();

        // Should only receive the matching one
        let received = sub.next().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().id, "sig_002");
    }

    #[tokio::test]
    async fn test_next_ignores_non_signal_notifications() {
        let (tx, rx) = broadcast::channel(10);
        let mut sub =
            Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        // Send a non-signal notification
        let other = JsonRpcNotification::new("cauce.ping".to_string(), None);
        tx.send(other).unwrap();

        // Send matching signal
        let signal = make_signal("sig_001", "signal.email.received");
        tx.send(make_signal_notification("signal.email.received", signal))
            .unwrap();

        // Should receive the signal
        let received = sub.next().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().id, "sig_001");
    }

    #[tokio::test]
    async fn test_next_returns_none_on_close() {
        let (tx, rx) = broadcast::channel::<JsonRpcNotification>(10);
        let mut sub =
            Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        // Drop sender to close channel
        drop(tx);

        // Should return None
        let received = sub.next().await;
        assert!(received.is_none());
    }

    #[test]
    fn test_try_next_empty() {
        let (tx, rx) = broadcast::channel(10);
        let mut sub =
            Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        // No messages available
        let result = sub.try_next();
        assert!(result.is_none());
        drop(tx);
    }

    #[test]
    fn test_try_next_closed() {
        let (tx, rx) = broadcast::channel::<JsonRpcNotification>(10);
        let mut sub =
            Subscription::new("sub_123".to_string(), vec!["signal.email.*".to_string()], rx);

        // Close channel
        drop(tx);

        // Should return None when closed
        let result = sub.try_next();
        assert!(result.is_none());
    }
}

//! Shared state for the Cauce server.
//!
//! This module provides the [`SharedState`] struct that holds references
//! to all server components for use in axum handlers.

use std::sync::Arc;

use crate::delivery::DeliveryTracker;
use crate::routing::MessageRouter;
use crate::session::SessionManager;
use crate::subscription::SubscriptionManager;

/// Shared state passed to all axum handlers.
///
/// This struct holds Arc references to all server components,
/// allowing handlers to access them without explicit dependency injection.
#[derive(Clone)]
pub struct SharedState {
    /// Subscription manager for managing client subscriptions.
    pub subscription_manager: Arc<dyn SubscriptionManager>,
    /// Message router for routing published messages.
    pub message_router: Arc<dyn MessageRouter>,
    /// Delivery tracker for tracking signal deliveries.
    pub delivery_tracker: Arc<dyn DeliveryTracker>,
    /// Session manager for managing client sessions.
    pub session_manager: Arc<dyn SessionManager>,
}

impl SharedState {
    /// Creates new shared state from component references.
    pub fn new(
        subscription_manager: Arc<dyn SubscriptionManager>,
        message_router: Arc<dyn MessageRouter>,
        delivery_tracker: Arc<dyn DeliveryTracker>,
        session_manager: Arc<dyn SessionManager>,
    ) -> Self {
        Self {
            subscription_manager,
            message_router,
            delivery_tracker,
            session_manager,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedeliveryConfig;
    use crate::delivery::InMemoryDeliveryTracker;
    use crate::routing::DefaultMessageRouter;
    use crate::session::InMemorySessionManager;
    use crate::subscription::InMemorySubscriptionManager;

    #[test]
    fn test_shared_state_new() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let message_router = Arc::new(DefaultMessageRouter::new(subscription_manager.clone()));
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(RedeliveryConfig::default()));
        let session_manager = Arc::new(InMemorySessionManager::default());

        let state = SharedState::new(
            subscription_manager as Arc<dyn SubscriptionManager>,
            message_router as Arc<dyn MessageRouter>,
            delivery_tracker as Arc<dyn DeliveryTracker>,
            session_manager as Arc<dyn SessionManager>,
        );

        // State should be accessible
        assert!(Arc::strong_count(&state.subscription_manager) >= 1);
        assert!(Arc::strong_count(&state.message_router) >= 1);
        assert!(Arc::strong_count(&state.delivery_tracker) >= 1);
        assert!(Arc::strong_count(&state.session_manager) >= 1);
    }

    #[test]
    fn test_shared_state_clone() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let message_router = Arc::new(DefaultMessageRouter::new(subscription_manager.clone()));
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(RedeliveryConfig::default()));
        let session_manager = Arc::new(InMemorySessionManager::default());

        let state = SharedState::new(
            subscription_manager as Arc<dyn SubscriptionManager>,
            message_router as Arc<dyn MessageRouter>,
            delivery_tracker as Arc<dyn DeliveryTracker>,
            session_manager as Arc<dyn SessionManager>,
        );

        let cloned = state.clone();

        // Both should reference the same underlying data
        assert!(Arc::ptr_eq(&state.subscription_manager, &cloned.subscription_manager));
        assert!(Arc::ptr_eq(&state.message_router, &cloned.message_router));
        assert!(Arc::ptr_eq(&state.delivery_tracker, &cloned.delivery_tracker));
        assert!(Arc::ptr_eq(&state.session_manager, &cloned.session_manager));
    }
}

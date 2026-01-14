//! Core server implementation for the Cauce server.
//!
//! This module provides the [`CauceServer`] struct which integrates
//! all components and provides the HTTP/WebSocket server.
//!
//! # Example
//!
//! ```ignore
//! use cauce_server_sdk::{CauceServer, ServerConfig};
//!
//! let config = ServerConfig::development();
//! let server = CauceServer::new(config);
//!
//! // Start serving
//! server.serve().await?;
//! ```

mod state;

pub use state::SharedState;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

use crate::auth::{AuthMiddleware, AuthValidator, InMemoryAuthValidator};
use crate::config::ServerConfig;
use crate::delivery::{DeliveryTracker, InMemoryDeliveryTracker};
use crate::error::{ServerError, ServerResult};
use crate::rate_limit::{InMemoryRateLimiter, RateLimitConfig, RateLimitMiddleware, RateLimiter};
use crate::routing::{DefaultMessageRouter, MessageRouter};
use crate::session::{InMemorySessionManager, SessionManager};
use crate::subscription::{InMemorySubscriptionManager, SubscriptionManager};
use crate::transport::{PollingHandler, SseHandler, WebSocketHandler, WebhookDelivery, WebhookDeliveryConfig};

/// The main Cauce server struct.
///
/// This struct integrates all the components needed to run a Cauce Protocol hub:
/// - Subscription management
/// - Message routing
/// - Signal delivery tracking
/// - Session management
/// - Transport handlers (WebSocket, SSE, Polling, Webhook)
/// - Authentication and rate limiting middleware
pub struct CauceServer<S, R, D, M, A, L>
where
    S: SubscriptionManager,
    R: MessageRouter,
    D: DeliveryTracker,
    M: SessionManager,
    A: AuthValidator,
    L: RateLimiter,
{
    config: ServerConfig,
    subscription_manager: Arc<S>,
    message_router: Arc<R>,
    delivery_tracker: Arc<D>,
    session_manager: Arc<M>,
    auth_validator: Arc<A>,
    rate_limiter: Arc<L>,
    webhook_delivery: Option<WebhookDelivery>,
}

/// Type alias for a server with default components.
pub type DefaultCauceServer = CauceServer<
    InMemorySubscriptionManager,
    DefaultMessageRouter<InMemorySubscriptionManager>,
    InMemoryDeliveryTracker,
    InMemorySessionManager,
    InMemoryAuthValidator,
    InMemoryRateLimiter,
>;

impl DefaultCauceServer {
    /// Creates a new server with default in-memory components.
    pub fn new(config: ServerConfig) -> Self {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let message_router = Arc::new(DefaultMessageRouter::new(Arc::clone(&subscription_manager)));
        let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(config.redelivery.clone()));
        let session_manager = Arc::new(InMemorySessionManager::default());

        // Set up auth validator with API keys from config
        let auth_validator = InMemoryAuthValidator::new();
        for key in &config.auth.api_keys {
            auth_validator.add_api_key("default", key);
        }
        let auth_validator = Arc::new(auth_validator);

        let rate_limiter = Arc::new(InMemoryRateLimiter::new(
            RateLimitConfig::default()
                .with_max_requests((config.limits.rate_limit_requests_per_second * 60) as u64)
                .with_window_secs(60),
        ));

        // Create webhook delivery with default secret (none for now)
        let webhook_delivery = Some(WebhookDelivery::new(WebhookDeliveryConfig::default()));

        Self {
            config,
            subscription_manager,
            message_router,
            delivery_tracker,
            session_manager,
            auth_validator,
            rate_limiter,
            webhook_delivery,
        }
    }

    /// Creates a server for development.
    pub fn development() -> Self {
        Self::new(ServerConfig::development())
    }
}

impl<S, R, D, M, A, L> CauceServer<S, R, D, M, A, L>
where
    S: SubscriptionManager + 'static,
    R: MessageRouter + 'static,
    D: DeliveryTracker + 'static,
    M: SessionManager + 'static,
    A: AuthValidator + 'static,
    L: RateLimiter + 'static,
{
    /// Sets a custom subscription manager.
    pub fn with_subscription_manager<S2: SubscriptionManager>(
        self,
        manager: S2,
    ) -> CauceServer<S2, R, D, M, A, L> {
        CauceServer {
            config: self.config,
            subscription_manager: Arc::new(manager),
            message_router: self.message_router,
            delivery_tracker: self.delivery_tracker,
            session_manager: self.session_manager,
            auth_validator: self.auth_validator,
            rate_limiter: self.rate_limiter,
            webhook_delivery: self.webhook_delivery,
        }
    }

    /// Sets a custom message router.
    pub fn with_message_router<R2: MessageRouter>(
        self,
        router: R2,
    ) -> CauceServer<S, R2, D, M, A, L> {
        CauceServer {
            config: self.config,
            subscription_manager: self.subscription_manager,
            message_router: Arc::new(router),
            delivery_tracker: self.delivery_tracker,
            session_manager: self.session_manager,
            auth_validator: self.auth_validator,
            rate_limiter: self.rate_limiter,
            webhook_delivery: self.webhook_delivery,
        }
    }

    /// Sets a custom delivery tracker.
    pub fn with_delivery_tracker<D2: DeliveryTracker>(
        self,
        tracker: D2,
    ) -> CauceServer<S, R, D2, M, A, L> {
        CauceServer {
            config: self.config,
            subscription_manager: self.subscription_manager,
            message_router: self.message_router,
            delivery_tracker: Arc::new(tracker),
            session_manager: self.session_manager,
            auth_validator: self.auth_validator,
            rate_limiter: self.rate_limiter,
            webhook_delivery: self.webhook_delivery,
        }
    }

    /// Sets a custom session manager.
    pub fn with_session_manager<M2: SessionManager>(
        self,
        manager: M2,
    ) -> CauceServer<S, R, D, M2, A, L> {
        CauceServer {
            config: self.config,
            subscription_manager: self.subscription_manager,
            message_router: self.message_router,
            delivery_tracker: self.delivery_tracker,
            session_manager: Arc::new(manager),
            auth_validator: self.auth_validator,
            rate_limiter: self.rate_limiter,
            webhook_delivery: self.webhook_delivery,
        }
    }

    /// Sets a custom auth validator.
    pub fn with_auth_validator<A2: AuthValidator>(
        self,
        validator: A2,
    ) -> CauceServer<S, R, D, M, A2, L> {
        CauceServer {
            config: self.config,
            subscription_manager: self.subscription_manager,
            message_router: self.message_router,
            delivery_tracker: self.delivery_tracker,
            session_manager: self.session_manager,
            auth_validator: Arc::new(validator),
            rate_limiter: self.rate_limiter,
            webhook_delivery: self.webhook_delivery,
        }
    }

    /// Sets a custom rate limiter.
    pub fn with_rate_limiter<L2: RateLimiter>(
        self,
        limiter: L2,
    ) -> CauceServer<S, R, D, M, A, L2> {
        CauceServer {
            config: self.config,
            subscription_manager: self.subscription_manager,
            message_router: self.message_router,
            delivery_tracker: self.delivery_tracker,
            session_manager: self.session_manager,
            auth_validator: self.auth_validator,
            rate_limiter: Arc::new(limiter),
            webhook_delivery: self.webhook_delivery,
        }
    }

    /// Gets the server configuration.
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Gets the subscription manager.
    pub fn subscription_manager(&self) -> Arc<S> {
        Arc::clone(&self.subscription_manager)
    }

    /// Gets the message router.
    pub fn message_router(&self) -> Arc<R> {
        Arc::clone(&self.message_router)
    }

    /// Gets the delivery tracker.
    pub fn delivery_tracker(&self) -> Arc<D> {
        Arc::clone(&self.delivery_tracker)
    }

    /// Gets the session manager.
    pub fn session_manager(&self) -> Arc<M> {
        Arc::clone(&self.session_manager)
    }

    /// Gets the webhook delivery handler.
    pub fn webhook_delivery(&self) -> Option<&WebhookDelivery> {
        self.webhook_delivery.as_ref()
    }

    /// Creates the axum Router for this server.
    pub fn router(&self) -> Router {
        let transports = &self.config.transports;

        // Create shared state
        let state = SharedState {
            subscription_manager: Arc::clone(&self.subscription_manager) as Arc<dyn SubscriptionManager>,
            message_router: Arc::clone(&self.message_router) as Arc<dyn MessageRouter>,
            delivery_tracker: Arc::clone(&self.delivery_tracker) as Arc<dyn DeliveryTracker>,
            session_manager: Arc::clone(&self.session_manager) as Arc<dyn SessionManager>,
        };

        let mut router = Router::new();

        // Add WebSocket handler
        if transports.websocket_enabled {
            let ws_handler = Arc::new(WebSocketHandler::new(
                Arc::clone(&self.subscription_manager),
                Arc::clone(&self.message_router),
                Arc::clone(&self.delivery_tracker),
                Arc::clone(&self.session_manager),
            ));

            router = router.route(
                "/cauce/v1/ws",
                get({
                    let handler = Arc::clone(&ws_handler);
                    move |ws| {
                        let h = Arc::clone(&handler);
                        async move { h.handle_upgrade(ws).await }
                    }
                }),
            );
        }

        // Add SSE handler
        if transports.sse_enabled {
            let sse_handler = Arc::new(SseHandler::new(
                Arc::clone(&self.subscription_manager),
                Arc::clone(&self.delivery_tracker),
                Arc::clone(&self.session_manager),
            ));

            router = router.route(
                "/cauce/v1/sse",
                get({
                    let handler = Arc::clone(&sse_handler);
                    move |query| {
                        let h = Arc::clone(&handler);
                        async move { h.handle_stream(query).await }
                    }
                }),
            );
        }

        // Add polling handlers
        if transports.polling_enabled {
            let polling_handler = Arc::new(PollingHandler::new(
                Arc::clone(&self.subscription_manager),
                Arc::clone(&self.delivery_tracker),
                Arc::clone(&self.session_manager),
            ));

            router = router
                .route(
                    "/cauce/v1/poll",
                    get({
                        let handler = Arc::clone(&polling_handler);
                        move |query| {
                            let h = Arc::clone(&handler);
                            async move { h.handle_poll(query).await }
                        }
                    }),
                )
                .route(
                    "/cauce/v1/ack",
                    post({
                        let handler = Arc::clone(&polling_handler);
                        move |query, body| {
                            let h = Arc::clone(&handler);
                            async move { h.handle_ack(query, body).await }
                        }
                    }),
                );
        }

        // Add health check endpoint
        router = router.route("/health", get(health_handler));

        // Add rate limiting middleware
        let rate_limiter = Arc::clone(&self.rate_limiter);
        let rate_limit_middleware = RateLimitMiddleware::with_shared(rate_limiter);

        // Add auth middleware if enabled
        let auth_enabled = self.config.auth.required;
        let auth_middleware = AuthMiddleware::with_shared(Arc::clone(&self.auth_validator));

        if auth_enabled {
            router = router.layer(auth_middleware.layer());
        }

        // Apply rate limiting
        router = router.layer(rate_limit_middleware.layer());

        router.with_state(state)
    }

    /// Starts the server and begins accepting connections.
    pub async fn serve(self) -> ServerResult<()> {
        let addr = self.config.address;
        let router = self.router();

        info!("Starting Cauce server on {}", addr);

        let listener = TcpListener::bind(addr).await.map_err(|e| ServerError::ConfigError {
            message: format!("Failed to bind to {}: {}", addr, e),
        })?;

        axum::serve(listener, router)
            .await
            .map_err(|e| ServerError::ConfigError {
                message: format!("Server error: {}", e),
            })?;

        Ok(())
    }

    /// Starts the server with graceful shutdown support.
    pub async fn serve_with_shutdown<F>(self, signal: F) -> ServerResult<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let addr = self.config.address;
        let router = self.router();

        info!("Starting Cauce server on {} (with graceful shutdown)", addr);

        let listener = TcpListener::bind(addr).await.map_err(|e| ServerError::ConfigError {
            message: format!("Failed to bind to {}: {}", addr, e),
        })?;

        axum::serve(listener, router)
            .with_graceful_shutdown(signal)
            .await
            .map_err(|e| ServerError::ConfigError {
                message: format!("Server error: {}", e),
            })?;

        info!("Server shut down gracefully");
        Ok(())
    }

    /// Returns the address the server will listen on.
    pub fn address(&self) -> SocketAddr {
        self.config.address
    }
}

/// Simple health check handler.
async fn health_handler() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_server_creation() {
        let config = ServerConfig::development();
        let server = DefaultCauceServer::new(config);

        assert_eq!(server.address(), "127.0.0.1:8080".parse().unwrap());
    }

    #[test]
    fn test_server_development() {
        let server = DefaultCauceServer::development();
        assert_eq!(server.address(), "127.0.0.1:8080".parse().unwrap());
    }

    #[test]
    fn test_server_accessors() {
        let server = DefaultCauceServer::development();

        // Verify we can access all components
        let _ = server.subscription_manager();
        let _ = server.message_router();
        let _ = server.delivery_tracker();
        let _ = server.session_manager();
        let _ = server.config();
    }

    #[test]
    fn test_router_creation() {
        let server = DefaultCauceServer::development();
        let _router = server.router();
    }

    #[test]
    fn test_with_custom_subscription_manager() {
        let server = DefaultCauceServer::development();
        let custom_manager = InMemorySubscriptionManager::default();
        let _server = server.with_subscription_manager(custom_manager);
    }

    #[test]
    fn test_with_custom_session_manager() {
        let server = DefaultCauceServer::development();
        let custom_manager = InMemorySessionManager::default();
        let _server = server.with_session_manager(custom_manager);
    }

    #[test]
    fn test_with_custom_delivery_tracker() {
        let server = DefaultCauceServer::development();
        let custom_tracker = InMemoryDeliveryTracker::new(crate::config::RedeliveryConfig::default());
        let _server = server.with_delivery_tracker(custom_tracker);
    }

    #[test]
    fn test_with_custom_message_router() {
        let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
        let custom_router = DefaultMessageRouter::new(subscription_manager);

        let server = DefaultCauceServer::development();
        let _server = server.with_message_router(custom_router);
    }

    #[test]
    fn test_with_custom_auth_validator() {
        let server = DefaultCauceServer::development();
        let custom_validator = InMemoryAuthValidator::new();
        let _server = server.with_auth_validator(custom_validator);
    }

    #[test]
    fn test_with_custom_rate_limiter() {
        let server = DefaultCauceServer::development();
        let custom_limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
        let _server = server.with_rate_limiter(custom_limiter);
    }

    #[test]
    fn test_webhook_delivery_accessor() {
        let server = DefaultCauceServer::development();
        let webhook = server.webhook_delivery();
        assert!(webhook.is_some());
    }

    #[test]
    fn test_server_config_accessor() {
        let config = ServerConfig::builder("0.0.0.0:9000".parse().unwrap())
            .build()
            .unwrap();
        let server = DefaultCauceServer::new(config);

        assert_eq!(server.config().address, "0.0.0.0:9000".parse().unwrap());
    }

    #[test]
    fn test_router_with_disabled_transports_fails() {
        use crate::config::TransportsConfig;

        // Config should fail when all transports are disabled
        let result = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
            .transports(TransportsConfig::none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_router_with_websocket_only() {
        use crate::config::TransportsConfig;

        let config = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
            .transports(TransportsConfig::websocket_only())
            .build()
            .unwrap();

        let server = DefaultCauceServer::new(config);
        let _router = server.router();
    }

    #[test]
    fn test_router_with_auth_required() {
        use crate::config::AuthConfig;

        let config = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
            .auth(AuthConfig::accept_bearer())
            .build()
            .unwrap();

        let server = DefaultCauceServer::new(config);
        let _router = server.router();
    }

    #[test]
    fn test_router_with_api_keys() {
        use crate::config::AuthConfig;

        let config = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
            .auth(AuthConfig::require_api_key(vec!["test_key_123".to_string()]))
            .build()
            .unwrap();

        let server = DefaultCauceServer::new(config);
        let _router = server.router();
    }

    #[tokio::test]
    async fn test_health_handler() {
        let result = health_handler().await;
        assert_eq!(result, "OK");
    }
}

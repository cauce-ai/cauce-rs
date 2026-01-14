//! Cauce Server SDK - Server-side infrastructure for the Cauce Protocol.
//!
//! This crate provides the building blocks for implementing Cauce Protocol hubs,
//! including transport handlers, subscription management, message routing,
//! and signal delivery tracking.
//!
//! # Quick Start
//!
//! ```ignore
//! use cauce_server_sdk::{CauceServer, ServerConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     let config = ServerConfig::builder(addr).build()?;
//!
//!     let server = CauceServer::new(config);
//!     server.serve().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! The server SDK is built around several core traits that allow customization:
//!
//! - [`SubscriptionManager`] - Manages client subscriptions to topics
//! - [`MessageRouter`] - Routes published messages to matching subscriptions
//! - [`DeliveryTracker`] - Tracks signal delivery and handles redelivery
//! - [`SessionManager`] - Manages client sessions and authentication state
//!
//! Default in-memory implementations are provided for all traits, suitable
//! for development and simple deployments. For production use, you can
//! implement these traits with persistent storage backends.
//!
//! # Transports
//!
//! The server supports multiple transport mechanisms:
//!
//! - **WebSocket** - Full-duplex bidirectional communication (recommended)
//! - **SSE (Server-Sent Events)** - Server-to-client streaming
//! - **HTTP Polling** - Short and long polling for environments without WebSocket
//! - **Webhook** - Server pushes signals to client-provided URLs
//!
//! # Example: Custom Components
//!
//! ```ignore
//! use cauce_server_sdk::{CauceServer, ServerConfig};
//! use cauce_server_sdk::subscription::InMemorySubscriptionManager;
//!
//! let config = ServerConfig::development();
//! let server = CauceServer::new(config)
//!     .with_subscription_manager(InMemorySubscriptionManager::new());
//! ```

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// Core modules
pub mod config;
pub mod error;

// Manager modules (to be implemented)
pub mod auth;
pub mod delivery;
pub mod rate_limit;
pub mod routing;
pub mod server;
pub mod session;
pub mod subscription;
pub mod transport;

// Re-export main types
pub use config::{
    AuthConfig, LimitsConfig, RedeliveryConfig, ServerConfig, ServerConfigBuilder,
    TransportsConfig,
};
pub use error::{ServerError, ServerResult};

// Re-export subscription types
pub use subscription::{InMemorySubscriptionManager, SubscriptionManager, TopicTrie};

// Re-export routing types
pub use routing::{DefaultMessageRouter, DeliveryResult, MessageRouter, RouteResult};

// Re-export delivery types
pub use delivery::{
    DeliveryStatus, DeliveryTracker, InMemoryDeliveryTracker, PendingDelivery, RedeliveryScheduler,
};

// Re-export session types
pub use session::{InMemorySessionManager, SessionInfo, SessionManager};

// Re-export auth types
pub use auth::{AuthInfo, AuthLayer, AuthMethod, AuthMiddleware, AuthResult, AuthValidator, InMemoryAuthValidator};

// Re-export rate limiting types
pub use rate_limit::{
    InMemoryRateLimiter, KeyExtractor, RateLimitConfig, RateLimitLayer, RateLimitMiddleware,
    RateLimitResult, RateLimitState, RateLimiter,
};

// Re-export server types
pub use server::{CauceServer, DefaultCauceServer, SharedState};

// Re-export commonly used types from cauce-core
pub use cauce_core::{
    // Client types
    Capability,
    ClientType,
    Transport as TransportType,
    // Error types
    CauceError,
    JsonRpcError,
    // JSON-RPC types
    JsonRpcNotification,
    JsonRpcRequest,
    JsonRpcResponse,
    RequestId,
    // Method parameters
    AckRequest,
    AckResponse,
    HelloRequest,
    HelloResponse,
    PublishMessage,
    PublishRequest,
    PublishResponse,
    SignalDelivery,
    SubscribeRequest,
    SubscribeResponse,
    SubscriptionInfo,
    SubscriptionStatus,
    UnsubscribeRequest,
    UnsubscribeResponse,
    // Core types
    Action,
    Signal,
    Topic,
    // Method constants
    METHOD_ACK,
    METHOD_GOODBYE,
    METHOD_HELLO,
    METHOD_PING,
    METHOD_PONG,
    METHOD_PUBLISH,
    METHOD_SIGNAL,
    METHOD_SUBSCRIBE,
    METHOD_SUBSCRIPTION_APPROVE,
    METHOD_SUBSCRIPTION_DENY,
    METHOD_SUBSCRIPTION_LIST,
    METHOD_SUBSCRIPTION_REQUEST,
    METHOD_SUBSCRIPTION_REVOKE,
    METHOD_SUBSCRIPTION_STATUS,
    METHOD_UNSUBSCRIBE,
    // Protocol constants
    PROTOCOL_VERSION,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that key types are accessible
        let _: ServerResult<()> = Ok(());

        // Verify config types
        let _config = ServerConfig::development();

        // Verify re-exports from cauce-core
        assert_eq!(PROTOCOL_VERSION, "1.0");
        assert_eq!(METHOD_HELLO, "cauce.hello");
    }
}

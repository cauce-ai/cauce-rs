//! # cauce-client-sdk
//!
//! Async Rust client SDK for connecting to Cauce Protocol Hubs.
//!
//! This crate provides a unified API for connecting to Cauce Hubs using
//! various transport mechanisms (WebSocket, SSE, HTTP Polling, etc.) with
//! automatic reconnection and local message queuing for resilience.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cauce_client_sdk::{CauceClient, ClientConfig, AuthConfig};
//! use cauce_core::ClientType;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure the client
//!     let config = ClientConfig::builder("wss://hub.example.com", "my-agent")
//!         .client_type(ClientType::Agent)
//!         .auth(AuthConfig::bearer("my-token"))
//!         .build()?;
//!
//!     // Connect to the Hub
//!     let client = CauceClient::connect(config).await?;
//!
//!     // Subscribe to topics
//!     let mut subscription = client
//!         .subscribe_to(&["signal.email.*", "signal.slack.*"])
//!         .await?;
//!
//!     // Receive signals
//!     while let Some(signal) = subscription.next().await {
//!         println!("Received signal: {:?}", signal.id());
//!
//!         // Acknowledge the signal
//!         client.ack_signal(&subscription, &signal).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Multiple Transports**: WebSocket, SSE, HTTP Polling, Long Polling, Webhooks
//! - **Automatic Reconnection**: Exponential backoff with configurable parameters
//! - **Local Queuing**: Buffer messages when Hub is unavailable
//! - **Type-Safe API**: Leverages cauce-core types for protocol compliance
//!
//! ## Modules
//!
//! - [`config`] - Client configuration types
//! - [`transport`] - Transport trait and implementations
//! - [`error`] - Client error types

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod config;
pub mod error;
pub mod transport;

// =============================================================================
// Public API Re-exports
// =============================================================================

pub use config::{AuthConfig, ClientConfig, ClientConfigBuilder, ReconnectConfig, TlsConfig};
pub use error::ClientError;
pub use transport::{ConnectionState, JsonRpcMessage, Transport};

// Re-export commonly used types from cauce-core for convenience
pub use cauce_core::{
    // Enums
    Capability,
    ClientType,
    Transport as TransportType,
    // Errors
    CauceError,
    // JSON-RPC types
    JsonRpcError,
    JsonRpcNotification,
    JsonRpcRequest,
    JsonRpcResponse,
    RequestId,
    // Protocol types
    Action,
    Signal,
    Topic,
    // Method types
    AckRequest,
    AckResponse,
    HelloRequest,
    HelloResponse,
    PublishRequest,
    PublishResponse,
    SubscribeRequest,
    SubscribeResponse,
    UnsubscribeRequest,
    UnsubscribeResponse,
};

/// Result type for client operations.
pub type ClientResult<T> = Result<T, ClientError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that key types are accessible
        let _ = std::any::type_name::<ClientConfig>();
        let _ = std::any::type_name::<ClientError>();
        let _ = std::any::type_name::<ConnectionState>();
    }
}

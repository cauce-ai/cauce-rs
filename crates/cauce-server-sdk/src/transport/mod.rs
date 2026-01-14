//! Transport handlers for the Cauce server.
//!
//! This module provides handlers for different transport mechanisms:
//! - WebSocket (full-duplex)
//! - SSE (Server-Sent Events)
//! - HTTP Polling (short and long)
//! - Webhook (server push)
//!
//! # Architecture
//!
//! Each transport handler integrates with axum and provides routes that can be
//! composed with your application's router.
//!
//! # Example
//!
//! ```ignore
//! use axum::Router;
//! use cauce_server_sdk::transport::websocket::WebSocketHandler;
//!
//! let handler = WebSocketHandler::new(/* components */);
//! let app = Router::new()
//!     .nest("/cauce/v1", handler.routes());
//! ```

mod message;
mod polling;
mod sse;
mod webhook;
mod websocket;

pub use message::JsonRpcMessage;
pub use polling::{AckQuery, ErrorResponse, PollQuery, PollResponse, PollSignal, PollingHandler};
pub use sse::{SseHandler, SseQuery, SseSignalEvent};
pub use webhook::{WebhookDelivery, WebhookDeliveryConfig, WebhookDeliveryResult};
pub use websocket::{WebSocketConnection, WebSocketHandler};

use crate::error::ServerResult;
use async_trait::async_trait;
use cauce_core::SignalDelivery;

/// Trait for signal delivery to connected clients.
///
/// This abstracts the mechanism for sending signals to clients
/// over different transports.
#[async_trait]
pub trait SignalSender: Send + Sync {
    /// Send a signal delivery to the client.
    async fn send_signal(&self, delivery: SignalDelivery) -> ServerResult<()>;

    /// Check if the sender is still connected/valid.
    fn is_connected(&self) -> bool;
}

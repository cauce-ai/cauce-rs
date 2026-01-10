//! Method parameter types for the Cauce Protocol JSON-RPC methods.
//!
//! This module provides request and response types for all Cauce Protocol methods:
//!
//! - Connection lifecycle: [`HelloRequest`], [`HelloResponse`]
//! - Subscription management: [`SubscribeRequest`], [`UnsubscribeRequest`]
//! - Publishing: [`PublishRequest`], [`PublishResponse`]
//! - Acknowledgment: [`AckRequest`], [`AckResponse`]
//! - Ping/Pong: [`PingParams`], [`PongParams`]
//! - Schema discovery: [`SchemasListRequest`], [`SchemasGetRequest`]

// Foundational types (shared enums)
mod auth;
mod client;
mod enums;
mod transport;

// Method-specific types
mod ack;
mod hello;
mod ping;
mod publish;
mod schemas;
mod signal_delivery;
mod subscribe;
mod subscription;
mod unsubscribe;

// Re-export foundational types
pub use auth::{Auth, AuthType};
pub use client::{Capability, ClientType};
pub use enums::{ApprovalType, SubscriptionStatus};
pub use transport::{E2eConfig, Transport, WebhookConfig};

// Re-export method types
pub use ack::{AckFailure, AckRequest, AckResponse};
pub use hello::{HelloRequest, HelloResponse};
pub use ping::{PingParams, PongParams};
pub use publish::{PublishMessage, PublishRequest, PublishResponse};
pub use schemas::{
    SchemaInfo, SchemasGetRequest, SchemasGetResponse, SchemasListRequest, SchemasListResponse,
};
pub use signal_delivery::SignalDelivery;
pub use subscribe::{SubscribeRequest, SubscribeResponse};
pub use subscription::{
    SubscriptionApproveRequest, SubscriptionDenyRequest, SubscriptionInfo, SubscriptionListRequest,
    SubscriptionListResponse, SubscriptionRestrictions, SubscriptionRevokeRequest,
    SubscriptionStatusNotification,
};
pub use unsubscribe::{UnsubscribeRequest, UnsubscribeResponse};

//! # cauce-core
//!
//! Core types and utilities for the Cauce Protocol.
//!
//! This crate provides the foundational types, validation utilities, and constants
//! used by all other Cauce crates (client-sdk, server-sdk, hub).
//!
//! ## Modules
//!
//! - [`types`] - Protocol types (Signal, Action, Topic, etc.)
//! - [`jsonrpc`] - JSON-RPC 2.0 request/response types
//! - [`methods`] - Method parameter types for JSON-RPC methods
//! - [`validation`] - Validation utilities and schema validation
//! - [`errors`] - Error types and error codes
//! - [`constants`] - Protocol constants (method names, limits)
//! - [`id`] - ID generation utilities
//! - [`matching`] - Topic pattern matching
//! - [`schemas`] - Embedded JSON schemas

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// Module declarations
pub mod builders;
pub mod constants;
pub mod errors;
pub mod id;
pub mod jsonrpc;
pub mod matching;
pub mod methods;
pub mod schemas;
pub mod types;
pub mod validation;

// =============================================================================
// Core Type Re-exports
// =============================================================================

// Re-exports for ergonomic imports
// Users can write `use cauce_core::Signal;` instead of `use cauce_core::types::Signal;`
pub use types::{
    Action, ActionBody, ActionContext, ActionType, Encrypted, EncryptionAlgorithm, Metadata,
    Payload, Priority, Signal, Source, Topic,
};

// =============================================================================
// Error Re-exports
// =============================================================================

// Re-export errors
pub use errors::{BuilderError, CauceError, ValidationError};

// =============================================================================
// Method Type Re-exports
// =============================================================================

// Authentication
pub use methods::{Auth, AuthType};

// Client
pub use methods::{Capability, ClientType};

// Transport
pub use methods::{E2eConfig, Transport, WebhookConfig};

// Enums
pub use methods::{ApprovalType, SubscriptionStatus};

// Hello
pub use methods::{HelloRequest, HelloResponse};

// Subscribe/Unsubscribe
pub use methods::{SubscribeRequest, SubscribeResponse};
pub use methods::{UnsubscribeRequest, UnsubscribeResponse};

// Publish
pub use methods::{PublishMessage, PublishRequest, PublishResponse};

// Ack
pub use methods::{AckFailure, AckRequest, AckResponse};

// Subscription Management
pub use methods::{
    SubscriptionApproveRequest, SubscriptionDenyRequest, SubscriptionInfo, SubscriptionListRequest,
    SubscriptionListResponse, SubscriptionRestrictions, SubscriptionRevokeRequest,
    SubscriptionStatusNotification,
};

// Ping/Pong
pub use methods::{PingParams, PongParams};

// Signal Delivery
pub use methods::SignalDelivery;

// Schemas
pub use methods::{
    SchemaInfo, SchemasGetRequest, SchemasGetResponse, SchemasListRequest, SchemasListResponse,
};

// =============================================================================
// Validation Re-exports
// =============================================================================

// Re-export validation functions
pub use validation::{
    is_valid_action_id, is_valid_message_id, is_valid_session_id, is_valid_signal_id,
    is_valid_subscription_id, is_valid_topic, validate_action, validate_signal,
    validate_topic_pattern,
};

// =============================================================================
// Constant Re-exports
// =============================================================================

// ID patterns and prefixes
pub use constants::{
    ACTION_ID_PATTERN, ACTION_ID_PREFIX, ID_RANDOM_LENGTH, MESSAGE_ID_PATTERN, PROTOCOL_VERSION,
    SESSION_ID_PATTERN, SIGNAL_ID_PATTERN, SIGNAL_ID_PREFIX, SUBSCRIPTION_ID_PATTERN,
    TOPIC_ALLOWED_CHARS, TOPIC_MAX_LENGTH, TOPIC_MIN_LENGTH,
};

// Method name constants
pub use constants::{
    METHOD_ACK, METHOD_GOODBYE, METHOD_HELLO, METHOD_PING, METHOD_PONG, METHOD_PUBLISH,
    METHOD_SCHEMAS_GET, METHOD_SCHEMAS_LIST, METHOD_SIGNAL, METHOD_SUBSCRIBE,
    METHOD_SUBSCRIPTION_APPROVE, METHOD_SUBSCRIPTION_DENY, METHOD_SUBSCRIPTION_LIST,
    METHOD_SUBSCRIPTION_REQUEST, METHOD_SUBSCRIPTION_REVOKE, METHOD_SUBSCRIPTION_STATUS,
    METHOD_UNSUBSCRIBE,
};

// Size limit constants
pub use constants::{
    MAX_SIGNALS_PER_BATCH, MAX_SIGNAL_PAYLOAD_SIZE, MAX_SUBSCRIPTIONS_PER_CLIENT,
    MAX_TOPICS_PER_SUBSCRIPTION, MAX_TOPIC_DEPTH, MAX_TOPIC_LENGTH,
};

// =============================================================================
// ID Generation Re-exports
// =============================================================================

pub use id::{
    generate_action_id, generate_message_id, generate_session_id, generate_signal_id,
    generate_subscription_id,
};

// =============================================================================
// Topic Matching Re-exports
// =============================================================================

pub use matching::{topic_matches, TopicMatcher};

// =============================================================================
// Builder Re-exports
// =============================================================================

// Re-export builders
pub use builders::{ActionBuilder, SignalBuilder};

// =============================================================================
// JSON-RPC Re-exports
// =============================================================================

// Re-export JSON-RPC types
pub use jsonrpc::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId, JSONRPC_VERSION,
};

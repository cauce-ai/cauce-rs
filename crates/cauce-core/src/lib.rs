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
//! - [`validation`] - Validation utilities and schema validation
//! - [`errors`] - Error types and error codes
//! - [`constants`] - Protocol constants (method names, limits)

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// Module declarations
pub mod builders;
pub mod constants;
pub mod errors;
pub mod jsonrpc;
pub mod types;
pub mod validation;

// Re-exports for ergonomic imports
// Users can write `use cauce_core::Signal;` instead of `use cauce_core::types::Signal;`
pub use types::{
    Action, ActionBody, ActionContext, ActionType, Encrypted, EncryptionAlgorithm, Metadata,
    Payload, Priority, Signal, Source, Topic,
};

// Re-export errors
pub use errors::{BuilderError, ValidationError};

// Re-export validation functions
pub use validation::{is_valid_action_id, is_valid_signal_id, is_valid_topic};

// Re-export constants
pub use constants::{
    ACTION_ID_PATTERN, ACTION_ID_PREFIX, ID_RANDOM_LENGTH, PROTOCOL_VERSION, SIGNAL_ID_PATTERN,
    SIGNAL_ID_PREFIX, TOPIC_ALLOWED_CHARS, TOPIC_MAX_LENGTH, TOPIC_MIN_LENGTH,
};

// Re-export builders
pub use builders::{ActionBuilder, SignalBuilder};

// Re-export JSON-RPC types
pub use jsonrpc::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId, JSONRPC_VERSION,
};

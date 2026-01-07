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
pub mod constants;
pub mod errors;
pub mod jsonrpc;
pub mod types;
pub mod validation;

// Re-exports for ergonomic imports
// Users can write `use cauce_core::Signal;` instead of `use cauce_core::types::Signal;`
pub use types::{Action, Signal, Topic};

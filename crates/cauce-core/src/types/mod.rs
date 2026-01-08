//! Protocol types for the Cauce Protocol.
//!
//! This module contains the core protocol types used throughout the Cauce ecosystem:
//!
//! - [`Signal`] - Messages sent from adapters to hubs
//! - [`Action`] - Commands sent from agents to adapters
//! - [`Topic`] - Validated hierarchical topic identifiers for pub/sub routing
//! - [`Source`] - Origin information for signals
//! - [`Payload`] - Message content with type information
//! - [`Metadata`] - Optional threading and priority information
//! - [`Encrypted`] - End-to-end encryption envelope
//! - [`Priority`] - Message priority levels
//! - [`ActionType`] - Types of actions that can be performed
//! - [`ActionBody`] - Details of an action to be performed
//! - [`ActionContext`] - Correlation and threading information for actions
//! - [`EncryptionAlgorithm`] - Supported E2E encryption algorithms

// Submodules
mod action;
mod encrypted;
mod enums;
mod metadata;
mod payload;
mod signal;
mod source;
mod topic;

// Re-export all public types
pub use action::{Action, ActionBody, ActionContext};
pub use encrypted::{Encrypted, EncryptionAlgorithm};
pub use enums::{ActionType, Priority};
pub use metadata::Metadata;
pub use payload::Payload;
pub use signal::Signal;
pub use source::Source;
pub use topic::Topic;

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "types: Protocol type definitions"
}

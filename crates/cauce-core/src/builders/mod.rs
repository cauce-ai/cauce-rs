//! Builder types for the Cauce Protocol.
//!
//! This module provides ergonomic builder patterns for creating
//! [`Signal`](crate::types::Signal) and [`Action`](crate::types::Action) instances.
//!
//! ## Overview
//!
//! - [`SignalBuilder`] - Fluent builder for creating Signal instances
//! - [`ActionBuilder`] - Fluent builder for creating Action instances
//!
//! ## Example
//!
//! ```
//! use cauce_core::builders::SignalBuilder;
//! use cauce_core::types::{Source, Payload, Topic};
//! use chrono::Utc;
//! use serde_json::json;
//!
//! let signal = SignalBuilder::new()
//!     .source(Source::new("email", "email-1", "msg-1"))
//!     .topic(Topic::new_unchecked("signal.email.received"))
//!     .payload(Payload::new(json!({}), "application/json"))
//!     .build()
//!     .expect("valid signal");
//! ```

mod action_builder;
mod signal_builder;

pub use action_builder::ActionBuilder;
pub use signal_builder::SignalBuilder;

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "builders: Builder patterns for Signal and Action"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_info() {
        assert!(module_info().contains("builders"));
    }
}

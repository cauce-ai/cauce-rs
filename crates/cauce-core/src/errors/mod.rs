//! Error types for the Cauce Protocol.
//!
//! This module provides error types and error codes:
//!
//! - **CauceError** - Main error enum for all Cauce operations (future)
//! - **ErrorCode** - Standardized error codes for protocol errors
//! - **Result type** - Convenient Result alias for Cauce operations
//!
//! ## Error Categories
//!
//! Errors are categorized by their source:
//! - Protocol errors (invalid messages, schema violations)
//! - Transport errors (connection, timeout)
//! - Validation errors (invalid fields, missing required data)
//!
//! ## Future Implementation
//!
//! The actual error implementations will be added in subsequent features (TODO.md 2.4).

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "errors: Error types and error codes"
}

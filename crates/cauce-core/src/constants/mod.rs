//! Protocol constants for the Cauce Protocol.
//!
//! This module provides protocol-wide constants:
//!
//! - **Method names** - Standard JSON-RPC method names (future)
//! - **Limits** - Protocol limits (max message size, topic depth, etc.)
//! - **Defaults** - Default values for optional parameters
//! - **Version** - Protocol version information
//!
//! ## Usage
//!
//! Constants are used throughout the codebase to ensure consistency
//! and make protocol limits configurable in one place.
//!
//! ## Future Implementation
//!
//! The actual constant definitions will be added in subsequent features (TODO.md 2.5).

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "constants: Protocol constants (method names, limits)"
}

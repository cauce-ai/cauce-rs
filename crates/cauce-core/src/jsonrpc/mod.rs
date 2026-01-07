//! JSON-RPC 2.0 types for the Cauce Protocol.
//!
//! This module provides JSON-RPC 2.0 compliant request and response types:
//!
//! - **Request** - JSON-RPC 2.0 request structure (future implementation)
//! - **Response** - JSON-RPC 2.0 response structure (future implementation)
//! - **Error** - JSON-RPC 2.0 error object (future implementation)
//! - **Notification** - JSON-RPC 2.0 notification (no response expected)
//!
//! ## Compliance
//!
//! All types conform to the JSON-RPC 2.0 specification per Constitution Principle V.
//!
//! ## Future Implementation
//!
//! The actual type definitions will be added in subsequent features (TODO.md 2.3).

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "jsonrpc: JSON-RPC 2.0 request/response types"
}

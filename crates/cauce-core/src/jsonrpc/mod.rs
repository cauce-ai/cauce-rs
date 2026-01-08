//! JSON-RPC 2.0 types for the Cauce Protocol.
//!
//! This module provides JSON-RPC 2.0 compliant message types for the Cauce Protocol:
//!
//! - [`RequestId`] - Request identifier (string or integer)
//! - [`JsonRpcRequest`] - JSON-RPC 2.0 request with id, method, and optional params
//! - [`JsonRpcResponse`] - JSON-RPC 2.0 response (success with result or error)
//! - [`JsonRpcNotification`] - JSON-RPC 2.0 notification (no response expected)
//! - [`JsonRpcError`] - JSON-RPC 2.0 error object with code, message, and optional data
//!
//! ## Compliance
//!
//! All types conform to the JSON-RPC 2.0 specification per Constitution Principle V.
//!
//! ## Examples
//!
//! ```
//! use cauce_core::jsonrpc::{JsonRpcRequest, JsonRpcResponse, RequestId};
//! use serde_json::json;
//!
//! // Create a request
//! let request = JsonRpcRequest::new(
//!     RequestId::from_number(1),
//!     "cauce.subscribe",
//!     Some(json!({"topics": ["signal.email.*"]})),
//! );
//!
//! // Create a success response
//! let response = JsonRpcResponse::success(
//!     RequestId::from_number(1),
//!     json!({"subscription_id": "sub_123"}),
//! );
//! ```

mod error;
mod id;
mod notification;
mod request;
mod response;

pub use error::JsonRpcError;
pub use id::RequestId;
pub use notification::JsonRpcNotification;
pub use request::{JsonRpcRequest, JSONRPC_VERSION};
pub use response::JsonRpcResponse;

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "jsonrpc: JSON-RPC 2.0 request/response types"
}

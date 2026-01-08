# Quickstart: JSON-RPC Types

**Feature**: 006-jsonrpc-types
**Date**: 2026-01-07

## Overview

This quickstart shows how to use the JSON-RPC 2.0 types in cauce-core for building Cauce Protocol clients and servers.

## Creating Requests

```rust
use cauce_core::jsonrpc::{JsonRpcRequest, RequestId};
use serde_json::json;

// Create a request with numeric id
let request = JsonRpcRequest::new(
    RequestId::Number(1),
    "cauce.subscribe",
    Some(json!({
        "topics": ["signal.email.*"]
    })),
);

// Serialize to JSON
let json_str = serde_json::to_string(&request)?;
// {"jsonrpc":"2.0","id":1,"method":"cauce.subscribe","params":{"topics":["signal.email.*"]}}

// Create a request with string id
let request = JsonRpcRequest::new(
    RequestId::String("req-abc-123".to_string()),
    "cauce.publish",
    None,  // No params
);
```

## Creating Notifications

```rust
use cauce_core::jsonrpc::JsonRpcNotification;
use serde_json::json;

// Notifications have no id (no response expected)
let notification = JsonRpcNotification::new(
    "cauce.signal",
    Some(json!({
        "topic": "signal.email.received",
        "signal": { "id": "sig_123_abc" }
    })),
);

// Serialize - note no "id" field
let json_str = serde_json::to_string(&notification)?;
// {"jsonrpc":"2.0","method":"cauce.signal","params":{...}}
```

## Parsing Responses

```rust
use cauce_core::jsonrpc::JsonRpcResponse;

// Parse a success response
let json = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
let response: JsonRpcResponse = serde_json::from_str(json)?;

match response {
    JsonRpcResponse::Success { id, result, .. } => {
        println!("Request {:?} succeeded with: {}", id, result);
    }
    JsonRpcResponse::Error { id, error, .. } => {
        println!("Request {:?} failed: {} (code {})", id, error.message, error.code);
    }
}
```

## Creating Success Responses

```rust
use cauce_core::jsonrpc::{JsonRpcResponse, RequestId};
use serde_json::json;

// Helper method for success responses
let response = JsonRpcResponse::success(
    RequestId::Number(1),
    json!({
        "subscription_id": "sub_abc123",
        "status": "active"
    }),
);
```

## Creating Error Responses

```rust
use cauce_core::jsonrpc::{JsonRpcResponse, JsonRpcError, RequestId};
use serde_json::json;

// Create an error object
let error = JsonRpcError::new(-32602, "Invalid params");

// Or with additional data
let error = JsonRpcError::with_data(
    -32602,
    "Invalid params",
    json!({
        "field": "topics",
        "reason": "must be non-empty array"
    }),
);

// Create error response
let response = JsonRpcResponse::error(
    Some(RequestId::Number(1)),
    error,
);

// For parse errors where request id is unknown
let response = JsonRpcResponse::error(
    None,  // null id
    JsonRpcError::new(-32700, "Parse error"),
);
```

## Standard Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Not a valid JSON-RPC request |
| -32601 | Method not found | Method doesn't exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal JSON-RPC error |

Note: Cauce-specific error codes (like `-32001 SubscriptionNotFound`) are defined in a separate feature (2.5 Error Codes).

## Validation

Invalid messages are rejected during deserialization:

```rust
// Wrong version - fails to parse
let bad_json = r#"{"jsonrpc":"1.0","id":1,"method":"test"}"#;
let result: Result<JsonRpcRequest, _> = serde_json::from_str(bad_json);
assert!(result.is_err());

// Both result and error - fails to parse
let bad_json = r#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-1,"message":""}}"#;
let result: Result<JsonRpcResponse, _> = serde_json::from_str(bad_json);
assert!(result.is_err());
```

## Type Re-exports

All types are re-exported at the crate root for convenience:

```rust
// Either import from module
use cauce_core::jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, JsonRpcError, RequestId};

// Or from crate root
use cauce_core::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, JsonRpcError, RequestId};
```

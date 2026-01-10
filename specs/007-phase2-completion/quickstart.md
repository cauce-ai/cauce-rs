# Quickstart: Phase 2 Completion - Cauce Core Library

**Date**: 2026-01-08
**Feature**: 007-phase2-completion

## Prerequisites

- Rust 1.75+ installed
- Repository cloned and on `007-phase2-completion` branch

## Build & Test

```bash
# Build the workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Run tests with coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace --lcov --output-path lcov.info --fail-under-lines 95
```

## Usage Examples

### Method Parameter Types

```rust
use cauce_core::{
    HelloRequest, HelloResponse, ClientType, Capability, Auth, AuthType,
    SubscribeRequest, Transport, ApprovalType,
};
use chrono::Utc;

// Create a hello request
let hello = HelloRequest {
    protocol_version: "1.0".to_string(),
    min_protocol_version: None,
    max_protocol_version: None,
    client_id: "my-agent".to_string(),
    client_type: ClientType::Agent,
    capabilities: vec![Capability::Subscribe, Capability::Publish],
    auth: Some(Auth {
        type_: AuthType::ApiKey,
        token: None,
        api_key: Some("my-api-key".to_string()),
    }),
};

// Serialize to JSON
let json = serde_json::to_string(&hello)?;

// Create a subscribe request
let subscribe = SubscribeRequest {
    topics: vec!["signal.email.*".to_string(), "signal.slack.**".to_string()],
    approval_type: Some(ApprovalType::Automatic),
    reason: Some("Monitor communications".to_string()),
    transport: Some(Transport::WebSocket),
    webhook: None,
    e2e: None,
};
```

### Error Handling

```rust
use cauce_core::{CauceError, JsonRpcError};

// Create a protocol error
let error = CauceError::RateLimited { retry_after_ms: 5000 };

// Get error code
assert_eq!(error.code(), -32006);

// Convert to JSON-RPC error
let rpc_error: JsonRpcError = error.into();
let json = serde_json::to_string(&rpc_error)?;
// {"code":-32006,"message":"Rate limited","data":{"retry_after_ms":5000}}
```

### ID Generation

```rust
use cauce_core::id::{
    generate_signal_id, generate_action_id, generate_subscription_id,
    generate_session_id, generate_message_id,
};

let signal_id = generate_signal_id();
// sig_1704672000_a1b2c3d4e5f6

let sub_id = generate_subscription_id();
// sub_550e8400-e29b-41d4-a716-446655440000

// IDs are always unique
assert_ne!(generate_signal_id(), generate_signal_id());
```

### Topic Matching

```rust
use cauce_core::matching::{TopicMatcher, topic_matches};

// Single segment wildcard
assert!(topic_matches("signal.email", "signal.*"));
assert!(!topic_matches("signal.email.received", "signal.*"));

// Multi-segment wildcard
assert!(topic_matches("signal.email.received", "signal.**"));
assert!(topic_matches("signal.email.inbox.unread", "signal.**"));

// Trailing wildcard
assert!(topic_matches("signal.email.received", "**.received"));

// Check against multiple patterns
let patterns = &["signal.email.*", "signal.slack.**"];
assert!(TopicMatcher::matches_any("signal.email.sent", patterns));
```

### Validation

```rust
use cauce_core::validation::{
    validate_signal, validate_topic_pattern, is_valid_subscription_id,
};
use serde_json::json;

// Validate topic pattern with wildcards
assert!(validate_topic_pattern("signal.*").is_ok());
assert!(validate_topic_pattern("signal.**").is_ok());
assert!(validate_topic_pattern("*foo").is_err()); // Invalid: wildcard mixed with text

// Validate IDs
assert!(is_valid_subscription_id("sub_550e8400-e29b-41d4-a716-446655440000").is_ok());
assert!(is_valid_subscription_id("invalid").is_err());

// Validate signal JSON against schema
let signal_json = json!({
    "id": "sig_1704672000_a1b2c3d4e5f6",
    "version": "1.0",
    "timestamp": "2024-01-08T12:00:00Z",
    "source": { "type": "email", "adapter_id": "gmail", "native_id": "123" },
    "topic": "signal.email.received",
    "payload": { "raw": "Hello", "content_type": "text/plain", "size_bytes": 5 }
});
let signal = validate_signal(&signal_json)?;
```

### Using Constants

```rust
use cauce_core::constants::{
    METHOD_SUBSCRIBE, METHOD_PUBLISH, METHOD_ACK,
    MAX_TOPICS_PER_SUBSCRIPTION, MAX_SIGNAL_PAYLOAD_SIZE,
};

// Build JSON-RPC request with method constant
use cauce_core::{JsonRpcRequest, RequestId};
use serde_json::json;

let request = JsonRpcRequest::new(
    RequestId::from_number(1),
    METHOD_SUBSCRIBE,
    Some(json!({
        "topics": ["signal.email.*"],
    })),
);

// Check limits
fn validate_subscription_topics(topics: &[String]) -> Result<(), String> {
    if topics.len() > MAX_TOPICS_PER_SUBSCRIPTION {
        return Err(format!("Too many topics (max {})", MAX_TOPICS_PER_SUBSCRIPTION));
    }
    Ok(())
}
```

## Module Structure

```
cauce_core
├── types          # Core types (Signal, Action, Topic) - existing
├── jsonrpc        # JSON-RPC 2.0 types - existing
├── methods        # NEW: Method parameter types
├── errors         # Enhanced: CauceError enum
├── constants      # Enhanced: Method names, size limits
├── validation     # Enhanced: Schema validation, ID validation
├── id             # NEW: ID generation utilities
├── matching       # NEW: Topic pattern matching
└── builders       # Builders - existing
```

## Running Tests

```bash
# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run specific test module
cargo test methods::hello

# Run tests with output
cargo test -- --nocapture
```

## Coverage Report

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate HTML report
cargo llvm-cov --workspace --html

# Open report
open target/llvm-cov/html/index.html
```

## Common Tasks

### Adding a New Method Type

1. Create file in `src/methods/` (e.g., `new_method.rs`)
2. Define request/response structs with serde derives
3. Add `pub mod new_method;` to `src/methods/mod.rs`
4. Re-export types in `src/methods/mod.rs`
5. Add re-exports to `src/lib.rs`
6. Write tests in the module

### Adding a New Error Code

1. Add variant to `CauceError` enum in `src/errors/protocol.rs`
2. Add match arm in `impl From<CauceError> for JsonRpcError`
3. Add test for error code and message
4. Update documentation

### Adding ID Validation

1. Add validation function in `src/validation/mod.rs`
2. Add regex pattern to constants if needed
3. Re-export from `src/lib.rs`
4. Write tests for valid/invalid cases

# Public API Contract: cauce-core Phase 2

**Date**: 2026-01-08
**Feature**: 007-phase2-completion

This document defines the public API surface for the cauce-core library additions in Phase 2.

---

## Module: `cauce_core::methods`

### Re-exports (from `lib.rs`)

```rust
// Authentication
pub use methods::{Auth, AuthType};

// Client
pub use methods::{ClientType, Capability};

// Transport
pub use methods::{Transport, WebhookConfig, E2eConfig};

// Enums
pub use methods::{ApprovalType, SubscriptionStatus};

// Hello
pub use methods::{HelloRequest, HelloResponse};

// Subscribe/Unsubscribe
pub use methods::{SubscribeRequest, SubscribeResponse};
pub use methods::{UnsubscribeRequest, UnsubscribeResponse};

// Publish
pub use methods::{PublishRequest, PublishResponse, PublishMessage};

// Ack
pub use methods::{AckRequest, AckResponse, AckFailure};

// Subscription Management
pub use methods::{
    SubscriptionApproveRequest, SubscriptionDenyRequest, SubscriptionRevokeRequest,
    SubscriptionListRequest, SubscriptionListResponse, SubscriptionInfo,
    SubscriptionRestrictions, SubscriptionStatusNotification,
};

// Ping/Pong
pub use methods::{PingParams, PongParams};

// Signal Delivery
pub use methods::SignalDelivery;

// Schemas
pub use methods::{SchemasListRequest, SchemasListResponse, SchemasGetRequest, SchemasGetResponse, SchemaInfo};
```

---

## Module: `cauce_core::errors`

### CauceError Enum

```rust
pub enum CauceError {
    // JSON-RPC Standard (-327xx)
    ParseError { message: String },
    InvalidRequest { message: String },
    MethodNotFound { method: String },
    InvalidParams { message: String },
    InternalError { message: String },

    // Cauce Protocol (-320xx)
    SubscriptionNotFound { id: String },
    TopicNotFound { topic: String },
    NotAuthorized { reason: String },
    SubscriptionPending { id: String },
    SubscriptionDenied { id: String, reason: Option<String> },
    RateLimited { retry_after_ms: u64 },
    SignalTooLarge { size: usize, max: usize },
    EncryptionRequired { topic: String },
    InvalidEncryption { reason: String },
    AdapterUnavailable { adapter: String },
    DeliveryFailed { signal_id: String, reason: String },
    QueueFull { capacity: usize },
    SessionExpired { session_id: String },
    UnsupportedTransport { transport: String },
    InvalidTopic { topic: String, reason: String },
}
```

### Trait Implementations

```rust
impl std::error::Error for CauceError {}
impl std::fmt::Display for CauceError {}
impl From<CauceError> for JsonRpcError {}
```

### Helper Methods

```rust
impl CauceError {
    pub fn code(&self) -> i32;
    pub fn message(&self) -> &'static str;
    pub fn to_json_rpc_error(&self) -> JsonRpcError;
}
```

---

## Module: `cauce_core::constants`

### Method Names

```rust
pub const METHOD_HELLO: &str = "cauce.hello";
pub const METHOD_GOODBYE: &str = "cauce.goodbye";
pub const METHOD_PING: &str = "cauce.ping";
pub const METHOD_PONG: &str = "cauce.pong";
pub const METHOD_PUBLISH: &str = "cauce.publish";
pub const METHOD_SUBSCRIBE: &str = "cauce.subscribe";
pub const METHOD_UNSUBSCRIBE: &str = "cauce.unsubscribe";
pub const METHOD_SIGNAL: &str = "cauce.signal";
pub const METHOD_ACK: &str = "cauce.ack";
pub const METHOD_SUBSCRIPTION_REQUEST: &str = "cauce.subscription.request";
pub const METHOD_SUBSCRIPTION_APPROVE: &str = "cauce.subscription.approve";
pub const METHOD_SUBSCRIPTION_DENY: &str = "cauce.subscription.deny";
pub const METHOD_SUBSCRIPTION_LIST: &str = "cauce.subscription.list";
pub const METHOD_SUBSCRIPTION_REVOKE: &str = "cauce.subscription.revoke";
pub const METHOD_SUBSCRIPTION_STATUS: &str = "cauce.subscription.status";
pub const METHOD_SCHEMAS_LIST: &str = "cauce.schemas.list";
pub const METHOD_SCHEMAS_GET: &str = "cauce.schemas.get";
```

### Size Limits

```rust
pub const MAX_SIGNAL_PAYLOAD_SIZE: usize = 10 * 1024 * 1024; // 10 MB
pub const MAX_TOPICS_PER_SUBSCRIPTION: usize = 100;
pub const MAX_SUBSCRIPTIONS_PER_CLIENT: usize = 1000;
pub const MAX_SIGNALS_PER_BATCH: usize = 100;
```

---

## Module: `cauce_core::id`

### ID Generation Functions

```rust
/// Generate a unique Signal ID: `sig_{timestamp}_{random12}`
pub fn generate_signal_id() -> String;

/// Generate a unique Action ID: `act_{timestamp}_{random12}`
pub fn generate_action_id() -> String;

/// Generate a unique Subscription ID: `sub_{uuid}`
pub fn generate_subscription_id() -> String;

/// Generate a unique Session ID: `sess_{uuid}`
pub fn generate_session_id() -> String;

/// Generate a unique Message ID: `msg_{uuid}`
pub fn generate_message_id() -> String;
```

---

## Module: `cauce_core::matching`

### TopicMatcher

```rust
pub struct TopicMatcher;

impl TopicMatcher {
    /// Check if a topic matches a pattern with wildcards
    /// - `*` matches exactly one segment
    /// - `**` matches one or more segments
    pub fn matches(topic: &str, pattern: &str) -> bool;

    /// Check if a topic matches any of the given patterns
    pub fn matches_any(topic: &str, patterns: &[&str]) -> bool;
}
```

### Standalone Function

```rust
/// Convenience function for single pattern matching
pub fn topic_matches(topic: &str, pattern: &str) -> bool;
```

---

## Module: `cauce_core::validation`

### New Validation Functions

```rust
/// Validate a JSON value against the Signal schema
pub fn validate_signal(value: &serde_json::Value) -> Result<Signal, ValidationError>;

/// Validate a JSON value against the Action schema
pub fn validate_action(value: &serde_json::Value) -> Result<Action, ValidationError>;

/// Validate a topic pattern (allows wildcards)
pub fn validate_topic_pattern(pattern: &str) -> Result<(), ValidationError>;

/// Validate a subscription ID format
pub fn is_valid_subscription_id(id: &str) -> Result<(), ValidationError>;

/// Validate a session ID format
pub fn is_valid_session_id(id: &str) -> Result<(), ValidationError>;

/// Validate a message ID format
pub fn is_valid_message_id(id: &str) -> Result<(), ValidationError>;
```

### Enhanced ValidationError

```rust
pub enum ValidationError {
    // Existing variants...
    InvalidTopic { reason: String },
    InvalidSignalId { reason: String },
    InvalidActionId { reason: String },

    // New variants
    InvalidSubscriptionId { reason: String },
    InvalidSessionId { reason: String },
    InvalidMessageId { reason: String },
    InvalidTopicPattern { reason: String },
    SchemaValidation { errors: Vec<String> },
    Deserialization { message: String },
}
```

---

## Serde Serialization Contracts

All types implement `Serialize` and `Deserialize` with the following conventions:

### Enum Serialization

```rust
// snake_case for all enum variants
#[serde(rename_all = "snake_case")]
```

### Field Renaming

```rust
// Reserved keywords use trailing underscore in Rust, renamed in JSON
#[serde(rename = "type")]
pub type_: AuthType,
```

### Optional Fields

```rust
// Optional fields serialize as absent when None (not null)
#[serde(skip_serializing_if = "Option::is_none")]
pub expires_at: Option<DateTime<Utc>>,
```

### DateTime Format

```rust
// ISO 8601 format via chrono's default serialization
pub timestamp: DateTime<Utc>,
```

---

## Error Code Mapping

| Error Variant | Code | Standard Message |
|---------------|------|------------------|
| ParseError | -32700 | "Parse error" |
| InvalidRequest | -32600 | "Invalid request" |
| MethodNotFound | -32601 | "Method not found" |
| InvalidParams | -32602 | "Invalid params" |
| InternalError | -32603 | "Internal error" |
| SubscriptionNotFound | -32001 | "Subscription not found" |
| TopicNotFound | -32002 | "Topic not found" |
| NotAuthorized | -32003 | "Not authorized" |
| SubscriptionPending | -32004 | "Subscription pending approval" |
| SubscriptionDenied | -32005 | "Subscription denied" |
| RateLimited | -32006 | "Rate limited" |
| SignalTooLarge | -32007 | "Signal too large" |
| EncryptionRequired | -32008 | "Encryption required" |
| InvalidEncryption | -32009 | "Invalid encryption" |
| AdapterUnavailable | -32010 | "Adapter unavailable" |
| DeliveryFailed | -32011 | "Delivery failed" |
| QueueFull | -32012 | "Queue full" |
| SessionExpired | -32013 | "Session expired" |
| UnsupportedTransport | -32014 | "Unsupported transport" |
| InvalidTopic | -32015 | "Invalid topic" |

---

## Backward Compatibility

All additions are **backward compatible**:
- New modules do not modify existing types
- Existing `ValidationError` variants preserved
- Existing constants preserved
- New re-exports do not shadow existing ones

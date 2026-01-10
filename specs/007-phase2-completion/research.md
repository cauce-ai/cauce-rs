# Research: Phase 2 Completion - Cauce Core Library

**Date**: 2026-01-08
**Feature**: 007-phase2-completion
**Status**: Complete

## Overview

This document captures research findings for implementing the remaining Phase 2 items of cauce-core. Since the Technical Context had no NEEDS CLARIFICATION items, research focused on best practices and implementation patterns.

---

## 1. Serde Patterns for Protocol Types

### Decision
Use `#[serde(rename_all = "snake_case")]` for enums and `#[serde(rename = "type")]` for reserved Rust keywords like `type_`.

### Rationale
- Protocol specification uses snake_case for JSON field names
- Rust's `type` keyword requires renaming (convention: `type_` in Rust, `type` in JSON)
- Consistent with existing types in `types/` module

### Alternatives Considered
- **camelCase**: Rejected - protocol spec uses snake_case
- **Manual rename for each field**: Rejected - verbose and error-prone

### Implementation Notes
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Bearer,
    ApiKey,
    Mtls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    #[serde(rename = "type")]
    pub type_: AuthType,
    pub token: Option<String>,
    pub api_key: Option<String>,
}
```

---

## 2. Error Code Implementation Pattern

### Decision
Implement `CauceError` as an enum with associated data, implementing `From<CauceError> for JsonRpcError`.

### Rationale
- Type-safe error handling with specific data per error type
- Enables pattern matching on error variants
- Consistent with existing `JsonRpcError` structure

### Alternatives Considered
- **Single struct with error code field**: Rejected - loses type safety
- **Trait-based errors**: Rejected - adds complexity without benefit for fixed error set

### Implementation Notes
```rust
#[derive(Debug, Clone, Error)]
pub enum CauceError {
    // JSON-RPC standard errors
    #[error("Parse error: {message}")]
    ParseError { message: String },  // -32700

    // Protocol errors
    #[error("Rate limited")]
    RateLimited { retry_after_ms: u64 },  // -32006
}

impl From<CauceError> for JsonRpcError {
    fn from(err: CauceError) -> Self {
        match err {
            CauceError::ParseError { message } =>
                JsonRpcError::new(-32700, "Parse error", Some(json!({ "message": message }))),
            CauceError::RateLimited { retry_after_ms } =>
                JsonRpcError::new(-32006, "Rate limited", Some(json!({ "retry_after_ms": retry_after_ms }))),
            // ...
        }
    }
}
```

---

## 3. ID Generation Strategy

### Decision
Use `chrono::Utc::now().timestamp()` for timestamps and `uuid::Uuid::new_v4()` for random components.

### Rationale
- `chrono` already in dependencies, provides reliable UTC timestamps
- `uuid` v4 provides cryptographically secure randomness
- For signal/action IDs: timestamp + 12 alphanumeric chars from UUID
- For subscription/session/message IDs: UUID v4 string

### Alternatives Considered
- **`rand` crate for randomness**: Rejected - uuid already provides secure randomness
- **Monotonic timestamps**: Rejected - Unix seconds sufficient, uniqueness from random component

### Implementation Notes
```rust
pub fn generate_signal_id() -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random: String = uuid::Uuid::new_v4()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(12)
        .collect();
    format!("sig_{}_{}", timestamp, random)
}

pub fn generate_subscription_id() -> String {
    format!("sub_{}", uuid::Uuid::new_v4())
}
```

---

## 4. Topic Pattern Matching Algorithm

### Decision
Implement recursive segment-by-segment matching for wildcards, with optional TopicTrie for bulk subscription matching.

### Rationale
- `*` matches single segment: simple equality check
- `**` matches one or more segments: requires backtracking or greedy matching
- Trie structure enables O(n) matching against many patterns

### Alternatives Considered
- **Regex conversion**: Rejected - overhead of regex compilation, harder to debug
- **State machine**: Rejected - complex for two simple wildcards

### Implementation Notes
```rust
pub fn matches(topic: &str, pattern: &str) -> bool {
    let topic_segments: Vec<&str> = topic.split('.').collect();
    let pattern_segments: Vec<&str> = pattern.split('.').collect();
    matches_segments(&topic_segments, &pattern_segments)
}

fn matches_segments(topic: &[&str], pattern: &[&str]) -> bool {
    match (topic.first(), pattern.first()) {
        (None, None) => true,
        (Some(_), None) => false,
        (None, Some(&"**")) => matches_segments(&[], &pattern[1..]),
        (None, Some(_)) => false,
        (Some(_), Some(&"**")) => {
            // Try matching 1 segment, then try matching more
            matches_segments(&topic[1..], pattern) ||
            matches_segments(&topic[1..], &pattern[1..])
        }
        (Some(t), Some(&"*")) => matches_segments(&topic[1..], &pattern[1..]),
        (Some(t), Some(p)) if t == p => matches_segments(&topic[1..], &pattern[1..]),
        _ => false,
    }
}
```

---

## 5. JSON Schema Embedding and Validation

### Decision
Embed JSON schemas using `include_str!` macro with lazy compilation via `once_cell::sync::Lazy`.

### Rationale
- Schemas embedded at compile time - no runtime file I/O
- `jsonschema` crate provides validation
- `once_cell::Lazy` compiles schemas once on first use (MSRV compatible)

### Alternatives Considered
- **Runtime file loading**: Rejected - adds deployment complexity, possible failures
- **LazyLock**: Rejected - requires Rust 1.80, MSRV is 1.75

### Implementation Notes
```rust
use once_cell::sync::Lazy;
use jsonschema::Validator;

static SIGNAL_SCHEMA: &str = include_str!("../../schemas/signal.schema.json");

static SIGNAL_VALIDATOR: Lazy<Validator> = Lazy::new(|| {
    let schema: serde_json::Value = serde_json::from_str(SIGNAL_SCHEMA)
        .expect("Invalid signal schema JSON");
    Validator::new(&schema).expect("Invalid signal schema")
});

pub fn validate_signal(value: &serde_json::Value) -> Result<Signal, ValidationError> {
    SIGNAL_VALIDATOR.validate(value)
        .map_err(|errors| ValidationError::SchemaValidation {
            errors: errors.map(|e| e.to_string()).collect()
        })?;
    serde_json::from_value(value.clone())
        .map_err(|e| ValidationError::Deserialization { message: e.to_string() })
}
```

---

## 6. Topic Pattern Validation

### Decision
Extend existing topic validation to support `*` and `**` wildcards for subscription patterns.

### Rationale
- Regular topics: alphanumeric, dots, hyphens, underscores (existing)
- Patterns: same rules plus `*` (single segment) and `**` (multi-segment)
- `**` must not appear mid-segment (e.g., `signal.**foo` invalid)

### Alternatives Considered
- **Separate TopicPattern type**: Rejected - pattern is superset of topic, validation sufficient

### Implementation Notes
```rust
pub fn validate_topic_pattern(pattern: &str) -> Result<(), ValidationError> {
    // Same base rules as validate_topic
    if pattern.is_empty() { return Err(ValidationError::EmptyPattern); }
    if pattern.len() > MAX_TOPIC_LENGTH { return Err(ValidationError::TooLong); }

    for segment in pattern.split('.') {
        match segment {
            "*" | "**" => continue,  // Valid wildcards
            s if s.contains('*') => return Err(ValidationError::InvalidWildcard),
            s => validate_segment(s)?,
        }
    }
    Ok(())
}
```

---

## 7. SubscriptionStatus and Lifecycle

### Decision
Define `SubscriptionStatus` enum with: Pending, Active, Denied, Revoked, Expired.

### Rationale
- Matches protocol specification subscription lifecycle
- Clear state transitions: Pending → Active/Denied, Active → Revoked/Expired
- Used in SubscribeResponse and SubscriptionInfo

### Alternatives Considered
- **String status**: Rejected - loses type safety and validation

### Implementation Notes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Pending,
    Active,
    Denied,
    Revoked,
    Expired,
}
```

---

## 8. PublishMessage Enum (Signal or Action)

### Decision
Use `#[serde(untagged)]` enum for PublishRequest message field to accept either Signal or Action.

### Rationale
- Protocol allows publishing either signals or actions
- Untagged enum tries each variant in order
- Signal has `sig_` prefix ID, Action has `act_` prefix - distinguishable

### Alternatives Considered
- **Tagged enum with "type" field**: Rejected - protocol doesn't define discriminator field
- **Separate endpoints**: Rejected - protocol uses single publish method

### Implementation Notes
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PublishMessage {
    Signal(Signal),
    Action(Action),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub topic: String,
    pub message: PublishMessage,
}
```

---

## Summary

All research items have been resolved. Key decisions:

| Area | Decision |
|------|----------|
| Serde naming | snake_case with `type_` → `type` rename |
| Error codes | Enum with `From<CauceError> for JsonRpcError` |
| ID generation | chrono timestamps + uuid v4 random |
| Topic matching | Recursive segment matching |
| Schema validation | `include_str!` + `once_cell::Lazy` |
| Wildcards | `*` single segment, `**` one-or-more segments |
| Publish message | Untagged enum for Signal/Action |

**Next step**: Phase 1 design (data-model.md, contracts/)

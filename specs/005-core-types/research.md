# Research: Core Types Module

**Feature**: 005-core-types
**Date**: 2026-01-07

## Overview

This document captures research decisions for implementing the Core Types module in `cauce-core`. No critical unknowns required clarification; research focuses on Rust best practices for the design patterns used.

## Research Topics

### 1. Serde Serialization for Optional Fields

**Decision**: Use `#[serde(skip_serializing_if = "Option::is_none")]` on all optional fields

**Rationale**:
- Cauce protocol expects optional fields to be omitted, not serialized as `null`
- This is idiomatic Rust/serde pattern for JSON APIs
- Reduces payload size and matches JavaScript/JSON conventions

**Alternatives considered**:
- Default serialization (includes `null` values): Rejected - protocol spec expects omission
- Custom serializer: Rejected - unnecessary complexity when skip_serializing_if works

### 2. Builder Pattern Implementation

**Decision**: Use the typestate builder pattern with compile-time enforcement of required fields

**Rationale**:
- Prevents runtime errors from missing required fields
- Provides excellent IDE autocomplete experience
- Zero runtime overhead (all checks at compile time)
- Idiomatic Rust pattern used by major crates (e.g., reqwest, tonic)

**Alternatives considered**:
- Runtime validation in `build()`: Rejected - pushes errors to runtime
- `derive_builder` crate: Rejected - adds dependency for simple use case; typestate is more idiomatic
- `bon` crate: Considered - newer builder derive macro, but adds dependency

**Implementation approach**:
```rust
// Typestate markers
pub struct NoId;
pub struct HasId(String);

pub struct SignalBuilder<Id, Source, Topic, Payload> {
    id: Id,
    source: Source,
    topic: Topic,
    payload: Payload,
    // Optional fields always available
    metadata: Option<Metadata>,
    encrypted: Option<Encrypted>,
}

impl SignalBuilder<NoId, NoSource, NoTopic, NoPayload> {
    pub fn new() -> Self { ... }
}

impl<S, T, P> SignalBuilder<NoId, S, T, P> {
    pub fn id(self, id: impl Into<String>) -> SignalBuilder<HasId, S, T, P> { ... }
}

// Only buildable when all required fields set
impl SignalBuilder<HasId, HasSource, HasTopic, HasPayload> {
    pub fn build(self) -> Signal { ... }
}
```

### 3. Topic Newtype Validation

**Decision**: Implement Topic as a newtype with validation in `TryFrom<&str>` and `FromStr`

**Rationale**:
- Newtype pattern enforces valid topics at type level
- Cannot accidentally use invalid topic strings
- `TryFrom`/`FromStr` is idiomatic Rust for fallible conversions
- Validation runs once at construction; subsequent use is zero-cost

**Alternatives considered**:
- Validate at each use site: Rejected - error-prone, repeated work
- Wrapper struct with pub String: Rejected - allows invalid construction
- Macro-based validation: Rejected - unnecessary complexity

**Validation rules** (from spec FR-014):
- Length: 1-255 characters
- Characters: alphanumeric, dots (`.`), hyphens (`-`), underscores (`_`)
- No leading or trailing dots
- No consecutive dots

### 4. ID Format Validation

**Decision**: Validate Signal/Action IDs using regex patterns with lazy_static/once_cell

**Rationale**:
- ID formats are well-defined: `sig_<timestamp>_<random>` and `act_<timestamp>_<random>`
- Regex provides clear, maintainable validation
- Compile regex once, reuse for all validations

**Alternatives considered**:
- Manual string parsing: Rejected - error-prone, hard to maintain
- No validation: Rejected - violates FR-002, FR-008

**ID format details** (from Constitution):
- Signal: `sig_<unix_timestamp_seconds>_<random_12_chars>`
- Action: `act_<unix_timestamp_seconds>_<random_12_chars>`

### 5. Timestamp Representation

**Decision**: Use `chrono::DateTime<Utc>` internally, serialize as ISO 8601 string

**Rationale**:
- `chrono` already in dependencies
- ISO 8601 is human-readable and unambiguous
- Serde support built into chrono with `serde` feature
- UTC timezone avoids ambiguity

**Alternatives considered**:
- Unix timestamp (i64): Rejected - less readable in JSON, timezone ambiguous
- String internally: Rejected - loses type safety for datetime operations

### 6. Error Type Design

**Decision**: Use `thiserror` for error enums with structured error data

**Rationale**:
- `thiserror` already in dependencies
- Provides automatic `std::error::Error` implementation
- Allows structured error variants for different failure modes
- Enables good error messages with context

**Error categories**:
- `ValidationError` - Topic validation, ID format validation
- `BuilderError` - Missing required fields (compile-time prevented, but useful for runtime cases)

### 7. Enum Serialization

**Decision**: Use `#[serde(rename_all = "snake_case")]` for enum variants

**Rationale**:
- JSON convention uses snake_case
- Rust convention uses PascalCase
- serde rename_all bridges both conventions automatically

**Example**:
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}
// Serializes as: "low", "normal", "high", "urgent"
```

### 8. Clone and Debug Derives

**Decision**: Derive `Clone`, `Debug`, `PartialEq` on all types

**Rationale**:
- Required by FR-017
- `Clone` needed for message passing and storage
- `Debug` essential for logging and debugging
- `PartialEq` needed for test assertions and comparisons

**Additional derives where appropriate**:
- `Default` for types with sensible defaults (e.g., Priority::Normal)
- `Hash` + `Eq` for types used as map keys (e.g., Topic)

## Dependencies Review

All dependencies are already in `Cargo.toml` (added in 004-cauce-core):

| Dependency | Version | Purpose |
|------------|---------|---------|
| serde | 1.0 | Serialization framework |
| serde_json | 1.0 | JSON serialization |
| thiserror | 1.0 | Error derive macro |
| chrono | 0.4 | DateTime handling |
| uuid | 1.0 | UUID generation (for IDs) |
| jsonschema | 0.27 | Schema validation (for contract tests) |

**No additional dependencies required.**

## Conclusion

All research topics resolved with clear decisions. The implementation can proceed with:
1. Typestate builder pattern for Signal/Action
2. Newtype with TryFrom validation for Topic
3. Serde with skip_serializing_if for optional fields
4. Regex validation for ID formats
5. Chrono DateTime for timestamps
6. Thiserror for structured errors

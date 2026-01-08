# Research: JSON-RPC Types

**Feature**: 006-jsonrpc-types
**Date**: 2026-01-07

## Overview

This document captures research findings for implementing JSON-RPC 2.0 message types in Rust. Since the Technical Context has no NEEDS CLARIFICATION items, this research focuses on best practices and implementation patterns.

## Research Topics

### 1. JSON-RPC 2.0 Specification Compliance

**Decision**: Implement strict JSON-RPC 2.0 compliance per https://www.jsonrpc.org/specification

**Rationale**: The specification is well-defined and widely adopted. Strict compliance ensures interoperability with any JSON-RPC 2.0 client/server.

**Key specification points**:
- `jsonrpc` field MUST be exactly "2.0"
- Request MUST have: jsonrpc, method, id (optional for notifications), params (optional)
- Response MUST have: jsonrpc, id, and EITHER result OR error (never both, never neither for success/error)
- Error MUST have: code (integer), message (string), data (optional)
- ID can be String, Number, or Null (null only for error responses to unidentifiable requests)

**Alternatives considered**:
- Loose parsing (accept "2.0" variations): Rejected - violates spec and reduces interoperability
- Custom extensions: Rejected - keep core types pure JSON-RPC 2.0

### 2. Serde Serialization Patterns for Union Types

**Decision**: Use `#[serde(untagged)]` enum for Response (success vs error) and RequestId (string vs integer)

**Rationale**:
- JSON-RPC 2.0 responses are discriminated by presence of "result" vs "error" fields
- RequestId must accept both `"abc"` (string) and `123` (integer) without type wrapper

**Implementation pattern for Response**:
```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Success { jsonrpc: String, id: RequestId, result: Value },
    Error { jsonrpc: String, id: Option<RequestId>, error: JsonRpcError },
}
```

**Implementation pattern for RequestId**:
```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}
```

**Alternatives considered**:
- Internally tagged enum: Rejected - JSON-RPC has no "type" field
- Struct with Option fields: Rejected - allows invalid states (both result and error present)
- Custom Deserialize impl: Possible but untagged enum is simpler and sufficient

### 3. Validation Strategy

**Decision**: Validate during deserialization using custom Deserialize implementations where needed

**Rationale**:
- Catching invalid messages early prevents propagation of bad data
- Serde custom deserialization is idiomatic Rust

**Validation rules**:
- FR-006: Validate `jsonrpc == "2.0"` - custom deserializer or post-parse check
- FR-011: Reject responses with both result and error - untagged enum prevents this structurally
- FR-010: Preserve id type - RequestId enum handles this naturally

**Implementation approach**:
- Use `#[serde(deserialize_with = "...")]` for jsonrpc field validation
- Response enum structure prevents both result+error
- RequestId enum preserves string vs integer naturally

**Alternatives considered**:
- Runtime validation after parse: Rejected - allows invalid types to exist temporarily
- Schema validation via jsonschema crate: Overkill for type-level validation

### 4. Optional Field Serialization

**Decision**: Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields

**Rationale**:
- JSON-RPC 2.0 allows omitting optional fields (params, data)
- Matches existing cauce-core patterns (see types/action.rs)
- Produces cleaner JSON output

**Fields affected**:
- Request.params: Optional<Value>
- Notification.params: Optional<Value>
- JsonRpcError.data: Optional<Value>

### 5. Error Type Design

**Decision**: Separate JsonRpcError (wire format) from application errors (ValidationError)

**Rationale**:
- JsonRpcError is a data structure for serialization
- ValidationError is for Rust error handling
- Mixing them creates awkward APIs

**Structure**:
- `JsonRpcError` - struct with code, message, data fields
- `ValidationError` - existing error enum in errors module for parse failures
- Helper: `JsonRpcError::parse_error()`, `::invalid_request()`, etc.

**Alternatives considered**:
- Single error type: Rejected - conflates wire format with Rust errors
- Error codes as enum: Deferred to 2.5 Error Codes feature

### 6. Helper Method Design

**Decision**: Provide static methods on Response type for construction

**Rationale**:
- Matches builder pattern used elsewhere in cauce-core
- Reduces boilerplate for common operations
- Ensures correct structure

**Helper methods**:
- `JsonRpcResponse::success(id: RequestId, result: Value) -> Self`
- `JsonRpcResponse::error(id: Option<RequestId>, error: JsonRpcError) -> Self`
- `JsonRpcError::new(code: i32, message: impl Into<String>) -> Self`
- `JsonRpcError::with_data(code: i32, message: impl Into<String>, data: Value) -> Self`

## Dependencies

| Dependency | Purpose | Already in workspace? |
|------------|---------|----------------------|
| serde | Serialization traits | Yes |
| serde_json | JSON serialization, Value type | Yes |
| thiserror | Error derive macro | Yes |

No new dependencies required.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Serde untagged enum ambiguity | Deserialization might pick wrong variant | Test with edge cases; Response variants are structurally distinct |
| i64 vs u64 for numeric ids | Overflow for large unsigned ids | JSON-RPC spec doesn't specify; i64 covers practical range |
| Null id handling | Complex edge case | Only allow null id in error responses per spec |

## Conclusion

The implementation is straightforward with existing dependencies. Key decisions:
1. Strict JSON-RPC 2.0 compliance
2. Untagged enums for polymorphic types (Response, RequestId)
3. Serde attributes for optional field handling
4. Validation during deserialization
5. Helper methods for ergonomic construction

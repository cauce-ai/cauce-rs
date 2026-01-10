# Feature Specification: Phase 2 Completion - Cauce Core Library

**Feature Branch**: `007-phase2-completion`
**Created**: 2026-01-08
**Status**: Draft
**Input**: User description: "Implement all remaining items for Phase 2 of cauce-core: Method Parameter Types, Error Codes, Method Constants, Validation enhancements, ID Generation Utilities, Topic Matching Utilities, and Testing"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Method Parameter Types for Protocol Operations (Priority: P1)

Developers using cauce-core need strongly-typed request and response structures for all Cauce Protocol JSON-RPC methods to safely build clients and servers that communicate over the protocol.

**Why this priority**: Without method parameter types, developers cannot build type-safe clients or servers. This is foundational for all protocol interactions.

**Independent Test**: Can be fully tested by creating request/response objects for each method and verifying serialization/deserialization roundtrips match expected JSON structures.

**Acceptance Scenarios**:

1. **Given** a developer wants to subscribe to topics, **When** they create a `SubscribeRequest` with topics and transport options, **Then** it serializes to valid JSON matching the protocol schema.
2. **Given** a server receives a subscribe request, **When** it parses the JSON into `SubscribeRequest`, **Then** all fields are correctly populated including optional fields.
3. **Given** a client receives a subscribe response, **When** it parses the JSON, **Then** it correctly deserializes into `SubscribeResponse` with subscription_id and status.
4. **Given** a `HelloRequest` is created, **When** serialized, **Then** it includes protocol_version, client_id, client_type, capabilities, and optional auth fields.

---

### User Story 2 - Protocol Error Codes and Handling (Priority: P1)

Developers need comprehensive error types matching all Cauce Protocol error codes (-32700 to -32015) to properly handle and report protocol errors in their applications.

**Why this priority**: Error handling is critical for robust protocol implementations. Without proper error codes, applications cannot respond appropriately to failures.

**Independent Test**: Can be fully tested by creating each error type and verifying its code, message, and conversion to JsonRpcError.

**Acceptance Scenarios**:

1. **Given** a JSON-RPC standard error occurs, **When** a `ParseError` is created, **Then** it has code -32700 and appropriate message.
2. **Given** a protocol-specific error occurs, **When** a `SubscriptionNotFound` error is created, **Then** it has code -32001 and can include additional data.
3. **Given** any `CauceError` variant, **When** converted to `JsonRpcError`, **Then** the code, message, and data fields are correctly populated.
4. **Given** a `RateLimited` error, **When** created with retry_after_ms, **Then** the data field includes the retry information.

---

### User Story 3 - Method Name Constants and Protocol Limits (Priority: P2)

Developers need constant definitions for all method names and protocol limits to avoid magic strings/numbers and ensure consistency across implementations.

**Why this priority**: Constants prevent typos and ensure protocol compliance, but implementations can function with string literals.

**Independent Test**: Can be fully tested by verifying each constant has the expected value.

**Acceptance Scenarios**:

1. **Given** a developer calls `cauce.subscribe`, **When** they use the `SUBSCRIBE` constant, **Then** it equals "cauce.subscribe".
2. **Given** protocol limits are needed, **When** checking `MAX_TOPIC_LENGTH`, **Then** it equals 255.
3. **Given** all method constants exist, **When** building protocol handlers, **Then** constants exist for all 17 protocol methods.

---

### User Story 4 - ID Generation for Protocol Messages (Priority: P2)

Developers need utilities to generate properly formatted IDs for signals, actions, subscriptions, sessions, and messages without manually constructing timestamp and random components.

**Why this priority**: ID generation is needed for creating protocol messages, but is a utility function that can be worked around.

**Independent Test**: Can be fully tested by generating IDs and validating they match the required format patterns.

**Acceptance Scenarios**:

1. **Given** a new signal is being created, **When** `generate_signal_id()` is called, **Then** it returns an ID matching `sig_<timestamp>_<12 alphanumeric chars>`.
2. **Given** multiple IDs are generated, **When** called in sequence, **Then** each ID is unique.
3. **Given** a subscription is created, **When** `generate_subscription_id()` is called, **Then** it returns an ID matching `sub_<uuid>` format.

---

### User Story 5 - Topic Pattern Matching for Subscriptions (Priority: P2)

Developers building Hub implementations need efficient topic matching with wildcard support to route signals to the correct subscriptions based on topic patterns.

**Why this priority**: Topic matching is essential for Hub routing but can be implemented with basic string operations initially.

**Independent Test**: Can be fully tested by matching various topics against patterns with `*` and `**` wildcards.

**Acceptance Scenarios**:

1. **Given** a subscription pattern `signal.*`, **When** matching against `signal.email`, **Then** it matches.
2. **Given** a subscription pattern `signal.*`, **When** matching against `signal.email.received`, **Then** it does NOT match (single segment only).
3. **Given** a subscription pattern `signal.**`, **When** matching against `signal.email.received`, **Then** it matches (multi-segment).
4. **Given** a pattern `**.received`, **When** matching `signal.email.received`, **Then** it matches.

---

### User Story 6 - Comprehensive Validation with Schema Support (Priority: P3)

Developers need validation functions that can validate signals, actions, and other protocol messages against JSON schemas embedded in the library.

**Why this priority**: Schema validation provides defense-in-depth but type-safe structs already provide compile-time guarantees.

**Independent Test**: Can be fully tested by passing valid and invalid JSON to validation functions.

**Acceptance Scenarios**:

1. **Given** a valid signal JSON, **When** `validate_signal()` is called, **Then** it returns Ok with the parsed Signal.
2. **Given** an invalid signal JSON (missing required field), **When** `validate_signal()` is called, **Then** it returns detailed ValidationError.
3. **Given** a topic pattern with wildcards, **When** `validate_topic_pattern()` is called, **Then** it validates wildcard syntax.

---

### User Story 7 - Complete Test Coverage for Core Library (Priority: P3)

The cauce-core library must achieve 95% code coverage with comprehensive unit tests, integration tests, and property-based tests to ensure protocol compliance and reliability.

**Why this priority**: Tests validate correctness but the implementation must exist first.

**Independent Test**: Can be verified by running coverage tools and checking the 95% threshold is met.

**Acceptance Scenarios**:

1. **Given** all cauce-core types, **When** coverage is measured, **Then** it exceeds 95%.
2. **Given** Signal serialization tests, **When** run, **Then** roundtrip serialization/deserialization preserves all data.
3. **Given** topic matching with arbitrary patterns, **When** property-based tests run, **Then** matching behavior is consistent.

---

### Edge Cases

- What happens when generating IDs at the same millisecond? IDs must still be unique due to random component.
- How does topic matching handle empty patterns or topics? Return error for empty strings.
- What happens when validation receives null or non-object JSON? Return appropriate ValidationError.
- How are deeply nested topic patterns handled? Limit nesting to reasonable depth (e.g., 10 segments).
- What if method parameters have extra unknown fields? Ignore unknown fields per JSON-RPC convention.

## Requirements *(mandatory)*

### Functional Requirements

#### Method Parameter Types (2.4)

- **FR-001**: System MUST provide `HelloRequest` struct with fields: protocol_version, min_protocol_version, max_protocol_version, client_id, client_type, capabilities, auth
- **FR-002**: System MUST provide `HelloResponse` struct with fields: session_id, server_version, capabilities, session_expires_at
- **FR-003**: System MUST provide `Auth` struct with fields: type_, token, api_key
- **FR-004**: System MUST provide `AuthType` enum with variants: Bearer, ApiKey, Mtls
- **FR-005**: System MUST provide `ClientType` enum with variants: Adapter, Agent, A2aAgent
- **FR-006**: System MUST provide `Capability` enum with variants: Subscribe, Publish, Ack, E2eEncryption
- **FR-007**: System MUST provide `SubscribeRequest` struct with fields: topics, approval_type, reason, transport, webhook, e2e
- **FR-008**: System MUST provide `SubscribeResponse` struct with fields: subscription_id, status, topics, created_at, expires_at
- **FR-009**: System MUST provide `ApprovalType` enum with variants: Automatic, UserApproved
- **FR-010**: System MUST provide `Transport` enum with variants: WebSocket, Sse, Polling, LongPolling, Webhook
- **FR-011**: System MUST provide `WebhookConfig` struct with fields: url, secret, headers
- **FR-012**: System MUST provide `E2eConfig` struct with fields: enabled, public_key, supported_algorithms
- **FR-013**: System MUST provide `UnsubscribeRequest` and `UnsubscribeResponse` structs
- **FR-014**: System MUST provide `PublishRequest` struct with fields: topic, message (Signal or Action)
- **FR-015**: System MUST provide `PublishResponse` struct with fields: message_id, delivered_to, queued_for
- **FR-016**: System MUST provide `AckRequest` struct with fields: signal_ids, subscription_id
- **FR-017**: System MUST provide `AckResponse` struct with fields: acknowledged, failed
- **FR-018**: System MUST provide `SignalDelivery` struct with fields: topic, signal
- **FR-019**: System MUST provide subscription management types: SubscriptionApproveRequest, SubscriptionDenyRequest, SubscriptionRevokeRequest, SubscriptionListRequest, SubscriptionListResponse, SubscriptionInfo, SubscriptionStatusNotification
- **FR-020**: System MUST provide `PingParams` and `PongParams` structs with timestamp field
- **FR-021**: System MUST provide schema types: SchemasListRequest, SchemasListResponse, SchemasGetRequest, SchemasGetResponse

#### Error Codes (2.5)

- **FR-022**: System MUST define `CauceError` enum with all protocol error variants
- **FR-023**: System MUST implement JSON-RPC standard errors: ParseError (-32700), InvalidRequest (-32600), MethodNotFound (-32601), InvalidParams (-32602), InternalError (-32603)
- **FR-024**: System MUST implement Cauce protocol errors: SubscriptionNotFound (-32001), TopicNotFound (-32002), NotAuthorized (-32003), SubscriptionPending (-32004), SubscriptionDenied (-32005), RateLimited (-32006), SignalTooLarge (-32007), EncryptionRequired (-32008), InvalidEncryption (-32009), AdapterUnavailable (-32010), DeliveryFailed (-32011), QueueFull (-32012), SessionExpired (-32013), UnsupportedTransport (-32014), InvalidTopic (-32015)
- **FR-025**: System MUST implement `From<CauceError> for JsonRpcError`
- **FR-026**: System MUST provide error data helpers for: details, suggestion, retry_after_ms, field

#### Method Constants (2.6)

- **FR-027**: System MUST define method name constants: HELLO, GOODBYE, PING, PONG, PUBLISH, SUBSCRIBE, UNSUBSCRIBE, SIGNAL, ACK, SUBSCRIPTION_REQUEST, SUBSCRIPTION_APPROVE, SUBSCRIPTION_DENY, SUBSCRIPTION_LIST, SUBSCRIPTION_REVOKE, SUBSCRIPTION_STATUS, SCHEMAS_LIST, SCHEMAS_GET
- **FR-028**: System MUST define size limits: MAX_TOPIC_LENGTH (255), MAX_SIGNAL_PAYLOAD_SIZE (10MB), MAX_TOPICS_PER_SUBSCRIPTION (100), MAX_SUBSCRIPTIONS_PER_CLIENT (1000), MAX_SIGNALS_PER_BATCH (100), MAX_TOPIC_DEPTH (10)

#### Validation (2.7)

- **FR-029**: System MUST embed JSON schemas as static assets using `include_str!` for: signal, action, jsonrpc, errors, method schemas, payload schemas
- **FR-030**: System MUST implement `validate_signal(value: &Value) -> Result<Signal, ValidationError>`
- **FR-031**: System MUST implement `validate_action(value: &Value) -> Result<Action, ValidationError>`
- **FR-032**: System MUST implement `validate_topic_pattern(pattern: &str) -> Result<(), ValidationError>` with wildcard support
- **FR-033**: System MUST implement ID validation for: subscription_id (sub_<uuid>), session_id (sess_<uuid>)
- **FR-034**: System MUST provide detailed `ValidationError` with field path and reason

#### ID Generation (2.8)

- **FR-035**: System MUST implement `generate_signal_id() -> String` returning `sig_<timestamp>_<random>`
- **FR-036**: System MUST implement `generate_action_id() -> String` returning `act_<timestamp>_<random>`
- **FR-037**: System MUST implement `generate_subscription_id() -> String` returning `sub_<uuid>`
- **FR-038**: System MUST implement `generate_session_id() -> String` returning `sess_<uuid>`
- **FR-039**: System MUST implement `generate_message_id() -> String` returning `msg_<uuid>`

#### Topic Matching (2.9)

- **FR-040**: System MUST implement `TopicMatcher` struct for efficient pattern matching
- **FR-041**: System MUST implement `matches(topic: &str, pattern: &str) -> bool` with wildcard support
- **FR-042**: `*` wildcard MUST match exactly one topic segment
- **FR-043**: `**` wildcard MUST match one or more topic segments
- **FR-044**: System SHOULD implement efficient topic trie for subscription matching

#### Testing (2.10)

- **FR-045**: System MUST have unit tests for all type serialization/deserialization
- **FR-046**: System MUST have unit tests for all error code conversions
- **FR-047**: System MUST have unit tests for topic validation (valid and invalid cases)
- **FR-048**: System MUST have unit tests for topic pattern matching with wildcards
- **FR-049**: System MUST have unit tests for ID generation format and uniqueness
- **FR-050**: System SHOULD have property-based tests for topic matching
- **FR-051**: System MUST have integration tests for roundtrip serialization of all types
- **FR-052**: System MUST achieve 95% code coverage

### Key Entities

- **Method Parameters**: Request and response types for each JSON-RPC method (HelloRequest, SubscribeRequest, etc.)
- **CauceError**: Enumeration of all protocol error codes with associated data
- **TopicMatcher**: Utility for matching topics against subscription patterns
- **ID Generators**: Functions producing correctly formatted IDs for protocol entities

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 21 method parameter types serialize/deserialize correctly per protocol schema
- **SC-002**: All 20 error codes are implemented with correct numeric codes and messages
- **SC-003**: All 17 method constants and 6 size limit constants are defined
- **SC-004**: Topic pattern matching handles `*` and `**` wildcards correctly in all test cases
- **SC-005**: ID generation produces unique, correctly formatted IDs for all entity types
- **SC-006**: Code coverage for cauce-core exceeds 95%
- **SC-007**: All tests pass in CI on Linux, macOS, and Windows

## Test-Driven Development Approach *(mandatory)*

### Testing Strategy

- **Unit Tests**: Each method parameter type, error code, constant, validation function, ID generator, and topic matcher component
- **Integration Tests**: Roundtrip serialization/deserialization for all types, schema validation against embedded schemas
- **Contract Tests**: Validate serialized JSON against protocol JSON schemas

### Coverage Requirement

Per Constitution Principle XI, this feature MUST:
- Have tests written BEFORE implementation code
- Follow Red-Green-Refactor cycle
- Achieve minimum **95% code coverage**
- Pass all tests in CI before merge

### Test Boundaries

| Component | Test Focus | Coverage Target |
|-----------|------------|-----------------|
| Method Parameters | Serialization, deserialization, optional fields | 95% |
| Error Codes | Code values, message formatting, JsonRpcError conversion | 95% |
| Constants | Value correctness | 100% |
| Validation | Valid/invalid inputs, error messages | 95% |
| ID Generation | Format compliance, uniqueness | 95% |
| Topic Matching | Wildcard patterns, edge cases | 95% |

## Protocol Impact *(Cauce-specific)*

### Schema Impact

| Schema | Change Type | Description |
|--------|-------------|-------------|
| `signal.schema.json` | None | No changes, used for validation |
| `action.schema.json` | None | No changes, used for validation |
| `jsonrpc.schema.json` | None | No changes, used for validation |
| `errors.schema.json` | None | No changes, error codes defined in spec |
| `methods/*.schema.json` | None | No changes, used for type definitions |
| `payloads/*.schema.json` | None | No changes, used for validation |

### Component Interactions

| Component | Responsibility in This Feature | NOT Responsible For |
|-----------|-------------------------------|---------------------|
| **Adapter** | Uses method parameters to communicate with Hub | Implementing Hub logic |
| **Hub** | Uses error codes, validation, topic matching | Client implementation |
| **Agent** | Uses method parameters, ID generation | Hub routing logic |

### Transport Considerations

| Transport | Supported | Notes |
|-----------|-----------|-------|
| WebSocket | Yes | Method parameters serialized as JSON text frames |
| Server-Sent Events | Yes | Same JSON serialization |
| HTTP Polling | Yes | Same JSON serialization |
| Webhooks | Yes | Same JSON serialization |

**Semantic consistency**: All types serialize to identical JSON regardless of transport.

### Wire Protocol

- **New methods**: None (implementing types for existing methods)
- **Modified methods**: None
- **A2A impact**: None (internal type definitions)
- **MCP impact**: None (internal type definitions)

### Version Impact

- **Change type**: MINOR (backward compatible feature addition)
- **Rationale**: Adding new types and utilities without changing existing interfaces

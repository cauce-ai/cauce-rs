# Feature Specification: Core Types Module

**Feature Branch**: `005-core-types`
**Created**: 2026-01-07
**Status**: Draft
**Input**: User description: "TODO.md section 2.2 - Implement Core Types module for cauce-core"

## User Scenarios & Testing

### User Story 1 - Create and Serialize Signals (Priority: P1)

A developer building an adapter needs to create Signal messages that represent incoming data from external platforms (email, chat, etc.) and serialize them to JSON for transmission to the Hub.

**Why this priority**: Signals are the fundamental data structure for all adapter-to-hub communication. Without Signal types, no data can flow through the Cauce protocol.

**Independent Test**: Can be fully tested by creating a Signal with all required fields, serializing to JSON, and deserializing back to verify round-trip integrity.

**Acceptance Scenarios**:

1. **Given** valid signal data (id, version, timestamp, source, topic, payload), **When** creating a Signal, **Then** the Signal is created with all fields populated correctly
2. **Given** a valid Signal struct, **When** serializing to JSON, **Then** the output matches the Cauce protocol JSON schema
3. **Given** valid JSON matching the signal schema, **When** deserializing, **Then** a valid Signal struct is created with all fields intact

---

### User Story 2 - Create and Serialize Actions (Priority: P1)

A developer building an agent needs to create Action messages that represent commands to be executed by adapters (send email, post message, etc.) and serialize them for transmission.

**Why this priority**: Actions are the fundamental data structure for all agent-to-adapter communication. Without Action types, agents cannot command adapters.

**Independent Test**: Can be fully tested by creating an Action with all required fields, serializing to JSON, and deserializing back to verify round-trip integrity.

**Acceptance Scenarios**:

1. **Given** valid action data (id, version, timestamp, topic, action body, context), **When** creating an Action, **Then** the Action is created with all fields populated correctly
2. **Given** a valid Action struct, **When** serializing to JSON, **Then** the output matches the Cauce protocol JSON schema
3. **Given** valid JSON matching the action schema, **When** deserializing, **Then** a valid Action struct is created with all fields intact

---

### User Story 3 - Validate Topics (Priority: P2)

A developer needs to create and validate Topic identifiers to ensure they conform to the Cauce protocol naming conventions before using them in signals or subscriptions.

**Why this priority**: Topics are required for routing but are simpler than Signal/Action. Valid topic patterns are essential for subscription matching.

**Independent Test**: Can be fully tested by creating topics with various valid and invalid patterns and verifying validation results.

**Acceptance Scenarios**:

1. **Given** a valid topic string (alphanumeric with dots, hyphens, underscores), **When** creating a Topic, **Then** the Topic is created successfully
2. **Given** an invalid topic string (leading dot, consecutive dots, too long), **When** creating a Topic, **Then** an appropriate validation error is returned
3. **Given** a valid Topic, **When** converting to string, **Then** the original string is returned unchanged

---

### User Story 4 - Build Signals with Builder Pattern (Priority: P2)

A developer needs an ergonomic way to construct complex Signal messages with optional fields without manually setting every field.

**Why this priority**: Improves developer experience but is not strictly required for protocol functionality.

**Independent Test**: Can be fully tested by using the builder to create signals with various combinations of required and optional fields.

**Acceptance Scenarios**:

1. **Given** a SignalBuilder, **When** setting required fields and calling build(), **Then** a valid Signal is created
2. **Given** a SignalBuilder with missing required fields, **When** calling build(), **Then** an error is returned indicating the missing fields
3. **Given** a SignalBuilder, **When** setting optional fields (metadata, encrypted), **Then** those fields are included in the built Signal

---

### User Story 5 - Handle Encrypted Message Envelopes (Priority: P3)

A developer needs to work with encrypted signal/action payloads for end-to-end encryption scenarios.

**Why this priority**: E2E encryption is optional in the protocol and can be deferred, but the types must exist for forward compatibility.

**Independent Test**: Can be fully tested by creating Encrypted structs with valid encryption metadata and verifying serialization.

**Acceptance Scenarios**:

1. **Given** encryption metadata (algorithm, public key, nonce, ciphertext), **When** creating an Encrypted struct, **Then** all fields are stored correctly
2. **Given** an Encrypted struct, **When** serializing to JSON, **Then** the output includes all encryption fields in the correct format

---

### Edge Cases

- What happens when a Signal ID doesn't match the required format (`sig_<timestamp>_<random>`)?
- What happens when a Topic exceeds 255 characters?
- What happens when a Topic contains invalid characters (spaces, special chars)?
- What happens when a Topic has leading/trailing dots or consecutive dots?
- How does the system handle empty payloads vs. missing payloads?
- What happens when optional fields are explicitly null vs. omitted in JSON?

## Requirements

### Functional Requirements

- **FR-001**: System MUST provide a `Signal` struct with fields: id, version, timestamp, source, topic, payload, metadata (optional), encrypted (optional)
- **FR-002**: System MUST validate Signal IDs match the format `sig_<timestamp>_<random>` where timestamp is milliseconds since epoch and random is alphanumeric
- **FR-003**: System MUST provide a `Source` struct with fields: type (adapter type), adapter_id, native_id
- **FR-004**: System MUST provide a `Payload` struct with fields: raw (JSON value), content_type (MIME type), size_bytes
- **FR-005**: System MUST provide a `Metadata` struct with fields: thread_id (optional), in_reply_to (optional), references (optional list), priority (optional), tags (optional list)
- **FR-006**: System MUST provide a `Priority` enum with variants: Low, Normal, High, Urgent
- **FR-007**: System MUST provide an `Action` struct with fields: id, version, timestamp, topic, action (body), context (optional), encrypted (optional)
- **FR-008**: System MUST validate Action IDs match the format `act_<timestamp>_<random>`
- **FR-009**: System MUST provide an `ActionBody` struct with fields: type (action type), target (optional), payload
- **FR-010**: System MUST provide an `ActionType` enum with variants: Send, Reply, Forward, React, Update, Delete
- **FR-011**: System MUST provide an `ActionContext` struct with fields: in_reply_to (optional), agent_id (optional), thread_id (optional), correlation_id (optional)
- **FR-012**: System MUST provide an `Encrypted` struct with fields: algorithm, recipient_public_key, nonce, ciphertext
- **FR-013**: System MUST provide an `EncryptionAlgorithm` enum with variants: X25519XSalsa20Poly1305, A256GCM, XChaCha20Poly1305
- **FR-014**: System MUST provide a `Topic` newtype that validates: length 1-255 characters, only alphanumeric chars plus dots, hyphens, and underscores, no leading/trailing dots, no consecutive dots
- **FR-015**: All types MUST implement Serialize and Deserialize for JSON round-trip
- **FR-016**: Signal and Action MUST provide builder patterns for ergonomic construction
- **FR-017**: All types MUST implement Debug and Clone traits
- **FR-018**: Optional fields MUST be properly handled during serialization (omit None values, include Some values)

### Key Entities

- **Signal**: Represents an inbound message from an adapter to the hub. Contains source information, topic routing, payload data, and optional metadata/encryption.
- **Action**: Represents a command from an agent to be executed by an adapter. Contains action type, target, payload, and optional context/encryption.
- **Topic**: A validated hierarchical identifier (e.g., `signal.email.received`) used for pub/sub routing.
- **Source**: Identifies where a signal originated (adapter type, adapter instance, native platform ID).
- **Payload**: The actual message content with type information and size.
- **Metadata**: Optional threading and priority information for message correlation.
- **Encrypted**: End-to-end encryption envelope containing algorithm info and ciphertext.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 14 type definitions compile and pass unit tests with 100% coverage
- **SC-002**: JSON serialization round-trip succeeds for all types (serialize then deserialize produces equal value)
- **SC-003**: Topic validation correctly accepts all valid patterns and rejects all invalid patterns per protocol spec
- **SC-004**: Signal and Action builders prevent construction of invalid instances (missing required fields)
- **SC-005**: All types work correctly with serde_json for real-world JSON interchange

## Test-Driven Development Approach

### Testing Strategy

- **Unit Tests**: Test each type's construction, field access, serialization, deserialization, and validation
- **Integration Tests**: Test Signal and Action builders produce valid instances that serialize correctly
- **Contract Tests**: Validate serialized JSON against Cauce protocol schemas (signal.schema.json, action.schema.json)

### Coverage Requirement

Per Constitution Principle XI, this feature MUST:
- Have tests written BEFORE implementation code
- Follow Red-Green-Refactor cycle
- Achieve minimum **95% code coverage**
- Pass all tests in CI before merge

### Test Boundaries

| Component | Test Focus | Coverage Target |
|-----------|------------|-----------------|
| Signal type | Construction, serialization, ID validation | 95% |
| Action type | Construction, serialization, ID validation | 95% |
| Topic newtype | Validation rules, string conversion | 95% |
| Builder patterns | Required field enforcement, optional fields | 95% |
| Supporting types | Source, Payload, Metadata, Encrypted, enums | 95% |

## Protocol Impact

### Schema Impact

| Schema | Change Type | Description |
|--------|-------------|-------------|
| `signal.schema.json` | None | Types implement existing schema, no changes |
| `action.schema.json` | None | Types implement existing schema, no changes |
| `jsonrpc.schema.json` | None | Not affected by core types |
| `errors.schema.json` | None | Not affected by core types |
| `methods/*.schema.json` | None | Not affected by core types |
| `payloads/*.schema.json` | None | Not affected by core types |

### Component Interactions

| Component | Responsibility in This Feature | NOT Responsible For |
|-----------|-------------------------------|---------------------|
| **Adapter** | Creates Signal instances from external data | Action creation |
| **Hub** | Routes Signals and Actions using Topic | Creating messages |
| **Agent** | Creates Action instances to command adapters | Signal creation |

### Transport Considerations

| Transport | Supported | Notes |
|-----------|-----------|-------|
| WebSocket | N/A | Core types are transport-agnostic |
| Server-Sent Events | N/A | Core types are transport-agnostic |
| HTTP Polling | N/A | Core types are transport-agnostic |
| Webhooks | N/A | Core types are transport-agnostic |

**Semantic consistency**: Core types define message structure independent of transport. The same Signal/Action serializes identically regardless of how it's transmitted.

### Wire Protocol

- **New methods**: None - this feature defines data types, not RPC methods
- **Modified methods**: None
- **A2A impact**: None - A2A will use these types but this feature doesn't modify A2A
- **MCP impact**: None - MCP will use these types but this feature doesn't modify MCP

### Version Impact

- **Change type**: MINOR (backward compatible)
- **Rationale**: Adding new types to the library without breaking existing functionality. This is a foundational addition that enables future features.

## Assumptions

- Signal and Action ID formats follow the pattern specified in the Cauce protocol spec
- The `version` field in Signal/Action refers to the protocol version (e.g., "1.0")
- Timestamps are stored as ISO 8601 strings in UTC
- The `raw` field in Payload is a serde_json::Value to support arbitrary JSON content
- Priority defaults to Normal when not specified
- Empty optional fields should be omitted from JSON output (not serialized as null)

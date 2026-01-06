<!--
  Sync Impact Report
  ==================
  Version change: 1.0.0 → 1.1.0

  Modified principles: None

  Added sections:
  - Principle XI: Test-Driven Development (TDD with 95% coverage requirement)
  - Development Workflow: Implementation Changes section

  Removed sections: None

  Templates status:
  - .specify/templates/plan-template.md: ✅ Updated (added coverage tool, threshold, constitution check table)
  - .specify/templates/spec-template.md: ✅ Updated (added TDD Approach section with Principle XI reference)
  - .specify/templates/tasks-template.md: ✅ Updated (tests now REQUIRED, added coverage verification)
  - .specify/templates/checklist-template.md: ✅ Compatible (generic)
  - .specify/templates/agent-file-template.md: ✅ Updated (added Protocol Principles section)

  Scripts status:
  - .specify/scripts/bash/check-prerequisites.sh: ✅ Updated (added --validate-constitution and --validate-coverage flags)
  - .specify/scripts/bash/update-agent-context.sh: ✅ Updated (added constitution awareness and Protocol Principles generation)

  Follow-up TODOs: None
-->

# Cauce Protocol Constitution

## Core Principles

### I. Spec-First Development

All protocol behavior MUST be defined in the specification before implementation begins.
The specification (`cauce-protocol-spec.md`) is the source of truth for:

- Message formats and schemas
- Wire protocol semantics (JSON-RPC 2.0)
- Transport bindings and their requirements
- Error codes and retry semantics

**Rationale**: A communication protocol requires unambiguous contracts. Implementation
without specification leads to incompatible clients and undefined behavior at boundaries.

### II. Schema-Driven Contracts

Core protocol messages MUST have corresponding JSON Schemas. Adapter payload formats
are extensible.

**Core schemas (REQUIRED):**
- `signal.schema.json` - signal envelope structure
- `action.schema.json` - action envelope structure
- `jsonrpc.schema.json` - wire protocol format
- `errors.schema.json` - error codes and formats
- `methods/*.schema.json` - pub/sub operation schemas

**Payload schemas (OPTIONAL/EXTENSIBLE):**
- `payloads/*.schema.json` - adapter-specific content formats
- Adapters MAY define their own payload structures
- The `payload.raw` field accepts any source-native format

**Rationale**: Core schemas enable validation and code generation. Extensible payloads
allow new adapters (Discord, Telegram, etc.) without protocol changes.

### III. Privacy First

- TLS 1.2+ MUST be used for all connections (TLS 1.3 SHOULD be preferred)
- End-to-end encryption MUST be supported as an optional extension
- Hub MUST NOT inspect E2E encrypted payloads
- Signal routing metadata (topic, timestamp, source type) remains visible for routing

**Rationale**: Personal communications contain sensitive data. The protocol MUST enable
users to trust that their data is protected in transit and optionally opaque to intermediaries.

### IV. Transport Agnostic

The protocol defines transport bindings; implementations choose which to support.
Message semantics MUST NOT change based on transport.

**Defined transports:**
- WebSocket (full-duplex, real-time)
- Server-Sent Events (unidirectional streaming)
- HTTP Polling / Long Polling (simple clients)
- Webhooks (server-to-server push)

A signal has the same meaning whether delivered via WebSocket or webhook.

**Rationale**: Different deployment contexts have different constraints. A serverless
function cannot maintain WebSocket connections; a mobile app may need real-time delivery.
The protocol serves all contexts by defining HOW each transport works, not mandating WHICH.

### V. Interoperability

Cauce MUST integrate with the broader AI agent ecosystem:

- **JSON-RPC 2.0**: Wire protocol MUST follow this established standard
- **A2A Protocol**: Hub MUST expose A2A endpoint for external agent interaction
- **MCP Protocol**: Hub MAY expose MCP interface for tool-based access

**Rationale**: No protocol exists in isolation. Cauce fills the Human ↔ AI Agent
gap alongside MCP (AI ↔ Tools) and A2A (Agent ↔ Agent). Integration is not optional.

### VI. Component Separation

The architecture MUST maintain strict boundaries:

| Component | Responsibility | NOT Responsible For |
|-----------|---------------|---------------------|
| **Adapter** | Source translation, local queuing, delivery | AI logic, content extraction, business rules |
| **Hub** | Pub/sub routing, subscription management, delivery tracking | Business logic, content inspection, AI decisions |
| **Agent** | Prioritization, filtering, response generation | Protocol details (explicitly out of scope) |

**Rationale**: Thin hub design enables scaling and trust. Smart logic belongs in agents
where users control behavior. Adapters remain simple translators. Mixing responsibilities
creates coupling and security risks.

### VII. Reliable Delivery

Signal delivery MUST be reliable with at-least-once semantics:

- Subscribers MUST acknowledge received signals
- Hub MUST track unacknowledged signals per subscription
- Hub MUST redeliver unacknowledged signals after timeout
- Hub MUST implement exponential backoff for redelivery
- Hub MAY move signals to dead-letter queue after max retries

**Rationale**: Personal communications cannot be silently lost. The protocol guarantees
that signals reach their destination or fail explicitly.

### VIII. Adapter Resilience

Adapters MUST remain operational when Hub is unavailable:

- Adapters MUST queue signals locally when Hub is unreachable
- Adapters MUST persist queue across restarts
- Adapters MUST retry delivery with exponential backoff
- Adapters SHOULD implement queue size limits
- Adapters SHOULD implement oldest-first eviction when queue is full

**Rationale**: Communication sources don't stop when infrastructure has issues.
Adapters buffer reality until the system recovers.

### IX. Semantic Versioning

The protocol version follows semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes to message format or semantics
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, clarifications

Version negotiation occurs during handshake:
- Clients advertise supported version range
- Hub responds with negotiated version
- Hub MUST reject unsupported versions with clear error

**Rationale**: Predictable versioning enables ecosystem growth. Clients can depend on
compatibility guarantees within major versions.

### X. Graceful Degradation

Unsupported features MUST NOT cause failures:

- Capability negotiation during handshake determines available features
- Unsupported extensions MUST be ignored, not rejected
- Hub MUST advertise supported capabilities in handshake response
- Clients MUST handle missing optional features gracefully

**Rationale**: The protocol evolves over time. Older clients connecting to newer hubs
(or vice versa) MUST continue to function for the features they share.

### XI. Test-Driven Development

All implementations MUST follow Test-Driven Development (TDD) with strict coverage:

- Tests MUST be written before implementation code
- Tests MUST fail before implementation (Red-Green-Refactor cycle)
- All crates MUST maintain minimum 95% code coverage
- Coverage MUST be measured and enforced in CI
- PRs reducing coverage below 95% MUST NOT be merged

**Test categories:**
- **Unit tests**: Individual functions and modules
- **Integration tests**: Cross-module and cross-crate interactions
- **Contract tests**: Schema validation and protocol compliance

**Rationale**: A protocol implementation must be correct. TDD ensures behavior is
specified before coded, and 95% coverage ensures edge cases are tested. Untested
code in a communication system risks silent data loss or corruption.

## Technical Constraints

### Message Limits

| Item | Limit | Enforcement |
|------|-------|-------------|
| Topic name | 255 characters | Schema validation |
| Signal payload | 10 MB | Hub rejection |
| Attachments per signal | 25 MB total | Hub rejection |
| Topics per subscription | 100 | Subscription rejection |
| Subscriptions per client | 1000 | Subscription rejection |

### ID Formats

- Signal IDs: `sig_<unix_timestamp_seconds>_<random_12_chars>`
- Action IDs: `act_<unix_timestamp_seconds>_<random_12_chars>`
- Subscription IDs: `sub_<random>`
- Session IDs: `sess_<random>`

### Topic Naming Convention (Recommended)

Topics SHOULD follow the pattern: `<category>.<source/target>.<event>`

Categories: `signal` (inbound), `action` (outbound), `system` (internal)

## Development Workflow

### Schema Changes

1. Update specification section with new/changed behavior
2. Update or create JSON Schema in `schemas/`
3. Ensure schema validates example messages in spec
4. Update changelog in specification

### New Transport Binding

1. Document in Section 7 (Transport Bindings) of spec
2. Define connection, handshake, and message exchange patterns
3. Specify keepalive and acknowledgment mechanisms
4. Implementations MAY add support

### Protocol Extensions

1. Extensions MUST be documented in appendices
2. Capability negotiation during handshake determines support
3. Unsupported extensions MUST NOT cause failures (Principle X)

### Implementation Changes

1. Write failing tests that specify the desired behavior
2. Implement the minimum code to pass tests
3. Refactor while maintaining green tests
4. Verify coverage meets 95% threshold
5. Submit PR with passing CI

## Governance

This constitution governs all Cauce Protocol development. Amendments require:

1. **Proposal**: Document the change with rationale
2. **Review**: Assess impact on specification, schemas, and implementations
3. **Migration**: Define backward compatibility strategy
4. **Update**: Amend constitution with version increment

All contributions MUST verify compliance with these principles before merge.

The specification (`cauce-protocol-spec.md`) is the normative reference.
This constitution provides guiding principles; the spec provides precise behavior.

**Version**: 1.1.0 | **Ratified**: 2026-01-05 | **Last Amended**: 2026-01-06

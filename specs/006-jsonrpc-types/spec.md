# Feature Specification: JSON-RPC Types

**Feature Branch**: `006-jsonrpc-types`
**Created**: 2026-01-07
**Status**: Draft
**Input**: User description: "2.3 JSON-RPC Types from TODO.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create JSON-RPC Request Messages (Priority: P1)

A developer building a Cauce client needs to construct properly formatted JSON-RPC 2.0 request messages to send method calls to the Hub. They need a type-safe way to create requests with required fields (jsonrpc version, method name) and optional fields (id, params).

**Why this priority**: JSON-RPC requests are the foundation of all client-to-server communication. Without request types, no protocol methods can be invoked.

**Independent Test**: Can be fully tested by creating request objects with various methods and params, serializing to JSON, and validating against the JSON-RPC 2.0 specification.

**Acceptance Scenarios**:

1. **Given** a developer creates a request, **When** they specify method and params, **Then** the jsonrpc field is automatically set to "2.0"
2. **Given** a request with an id, **When** serialized to JSON, **Then** the output conforms to JSON-RPC 2.0 request format
3. **Given** a request id as either string or integer, **When** the request is created, **Then** both id types are accepted and preserved correctly

---

### User Story 2 - Parse JSON-RPC Response Messages (Priority: P1)

A developer receiving responses from the Hub needs to parse them into typed structures that distinguish between success responses (containing a result) and error responses (containing an error object). This enables proper error handling and result extraction.

**Why this priority**: Responses complete the request/response cycle. Without response types, clients cannot process Hub replies.

**Independent Test**: Can be fully tested by deserializing JSON response strings into response types and verifying correct variant detection (success vs error).

**Acceptance Scenarios**:

1. **Given** a JSON string with "result" field, **When** parsed as response, **Then** it is recognized as a success response
2. **Given** a JSON string with "error" field, **When** parsed as response, **Then** it is recognized as an error response
3. **Given** a response with id matching a request, **When** parsed, **Then** the id is correctly extracted for correlation

---

### User Story 3 - Send JSON-RPC Notifications (Priority: P2)

A developer needs to send one-way messages (notifications) that don't expect a response. Notifications are requests without an id field and are used for events like ping/keepalive or signal delivery.

**Why this priority**: Notifications enable efficient one-way communication patterns but are secondary to the core request/response flow.

**Independent Test**: Can be fully tested by creating notification objects, verifying they have no id field, and confirming proper serialization.

**Acceptance Scenarios**:

1. **Given** a developer creates a notification, **When** serialized, **Then** no "id" field appears in the JSON output
2. **Given** a method name and params, **When** a notification is created, **Then** jsonrpc is set to "2.0" automatically

---

### User Story 4 - Handle JSON-RPC Errors (Priority: P2)

A developer needs structured error information when operations fail. Error objects contain a code (integer), message (string), and optional data for additional context. Standard JSON-RPC error codes and Cauce-specific codes must be representable.

**Why this priority**: Proper error handling is essential for robust clients but depends on the response types being in place.

**Independent Test**: Can be fully tested by creating error objects with various codes and data, serializing them, and validating the structure.

**Acceptance Scenarios**:

1. **Given** an error with code, message, and data, **When** serialized, **Then** all fields are correctly included in the error object
2. **Given** standard JSON-RPC error codes (-32700, -32600, etc.), **When** creating errors, **Then** these codes are representable
3. **Given** an error without optional data, **When** serialized, **Then** the data field is omitted

---

### User Story 5 - Build Responses Easily (Priority: P3)

A developer implementing a Hub or server component needs convenient helper methods to construct success responses from results and error responses from error objects. This reduces boilerplate and ensures correct structure.

**Why this priority**: Helper methods improve developer ergonomics but are not required for basic functionality.

**Independent Test**: Can be fully tested by using helper methods to create responses and verifying output matches expected JSON structure.

**Acceptance Scenarios**:

1. **Given** a result value and request id, **When** using a success helper, **Then** a properly structured success response is created
2. **Given** an error object and request id, **When** using an error helper, **Then** a properly structured error response is created
3. **Given** any response created via helpers, **When** serialized, **Then** it conforms to JSON-RPC 2.0 specification

---

### Edge Cases

- What happens when deserializing a response with both "result" and "error" fields? (Invalid per spec - must error)
- How does the system handle a request with jsonrpc field not equal to "2.0"? (Must reject with validation error)
- What happens when deserializing a request with null id vs missing id? (null id = valid request expecting response; missing id = notification)
- How are empty params (null vs missing vs empty object) handled? (All are valid - null and missing are equivalent, empty object is a valid params value)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a Request type with fields: jsonrpc (always "2.0"), id (required), method (required), params (optional)
- **FR-002**: System MUST provide a Response type that can represent either success (with result) or error (with error object)
- **FR-003**: System MUST provide a Notification type with fields: jsonrpc (always "2.0"), method (required), params (optional), but no id field
- **FR-004**: System MUST provide an Error type with fields: code (integer), message (string), data (optional)
- **FR-005**: System MUST provide a RequestId type that accepts either string or integer values
- **FR-006**: System MUST validate that jsonrpc field equals "2.0" during deserialization
- **FR-007**: System MUST provide helper method to create success response from result and id
- **FR-008**: System MUST provide helper method to create error response from error object and id
- **FR-009**: System MUST serialize optional fields correctly (omit when None, include when Some)
- **FR-010**: System MUST preserve id type through serialization roundtrip (string stays string, integer stays integer)
- **FR-011**: System MUST reject responses that contain both "result" and "error" fields during deserialization
- **FR-012**: System MUST support null id values for error responses to unidentifiable requests (per JSON-RPC 2.0 spec)

### Key Entities

- **Request**: A JSON-RPC 2.0 request message with jsonrpc version, id, method name, and optional params
- **Response**: A JSON-RPC 2.0 response that is either success (id + result) or error (id + error object)
- **Notification**: A JSON-RPC 2.0 notification (request without id, no response expected)
- **Error**: A JSON-RPC 2.0 error object with numeric code, string message, and optional structured data
- **RequestId**: A polymorphic identifier that can be either a string or an integer

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All JSON-RPC types can be serialized and deserialized with 100% fidelity (roundtrip test)
- **SC-002**: Invalid JSON-RPC messages (wrong version, both result and error) are rejected with clear error messages
- **SC-003**: Developers can create request, response, and notification types in a single line using builder or constructor methods
- **SC-004**: Serialized output exactly matches JSON-RPC 2.0 specification format
- **SC-005**: All 12 functional requirements have passing tests demonstrating correct behavior
- **SC-006**: Code coverage for the jsonrpc module reaches minimum 95%

## Test-Driven Development Approach *(mandatory)*

### Testing Strategy

- **Unit Tests**: Test each type's serialization/deserialization, validation logic, and helper methods independently
- **Integration Tests**: Test roundtrip serialization through the full serialize/deserialize cycle
- **Contract Tests**: Validate serialized output against JSON-RPC 2.0 specification examples

### Coverage Requirement

Per Constitution Principle XI, this feature MUST:
- Have tests written BEFORE implementation code
- Follow Red-Green-Refactor cycle
- Achieve minimum **95% code coverage**
- Pass all tests in CI before merge

### Test Boundaries

| Component       | Test Focus                              | Coverage Target |
|-----------------|----------------------------------------|-----------------|
| Request         | Construction, serialization, id handling | 95%             |
| Response        | Success/error variants, deserialization | 95%             |
| Notification    | No-id constraint, serialization         | 95%             |
| Error           | Code/message/data handling              | 95%             |
| RequestId       | String/integer polymorphism             | 95%             |
| Helpers         | Response construction convenience       | 95%             |

## Protocol Impact *(Cauce-specific)*

### Schema Impact

| Schema                   | Change Type | Description                      |
|--------------------------|-------------|----------------------------------|
| `signal.schema.json`     | None        | N/A                              |
| `action.schema.json`     | None        | N/A                              |
| `jsonrpc.schema.json`    | New         | Core JSON-RPC 2.0 message types  |
| `errors.schema.json`     | None        | Error codes defined separately in 2.5 |
| `methods/*.schema.json`  | None        | Method params defined separately in 2.4 |
| `payloads/*.schema.json` | None        | N/A                              |

### Component Interactions

| Component    | Responsibility in This Feature           | NOT Responsible For           |
|--------------|------------------------------------------|-------------------------------|
| **Adapter**  | Uses these types to communicate with Hub | Defining protocol methods     |
| **Hub**      | Uses these types to parse requests and send responses | Message routing logic |
| **Agent**    | Uses these types to communicate via Hub  | Transport handling            |

### Transport Considerations

| Transport           | Supported | Notes                                |
|---------------------|-----------|--------------------------------------|
| WebSocket           | Yes       | Messages sent as text frames         |
| Server-Sent Events  | Yes       | Responses/notifications in event data |
| HTTP Polling        | Yes       | Request/response in HTTP body        |
| Webhooks            | Yes       | Notifications in webhook payload     |

**Semantic consistency**: JSON-RPC message structure is identical regardless of transport - only the framing differs.

### Wire Protocol

- **New methods**: None (this feature provides types, not methods)
- **Modified methods**: None
- **A2A impact**: None (A2A uses these types internally)
- **MCP impact**: None (MCP uses these types internally)

### Version Impact

- **Change type**: MINOR (backward compatible)
- **Rationale**: Adds new types without changing existing functionality. Existing cauce-core users are unaffected.

## Assumptions

1. JSON-RPC 2.0 specification (https://www.jsonrpc.org/specification) is the authoritative reference
2. Batch requests (arrays of requests) are out of scope for this feature - will be added if needed later
3. The `params` field uses flexible JSON value type to support any valid JSON structure
4. Error data field uses flexible JSON value type for extensibility

## Dependencies

- cauce-core crate (existing types module for re-exports)
- Serialization library for JSON handling
- Error handling library for validation errors

## Out of Scope

- Method-specific parameter types (covered in 2.4 Method Parameter Types)
- Cauce-specific error codes (covered in 2.5 Error Codes)
- Batch request/response handling
- Transport-level framing

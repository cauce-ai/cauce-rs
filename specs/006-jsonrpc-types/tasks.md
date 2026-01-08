# Tasks: JSON-RPC Types

**Input**: Design documents from `/specs/006-jsonrpc-types/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Tests are REQUIRED per Constitution Principle XI (TDD). All implementations MUST:
- Write failing tests BEFORE implementation code
- Maintain minimum 95% code coverage
- Include unit, integration, and contract tests

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Project root**: `crates/cauce-core/`
- **Source**: `crates/cauce-core/src/jsonrpc/`
- **Tests**: `crates/cauce-core/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Module structure and shared dependencies

- [x] T001 Create jsonrpc module structure with mod.rs in crates/cauce-core/src/jsonrpc/mod.rs
- [x] T002 [P] Create id.rs file for RequestId type in crates/cauce-core/src/jsonrpc/id.rs
- [x] T003 [P] Create error.rs file for JsonRpcError type in crates/cauce-core/src/jsonrpc/error.rs
- [x] T004 [P] Create request.rs file for JsonRpcRequest type in crates/cauce-core/src/jsonrpc/request.rs
- [x] T005 [P] Create notification.rs file for JsonRpcNotification type in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T006 [P] Create response.rs file for JsonRpcResponse type in crates/cauce-core/src/jsonrpc/response.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: RequestId type that all other types depend on

**⚠️ CRITICAL**: RequestId must be complete before Request/Response/Notification can use it

### Tests for RequestId (REQUIRED - TDD)

- [x] T007 [P] Write unit tests for RequestId string variant serialization in crates/cauce-core/src/jsonrpc/id.rs
- [x] T008 [P] Write unit tests for RequestId integer variant serialization in crates/cauce-core/src/jsonrpc/id.rs
- [x] T009 [P] Write unit tests for RequestId roundtrip (type preservation) in crates/cauce-core/src/jsonrpc/id.rs

### Implementation for RequestId

- [x] T010 Implement RequestId enum with String and Number variants in crates/cauce-core/src/jsonrpc/id.rs
- [x] T011 Implement serde Serialize/Deserialize for RequestId (untagged) in crates/cauce-core/src/jsonrpc/id.rs
- [x] T012 Add RequestId helper constructors (from_string, from_number) in crates/cauce-core/src/jsonrpc/id.rs

**Checkpoint**: RequestId ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Create JSON-RPC Request Messages (Priority: P1) 🎯 MVP

**Goal**: Developers can construct properly formatted JSON-RPC 2.0 request messages

**Independent Test**: Create request objects with various methods and params, serialize to JSON, validate against JSON-RPC 2.0 specification

### Tests for User Story 1 (REQUIRED - TDD)

- [x] T013 [P] [US1] Write test: request serialization includes jsonrpc "2.0" in crates/cauce-core/src/jsonrpc/request.rs
- [x] T014 [P] [US1] Write test: request with string id serializes correctly in crates/cauce-core/src/jsonrpc/request.rs
- [x] T015 [P] [US1] Write test: request with integer id serializes correctly in crates/cauce-core/src/jsonrpc/request.rs
- [x] T016 [P] [US1] Write test: request with params serializes correctly in crates/cauce-core/src/jsonrpc/request.rs
- [x] T017 [P] [US1] Write test: request without params omits params field in crates/cauce-core/src/jsonrpc/request.rs
- [x] T018 [P] [US1] Write test: request deserialization validates jsonrpc == "2.0" in crates/cauce-core/src/jsonrpc/request.rs

### Implementation for User Story 1

- [x] T019 [US1] Implement JsonRpcRequest struct with fields (jsonrpc, id, method, params) in crates/cauce-core/src/jsonrpc/request.rs
- [x] T020 [US1] Implement serde Serialize for JsonRpcRequest (skip_serializing_if for params) in crates/cauce-core/src/jsonrpc/request.rs
- [x] T021 [US1] Implement serde Deserialize with jsonrpc "2.0" validation in crates/cauce-core/src/jsonrpc/request.rs
- [x] T022 [US1] Add JsonRpcRequest::new() constructor in crates/cauce-core/src/jsonrpc/request.rs
- [x] T023 [US1] Export JsonRpcRequest from mod.rs in crates/cauce-core/src/jsonrpc/mod.rs

**Checkpoint**: User Story 1 complete - requests can be created and serialized

---

## Phase 4: User Story 2 - Parse JSON-RPC Response Messages (Priority: P1)

**Goal**: Developers can parse responses and distinguish success from error

**Independent Test**: Deserialize JSON response strings, verify correct variant detection (success vs error)

### Tests for User Story 2 (REQUIRED - TDD)

- [x] T024 [P] [US2] Write test: success response deserializes with result field in crates/cauce-core/src/jsonrpc/response.rs
- [x] T025 [P] [US2] Write test: error response deserializes with error field in crates/cauce-core/src/jsonrpc/response.rs
- [x] T026 [P] [US2] Write test: response id is correctly extracted for correlation in crates/cauce-core/src/jsonrpc/response.rs
- [x] T027 [P] [US2] Write test: response with both result and error is rejected (FR-011) in crates/cauce-core/src/jsonrpc/response.rs
- [x] T028 [P] [US2] Write test: null id in error response is accepted (FR-012) in crates/cauce-core/src/jsonrpc/response.rs
- [x] T029 [P] [US2] Write test: success response serializes correctly in crates/cauce-core/src/jsonrpc/response.rs
- [x] T030 [P] [US2] Write test: error response serializes correctly in crates/cauce-core/src/jsonrpc/response.rs

### Implementation for User Story 2

- [x] T031 [US2] Implement JsonRpcResponse enum with Success and Error variants in crates/cauce-core/src/jsonrpc/response.rs
- [x] T032 [US2] Implement custom Deserialize to reject responses with both result and error in crates/cauce-core/src/jsonrpc/response.rs
- [x] T033 [US2] Implement Serialize for JsonRpcResponse (untagged variants) in crates/cauce-core/src/jsonrpc/response.rs
- [x] T034 [US2] Add JsonRpcResponse accessor methods (id, is_success, is_error) in crates/cauce-core/src/jsonrpc/response.rs
- [x] T035 [US2] Export JsonRpcResponse from mod.rs in crates/cauce-core/src/jsonrpc/mod.rs

**Checkpoint**: User Stories 1 AND 2 complete - full request/response cycle works

---

## Phase 5: User Story 3 - Send JSON-RPC Notifications (Priority: P2)

**Goal**: Developers can send one-way messages that don't expect a response

**Independent Test**: Create notification objects, verify no id field in serialized output

### Tests for User Story 3 (REQUIRED - TDD)

- [x] T036 [P] [US3] Write test: notification serialization has no id field in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T037 [P] [US3] Write test: notification includes jsonrpc "2.0" in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T038 [P] [US3] Write test: notification with params serializes correctly in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T039 [P] [US3] Write test: notification without params omits params field in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T040 [P] [US3] Write test: notification deserialization validates jsonrpc == "2.0" in crates/cauce-core/src/jsonrpc/notification.rs

### Implementation for User Story 3

- [x] T041 [US3] Implement JsonRpcNotification struct (jsonrpc, method, params - no id) in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T042 [US3] Implement serde Serialize/Deserialize for JsonRpcNotification in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T043 [US3] Add JsonRpcNotification::new() constructor in crates/cauce-core/src/jsonrpc/notification.rs
- [x] T044 [US3] Export JsonRpcNotification from mod.rs in crates/cauce-core/src/jsonrpc/mod.rs

**Checkpoint**: User Story 3 complete - notifications can be sent

---

## Phase 6: User Story 4 - Handle JSON-RPC Errors (Priority: P2)

**Goal**: Developers can create and parse structured error information

**Independent Test**: Create error objects with various codes and data, serialize, validate structure

### Tests for User Story 4 (REQUIRED - TDD)

- [x] T045 [P] [US4] Write test: error with code, message, data serializes all fields in crates/cauce-core/src/jsonrpc/error.rs
- [x] T046 [P] [US4] Write test: error without data omits data field in crates/cauce-core/src/jsonrpc/error.rs
- [x] T047 [P] [US4] Write test: standard error codes (-32700, -32600, etc.) are representable in crates/cauce-core/src/jsonrpc/error.rs
- [x] T048 [P] [US4] Write test: error deserialization works correctly in crates/cauce-core/src/jsonrpc/error.rs
- [x] T049 [P] [US4] Write test: error roundtrip preserves all fields in crates/cauce-core/src/jsonrpc/error.rs

### Implementation for User Story 4

- [x] T050 [US4] Implement JsonRpcError struct (code, message, data) in crates/cauce-core/src/jsonrpc/error.rs
- [x] T051 [US4] Implement serde Serialize/Deserialize with skip_serializing_if for data in crates/cauce-core/src/jsonrpc/error.rs
- [x] T052 [US4] Add JsonRpcError::new() and ::with_data() constructors in crates/cauce-core/src/jsonrpc/error.rs
- [x] T053 [US4] Export JsonRpcError from mod.rs in crates/cauce-core/src/jsonrpc/mod.rs

**Checkpoint**: User Story 4 complete - errors can be created and parsed

---

## Phase 7: User Story 5 - Build Responses Easily (Priority: P3)

**Goal**: Developers have convenient helper methods to construct responses

**Independent Test**: Use helper methods to create responses, verify output matches expected JSON structure

### Tests for User Story 5 (REQUIRED - TDD)

- [x] T054 [P] [US5] Write test: success() helper creates valid success response in crates/cauce-core/src/jsonrpc/response.rs
- [x] T055 [P] [US5] Write test: error() helper creates valid error response in crates/cauce-core/src/jsonrpc/response.rs
- [x] T056 [P] [US5] Write test: error() helper with null id creates valid response in crates/cauce-core/src/jsonrpc/response.rs
- [x] T057 [P] [US5] Write test: helper-created responses serialize to valid JSON-RPC 2.0 in crates/cauce-core/src/jsonrpc/response.rs

### Implementation for User Story 5

- [x] T058 [US5] Add JsonRpcResponse::success(id, result) helper in crates/cauce-core/src/jsonrpc/response.rs
- [x] T059 [US5] Add JsonRpcResponse::error(id, error) helper in crates/cauce-core/src/jsonrpc/response.rs
- [x] T060 [US5] Add convenience accessors (result(), error_obj(), into_result()) in crates/cauce-core/src/jsonrpc/response.rs

**Checkpoint**: All user stories complete - full JSON-RPC type system implemented

---

## Phase 8: Verification & Polish

**Purpose**: Coverage verification, re-exports, and integration tests

### Coverage Verification (REQUIRED)

- [x] T061 Run coverage report and verify ≥95% threshold with cargo llvm-cov (98.22% achieved)
- [x] T062 Add missing unit tests to reach coverage target if needed (not needed - already at 98.22%)

### Integration & Re-exports

- [x] T063 Create integration test file for JSON-RPC types in crates/cauce-core/tests/jsonrpc_test.rs
- [x] T064 [P] Write integration test: request/response roundtrip in crates/cauce-core/tests/jsonrpc_test.rs
- [x] T065 [P] Write integration test: all types can be imported from crate root in crates/cauce-core/tests/jsonrpc_test.rs
- [x] T066 Add JSON-RPC type re-exports to lib.rs in crates/cauce-core/src/lib.rs
- [x] T067 Verify quickstart.md examples work with cargo test --doc (72 doc tests passed)

### Polish

- [x] T068 Add module-level documentation to mod.rs in crates/cauce-core/src/jsonrpc/mod.rs
- [x] T069 Add doc comments with examples to all public types and methods

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel (both P1)
  - US3 and US4 can proceed in parallel (both P2)
  - US5 depends on US2 (needs Response type)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Phase 2 (RequestId) - No dependencies on other stories
- **User Story 2 (P1)**: Depends on Phase 2 (RequestId) and US4 (JsonRpcError) - Can run in parallel with US1 if error.rs is done first
- **User Story 3 (P2)**: Depends on Phase 2 only - Independent of other stories
- **User Story 4 (P2)**: Depends on Phase 2 only - Independent of other stories
- **User Story 5 (P3)**: Depends on US2 (extends Response with helpers) and US4 (uses Error)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Types before helpers
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- T002-T006 (Setup file creation) can all run in parallel
- T007-T009 (RequestId tests) can all run in parallel
- All tests within each user story marked [P] can run in parallel
- US1 and US2 can run in parallel after Phase 2
- US3 and US4 can run in parallel
- T064-T065 (integration tests) can run in parallel

---

## Parallel Example: Phase 2 (RequestId)

```bash
# Launch all tests together:
Task: "Write unit tests for RequestId string variant serialization"
Task: "Write unit tests for RequestId integer variant serialization"
Task: "Write unit tests for RequestId roundtrip (type preservation)"

# Then implement sequentially (same file):
Task: "Implement RequestId enum"
Task: "Implement serde Serialize/Deserialize"
Task: "Add RequestId helper constructors"
```

---

## Parallel Example: User Story 1

```bash
# Launch all tests together:
Task: "Write test: request serialization includes jsonrpc '2.0'"
Task: "Write test: request with string id serializes correctly"
Task: "Write test: request with integer id serializes correctly"
Task: "Write test: request with params serializes correctly"
Task: "Write test: request without params omits params field"
Task: "Write test: request deserialization validates jsonrpc == '2.0'"

# Then implement sequentially (dependencies):
Task: "Implement JsonRpcRequest struct"
Task: "Implement serde Serialize"
Task: "Implement serde Deserialize with validation"
Task: "Add constructor"
Task: "Export from mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (RequestId)
3. Complete Phase 3: User Story 1 (Request)
4. **STOP and VALIDATE**: Test request creation and serialization
5. Can deploy if only requests are needed

### Incremental Delivery

1. Complete Setup + Foundational → RequestId ready
2. Add User Story 1 → Request types work → Partial functionality
3. Add User Story 2 → Response types work → Full request/response cycle
4. Add User Story 3 → Notifications work → One-way messaging
5. Add User Story 4 → Structured errors work → Better error handling
6. Add User Story 5 → Helper methods → Improved ergonomics

### Optimal Parallel Strategy

With two developers:
1. Both complete Setup + Foundational together
2. After Phase 2:
   - Developer A: US1 (Request) → US3 (Notification)
   - Developer B: US4 (Error) → US2 (Response) → US5 (Helpers)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Verify tests fail before implementing (TDD)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All types use untagged serde enums where polymorphism is needed (RequestId, Response)

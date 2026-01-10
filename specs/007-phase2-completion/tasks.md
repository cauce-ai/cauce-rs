# Tasks: Phase 2 Completion - Cauce Core Library

**Input**: Design documents from `/specs/007-phase2-completion/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

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

All paths are relative to `crates/cauce-core/`:
- Source files: `src/`
- Unit tests: `src/<module>/mod.rs` (inline tests)
- Integration tests: `tests/`

---

## Phase 1: Setup (Module Infrastructure)

**Purpose**: Create new module structure and prepare for implementation

- [X] T001 Create `src/methods/` directory with `mod.rs` in crates/cauce-core/src/methods/mod.rs
- [X] T002 [P] Create `src/id/` directory with `mod.rs` in crates/cauce-core/src/id/mod.rs
- [X] T003 [P] Create `src/matching/` directory with `mod.rs` in crates/cauce-core/src/matching/mod.rs
- [X] T004 [P] Create `src/errors/protocol.rs` for CauceError in crates/cauce-core/src/errors/protocol.rs
- [X] T005 [P] Create `src/validation/schema.rs` for schema validation in crates/cauce-core/src/validation/schema.rs
- [X] T005b [P] Create `src/schemas/` directory with embedded JSON schemas (signal, action, jsonrpc, errors) using include_str! in crates/cauce-core/src/schemas/mod.rs
- [X] T006 Update `src/lib.rs` to declare new modules in crates/cauce-core/src/lib.rs

---

## Phase 2: Foundational (Shared Enums and Base Types)

**Purpose**: Implement enums and base types used by multiple user stories

**⚠️ CRITICAL**: User Story 1 depends on these shared types

### Tests for Foundational Types (REQUIRED - TDD) ✅

- [X] T007 [P] Write tests for AuthType enum serialization in crates/cauce-core/src/methods/auth.rs
- [X] T008 [P] Write tests for ClientType, Capability enums in crates/cauce-core/src/methods/client.rs
- [X] T009 [P] Write tests for Transport enum serialization in crates/cauce-core/src/methods/transport.rs
- [X] T010 [P] Write tests for ApprovalType, SubscriptionStatus enums in crates/cauce-core/src/methods/enums.rs

### Implementation for Foundational Types

- [X] T011 [P] Implement AuthType enum and Auth struct in crates/cauce-core/src/methods/auth.rs
- [X] T012 [P] Implement ClientType and Capability enums in crates/cauce-core/src/methods/client.rs
- [X] T013 [P] Implement Transport, WebhookConfig, E2eConfig in crates/cauce-core/src/methods/transport.rs
- [X] T014 [P] Implement ApprovalType and SubscriptionStatus enums in crates/cauce-core/src/methods/enums.rs
- [X] T015 Update methods/mod.rs to export foundational types in crates/cauce-core/src/methods/mod.rs

**Checkpoint**: Foundational types ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Method Parameter Types (Priority: P1) 🎯 MVP

**Goal**: Developers can create type-safe request/response objects for all Cauce Protocol JSON-RPC methods

**Independent Test**: Create HelloRequest, SubscribeRequest, PublishRequest and verify JSON serialization roundtrips correctly

### Tests for User Story 1 (REQUIRED - TDD) ✅

> **TDD: Write these tests FIRST, ensure they FAIL before implementation (Red-Green-Refactor)**

- [X] T016 [P] [US1] Write tests for HelloRequest/Response serialization in crates/cauce-core/src/methods/hello.rs
- [X] T017 [P] [US1] Write tests for SubscribeRequest/Response serialization in crates/cauce-core/src/methods/subscribe.rs
- [X] T018 [P] [US1] Write tests for UnsubscribeRequest/Response serialization in crates/cauce-core/src/methods/unsubscribe.rs
- [X] T019 [P] [US1] Write tests for PublishRequest/Response and PublishMessage enum in crates/cauce-core/src/methods/publish.rs
- [X] T020 [P] [US1] Write tests for AckRequest/Response and AckFailure in crates/cauce-core/src/methods/ack.rs
- [X] T021 [P] [US1] Write tests for subscription management types in crates/cauce-core/src/methods/subscription.rs
- [X] T022 [P] [US1] Write tests for PingParams/PongParams in crates/cauce-core/src/methods/ping.rs
- [X] T023 [P] [US1] Write tests for SignalDelivery struct in crates/cauce-core/src/methods/signal_delivery.rs
- [X] T024 [P] [US1] Write tests for schema method types in crates/cauce-core/src/methods/schemas.rs

### Implementation for User Story 1

- [X] T025 [P] [US1] Implement HelloRequest and HelloResponse in crates/cauce-core/src/methods/hello.rs
- [X] T026 [P] [US1] Implement SubscribeRequest and SubscribeResponse in crates/cauce-core/src/methods/subscribe.rs
- [X] T027 [P] [US1] Implement UnsubscribeRequest and UnsubscribeResponse in crates/cauce-core/src/methods/unsubscribe.rs
- [X] T028 [P] [US1] Implement PublishMessage, PublishRequest, PublishResponse in crates/cauce-core/src/methods/publish.rs
- [X] T029 [P] [US1] Implement AckRequest, AckResponse, AckFailure in crates/cauce-core/src/methods/ack.rs
- [X] T030 [US1] Implement subscription management types (SubscriptionApproveRequest, SubscriptionDenyRequest, SubscriptionRevokeRequest, SubscriptionListRequest, SubscriptionListResponse, SubscriptionInfo, SubscriptionRestrictions, SubscriptionStatusNotification) in crates/cauce-core/src/methods/subscription.rs
- [X] T031 [P] [US1] Implement PingParams and PongParams in crates/cauce-core/src/methods/ping.rs
- [X] T032 [P] [US1] Implement SignalDelivery struct in crates/cauce-core/src/methods/signal_delivery.rs
- [X] T033 [P] [US1] Implement SchemasListRequest, SchemasListResponse, SchemasGetRequest, SchemasGetResponse, SchemaInfo in crates/cauce-core/src/methods/schemas.rs
- [X] T034 [US1] Update methods/mod.rs to export all method types in crates/cauce-core/src/methods/mod.rs
- [X] T035 [US1] Add re-exports to lib.rs for all method parameter types in crates/cauce-core/src/lib.rs

**Checkpoint**: User Story 1 complete - all 21 method parameter types serialize/deserialize correctly

---

## Phase 4: User Story 2 - Protocol Error Codes (Priority: P1)

**Goal**: Developers can create and handle all 20 Cauce Protocol error codes with proper JsonRpcError conversion

**Independent Test**: Create each CauceError variant and verify code(), message(), and JsonRpcError conversion

### Tests for User Story 2 (REQUIRED - TDD) ✅

- [X] T036 [P] [US2] Write tests for JSON-RPC standard error variants (ParseError, InvalidRequest, MethodNotFound, InvalidParams, InternalError) in crates/cauce-core/src/errors/protocol.rs
- [X] T037 [P] [US2] Write tests for Cauce protocol error variants (all 15 protocol-specific errors) in crates/cauce-core/src/errors/protocol.rs
- [X] T038 [P] [US2] Write tests for CauceError code() and message() methods in crates/cauce-core/src/errors/protocol.rs
- [X] T039 [P] [US2] Write tests for From<CauceError> for JsonRpcError conversion in crates/cauce-core/src/errors/protocol.rs
- [ ] T040 [P] [US2] Write integration test for error code roundtrip in crates/cauce-core/tests/error_codes.rs

### Implementation for User Story 2

- [X] T041 [US2] Implement CauceError enum with all 20 variants in crates/cauce-core/src/errors/protocol.rs
- [X] T042 [US2] Implement code() method returning correct error codes in crates/cauce-core/src/errors/protocol.rs
- [X] T043 [US2] Implement message() method returning standard messages in crates/cauce-core/src/errors/protocol.rs
- [X] T044 [US2] Implement Display trait for CauceError in crates/cauce-core/src/errors/protocol.rs
- [X] T045 [US2] Implement From<CauceError> for JsonRpcError with data field population in crates/cauce-core/src/errors/protocol.rs
- [X] T046 [US2] Update errors/mod.rs to export CauceError in crates/cauce-core/src/errors/mod.rs
- [X] T047 [US2] Add CauceError re-export to lib.rs in crates/cauce-core/src/lib.rs

**Checkpoint**: User Story 2 complete - all 20 error codes work with correct JsonRpcError conversion

---

## Phase 5: User Story 3 - Method Constants and Limits (Priority: P2)

**Goal**: Developers can use predefined constants for all method names and protocol limits

**Independent Test**: Verify each constant has the expected value

### Tests for User Story 3 (REQUIRED - TDD) ✅

- [X] T048 [P] [US3] Write tests for all 17 method name constants in crates/cauce-core/src/constants/mod.rs
- [X] T049 [P] [US3] Write tests for all 5 new size limit constants in crates/cauce-core/src/constants/mod.rs (note: MAX_TOPIC_LENGTH exists as TOPIC_MAX_LENGTH)

### Implementation for User Story 3

- [X] T050 [US3] Add METHOD_HELLO, METHOD_GOODBYE, METHOD_PING, METHOD_PONG constants in crates/cauce-core/src/constants/mod.rs
- [X] T051 [US3] Add METHOD_PUBLISH, METHOD_SUBSCRIBE, METHOD_UNSUBSCRIBE, METHOD_SIGNAL, METHOD_ACK constants in crates/cauce-core/src/constants/mod.rs
- [X] T052 [US3] Add METHOD_SUBSCRIPTION_REQUEST, METHOD_SUBSCRIPTION_APPROVE, METHOD_SUBSCRIPTION_DENY, METHOD_SUBSCRIPTION_LIST, METHOD_SUBSCRIPTION_REVOKE, METHOD_SUBSCRIPTION_STATUS constants in crates/cauce-core/src/constants/mod.rs
- [X] T053 [US3] Add METHOD_SCHEMAS_LIST, METHOD_SCHEMAS_GET constants in crates/cauce-core/src/constants/mod.rs
- [X] T054 [US3] Add MAX_SIGNAL_PAYLOAD_SIZE, MAX_TOPICS_PER_SUBSCRIPTION, MAX_SUBSCRIPTIONS_PER_CLIENT, MAX_SIGNALS_PER_BATCH, MAX_TOPIC_DEPTH constants in crates/cauce-core/src/constants/mod.rs
- [X] T054b [US3] Add MAX_TOPIC_LENGTH as alias for TOPIC_MAX_LENGTH in crates/cauce-core/src/constants/mod.rs
- [X] T055 [US3] Add re-exports for new constants to lib.rs in crates/cauce-core/src/lib.rs

**Checkpoint**: User Story 3 complete - all 24 constants defined and exported (17 methods + 6 limits + MAX_TOPIC_LENGTH alias)

---

## Phase 6: User Story 4 - ID Generation Utilities (Priority: P2)

**Goal**: Developers can generate properly formatted unique IDs for all protocol entities

**Independent Test**: Generate IDs and validate format with regex, verify uniqueness

### Tests for User Story 4 (REQUIRED - TDD) ✅

- [X] T056 [P] [US4] Write tests for generate_signal_id() format and uniqueness in crates/cauce-core/src/id/mod.rs
- [X] T057 [P] [US4] Write tests for generate_action_id() format and uniqueness in crates/cauce-core/src/id/mod.rs
- [X] T058 [P] [US4] Write tests for generate_subscription_id() format (sub_<uuid>) in crates/cauce-core/src/id/mod.rs
- [X] T059 [P] [US4] Write tests for generate_session_id() format (sess_<uuid>) in crates/cauce-core/src/id/mod.rs
- [X] T060 [P] [US4] Write tests for generate_message_id() format (msg_<uuid>) in crates/cauce-core/src/id/mod.rs

### Implementation for User Story 4

- [X] T061 [P] [US4] Implement generate_signal_id() returning sig_<timestamp>_<random12> in crates/cauce-core/src/id/mod.rs
- [X] T062 [P] [US4] Implement generate_action_id() returning act_<timestamp>_<random12> in crates/cauce-core/src/id/mod.rs
- [X] T063 [P] [US4] Implement generate_subscription_id() returning sub_<uuid> in crates/cauce-core/src/id/mod.rs
- [X] T064 [P] [US4] Implement generate_session_id() returning sess_<uuid> in crates/cauce-core/src/id/mod.rs
- [X] T065 [P] [US4] Implement generate_message_id() returning msg_<uuid> in crates/cauce-core/src/id/mod.rs
- [X] T066 [US4] Add re-exports for ID generation functions to lib.rs in crates/cauce-core/src/lib.rs

**Checkpoint**: User Story 4 complete - all 5 ID generators produce correctly formatted unique IDs

---

## Phase 7: User Story 5 - Topic Pattern Matching (Priority: P2)

**Goal**: Developers can match topics against subscription patterns with `*` and `**` wildcards

**Independent Test**: Match various topics against patterns and verify wildcard behavior

### Tests for User Story 5 (REQUIRED - TDD) ✅

- [X] T067 [P] [US5] Write tests for exact topic matching (no wildcards) in crates/cauce-core/src/matching/mod.rs
- [X] T068 [P] [US5] Write tests for single-segment wildcard `*` matching in crates/cauce-core/src/matching/mod.rs
- [X] T069 [P] [US5] Write tests for multi-segment wildcard `**` matching in crates/cauce-core/src/matching/mod.rs
- [X] T070 [P] [US5] Write tests for TopicMatcher::matches_any() in crates/cauce-core/src/matching/mod.rs
- [X] T071 [P] [US5] Write tests for edge cases (empty topic, empty pattern, leading/trailing wildcards) in crates/cauce-core/src/matching/mod.rs
- [ ] T072 [P] [US5] Write integration test for topic matching patterns in crates/cauce-core/tests/topic_matching.rs

### Implementation for User Story 5

- [X] T073 [US5] Implement matches_segments() recursive helper function in crates/cauce-core/src/matching/mod.rs
- [X] T074 [US5] Implement TopicMatcher::matches() with wildcard support in crates/cauce-core/src/matching/mod.rs
- [X] T075 [US5] Implement TopicMatcher::matches_any() in crates/cauce-core/src/matching/mod.rs
- [X] T076 [US5] Implement topic_matches() convenience function in crates/cauce-core/src/matching/mod.rs
- [X] T077 [US5] Add re-exports for topic matching to lib.rs in crates/cauce-core/src/lib.rs

**Checkpoint**: User Story 5 complete - topic matching handles all wildcard patterns correctly

---

## Phase 8: User Story 6 - Enhanced Validation (Priority: P3)

**Goal**: Developers can validate signals, actions, topic patterns, and new ID formats

**Independent Test**: Pass valid/invalid JSON and patterns to validation functions

### Tests for User Story 6 (REQUIRED - TDD) ✅

- [X] T077b [P] [US6] Write tests for validate_signal() with valid/invalid JSON in crates/cauce-core/src/validation/schema.rs
- [X] T077c [P] [US6] Write tests for validate_action() with valid/invalid JSON in crates/cauce-core/src/validation/schema.rs
- [X] T078 [P] [US6] Write tests for validate_topic_pattern() with wildcards in crates/cauce-core/src/validation/mod.rs
- [X] T079 [P] [US6] Write tests for is_valid_subscription_id() format in crates/cauce-core/src/validation/mod.rs
- [X] T080 [P] [US6] Write tests for is_valid_session_id() format in crates/cauce-core/src/validation/mod.rs
- [X] T081 [P] [US6] Write tests for is_valid_message_id() format in crates/cauce-core/src/validation/mod.rs
- [X] T082 [P] [US6] Write tests for new ValidationError variants in crates/cauce-core/src/errors/mod.rs

### Implementation for User Story 6

- [X] T082b [US6] Implement validate_signal(value: &Value) -> Result<Signal, ValidationError> in crates/cauce-core/src/validation/schema.rs
- [X] T082c [US6] Implement validate_action(value: &Value) -> Result<Action, ValidationError> in crates/cauce-core/src/validation/schema.rs
- [X] T083 [US6] Add new ValidationError variants (InvalidSubscriptionId, InvalidSessionId, InvalidMessageId, InvalidTopicPattern, SchemaValidation, Deserialization) in crates/cauce-core/src/errors/mod.rs
- [X] T084 [US6] Add regex patterns for subscription_id, session_id, message_id to constants in crates/cauce-core/src/constants/mod.rs
- [X] T085 [US6] Implement validate_topic_pattern() supporting `*` and `**` wildcards in crates/cauce-core/src/validation/mod.rs
- [X] T086 [US6] Implement is_valid_subscription_id() with sub_<uuid> pattern in crates/cauce-core/src/validation/mod.rs
- [X] T087 [US6] Implement is_valid_session_id() with sess_<uuid> pattern in crates/cauce-core/src/validation/mod.rs
- [X] T088 [US6] Implement is_valid_message_id() with msg_<uuid> pattern in crates/cauce-core/src/validation/mod.rs
- [X] T089 [US6] Add re-exports for new validation functions to lib.rs in crates/cauce-core/src/lib.rs

**Checkpoint**: User Story 6 complete - all new validation functions work correctly

---

## Phase 9: User Story 7 - Complete Test Coverage (Priority: P3)

**Goal**: Achieve 95% code coverage with comprehensive tests

**Independent Test**: Run coverage tools and verify threshold is met

### Integration Tests (REQUIRED - TDD) ✅

- [ ] T090 [P] [US7] Write integration test for method types serialization roundtrip in crates/cauce-core/tests/serialization_roundtrip.rs
- [ ] T091 [P] [US7] Write integration test for all type roundtrips (HelloRequest, SubscribeRequest, PublishRequest, etc.) in crates/cauce-core/tests/serialization_roundtrip.rs
- [ ] T092 [P] [US7] Add property-based tests for topic matching (if proptest available) in crates/cauce-core/tests/topic_matching.rs

### Coverage Gap Analysis

- [ ] T093 [US7] Run coverage report with `cargo llvm-cov --workspace --html` and identify gaps
- [ ] T094 [US7] Add missing unit tests for any uncovered branches in method types
- [ ] T095 [US7] Add missing unit tests for any uncovered error code paths
- [ ] T096 [US7] Add missing edge case tests for validation functions
- [ ] T097 [US7] Verify all public API functions have at least one test

**Checkpoint**: User Story 7 complete - code coverage exceeds 95%

---

## Phase 10: Verification & Polish

**Purpose**: Coverage verification, schema validation, and cross-cutting improvements

### Coverage Verification (REQUIRED) ✅

- [ ] T098 Run `cargo llvm-cov --workspace --fail-under-lines 95` to verify coverage threshold
- [X] T099 Run `cargo test --workspace` to verify all tests pass
- [X] T100 Run `cargo clippy --workspace -- -D warnings` to verify no lints

### Polish & Cross-Cutting Concerns

- [X] T101 [P] Verify all doc comments are complete for public API in crates/cauce-core/src/lib.rs
- [X] T102 [P] Run `cargo fmt --all --check` to verify formatting
- [X] T103 Update lib.rs doc comments with new module documentation in crates/cauce-core/src/lib.rs
- [ ] T104 Verify quickstart.md examples compile correctly

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 - shared enums needed by US1
- **User Story 1 (Phase 3)**: Depends on Phase 2 - uses shared enums
- **User Story 2 (Phase 4)**: Depends on Phase 1 - no dependency on US1
- **User Story 3 (Phase 5)**: Depends on Phase 1 - no dependency on US1/US2
- **User Story 4 (Phase 6)**: Depends on Phase 1 - no dependency on US1-3
- **User Story 5 (Phase 7)**: Depends on Phase 1 - no dependency on US1-4
- **User Story 6 (Phase 8)**: Depends on Phase 1 - no dependency on US1-5
- **User Story 7 (Phase 9)**: Depends on US1-6 completion (tests all code)
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Requires foundational types (Phase 2)
- **User Story 2 (P1)**: Independent - only needs Phase 1
- **User Story 3 (P2)**: Independent - only needs Phase 1
- **User Story 4 (P2)**: Independent - only needs Phase 1
- **User Story 5 (P2)**: Independent - only needs Phase 1
- **User Story 6 (P3)**: Independent - only needs Phase 1
- **User Story 7 (P3)**: Depends on all other stories (coverage testing)

### Parallel Opportunities

Once Phase 2 is complete:
- US1 (method types) can proceed
- US2, US3, US4, US5, US6 can all proceed in parallel (different modules)

Within each user story:
- All [P] tests can run in parallel
- All [P] implementations can run in parallel

---

## Parallel Example: After Phase 2

```bash
# Launch all independent user stories in parallel:
# Developer A: User Story 1 - Method Parameter Types
# Developer B: User Story 2 - Error Codes
# Developer C: User Story 3 - Constants
# Developer D: User Story 4 - ID Generation
# Developer E: User Story 5 - Topic Matching
# Developer F: User Story 6 - Validation

# Each story is in a different module/file, no conflicts
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational types
3. Complete Phase 3: User Story 1 (method types)
4. Complete Phase 4: User Story 2 (error codes)
5. **STOP and VALIDATE**: Test serialization and error handling
6. This gives a functional protocol types library

### Incremental Delivery

1. Setup + Foundational → Module structure ready
2. Add User Story 1 → Method parameter types available
3. Add User Story 2 → Error handling available
4. Add User Story 3 → Constants available
5. Add User Story 4 → ID generation available
6. Add User Story 5 → Topic matching available
7. Add User Story 6 → Enhanced validation available
8. Add User Story 7 → Coverage verified
9. Each addition is independently usable

---

## Summary

| Phase | User Story | Priority | Task Count |
|-------|------------|----------|------------|
| 1 | Setup | - | 7 |
| 2 | Foundational | - | 9 |
| 3 | Method Parameter Types | P1 | 20 |
| 4 | Protocol Error Codes | P1 | 12 |
| 5 | Constants & Limits | P2 | 9 |
| 6 | ID Generation | P2 | 11 |
| 7 | Topic Matching | P2 | 11 |
| 8 | Enhanced Validation | P3 | 16 |
| 9 | Test Coverage | P3 | 8 |
| 10 | Verification & Polish | - | 7 |
| **Total** | | | **110** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD Red-Green-Refactor)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- 95% coverage is REQUIRED per Constitution Principle XI

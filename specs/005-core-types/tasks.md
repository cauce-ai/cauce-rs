# Tasks: Core Types Module

**Input**: Design documents from `/specs/005-core-types/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

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

- **Crate root**: `crates/cauce-core/`
- **Source**: `crates/cauce-core/src/`
- **Tests**: `crates/cauce-core/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Module structure and error types needed by all user stories

- [ ] T001 Expand types module structure with submodules in crates/cauce-core/src/types/mod.rs
- [ ] T002 [P] Implement ValidationError and BuilderError in crates/cauce-core/src/errors/mod.rs
- [ ] T003 [P] Add protocol constants (ID patterns, limits) in crates/cauce-core/src/constants/mod.rs
- [ ] T004 [P] Add ID validation utilities in crates/cauce-core/src/validation/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared types required by Signal and Action - MUST complete before user stories

**⚠️ CRITICAL**: US1 and US2 depend on these types

- [ ] T005 [P] Implement Priority enum in crates/cauce-core/src/types/enums.rs
- [ ] T006 [P] Implement ActionType enum in crates/cauce-core/src/types/enums.rs
- [ ] T007 [P] Implement Source struct with serde in crates/cauce-core/src/types/source.rs
- [ ] T008 [P] Implement Payload struct with serde in crates/cauce-core/src/types/payload.rs
- [ ] T009 [P] Implement Metadata struct with serde in crates/cauce-core/src/types/metadata.rs
- [ ] T010 [P] Implement ActionBody struct with serde in crates/cauce-core/src/types/action.rs
- [ ] T011 [P] Implement ActionContext struct with serde in crates/cauce-core/src/types/action.rs
- [ ] T012 Update types/mod.rs to re-export all foundational types in crates/cauce-core/src/types/mod.rs

**Checkpoint**: All supporting types ready - Signal/Action implementation can begin

---

## Phase 3: User Story 1 - Create and Serialize Signals (Priority: P1) 🎯 MVP

**Goal**: Developers can create Signal messages and serialize/deserialize them to JSON

**Independent Test**: Create Signal with all fields, serialize to JSON, deserialize back, verify equality

### Tests for User Story 1 (REQUIRED - TDD) ✅

> **TDD: Write these tests FIRST, ensure they FAIL before implementation (Red-Green-Refactor)**

- [ ] T013 [P] [US1] Unit tests for Signal struct creation in crates/cauce-core/tests/signal_test.rs
- [ ] T014 [P] [US1] Unit tests for Signal ID validation in crates/cauce-core/tests/signal_test.rs
- [ ] T015 [P] [US1] Serde round-trip tests for Signal in crates/cauce-core/tests/serde_test.rs
- [ ] T016 [P] [US1] Unit tests for Source, Payload, Metadata in crates/cauce-core/tests/types_test.rs

### Implementation for User Story 1

- [ ] T017 [US1] Implement Signal struct with all fields in crates/cauce-core/src/types/signal.rs
- [ ] T018 [US1] Add Signal ID validation (sig_<timestamp>_<random>) in crates/cauce-core/src/types/signal.rs
- [ ] T019 [US1] Add serde derives with skip_serializing_if for optional fields in crates/cauce-core/src/types/signal.rs
- [ ] T020 [US1] Re-export Signal from crate root in crates/cauce-core/src/lib.rs

**Checkpoint**: Signal struct fully functional with serialization - US1 complete

---

## Phase 4: User Story 2 - Create and Serialize Actions (Priority: P1)

**Goal**: Developers can create Action messages and serialize/deserialize them to JSON

**Independent Test**: Create Action with all fields, serialize to JSON, deserialize back, verify equality

### Tests for User Story 2 (REQUIRED - TDD) ✅

- [ ] T021 [P] [US2] Unit tests for Action struct creation in crates/cauce-core/tests/action_test.rs
- [ ] T022 [P] [US2] Unit tests for Action ID validation in crates/cauce-core/tests/action_test.rs
- [ ] T023 [P] [US2] Serde round-trip tests for Action in crates/cauce-core/tests/serde_test.rs
- [ ] T024 [P] [US2] Unit tests for ActionBody, ActionContext in crates/cauce-core/tests/types_test.rs

### Implementation for User Story 2

- [ ] T025 [US2] Implement Action struct with all fields in crates/cauce-core/src/types/action.rs
- [ ] T026 [US2] Add Action ID validation (act_<timestamp>_<random>) in crates/cauce-core/src/types/action.rs
- [ ] T027 [US2] Add serde derives with skip_serializing_if for optional fields in crates/cauce-core/src/types/action.rs
- [ ] T028 [US2] Re-export Action from crate root in crates/cauce-core/src/lib.rs

**Checkpoint**: Action struct fully functional with serialization - US2 complete

---

## Phase 5: User Story 3 - Validate Topics (Priority: P2)

**Goal**: Developers can create validated Topic identifiers with protocol-compliant patterns

**Independent Test**: Create valid/invalid topics, verify validation accepts/rejects correctly

### Tests for User Story 3 (REQUIRED - TDD) ✅

- [ ] T029 [P] [US3] Unit tests for valid topic patterns in crates/cauce-core/tests/topic_test.rs
- [ ] T030 [P] [US3] Unit tests for invalid topic patterns in crates/cauce-core/tests/topic_test.rs
- [ ] T031 [P] [US3] Unit tests for Topic length limits in crates/cauce-core/tests/topic_test.rs
- [ ] T032 [P] [US3] Serde round-trip tests for Topic in crates/cauce-core/tests/serde_test.rs

### Implementation for User Story 3

- [ ] T033 [US3] Implement Topic newtype wrapper in crates/cauce-core/src/types/topic.rs
- [ ] T034 [US3] Implement TryFrom<&str> with validation in crates/cauce-core/src/types/topic.rs
- [ ] T035 [US3] Implement FromStr for Topic in crates/cauce-core/src/types/topic.rs
- [ ] T036 [US3] Add serde Serialize/Deserialize for Topic in crates/cauce-core/src/types/topic.rs
- [ ] T037 [US3] Re-export Topic from crate root in crates/cauce-core/src/lib.rs

**Checkpoint**: Topic validation fully functional - US3 complete

---

## Phase 6: User Story 4 - Build Signals with Builder Pattern (Priority: P2)

**Goal**: Developers can construct Signals ergonomically with compile-time required field enforcement

**Independent Test**: Use builder to create Signal with various field combinations, verify behavior

### Tests for User Story 4 (REQUIRED - TDD) ✅

- [ ] T038 [P] [US4] Unit tests for SignalBuilder with all required fields in crates/cauce-core/tests/builder_test.rs
- [ ] T039 [P] [US4] Unit tests for SignalBuilder with optional fields in crates/cauce-core/tests/builder_test.rs
- [ ] T040 [P] [US4] Unit tests for ActionBuilder with all required fields in crates/cauce-core/tests/builder_test.rs
- [ ] T041 [P] [US4] Unit tests for ActionBuilder with optional fields in crates/cauce-core/tests/builder_test.rs

### Implementation for User Story 4

- [ ] T042 [US4] Implement SignalBuilder with typestate pattern in crates/cauce-core/src/types/signal.rs
- [ ] T043 [US4] Add builder method to Signal in crates/cauce-core/src/types/signal.rs
- [ ] T044 [US4] Implement ActionBuilder with typestate pattern in crates/cauce-core/src/types/action.rs
- [ ] T045 [US4] Add builder method to Action in crates/cauce-core/src/types/action.rs

**Checkpoint**: Builder patterns functional for Signal and Action - US4 complete

---

## Phase 7: User Story 5 - Handle Encrypted Message Envelopes (Priority: P3)

**Goal**: Developers can work with E2E encryption envelope types

**Independent Test**: Create Encrypted struct, serialize to JSON, verify all fields present

### Tests for User Story 5 (REQUIRED - TDD) ✅

- [ ] T046 [P] [US5] Unit tests for Encrypted struct in crates/cauce-core/tests/types_test.rs
- [ ] T047 [P] [US5] Unit tests for EncryptionAlgorithm enum in crates/cauce-core/tests/types_test.rs
- [ ] T048 [P] [US5] Serde round-trip tests for Encrypted in crates/cauce-core/tests/serde_test.rs

### Implementation for User Story 5

- [ ] T049 [US5] Implement EncryptionAlgorithm enum in crates/cauce-core/src/types/encrypted.rs
- [ ] T050 [US5] Implement Encrypted struct with serde in crates/cauce-core/src/types/encrypted.rs
- [ ] T051 [US5] Re-export Encrypted and EncryptionAlgorithm from crate root in crates/cauce-core/src/lib.rs

**Checkpoint**: Encryption types functional - US5 complete

---

## Phase 8: Verification & Polish

**Purpose**: Coverage verification, integration, and cross-cutting improvements

### Coverage Verification (REQUIRED) ✅

- [ ] T052 Run coverage report with cargo llvm-cov and verify ≥95% threshold
- [ ] T053 Add missing unit tests to reach coverage target
- [ ] T054 Verify all tests pass with cargo test -p cauce-core

### Polish & Cross-Cutting Concerns

- [ ] T055 [P] Remove placeholder types (Signal, Action, Topic) from crates/cauce-core/src/types/mod.rs
- [ ] T056 [P] Update module documentation in crates/cauce-core/src/lib.rs
- [ ] T057 Run cargo clippy -p cauce-core and fix any warnings
- [ ] T058 Run cargo fmt -p cauce-core to ensure formatting
- [ ] T059 Validate quickstart.md examples compile and work

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories (provides shared types)
- **User Stories (Phase 3-7)**: All depend on Foundational completion
  - US1 and US2 can proceed in parallel (both P1 priority)
  - US3 can proceed after Foundational (no dependency on US1/US2)
  - US4 depends on US1 and US2 (adds builders to Signal/Action)
  - US5 can proceed after Foundational (no dependency on other stories)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational - provides Signal type
- **User Story 2 (P1)**: Depends on Foundational - provides Action type
- **User Story 3 (P2)**: Depends on Foundational - provides Topic type (used by US1/US2 but can be developed in parallel)
- **User Story 4 (P2)**: Depends on US1 and US2 - adds builders to existing types
- **User Story 5 (P3)**: Depends on Foundational - provides Encrypted type (optional field in Signal/Action)

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Tests can run in parallel within a story
- Implementation tasks follow test tasks
- Re-export tasks depend on implementation

### Parallel Opportunities

**Setup phase:**
- T002, T003, T004 can run in parallel

**Foundational phase:**
- T005, T006, T007, T008, T009, T010, T011 can ALL run in parallel (different files)

**After Foundational:**
- US1, US2, US3, US5 can all start in parallel
- US4 must wait for US1 and US2

**Within each story:**
- All test tasks marked [P] can run in parallel
- Implementation follows tests

---

## Parallel Example: Foundational Phase

```bash
# Launch all foundational types together (different files):
Task: "Implement Priority enum in crates/cauce-core/src/types/enums.rs"
Task: "Implement ActionType enum in crates/cauce-core/src/types/enums.rs"
Task: "Implement Source struct with serde in crates/cauce-core/src/types/source.rs"
Task: "Implement Payload struct with serde in crates/cauce-core/src/types/payload.rs"
Task: "Implement Metadata struct with serde in crates/cauce-core/src/types/metadata.rs"
Task: "Implement ActionBody struct with serde in crates/cauce-core/src/types/action.rs"
Task: "Implement ActionContext struct with serde in crates/cauce-core/src/types/action.rs"
```

## Parallel Example: User Story 1

```bash
# Launch all US1 tests together:
Task: "Unit tests for Signal struct creation in crates/cauce-core/tests/signal_test.rs"
Task: "Unit tests for Signal ID validation in crates/cauce-core/tests/signal_test.rs"
Task: "Serde round-trip tests for Signal in crates/cauce-core/tests/serde_test.rs"
Task: "Unit tests for Source, Payload, Metadata in crates/cauce-core/tests/types_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - provides shared types)
3. Complete Phase 3: User Story 1 (Signal)
4. Complete Phase 4: User Story 2 (Action)
5. **STOP and VALIDATE**: Test Signal + Action independently
6. This provides functional core types for protocol communication

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 (Signal) + US2 (Action) → Test independently → **MVP ready**
3. Add US3 (Topic validation) → Improved validation
4. Add US4 (Builders) → Improved DX
5. Add US5 (Encrypted) → E2E encryption support
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Signal)
   - Developer B: User Story 2 (Action)
   - Developer C: User Story 3 (Topic) + User Story 5 (Encrypted)
3. After US1+US2: Developer A or B takes User Story 4 (Builders)
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests FAIL before implementing (TDD Red-Green-Refactor)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Total tasks: 59
- Coverage target: 95%

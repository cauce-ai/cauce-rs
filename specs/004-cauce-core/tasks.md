# Tasks: cauce-core Project Setup

**Input**: Design documents from `/specs/004-cauce-core/`
**Prerequisites**: plan.md, spec.md, research.md

**Tests**: Per Constitution Principle XI, TDD is required. Tests written BEFORE implementation, 95% coverage target.

**Organization**: Tasks grouped by user story for independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Rust workspace**: `crates/cauce-core/` for this crate
- **Source**: `crates/cauce-core/src/`
- **Tests**: `crates/cauce-core/tests/` (integration tests)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace configuration and crate initialization

- [x] T001 Add workspace dependencies to root Cargo.toml (serde, serde_json, thiserror, chrono, uuid, jsonschema)
- [x] T002 Create crates/cauce-core/ directory structure
- [x] T003 Create crates/cauce-core/Cargo.toml with crate metadata and workspace dependency references

**Checkpoint**: Crate structure exists, workspace configured

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None required - this is a minimal setup feature with no blocking prerequisites beyond Phase 1

**Checkpoint**: Ready for user story implementation

---

## Phase 3: User Story 1 - Library Foundation (Priority: P1) MVP

**Goal**: cauce-core compiles as a workspace member and can be used as a dependency

**Independent Test**: `cargo build -p cauce-core` and `cargo test -p cauce-core` succeed

### Tests for User Story 1 (TDD)

- [x] T004 [US1] Write integration test verifying crate compiles in crates/cauce-core/tests/crate_test.rs

### Implementation for User Story 1

- [x] T005 [US1] Create crates/cauce-core/src/lib.rs with module declarations
- [x] T006 [US1] Verify workspace build with `cargo build --workspace`
- [x] T007 [US1] Verify crate-specific build with `cargo build -p cauce-core`

**Checkpoint**: cauce-core builds successfully as workspace member

---

## Phase 4: User Story 2 - Module Discovery (Priority: P1)

**Goal**: Clear module structure with 5 documented modules

**Independent Test**: lib.rs shows all modules, each mod.rs has documentation

### Tests for User Story 2 (TDD)

- [x] T008 [US2] Write test verifying all 5 modules are accessible in crates/cauce-core/tests/module_test.rs

### Implementation for User Story 2

- [x] T009 [P] [US2] Create crates/cauce-core/src/types/mod.rs with module documentation
- [x] T010 [P] [US2] Create crates/cauce-core/src/jsonrpc/mod.rs with module documentation
- [x] T011 [P] [US2] Create crates/cauce-core/src/validation/mod.rs with module documentation
- [x] T012 [P] [US2] Create crates/cauce-core/src/errors/mod.rs with module documentation
- [x] T013 [P] [US2] Create crates/cauce-core/src/constants/mod.rs with module documentation
- [x] T014 [US2] Update crates/cauce-core/src/lib.rs to declare all 5 modules

**Checkpoint**: All 5 modules exist with documentation, accessible via lib.rs

---

## Phase 5: User Story 3 - Re-exports for Ergonomics (Priority: P2)

**Goal**: Key types re-exported at crate root for ergonomic imports

**Independent Test**: `use cauce_core::Signal;` works (once Signal is defined in future features)

### Tests for User Story 3 (TDD)

- [x] T015 [US3] Write test verifying placeholder types are re-exported at crate root in crates/cauce-core/tests/reexport_test.rs

### Implementation for User Story 3

- [x] T016 [US3] Add placeholder types (Signal, Action, Topic) to crates/cauce-core/src/types/mod.rs
- [x] T017 [US3] Add re-exports to crates/cauce-core/src/lib.rs for placeholder types

**Checkpoint**: Placeholder types accessible via `use cauce_core::{Signal, Action, Topic};`

---

## Phase 6: Verification & Polish

**Purpose**: Linting, formatting, and final validation

### Coverage Verification

- [x] T018 Run `cargo test -p cauce-core` and verify all tests pass
- [x] T019 Run `cargo llvm-cov --package cauce-core` and verify coverage meets threshold

### Polish & Cross-Cutting Concerns

- [x] T020 Run `cargo clippy -p cauce-core` and fix any warnings
- [x] T021 Run `cargo fmt --check -p cauce-core` and fix formatting issues
- [x] T022 Run `cargo deny check` and verify license compliance
- [x] T023 Verify quickstart.md validation steps pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: N/A for this feature
- **User Story 1 (Phase 3)**: Depends on Phase 1
- **User Story 2 (Phase 4)**: Depends on Phase 3 (needs lib.rs to exist)
- **User Story 3 (Phase 5)**: Depends on Phase 4 (needs modules to exist)
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Setup - No dependencies on other stories
- **User Story 2 (P1)**: Requires US1 complete (needs lib.rs structure)
- **User Story 3 (P2)**: Requires US2 complete (needs modules to add placeholders)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Module files before lib.rs updates
- Core implementation before re-exports

### Parallel Opportunities

- T009, T010, T011, T012, T013 can all run in parallel (different module files)
- T020, T021, T022 can run in parallel (different tools)

---

## Parallel Example: User Story 2 Modules

```bash
# Launch all module creations together:
Task: "Create crates/cauce-core/src/types/mod.rs"
Task: "Create crates/cauce-core/src/jsonrpc/mod.rs"
Task: "Create crates/cauce-core/src/validation/mod.rs"
Task: "Create crates/cauce-core/src/errors/mod.rs"
Task: "Create crates/cauce-core/src/constants/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (workspace deps, crate structure)
2. Complete Phase 3: User Story 1 (lib.rs, build verification)
3. **STOP and VALIDATE**: `cargo build -p cauce-core` succeeds
4. Minimal viable crate ready

### Incremental Delivery

1. Setup + US1 → Crate builds (MVP!)
2. Add US2 → Modules documented
3. Add US3 → Ergonomic re-exports
4. Polish → CI-ready

---

## Notes

- This is a setup feature - modules will be populated by features 2.2-2.9
- Placeholder types (Signal, Action, Topic) are empty structs for now
- Empty modules with doc comments don't require test coverage
- 95% coverage applies to actual code (placeholder structs)

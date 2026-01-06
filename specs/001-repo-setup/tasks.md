# Tasks: Repository Setup

**Input**: Design documents from `/specs/001-repo-setup/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: This feature is configuration-only (no executable code). Validation is performed by running tools against the configuration files and sample code.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Cargo workspace at repository root
- Configuration files at repository root
- Hooks in `.githooks/` directory
- All crates in `crates/` directory

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the base directory structure and core project files

- [ ] T001 Create `crates/` directory for workspace members
- [ ] T002 Create `.githooks/` directory for git hooks

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None for this feature - all user stories are independent configuration tasks

**⚠️ NOTE**: This feature has no blocking prerequisites. Each user story creates independent configuration files.

---

## Phase 3: User Story 1 - Initialize Workspace Structure (Priority: P1) 🎯 MVP

**Goal**: Developers can clone the repository and immediately run build commands with a properly configured workspace

**Independent Test**: Run `cargo build --workspace` - should succeed without errors (even with no code yet)

### Implementation for User Story 1

- [ ] T003 [US1] Create workspace `Cargo.toml` at repository root with:
  - `[workspace]` section with `resolver = "2"`
  - `members = ["crates/*"]` glob pattern
  - `[workspace.package]` with shared metadata (edition, license, repository)
  - `[workspace.dependencies]` with shared dependencies (tokio, serde, thiserror, chrono, uuid, serde_json)
  - Comments explaining each section

- [ ] T004 [US1] Create `.gitignore` at repository root with:
  - Rust build artifacts (`/target/`, `Cargo.lock` for libraries)
  - IDE files (`.idea/`, `.vscode/`, `*.swp`)
  - OS files (`.DS_Store`, `Thumbs.db`)
  - Environment files (`.env`)
  - Comments grouping each category

**Checkpoint**: Workspace initialized - `cargo build --workspace` should run without errors

---

## Phase 4: User Story 2 - Consistent Code Formatting (Priority: P1)

**Goal**: All code is automatically formatted to project standards

**Independent Test**: Create a `.rs` file with bad formatting, run `cargo fmt`, verify it's reformatted correctly

### Implementation for User Story 2

- [ ] T005 [US2] Create `rustfmt.toml` at repository root with:
  - `edition = "2021"`
  - `imports_granularity = "Module"`
  - `group_imports = "StdExternalCrate"`
  - Comments explaining each setting (per research.md Decision 9)

**Checkpoint**: Format configuration complete - `cargo fmt --check` should pass on formatted code

---

## Phase 5: User Story 3 - Automated Code Quality Checks (Priority: P1)

**Goal**: Developers receive feedback on code quality issues via linting

**Independent Test**: Create code with a known clippy warning, run `cargo clippy`, verify warning is reported

### Implementation for User Story 3

- [ ] T006 [US3] Verify clippy works with default configuration (no `clippy.toml` needed initially)
- [ ] T007 [US3] Document clippy usage in quickstart.md (already done, verify accuracy)

**Checkpoint**: Lint configuration complete - `cargo clippy` should run and report issues

---

## Phase 6: User Story 4 - Dependency Security Verification (Priority: P2)

**Goal**: Dependencies are audited for security vulnerabilities and license compliance

**Independent Test**: Run `cargo deny check` - should pass with no violations on initial empty workspace

### Implementation for User Story 4

- [ ] T008 [US4] Create `deny.toml` at repository root with:
  - `[advisories]` section with vulnerability database settings
  - `[licenses]` section allowing: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0, Unlicense
  - `[bans]` section with basic duplicate detection
  - `[sources]` section allowing crates.io
  - Comments explaining each policy (per research.md Decision 10)

**Checkpoint**: Audit configuration complete - `cargo deny check` should pass

---

## Phase 7: User Story 5 - Pre-Commit Quality Gate (Priority: P2)

**Goal**: Commits are automatically validated against quality standards

**Independent Test**:
1. Enable hooks with `git config core.hooksPath .githooks`
2. Create a file with bad formatting
3. Attempt to commit - should be rejected
4. Fix formatting and commit - should succeed

### Implementation for User Story 5

- [ ] T009 [US5] Create `.githooks/pre-commit` script with:
  - Shebang for bash/sh portability (`#!/usr/bin/env bash`)
  - Check for required tools (rustfmt, clippy) with helpful error messages
  - Run `cargo fmt --check` (reject if fails)
  - Run `cargo clippy -- -D warnings` (reject if fails)
  - Run `cargo test --workspace` (reject if fails)
  - Clear success/failure messages
  - Make script executable (`chmod +x`)

- [ ] T010 [US5] Create `.pre-commit-config.yaml` at repository root documenting:
  - What checks run on pre-commit
  - How to enable hooks (`git config core.hooksPath .githooks`)
  - How to bypass temporarily (`git commit --no-verify`)

**Checkpoint**: Pre-commit hooks complete - commits should be validated automatically

---

## Phase 8: Verification & Polish

**Purpose**: Validate all configurations work together and documentation is complete

### Validation Tasks

- [ ] T011 [P] Validate workspace: Run `cargo build --workspace` (should succeed)
- [ ] T012 [P] Validate format: Create test file, run `cargo fmt`, verify reformatting
- [ ] T013 [P] Validate lint: Run `cargo clippy` (should run without errors)
- [ ] T014 [P] Validate audit: Run `cargo deny check` (should pass)
- [ ] T015 Validate hooks: Enable hooks, attempt commit with bad code, verify rejection

### Documentation Tasks

- [ ] T016 [P] Verify `specs/001-repo-setup/quickstart.md` is accurate and complete
- [ ] T017 [P] Add configuration file comments per FR-009 (review all created files)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: N/A - no blocking prerequisites for this feature
- **User Stories (Phases 3-7)**: US1 should be first (workspace is foundation)
  - US2, US3, US4 can proceed in parallel after US1
  - US5 depends on US2 (format) and US3 (lint) being complete
- **Phase 8 (Verification)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (Workspace)**: No dependencies - START HERE
- **User Story 2 (Formatting)**: Requires US1 (workspace must exist to format code)
- **User Story 3 (Linting)**: Requires US1 (workspace must exist to lint code)
- **User Story 4 (Audit)**: Requires US1 (workspace must exist to audit dependencies)
- **User Story 5 (Hooks)**: Requires US2 + US3 (hooks call format and lint)

### Parallel Opportunities

- T011, T012, T013, T014 can all run in parallel (different validation targets)
- T016, T017 can run in parallel (different documentation files)
- US2, US3, US4 can all run in parallel after US1 completes

---

## Parallel Example: Validation Phase

```bash
# Launch all validation tasks together:
Task: "Validate workspace: Run cargo build --workspace"
Task: "Validate format: Create test file, run cargo fmt"
Task: "Validate lint: Run cargo clippy"
Task: "Validate audit: Run cargo deny check"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 3: User Story 1 (T003-T004)
3. **STOP and VALIDATE**: `cargo build --workspace` should succeed
4. Workspace is usable - other stories add developer experience improvements

### Incremental Delivery

1. US1: Workspace initialized → Can start coding
2. US2 + US3: Format + Lint → Consistent code quality
3. US4: Audit → Security compliance
4. US5: Hooks → Automated enforcement
5. Each story adds value without breaking previous stories

### Recommended Execution Order

```
T001 → T002 → T003 → T004 (US1 complete - MVP)
           ↓
    ┌──────┼──────┐
    ↓      ↓      ↓
   T005   T006   T008  (US2, US3, US4 in parallel)
   T007
    └──────┼──────┘
           ↓
    T009 → T010 (US5 - needs US2+US3)
           ↓
    T011-T017 (Verification)
```

---

## Notes

- This feature creates configuration files only - no executable code
- All configuration files should include explanatory comments (FR-009)
- Pre-commit hooks must be enabled manually via `git config core.hooksPath .githooks`
- Validation tests use actual tool execution rather than unit tests
- Total tasks: 17
- Parallelizable tasks: 8 (marked [P])

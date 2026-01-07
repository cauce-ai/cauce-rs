# Tasks: Documentation Structure

**Input**: Design documents from `/specs/003-docs-setup/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: This feature is documentation-only (Markdown files). Validation is performed by visual inspection and markdown linting, not executable tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Documentation at repository root and in `docs/` directory
- CONTRIBUTING.md at repository root (GitHub convention)
- ARCHITECTURE.md in docs/ (technical documentation)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the documentation directory structure

- [x] T001 Create `docs/` directory at repository root
- [x] T002 [P] Create `docs/architecture/` subdirectory for future detailed architecture docs
- [x] T003 [P] Create `docs/guides/` subdirectory for future how-to guides
- [x] T004 [P] Create `docs/reference/` subdirectory for future API reference

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None for this feature - each user story creates independent documentation files

**⚠️ NOTE**: All documentation files can be created independently once the directory structure exists.

---

## Phase 3: User Story 1 - Development Guidelines Access (Priority: P1) 🎯 MVP

**Goal**: Contributors can find and follow development guidelines in CONTRIBUTING.md

**Independent Test**: A new contributor reads CONTRIBUTING.md and understands how to submit a PR

### Implementation for User Story 1

- [x] T005 [US1] Create `CONTRIBUTING.md` at repository root with header and overview
- [x] T006 [US1] Add Prerequisites section to `CONTRIBUTING.md` (Rust toolchain, cargo-deny, cargo-llvm-cov)
- [x] T007 [US1] Add Getting Started section to `CONTRIBUTING.md` (clone, build, test commands)
- [x] T008 [US1] Add Code Standards section to `CONTRIBUTING.md` (rustfmt, clippy, 95% coverage per Constitution XI)
- [x] T009 [US1] Add Commit Conventions section to `CONTRIBUTING.md` (Conventional Commits format)
- [x] T010 [US1] Add Branch Naming section to `CONTRIBUTING.md` (SpecKit pattern: ###-feature-name)
- [x] T011 [US1] Add Pull Request Process section to `CONTRIBUTING.md` (CI requirements, squash merge)

**Checkpoint**: CONTRIBUTING.md complete - contributors have all guidelines needed

---

## Phase 4: User Story 2 - Architecture Understanding (Priority: P1)

**Goal**: Developers understand crate structure and responsibilities from ARCHITECTURE.md

**Independent Test**: A developer reads ARCHITECTURE.md and can identify which crate handles a specific responsibility

### Implementation for User Story 2

- [x] T012 [US2] Create `docs/ARCHITECTURE.md` with header and overview
- [x] T013 [US2] Add Crate Overview table to `docs/ARCHITECTURE.md` (all 8 planned crates with status)
- [x] T014 [US2] Add Mermaid dependency diagram to `docs/ARCHITECTURE.md` showing crate relationships
- [x] T015 [US2] Add Crate Responsibilities section to `docs/ARCHITECTURE.md` (what each crate does)
- [x] T016 [US2] Add Crate Boundaries section to `docs/ARCHITECTURE.md` (what each crate does NOT do per Constitution VI)
- [x] T017 [US2] Add Layer diagram to `docs/ARCHITECTURE.md` showing Adapter/Hub/Agent separation

**Checkpoint**: ARCHITECTURE.md complete - developers understand crate structure

---

## Phase 5: User Story 3 - Documentation Discoverability (Priority: P2)

**Goal**: Users can navigate the docs/ directory and find documentation organized by category

**Independent Test**: A user browses docs/ and finds documentation index with navigation

### Implementation for User Story 3

- [x] T018 [US3] Create `docs/README.md` with documentation index and navigation
- [x] T019 [US3] Add links to ARCHITECTURE.md from `docs/README.md`
- [x] T020 [US3] Add placeholder descriptions for future documentation categories in `docs/README.md`

**Checkpoint**: docs/README.md complete - documentation is discoverable and navigable

---

## Phase 6: Verification & Polish

**Purpose**: Validate documentation renders correctly and links work

### Validation Tasks

- [x] T021 [P] Verify CONTRIBUTING.md renders correctly on GitHub
- [x] T022 [P] Verify Mermaid diagrams in ARCHITECTURE.md render on GitHub
- [x] T023 [P] Verify all internal links in documentation work
- [x] T024 Add documentation file comments/metadata where appropriate

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: N/A - no blocking prerequisites for this feature
- **User Stories (Phases 3-5)**: Can all proceed after Phase 1 is complete
  - User Story 1 (CONTRIBUTING.md) is independent
  - User Story 2 (ARCHITECTURE.md) is independent
  - User Story 3 (docs/README.md) can link to ARCHITECTURE.md once it exists
- **Phase 6 (Verification)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: No dependencies - can start after Phase 1
- **User Story 2 (P1)**: No dependencies - can start after Phase 1
- **User Story 3 (P2)**: Soft dependency on US2 for linking to ARCHITECTURE.md

### Parallel Opportunities

- T002, T003, T004 can run in parallel (different directories)
- T021, T022, T023 can run in parallel (different validation targets)
- US1 and US2 can run in parallel (different files)

---

## Parallel Example: After Phase 1 Complete

```bash
# These can all run in parallel after directory structure exists:
Task: "Create CONTRIBUTING.md at repository root" (US1)
Task: "Create docs/ARCHITECTURE.md with header" (US2)
Task: "Create docs/README.md with navigation" (US3)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 3: User Story 1 (T005-T011)
3. **STOP and VALIDATE**: CONTRIBUTING.md exists and is complete
4. Contributors can now follow guidelines

### Incremental Delivery

1. US1: CONTRIBUTING.md → Contributors have guidelines
2. US2: ARCHITECTURE.md → Developers understand crate structure
3. US3: docs/README.md → Documentation is discoverable

### Recommended Execution Order

```
T001 (directory)
   ↓
T002, T003, T004 (subdirectories - parallel)
   ↓
┌──────────────┐
↓              ↓
T005-T011    T012-T017  (US1 and US2 in parallel)
└──────┬───────┘
       ↓
   T018-T020 (US3)
       ↓
T021-T024 (Verification - parallel)
```

---

## Notes

- This feature creates documentation files only - no executable code
- All Markdown files should use GitHub-flavored Markdown
- Mermaid diagrams must be validated for correct rendering on GitHub
- Total tasks: 24
- Parallelizable tasks: 10 (marked [P])

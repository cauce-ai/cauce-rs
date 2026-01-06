# Tasks: CI/CD Pipeline

**Input**: Design documents from `/specs/002-ci-cd-pipeline/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: This feature is configuration-only (YAML workflow files). Validation is performed by GitHub Actions executing the workflows on actual pushes.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Workflows in `.github/workflows/` directory
- All crates in `crates/` directory
- Configuration files at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the workflow directory structure

- [x] T001 Create `.github/workflows/` directory for GitHub Actions workflows

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None for this feature - each user story creates independent workflow files or sections

**⚠️ NOTE**: US1-US4 all contribute to `ci.yml`. They are implemented sequentially but each adds independent functionality.

---

## Phase 3: User Story 1 - Automated Quality Verification (Priority: P1) 🎯 MVP

**Goal**: Developers receive automatic feedback on code quality when pushing code

**Independent Test**: Push a commit and verify CI runs format check, linting, and basic build

### Implementation for User Story 1

- [x] T002 [US1] Create `.github/workflows/ci.yml` with:
  - Workflow name and trigger configuration (push, pull_request)
  - Rust toolchain setup using `dtolnay/rust-toolchain@stable`
  - Cargo caching using `Swatinem/rust-cache@v2`

- [x] T003 [US1] Add format check job to `.github/workflows/ci.yml`:
  - Job: `format`
  - Run `cargo fmt --check`
  - Fail pipeline if formatting differs

- [x] T004 [US1] Add lint check job to `.github/workflows/ci.yml`:
  - Job: `lint`
  - Run `cargo clippy --workspace --all-targets -- -D warnings`
  - Fail pipeline on any warnings

- [x] T005 [US1] Add build job to `.github/workflows/ci.yml`:
  - Job: `build`
  - Run `cargo build --workspace`
  - Depends on format and lint passing

- [x] T006 [US1] Add test job to `.github/workflows/ci.yml`:
  - Job: `test`
  - Run `cargo test --workspace`
  - Depends on build passing

**Checkpoint**: Basic CI working - push triggers format, lint, build, test

---

## Phase 4: User Story 2 - Coverage Enforcement (Priority: P1)

**Goal**: CI enforces 95% code coverage requirement automatically

**Independent Test**: Push code with <95% coverage and verify CI fails; push code with ≥95% and verify CI passes

### Implementation for User Story 2

- [x] T007 [US2] Add coverage job to `.github/workflows/ci.yml`:
  - Job: `coverage`
  - Install `cargo-llvm-cov` via `taiki-e/install-action@cargo-llvm-cov`
  - Run `cargo llvm-cov --workspace --fail-under 95`
  - Upload coverage report as artifact
  - Depends on test passing

**Checkpoint**: Coverage enforcement active - 95% threshold enforced

---

## Phase 5: User Story 3 - Cross-Platform Build Verification (Priority: P1)

**Goal**: CI verifies code builds and tests pass on Linux, macOS, and Windows

**Independent Test**: Push code and verify builds execute on all three platforms

### Implementation for User Story 3

- [x] T008 [US3] Convert build job to matrix in `.github/workflows/ci.yml`:
  - Matrix: `os: [ubuntu-latest, macos-latest, windows-latest]`
  - Build on all platforms in parallel
  - Update job name to include platform

- [x] T009 [US3] Convert test job to matrix in `.github/workflows/ci.yml`:
  - Matrix: `os: [ubuntu-latest, macos-latest, windows-latest]`
  - Test on all platforms in parallel
  - Coverage remains Linux-only

**Checkpoint**: Cross-platform CI active - all platforms tested before merge

---

## Phase 6: User Story 4 - Dependency Security Audit (Priority: P2)

**Goal**: CI audits dependencies for security vulnerabilities and license compliance

**Independent Test**: Add a dependency with known vulnerability and verify CI fails

### Implementation for User Story 4

- [x] T010 [US4] Add audit job to `.github/workflows/ci.yml`:
  - Job: `audit`
  - Install `cargo-deny` via `taiki-e/install-action@cargo-deny`
  - Run `cargo deny check`
  - Runs in parallel with format/lint

**Checkpoint**: Dependency audit active - vulnerabilities and license violations caught

---

## Phase 7: User Story 5 - Automated Release Builds (Priority: P2)

**Goal**: Version tags trigger automated release binary builds for all platforms

**Independent Test**: Create a version tag (v0.0.1-test) and verify release workflow triggers

### Implementation for User Story 5

- [x] T011 [P] [US5] Create `.github/workflows/release.yml` with:
  - Trigger: `push` tags matching `v*.*.*`
  - Workflow name: `Release`

- [x] T012 [US5] Add release build matrix to `.github/workflows/release.yml`:
  - Matrix targets per research.md Decision 5:
    - x86_64-unknown-linux-gnu (ubuntu-latest)
    - x86_64-unknown-linux-musl (ubuntu-latest)
    - aarch64-unknown-linux-gnu (ubuntu-latest, cross-compile)
    - x86_64-apple-darwin (macos-latest)
    - aarch64-apple-darwin (macos-latest)
    - x86_64-pc-windows-msvc (windows-latest)
  - Build with `--release` flag

- [x] T013 [US5] Add artifact upload to `.github/workflows/release.yml`:
  - Package binaries with appropriate names
  - Upload as release assets using `softprops/action-gh-release@v1`

**Checkpoint**: Release automation active - tags produce downloadable binaries

---

## Phase 8: User Story 6 - Docker Image Builds (Priority: P3)

**Goal**: CI builds and publishes Docker images on push to main and version tags

**Independent Test**: Push to main and verify Docker image is built and tagged

### Implementation for User Story 6

- [x] T014 [P] [US6] Create `Dockerfile` at repository root:
  - Multi-stage build per research.md Decision 6
  - Builder stage: `rust:1.75-slim`
  - Runtime stage: `debian:bookworm-slim` or distroless
  - Copy binary from builder to runtime

- [x] T015 [P] [US6] Create `.github/workflows/docker.yml` with:
  - Triggers: push to main, push tags `v*.*.*`
  - Workflow name: `Docker`

- [x] T016 [US6] Add Docker build job to `.github/workflows/docker.yml`:
  - Login to GitHub Container Registry (ghcr.io)
  - Build image using `docker/build-push-action@v5`
  - Tag with branch name or version tag
  - Push to `ghcr.io/cauce-ai/cauce-rs`

**Checkpoint**: Docker automation active - images available on ghcr.io

---

## Phase 9: Verification & Polish

**Purpose**: Validate all workflows work together and documentation is complete

### Validation Tasks

- [x] T017 [P] Verify ci.yml syntax by pushing test commit
- [x] T018 [P] Verify release.yml syntax by creating test tag
- [x] T019 [P] Verify docker.yml syntax by pushing to main
- [x] T020 Verify all jobs report status to PR checks

### Documentation Tasks

- [x] T021 [P] Update `specs/002-ci-cd-pipeline/quickstart.md` with actual workflow details
- [x] T022 [P] Add workflow file comments explaining each section

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: N/A - no blocking prerequisites for this feature
- **User Stories (Phases 3-8)**:
  - US1-US4 are sequential (all modify ci.yml)
  - US5 (release.yml) can start after US1 setup
  - US6 (docker.yml) can start after US1 setup
- **Phase 9 (Verification)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (Quality)**: No dependencies - START HERE (creates ci.yml)
- **User Story 2 (Coverage)**: Requires US1 (adds to ci.yml)
- **User Story 3 (Cross-Platform)**: Requires US2 (modifies ci.yml)
- **User Story 4 (Audit)**: Requires US1 (adds to ci.yml), can parallel US2/US3
- **User Story 5 (Release)**: Can start after US1 (separate file)
- **User Story 6 (Docker)**: Can start after US1 (separate file)

### Parallel Opportunities

- T014, T015 can run in parallel (different files)
- T017, T018, T019 can run in parallel (different validation targets)
- T021, T022 can run in parallel (different documentation files)
- US5 and US6 can run in parallel (different workflow files)

---

## Parallel Example: After US1 Complete

```bash
# These can all run in parallel after ci.yml basics are in place:
Task: "Create release.yml" (US5)
Task: "Create docker.yml" (US6)
Task: "Add audit job to ci.yml" (US4 - if US2/US3 not blocking)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 3: User Story 1 (T002-T006)
3. **STOP and VALIDATE**: Push code, verify CI runs
4. Basic CI is functional - quality feedback on every push

### Incremental Delivery

1. US1: Basic CI → Immediate feedback on pushes
2. US2: Coverage → 95% enforcement active
3. US3: Cross-platform → All platforms verified
4. US4: Audit → Security/license compliance
5. US5: Release → Automated binary releases
6. US6: Docker → Container deployment ready

### Recommended Execution Order

```
T001 (Setup)
   ↓
T002 → T003 → T004 → T005 → T006 (US1 complete - MVP)
   ↓
T007 (US2 - Coverage)
   ↓
T008 → T009 (US3 - Cross-Platform)
   ↓
T010 (US4 - Audit)
   ↓
┌──────────────┐
↓              ↓
T011-T013    T014-T016  (US5 and US6 in parallel)
└──────┬───────┘
       ↓
T017-T022 (Verification)
```

---

## Notes

- This feature creates configuration files only - no executable code to test
- All workflow YAML files should include explanatory comments
- Validation is performed by actual CI runs on pushes/tags
- Total tasks: 22
- Parallelizable tasks: 8 (marked [P])

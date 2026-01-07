# Feature Specification: cauce-core Project Setup

**Feature Branch**: `004-cauce-core`
**Created**: 2026-01-07
**Status**: Draft
**Input**: User description: "Section 2.1 from TODO.md - Create crates/cauce-core with Cargo.toml and module structure"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Library Foundation (Priority: P1)

A developer working on other cauce-rs crates (client-sdk, server-sdk, hub) needs to import core protocol types and utilities from a shared library. They need the cauce-core crate to exist as a workspace member with proper dependencies so they can start building on top of it.

**Why this priority**: This is the foundational crate that all other crates depend on. Without it, no other development can proceed.

**Independent Test**: Run `cargo build -p cauce-core` and `cargo test -p cauce-core` successfully.

**Acceptance Scenarios**:

1. **Given** a developer checks out the repository, **When** they run `cargo build --workspace`, **Then** the cauce-core crate compiles without errors.
2. **Given** the cauce-core crate exists, **When** another crate adds `cauce-core` as a dependency, **Then** it can import types from cauce-core.
3. **Given** a developer wants to run tests, **When** they run `cargo test -p cauce-core`, **Then** the test harness runs (even if no tests exist yet).

---

### User Story 2 - Module Discovery (Priority: P1)

A developer exploring the cauce-core crate needs to understand its structure and find where to implement specific functionality. The module structure should clearly separate concerns and provide a logical home for each type of functionality.

**Why this priority**: A clear module structure is essential for maintainability and for developers to navigate the codebase.

**Independent Test**: A developer can look at lib.rs and immediately understand what modules exist and their purposes.

**Acceptance Scenarios**:

1. **Given** a developer opens `lib.rs`, **When** they review the module declarations, **Then** they see modules for types, jsonrpc, validation, errors, and constants.
2. **Given** a developer needs to add a new type, **When** they consult the module structure, **Then** they know to add it in the `types` module.
3. **Given** a developer needs to add JSON-RPC request/response types, **When** they look at the module structure, **Then** they find the `jsonrpc` module.

---

### User Story 3 - Re-exports for Ergonomics (Priority: P2)

A developer using cauce-core from another crate wants convenient access to commonly used types without deep module paths. The library should re-export key types at the crate root for ergonomic imports.

**Why this priority**: Good API ergonomics improve developer experience but are secondary to the core structure.

**Independent Test**: A developer can write `use cauce_core::Signal;` without navigating into submodules.

**Acceptance Scenarios**:

1. **Given** a developer wants to use the Signal type, **When** they write `use cauce_core::Signal;`, **Then** the import works.
2. **Given** a developer wants to use multiple core types, **When** they write `use cauce_core::{Signal, Action, Topic};`, **Then** all imports work.

---

### Edge Cases

- What happens if a dependency version conflicts with workspace dependencies? Use workspace dependencies to ensure consistency.
- What if a module is empty? Include a module-level doc comment explaining its purpose.
- What happens during partial implementation? Modules can be created with placeholder types that get filled in by subsequent features.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A `crates/cauce-core/` directory MUST exist at the repository root
- **FR-002**: `crates/cauce-core/Cargo.toml` MUST define the crate with name `cauce-core`
- **FR-003**: Cargo.toml MUST include required dependencies: `serde`, `serde_json`, `thiserror`, `chrono`, `uuid`, `jsonschema`
- **FR-004**: The crate MUST be a workspace member in the root `Cargo.toml`
- **FR-005**: `src/lib.rs` MUST exist and declare the module structure
- **FR-006**: A `src/types/` module MUST exist for protocol type definitions
- **FR-007**: A `src/jsonrpc/` module MUST exist for JSON-RPC type definitions
- **FR-008**: A `src/validation/` module MUST exist for validation utilities
- **FR-009**: A `src/errors/` module MUST exist for error type definitions
- **FR-010**: A `src/constants/` module MUST exist for protocol constants
- **FR-011**: Each module MUST have a `mod.rs` file with appropriate documentation
- **FR-012**: The crate MUST compile successfully with `cargo build`
- **FR-013**: The crate MUST pass `cargo clippy` without warnings
- **FR-014**: The crate MUST pass `cargo fmt --check`

### Key Entities

- **cauce-core crate**: The shared library containing protocol types, validation, and utilities
- **Module structure**: Logical organization of code by concern (types, jsonrpc, validation, errors, constants)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo build -p cauce-core` completes in under 30 seconds on a standard developer machine
- **SC-002**: All 5 modules are present and documented
- **SC-003**: The crate has 100% of required dependencies declared
- **SC-004**: The crate integrates seamlessly as a workspace member (no workspace build errors)

## Test-Driven Development Approach *(mandatory)*

### Testing Strategy

- **Unit Tests**: Test that modules are properly exported and accessible
- **Integration Tests**: Verify crate can be used as a dependency by other workspace members
- **Contract Tests**: N/A for this setup phase (no protocol behavior yet)

### Coverage Requirement

Per Constitution Principle XI, this feature MUST:
- Have tests written BEFORE implementation code
- Follow Red-Green-Refactor cycle
- Achieve minimum **95% code coverage**
- Pass all tests in CI before merge

Note: For this project setup phase, coverage applies to any code written. Empty modules with only doc comments don't require tests.

### Test Boundaries

| Component | Test Focus | Coverage Target |
|-----------|------------|-----------------|
| lib.rs | Module exports, re-exports | 95% |
| types/mod.rs | Type availability | 95% |
| Other modules | Module initialization | 95% |

## Protocol Impact *(Cauce-specific)*

### Schema Impact

| Schema | Change Type | Description |
|--------|-------------|-------------|
| `signal.schema.json` | None | No schemas implemented in this phase |
| `action.schema.json` | None | No schemas implemented in this phase |
| `jsonrpc.schema.json` | None | No schemas implemented in this phase |
| `errors.schema.json` | None | No schemas implemented in this phase |
| `methods/*.schema.json` | None | No schemas implemented in this phase |
| `payloads/*.schema.json` | None | No schemas implemented in this phase |

### Component Interactions

This feature creates the foundation for component implementations but does not implement component behavior.

| Component | Responsibility in This Feature | NOT Responsible For |
|-----------|-------------------------------|---------------------|
| **Adapter** | Will use types from cauce-core | N/A - not implemented |
| **Hub** | Will use types from cauce-core | N/A - not implemented |
| **Agent** | Will use types from cauce-core | N/A - not implemented |

### Transport Considerations

| Transport | Supported | Notes |
|-----------|-----------|-------|
| WebSocket | N/A | Types will be transport-agnostic |
| Server-Sent Events | N/A | Types will be transport-agnostic |
| HTTP Polling | N/A | Types will be transport-agnostic |
| Webhooks | N/A | Types will be transport-agnostic |

**Semantic consistency**: Types defined in cauce-core will have identical semantics regardless of transport (per Constitution Principle IV).

### Wire Protocol

- **New methods**: None (setup phase only)
- **Modified methods**: None
- **A2A impact**: None
- **MCP impact**: None

### Version Impact

- **Change type**: MINOR (new functionality, backward compatible)
- **Rationale**: Adding a new crate with new public API; no breaking changes to existing code

## Assumptions

- The workspace is already initialized (from feature 001-repo-setup)
- Dependencies will be managed at workspace level where appropriate
- Empty modules are acceptable in this phase; they will be populated by subsequent features (2.2-2.9)
- Apache 2.0 license compliance is enforced by cargo-deny (from feature 002)

## Dependencies

- Feature 001-repo-setup: Workspace configuration must exist
- Feature 002-ci-cd-pipeline: CI will validate the new crate

## Out of Scope

- Implementing actual protocol types (covered in TODO.md 2.2-2.8)
- JSON schema embedding (covered in TODO.md 2.7)
- ID generation utilities (covered in TODO.md 2.8)
- Topic matching utilities (covered in TODO.md 2.9)
- Tests for protocol behavior (covered in TODO.md 2.10)

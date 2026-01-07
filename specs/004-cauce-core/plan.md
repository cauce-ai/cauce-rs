# Implementation Plan: cauce-core Project Setup

**Branch**: `004-cauce-core` | **Date**: 2026-01-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-cauce-core/spec.md`

## Summary

Create the foundational `cauce-core` crate as a workspace member with proper Cargo.toml configuration, module structure (types, jsonrpc, validation, errors, constants), and required dependencies. This is the base library that all other Cauce crates will depend on.

## Technical Context

**Language/Version**: Rust stable (1.75+)
**Primary Dependencies**: serde, serde_json, thiserror, chrono, uuid, jsonschema
**Storage**: N/A (library crate, no persistence)
**Testing Framework**: cargo test (built-in)
**Coverage Tool**: cargo-llvm-cov (established in feature 002)
**Coverage Threshold**: 95% (per Constitution Principle XI)
**Target Platform**: Cross-platform library (Linux, macOS, Windows)
**Project Type**: Rust workspace member (library crate)
**Performance Goals**: N/A for setup phase (types will be optimized in implementation)
**Constraints**: Apache 2.0 license only dependencies (per deny.toml)
**Scale/Scope**: 5 modules, ~10 initial files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|-----------|------|--------|
| **I. Spec-First** | Feature behavior defined in spec before implementation | ☑ Pass |
| **II. Schema-Driven** | Required JSON schemas identified (signal, action, jsonrpc, errors, methods/*) | ☑ N/A (schemas deferred to 2.7) |
| **III. Privacy First** | TLS 1.2+ requirement acknowledged; E2E encryption impact assessed | ☑ N/A (no network code) |
| **IV. Transport Agnostic** | Transport bindings identified (WebSocket/SSE/HTTP/Webhook); semantics unchanged across transports | ☑ N/A (types are transport-agnostic by design) |
| **V. Interoperability** | JSON-RPC 2.0 compliance verified; A2A/MCP impact assessed | ☑ N/A (jsonrpc module created but not implemented) |
| **VI. Component Separation** | Responsibilities mapped to Adapter/Hub/Agent; no boundary violations | ☑ N/A (shared types, no component logic) |
| **VII. Reliable Delivery** | At-least-once semantics maintained; ack/redelivery handled | ☑ N/A (no delivery logic) |
| **VIII. Adapter Resilience** | Local queuing/persistence requirements addressed | ☑ N/A (no adapter logic) |
| **IX. Semantic Versioning** | Version impact assessed (MAJOR/MINOR/PATCH) | ☑ Pass (MINOR - new crate) |
| **X. Graceful Degradation** | Capability negotiation supported; unsupported features handled | ☑ N/A (no runtime behavior) |
| **XI. TDD** | Test strategy defined; 95% coverage target confirmed | ☑ Pass |

**Blocking violations**: None. This is a project setup feature with no protocol behavior.

## Project Structure

### Documentation (this feature)

```text
specs/004-cauce-core/
├── plan.md              # This file
├── research.md          # Phase 0 output (dependency research)
├── quickstart.md        # Phase 1 output (how to use the crate)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

Note: `data-model.md` and `contracts/` are N/A - this is a library setup, not a data/API feature.

### Source Code (repository root)

```text
crates/cauce-core/
├── Cargo.toml           # Crate manifest with dependencies
└── src/
    ├── lib.rs           # Crate root with module declarations and re-exports
    ├── types/
    │   └── mod.rs       # Protocol types (Signal, Action, Topic, etc.)
    ├── jsonrpc/
    │   └── mod.rs       # JSON-RPC 2.0 types (Request, Response, etc.)
    ├── validation/
    │   └── mod.rs       # Validation utilities and schema validation
    ├── errors/
    │   └── mod.rs       # Error types and codes
    └── constants/
        └── mod.rs       # Protocol constants (method names, limits)
```

**Structure Decision**: Standard Rust library crate structure within the workspace `crates/` directory. Each module gets its own directory for future expansion (submodules in 2.2-2.9).

## Complexity Tracking

No violations to justify. This is a minimal setup feature.

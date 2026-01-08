# Implementation Plan: Core Types Module

**Branch**: `005-core-types` | **Date**: 2026-01-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-core-types/spec.md`

## Summary

Implement the Core Types module for `cauce-core` crate, providing foundational data structures for the Cauce Protocol: `Signal`, `Action`, `Topic`, and supporting types (`Source`, `Payload`, `Metadata`, `Encrypted`, enums). All types implement serde Serialize/Deserialize for JSON round-trip, with builder patterns for Signal and Action construction. Topic provides validated newtype with protocol-compliant patterns.

## Technical Context

**Language/Version**: Rust stable (1.75+)
**Primary Dependencies**: serde 1.0, serde_json 1.0, thiserror 1.0, chrono 0.4, uuid 1.0
**Storage**: N/A (library crate, in-memory types only)
**Testing Framework**: cargo test (built-in)
**Coverage Tool**: cargo-llvm-cov
**Coverage Threshold**: 95% (per Constitution Principle XI)
**Target Platform**: Cross-platform library (Linux, macOS, Windows)
**Project Type**: Single Rust library crate
**Performance Goals**: N/A (data types have no runtime performance requirements beyond normal struct operations)
**Constraints**: Types must serialize to JSON matching Cauce protocol schemas; validation must be fast enough for message processing
**Scale/Scope**: 14 type definitions, ~20 test files, targeting 95%+ coverage

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|-----------|------|--------|
| **I. Spec-First** | Feature behavior defined in spec before implementation | ✅ Pass |
| **II. Schema-Driven** | Required JSON schemas identified (signal, action, jsonrpc, errors, methods/*) | ✅ Pass - Types implement existing schemas |
| **III. Privacy First** | TLS 1.2+ requirement acknowledged; E2E encryption impact assessed | ✅ Pass - Encrypted type supports E2E |
| **IV. Transport Agnostic** | Transport bindings identified (WebSocket/SSE/HTTP/Webhook); semantics unchanged across transports | ✅ N/A - Core types are transport-agnostic |
| **V. Interoperability** | JSON-RPC 2.0 compliance verified; A2A/MCP impact assessed | ✅ Pass - Types used by JSON-RPC layer |
| **VI. Component Separation** | Responsibilities mapped to Adapter/Hub/Agent; no boundary violations | ✅ Pass - Types shared across components |
| **VII. Reliable Delivery** | At-least-once semantics maintained; ack/redelivery handled | ✅ N/A - Types don't handle delivery |
| **VIII. Adapter Resilience** | Local queuing/persistence requirements addressed | ✅ N/A - Types don't handle queuing |
| **IX. Semantic Versioning** | Version impact assessed (MAJOR/MINOR/PATCH) | ✅ Pass - MINOR version bump |
| **X. Graceful Degradation** | Capability negotiation supported; unsupported features handled | ✅ N/A - Types are foundational |
| **XI. TDD** | Test strategy defined; 95% coverage target confirmed | ✅ Pass - TDD strategy in spec |

**Blocking violations**: None

## Project Structure

### Documentation (this feature)

```text
specs/005-core-types/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/cauce-core/
├── Cargo.toml           # Already exists with dependencies
├── src/
│   ├── lib.rs           # Crate root with module re-exports
│   ├── types/
│   │   ├── mod.rs       # Types module (Signal, Action, Topic, etc.)
│   │   ├── signal.rs    # Signal struct and SignalBuilder
│   │   ├── action.rs    # Action struct and ActionBuilder
│   │   ├── topic.rs     # Topic newtype with validation
│   │   ├── source.rs    # Source struct
│   │   ├── payload.rs   # Payload struct
│   │   ├── metadata.rs  # Metadata struct
│   │   ├── encrypted.rs # Encrypted struct and EncryptionAlgorithm
│   │   └── enums.rs     # Priority, ActionType enums
│   ├── errors/
│   │   └── mod.rs       # Error types (ValidationError, BuilderError)
│   ├── jsonrpc/
│   │   └── mod.rs       # Placeholder (future feature)
│   ├── validation/
│   │   └── mod.rs       # Validation utilities
│   └── constants/
│       └── mod.rs       # Protocol constants
└── tests/
    ├── signal_test.rs   # Signal unit and integration tests
    ├── action_test.rs   # Action unit and integration tests
    ├── topic_test.rs    # Topic validation tests
    ├── builder_test.rs  # Builder pattern tests
    ├── serde_test.rs    # Serialization round-trip tests
    └── types_test.rs    # Supporting types tests
```

**Structure Decision**: Single library crate structure. The `types/` module is expanded into individual files per major type for maintainability, while smaller types (enums) are grouped. Tests are organized by domain area.

## Complexity Tracking

> No violations requiring justification. All Constitution principles satisfied or N/A for this feature.

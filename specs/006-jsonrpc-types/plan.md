# Implementation Plan: JSON-RPC Types

**Branch**: `006-jsonrpc-types` | **Date**: 2026-01-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-jsonrpc-types/spec.md`

## Summary

Implement JSON-RPC 2.0 message types for the cauce-core crate's `jsonrpc/` module. This provides the wire protocol foundation for all Cauce Protocol communication: Request, Response, Notification, Error, and RequestId types with full serialization support.

## Technical Context

**Language/Version**: Rust stable (1.75+)
**Primary Dependencies**: serde, serde_json, thiserror (all already in workspace)
**Storage**: N/A (library crate, no persistence)
**Testing Framework**: cargo test (built-in)
**Coverage Tool**: cargo-llvm-cov (configured in CI)
**Coverage Threshold**: 95% (per Constitution Principle XI)
**Target Platform**: All platforms (library crate)
**Project Type**: single (Rust crate within workspace)
**Performance Goals**: N/A (types only, no runtime behavior)
**Constraints**: MSRV 1.75 (no std::sync::LazyLock)
**Scale/Scope**: 5 types, ~12 functional requirements

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|-----------|------|--------|
| **I. Spec-First** | Feature behavior defined in spec before implementation | ☑ Pass |
| **II. Schema-Driven** | Required JSON schemas identified (signal, action, jsonrpc, errors, methods/*) | ☑ Pass |
| **III. Privacy First** | TLS 1.2+ requirement acknowledged; E2E encryption impact assessed | ☑ N/A |
| **IV. Transport Agnostic** | Transport bindings identified (WebSocket/SSE/HTTP/Webhook); semantics unchanged across transports | ☑ Pass |
| **V. Interoperability** | JSON-RPC 2.0 compliance verified; A2A/MCP impact assessed | ☑ Pass |
| **VI. Component Separation** | Responsibilities mapped to Adapter/Hub/Agent; no boundary violations | ☑ Pass |
| **VII. Reliable Delivery** | At-least-once semantics maintained; ack/redelivery handled | ☑ N/A |
| **VIII. Adapter Resilience** | Local queuing/persistence requirements addressed | ☑ N/A |
| **IX. Semantic Versioning** | Version impact assessed (MAJOR/MINOR/PATCH) | ☑ Pass |
| **X. Graceful Degradation** | Capability negotiation supported; unsupported features handled | ☑ N/A |
| **XI. TDD** | Test strategy defined; 95% coverage target confirmed | ☑ Pass |

**Blocking violations**: None

**Notes**:
- III, VII, VIII, X are N/A - this feature provides data types only, no transport/encryption/delivery behavior
- V verified: JSON-RPC 2.0 spec (https://www.jsonrpc.org/specification) is the authoritative reference
- IX: MINOR version bump - adds new types without breaking existing functionality

## Project Structure

### Documentation (this feature)

```text
specs/006-jsonrpc-types/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (N/A for library types)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/cauce-core/
├── src/
│   ├── lib.rs              # Re-exports jsonrpc types
│   ├── jsonrpc/
│   │   ├── mod.rs          # Module root with re-exports
│   │   ├── request.rs      # JsonRpcRequest type
│   │   ├── response.rs     # JsonRpcResponse type (success/error variants)
│   │   ├── notification.rs # JsonRpcNotification type
│   │   ├── error.rs        # JsonRpcError type
│   │   └── id.rs           # RequestId type (string/integer)
│   ├── types/              # Existing core types
│   ├── builders/           # Existing builders
│   ├── constants/          # Existing constants
│   ├── errors/             # Existing error types
│   └── validation/         # Existing validation
└── tests/
    └── jsonrpc_test.rs     # Integration tests for JSON-RPC types
```

**Structure Decision**: Extends existing cauce-core crate with new `jsonrpc/` submodule files. Each JSON-RPC type gets its own file for clarity and maintainability.

## Complexity Tracking

> No Constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

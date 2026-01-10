# Implementation Plan: Phase 2 Completion - Cauce Core Library

**Branch**: `007-phase2-completion` | **Date**: 2026-01-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-phase2-completion/spec.md`

## Summary

Complete the remaining Phase 2 items for cauce-core library: Method Parameter Types (21 types for JSON-RPC methods), Protocol Error Codes (20 error codes with JsonRpcError conversion), Method Constants (18 method names + 5 size limits), enhanced Validation (schema validation, topic patterns, ID validation), ID Generation utilities (5 generators), Topic Matching with wildcard support, and comprehensive testing to achieve 95% coverage.

## Technical Context

**Language/Version**: Rust 1.75+ (stable)
**Primary Dependencies**: serde 1.0, serde_json 1.0, thiserror 1.0, chrono 0.4, uuid 1.0, jsonschema 0.27, regex 1.10, once_cell 1.19
**Storage**: N/A (library crate, in-memory types only)
**Testing Framework**: cargo test (built-in)
**Coverage Tool**: cargo-llvm-cov (with llvm-tools-preview)
**Coverage Threshold**: 95% (per Constitution Principle XI)
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Library crate within Cargo workspace
**Performance Goals**: Topic matching should handle 10,000 patterns in < 10ms; ID generation < 1ms
**Constraints**: MSRV 1.75 (cannot use LazyLock, use once_cell instead)
**Scale/Scope**: 52 functional requirements, 21 method parameter types, 20 error codes, 17 method constants

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|-----------|------|--------|
| **I. Spec-First** | Feature behavior defined in spec before implementation | ☑ Pass |
| **II. Schema-Driven** | Required JSON schemas identified (signal, action, jsonrpc, errors, methods/*) | ☑ Pass |
| **III. Privacy First** | TLS 1.2+ requirement acknowledged; E2E encryption impact assessed | ☑ N/A (library types only) |
| **IV. Transport Agnostic** | Transport bindings identified (WebSocket/SSE/HTTP/Webhook); semantics unchanged across transports | ☑ Pass |
| **V. Interoperability** | JSON-RPC 2.0 compliance verified; A2A/MCP impact assessed | ☑ Pass |
| **VI. Component Separation** | Responsibilities mapped to Adapter/Hub/Agent; no boundary violations | ☑ Pass |
| **VII. Reliable Delivery** | At-least-once semantics maintained; ack/redelivery handled | ☑ N/A (types only, no runtime behavior) |
| **VIII. Adapter Resilience** | Local queuing/persistence requirements addressed | ☑ N/A (types only, no runtime behavior) |
| **IX. Semantic Versioning** | Version impact assessed (MAJOR/MINOR/PATCH) | ☑ Pass (MINOR - backward compatible) |
| **X. Graceful Degradation** | Capability negotiation supported; unsupported features handled | ☑ Pass (Capability enum defined) |
| **XI. TDD** | Test strategy defined; 95% coverage target confirmed | ☑ Pass |

**Blocking violations**: None

## Project Structure

### Documentation (this feature)

```text
specs/007-phase2-completion/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/cauce-core/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Public API re-exports
    ├── types/                    # Core protocol types (existing)
    │   ├── mod.rs
    │   ├── signal.rs
    │   ├── action.rs
    │   ├── topic.rs
    │   ├── source.rs
    │   ├── payload.rs
    │   ├── metadata.rs
    │   ├── encrypted.rs
    │   └── enums.rs
    ├── jsonrpc/                  # JSON-RPC 2.0 types (existing)
    │   ├── mod.rs
    │   ├── request.rs
    │   ├── response.rs
    │   ├── notification.rs
    │   ├── error.rs
    │   └── id.rs
    ├── methods/                  # NEW: Method parameter types
    │   ├── mod.rs
    │   ├── hello.rs              # HelloRequest, HelloResponse
    │   ├── subscribe.rs          # SubscribeRequest, SubscribeResponse
    │   ├── unsubscribe.rs        # UnsubscribeRequest, UnsubscribeResponse
    │   ├── publish.rs            # PublishRequest, PublishResponse
    │   ├── ack.rs                # AckRequest, AckResponse
    │   ├── subscription.rs       # Subscription management types
    │   ├── schemas.rs            # SchemasListRequest, etc.
    │   ├── ping.rs               # PingParams, PongParams
    │   ├── signal_delivery.rs    # SignalDelivery notification
    │   ├── auth.rs               # Auth, AuthType
    │   ├── client.rs             # ClientType, Capability
    │   ├── transport.rs          # Transport, WebhookConfig, E2eConfig
    │   └── enums.rs              # ApprovalType, SubscriptionStatus
    ├── errors/                   # Enhanced error types
    │   ├── mod.rs                # Existing ValidationError, BuilderError
    │   └── protocol.rs           # NEW: CauceError enum
    ├── constants/                # Enhanced constants
    │   └── mod.rs                # Existing + NEW method names, limits
    ├── validation/               # Enhanced validation
    │   ├── mod.rs                # Existing topic/ID validation
    │   └── schema.rs             # NEW: JSON schema validation
    ├── id/                       # NEW: ID generation utilities
    │   └── mod.rs                # generate_signal_id, etc.
    ├── matching/                 # NEW: Topic matching
    │   ├── mod.rs
    │   └── trie.rs               # TopicMatcher, TopicTrie
    ├── schemas/                  # NEW: Embedded JSON schemas
    │   └── mod.rs                # include_str! for signal, action, jsonrpc, errors
    └── builders/                 # Existing builders
        ├── mod.rs
        ├── signal_builder.rs
        └── action_builder.rs

tests/                           # Integration tests (at crate root)
├── serialization_roundtrip.rs   # All types roundtrip tests
├── error_codes.rs               # Error code conversion tests
└── topic_matching.rs            # Topic pattern matching tests
```

**Structure Decision**: Single library crate structure within the existing Cargo workspace. New modules (`methods/`, `id/`, `matching/`) added alongside existing modules. Integration tests placed in `crates/cauce-core/tests/` directory.

## Complexity Tracking

> No Constitution Check violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

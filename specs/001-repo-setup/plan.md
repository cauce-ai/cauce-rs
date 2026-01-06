# Implementation Plan: Repository Setup

**Branch**: `001-repo-setup` | **Date**: 2026-01-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-repo-setup/spec.md`

## Summary

This feature establishes the foundational Rust workspace structure for the cauce-rs project, including workspace configuration, code formatting rules, linting configuration, dependency auditing, and pre-commit hooks. This is infrastructure-only with no runtime code.

## Technical Context

**Language/Version**: Rust stable (1.75+)
**Primary Dependencies**: None (configuration files only; tools are external)
**Storage**: N/A (no runtime storage)
**Testing Framework**: cargo test (for future crates)
**Coverage Tool**: cargo-tarpaulin or cargo-llvm-cov
**Coverage Threshold**: 95% (per Constitution Principle XI)
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Cargo workspace (multi-crate)
**Performance Goals**: N/A (configuration only)
**Constraints**: Pre-commit hooks must complete in <60 seconds for typical commits
**Scale/Scope**: Workspace will eventually contain ~15 crates per TODO.md

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|-----------|------|--------|
| **I. Spec-First** | Feature behavior defined in spec before implementation | ☑ Pass |
| **II. Schema-Driven** | Required JSON schemas identified (signal, action, jsonrpc, errors, methods/*) | ☐ N/A |
| **III. Privacy First** | TLS 1.2+ requirement acknowledged; E2E encryption impact assessed | ☐ N/A |
| **IV. Transport Agnostic** | Transport bindings identified (WebSocket/SSE/HTTP/Webhook); semantics unchanged across transports | ☐ N/A |
| **V. Interoperability** | JSON-RPC 2.0 compliance verified; A2A/MCP impact assessed | ☐ N/A |
| **VI. Component Separation** | Responsibilities mapped to Adapter/Hub/Agent; no boundary violations | ☐ N/A |
| **VII. Reliable Delivery** | At-least-once semantics maintained; ack/redelivery handled | ☐ N/A |
| **VIII. Adapter Resilience** | Local queuing/persistence requirements addressed | ☐ N/A |
| **IX. Semantic Versioning** | Version impact assessed (MAJOR/MINOR/PATCH) | ☐ N/A |
| **X. Graceful Degradation** | Capability negotiation supported; unsupported features handled | ☐ N/A |
| **XI. TDD** | Test strategy defined; 95% coverage target confirmed | ☑ Pass |

**Blocking violations**: None. This is infrastructure setup that doesn't affect protocol behavior. Principles II-X are N/A as they apply to protocol runtime, not project tooling.

## Project Structure

### Documentation (this feature)

```text
specs/001-repo-setup/
├── plan.md              # This file
├── research.md          # Phase 0 output - tooling decisions
├── data-model.md        # Phase 1 output - N/A for this feature
├── quickstart.md        # Phase 1 output - developer setup guide
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
# Cargo workspace structure
Cargo.toml               # Workspace manifest with shared dependencies
crates/                  # All workspace member crates (empty initially)
├── cauce-core/          # Core types (Phase 2)
├── cauce-client-sdk/    # Client SDK (Phase 3)
├── cauce-server-sdk/    # Server SDK (Phase 4)
├── cauce-hub/           # Reference Hub (Phase 5)
├── cauce-cli/           # Hub CLI (Phase 6)
├── cauce-agent-cli/     # Agent CLI (Phase 7)
└── cauce-adapter-*/     # Adapters (Phases 8, 11, 12)

# Configuration files
.gitignore               # Rust/IDE ignore patterns
rustfmt.toml             # Formatting rules
deny.toml                # cargo-deny license/security policy
.pre-commit-config.yaml  # Hook documentation (reference only, not the runner)

# Note: clippy.toml is optional - clippy defaults are used initially

# Pre-commit hooks
.githooks/               # Custom git hooks directory
└── pre-commit           # Main pre-commit script
```

**Structure Decision**: Cargo workspace with `crates/` directory for all member crates. This follows the common Rust monorepo pattern and matches the TODO.md structure.

## Complexity Tracking

No Constitution violations requiring justification.

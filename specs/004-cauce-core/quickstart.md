# Quickstart: cauce-core

**Feature**: 004-cauce-core
**Date**: 2026-01-07

## What This Feature Delivers

This feature creates the foundational `cauce-core` crate with:

1. **Cargo.toml** - Crate manifest with required dependencies
2. **Module structure** - Organized layout for protocol types and utilities
3. **Workspace integration** - Proper setup as a workspace member

## Files Created

| File | Location | Purpose |
|------|----------|---------|
| Cargo.toml | crates/cauce-core/ | Crate manifest with dependencies |
| lib.rs | crates/cauce-core/src/ | Crate root with module declarations |
| types/mod.rs | crates/cauce-core/src/ | Protocol types module |
| jsonrpc/mod.rs | crates/cauce-core/src/ | JSON-RPC 2.0 types module |
| validation/mod.rs | crates/cauce-core/src/ | Validation utilities module |
| errors/mod.rs | crates/cauce-core/src/ | Error types module |
| constants/mod.rs | crates/cauce-core/src/ | Protocol constants module |

## Directory Structure

```
crates/cauce-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types/
    │   └── mod.rs
    ├── jsonrpc/
    │   └── mod.rs
    ├── validation/
    │   └── mod.rs
    ├── errors/
    │   └── mod.rs
    └── constants/
        └── mod.rs
```

## Usage

### Building the Crate

```bash
# Build cauce-core only
cargo build -p cauce-core

# Build entire workspace
cargo build --workspace
```

### Running Tests

```bash
# Run cauce-core tests
cargo test -p cauce-core

# Run with coverage
cargo llvm-cov --package cauce-core
```

### Using as a Dependency

In another workspace crate's Cargo.toml:

```toml
[dependencies]
cauce-core = { path = "../cauce-core" }
```

Then import types:

```rust
use cauce_core::Signal;
use cauce_core::Action;
use cauce_core::Topic;
```

## Module Purposes

| Module | Responsibility | Future Content |
|--------|----------------|----------------|
| `types` | Protocol type definitions | Signal, Action, Topic, Subscription |
| `jsonrpc` | JSON-RPC 2.0 compliance | Request, Response, Error types |
| `validation` | Input validation | Schema validation, field validators |
| `errors` | Error handling | CauceError enum, error codes |
| `constants` | Protocol constants | Method names, limits, defaults |

## Dependencies

| Crate | Purpose |
|-------|---------|
| serde | Serialization framework |
| serde_json | JSON serialization |
| thiserror | Error type derivation |
| chrono | Timestamp handling |
| uuid | ID generation |
| jsonschema | Schema validation |

## Validation

After implementation, verify:

- [ ] `cargo build -p cauce-core` succeeds
- [ ] `cargo test -p cauce-core` runs (even if no tests yet)
- [ ] `cargo clippy -p cauce-core` passes without warnings
- [ ] `cargo fmt --check -p cauce-core` passes
- [ ] All 5 modules visible in documentation

## Related

- **TODO.md** - Phases 2.2-2.9 will populate these modules
- **deny.toml** - License compliance (requires MIT addition for jsonschema)
- **Constitution** - Principle XI (TDD), Principle VI (Component Separation)

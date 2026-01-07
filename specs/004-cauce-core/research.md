# Research: cauce-core Project Setup

**Feature**: 004-cauce-core | **Date**: 2026-01-07 | **Phase**: 0

## Purpose

Document dependency decisions and license compliance for the cauce-core crate.

## Dependency Research

### Required Dependencies (from spec.md FR-003)

| Dependency | Purpose | Version | License | Status |
|------------|---------|---------|---------|--------|
| serde | Serialization/deserialization framework | 1.x | MIT OR Apache-2.0 | ✅ Approved |
| serde_json | JSON serialization | 1.x | MIT OR Apache-2.0 | ✅ Approved |
| thiserror | Ergonomic error types | 1.x | MIT OR Apache-2.0 | ✅ Approved |
| chrono | Date/time handling | 0.4.x | MIT OR Apache-2.0 | ✅ Approved |
| uuid | UUID generation/parsing | 1.x | MIT OR Apache-2.0 | ✅ Approved |
| jsonschema | JSON Schema validation | 0.27.x | MIT | ⚠️ Review |

### License Compliance Analysis

**Project Constraint**: deny.toml allows only Apache-2.0 licensed dependencies.

**Dual-Licensed Crates (MIT OR Apache-2.0)**:
- serde, serde_json, thiserror, chrono, uuid
- These are compliant - Apache-2.0 option satisfies the policy

**MIT-Only Crates**:
- `jsonschema` - Licensed under MIT only

**Decision Required**: The `jsonschema` crate is MIT-licensed, not Apache-2.0. Options:

1. **Update deny.toml** to allow MIT license (common permissive license)
2. **Find alternative** JSON Schema validation crate with Apache-2.0 license
3. **Defer jsonschema** to a later feature and use manual validation initially

**Recommendation**: Option 1 - Update deny.toml to allow MIT. MIT is a permissive license compatible with Apache-2.0 projects. Most Rust ecosystem crates are dual-licensed or MIT.

### Alternative JSON Schema Crates

| Crate | License | Notes |
|-------|---------|-------|
| jsonschema | MIT | Most mature, well-maintained |
| valico | MIT | Older, less maintained |
| jsonschema-rs | MIT | Wrapper around jsonschema |

All JSON Schema validation crates in the Rust ecosystem are MIT-licensed. The recommendation stands: update deny.toml to allow MIT.

## Workspace Dependencies

To ensure version consistency across workspace members, dependencies should be defined in the root `Cargo.toml`:

```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
jsonschema = "0.27"
```

## Module Structure Rationale

| Module | Purpose | Future Content (2.2-2.9) |
|--------|---------|--------------------------|
| types/ | Protocol types | Signal, Action, Topic, etc. |
| jsonrpc/ | JSON-RPC 2.0 | Request, Response, Error types |
| validation/ | Validation utilities | Schema validation, field validation |
| errors/ | Error types | CauceError, error codes |
| constants/ | Protocol constants | Method names, limits, defaults |

This structure aligns with the TODO.md phases 2.2-2.9 and provides clear separation of concerns.

## Open Questions

None - all clarifications resolved through reasonable defaults.

## Decisions Made

1. **Dependency versions**: Use latest stable versions for all dependencies
2. **License policy**: Recommend updating deny.toml to allow MIT (required for jsonschema)
3. **Feature flags**: Enable `derive` for serde, `serde` for chrono/uuid, `v4` for uuid
4. **Module structure**: Directory-based modules (mod.rs) for future expansion

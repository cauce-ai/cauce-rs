# Research: Repository Setup

**Feature**: 001-repo-setup
**Date**: 2026-01-06

## Research Tasks

This feature involves configuration choices for Rust development tooling. No external unknowns require research—all tools are well-established in the Rust ecosystem.

---

## Decision 1: Workspace Layout

**Decision**: Use `crates/` directory for all workspace members

**Rationale**:
- Common pattern in large Rust projects (tokio, serde, bevy)
- Keeps repository root clean
- Matches TODO.md planned structure with ~15 crates
- Enables glob patterns for workspace members: `crates/*`

**Alternatives Considered**:
- Flat layout (all crates at root): Rejected—clutters root with 15+ directories
- `packages/` or `libs/`: Rejected—`crates/` is the Rust convention

---

## Decision 2: Code Formatting Tool

**Decision**: rustfmt (standard Rust formatter)

**Rationale**:
- Official Rust project tooling
- Ships with rustup
- Universal adoption ensures contributor familiarity
- Deterministic output across platforms

**Alternatives Considered**:
- None viable—rustfmt is the only production-ready Rust formatter

---

## Decision 3: Linting Tool

**Decision**: clippy (standard Rust linter)

**Rationale**:
- Official Rust project tooling
- Ships with rustup
- Comprehensive lint coverage (correctness, style, performance, complexity)
- Widely adopted, well-documented

**Alternatives Considered**:
- None viable—clippy is the only production-ready Rust linter

---

## Decision 4: Dependency Auditing Tool

**Decision**: cargo-deny

**Rationale**:
- Checks licenses, vulnerabilities, duplicate dependencies, and banned crates
- Single tool covers all audit requirements (FR-006)
- Active maintenance, good ecosystem adoption
- CI-friendly with configurable policies

**Alternatives Considered**:
- cargo-audit: Only checks vulnerabilities, not licenses—insufficient for FR-006
- cargo-license: Only lists licenses, no policy enforcement
- Manual review: Not scalable, error-prone

---

## Decision 5: Pre-commit Hook Framework

**Decision**: Native git hooks with shell script

**Rationale**:
- Zero external dependencies (no Python/Node required)
- Full control over hook behavior
- Faster execution (no framework overhead)
- Works on all platforms with bash/sh
- Simpler setup for contributors

**Alternatives Considered**:
- pre-commit (Python): Adds Python dependency to Rust project—unnecessary complexity
- husky (Node): Adds Node dependency to Rust project—unnecessary complexity
- rusty-hook: Less mature, limited adoption

---

## Decision 6: Coverage Tool

**Decision**: cargo-tarpaulin (primary), with cargo-llvm-cov as documented alternative

**Rationale**:
- cargo-tarpaulin: Mature, widely used, good CI integration
- cargo-llvm-cov: More accurate but requires LLVM tools, harder to set up
- Document both to allow contributor choice based on environment

**Alternatives Considered**:
- grcov: Works but less Rust-native than tarpaulin
- kcov: Platform-specific limitations

---

## Decision 7: Git Hooks Installation Method

**Decision**: Configure `core.hooksPath` to `.githooks/` directory

**Rationale**:
- Hooks versioned in repository (consistent across team)
- No symlink management required
- Single git config command to enable
- Works across platforms

**Alternatives Considered**:
- Symlinks in `.git/hooks/`: Not version-controlled, requires per-clone setup
- Copy hooks on build: Adds build step, can get out of sync

---

## Decision 8: Clippy Strictness Level

**Decision**: `--deny warnings` in CI, standard warnings locally

**Rationale**:
- CI enforces zero warnings (prevents warning accumulation)
- Local development allows warnings during iteration
- Matches common Rust project patterns

**Alternatives Considered**:
- Always deny: Too strict for local development workflow
- Allow warnings in CI: Leads to warning debt over time

---

## Decision 9: rustfmt Configuration Style

**Decision**: Minimal configuration with opinionated defaults

**Rationale**:
- rustfmt defaults are well-considered and widely accepted
- Fewer config options = less bikeshedding
- Only configure where project has specific needs (e.g., imports_granularity)

**Configuration items to set**:
- `edition = "2021"` (explicit edition)
- `imports_granularity = "Module"` (cleaner diffs)
- `group_imports = "StdExternalCrate"` (stdlib first, then external, then local)

**Alternatives Considered**:
- Extensive customization: More maintenance burden, style debates
- No configuration file: Risk of differing defaults across rustfmt versions

---

## Decision 10: License Policy for Dependencies

**Decision**: Allow MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0, Unlicense

**Rationale**:
- Standard permissive licenses compatible with any project license
- Covers 95%+ of Rust ecosystem dependencies
- Excludes copyleft (GPL, LGPL) which may have compatibility concerns

**Alternatives Considered**:
- Allow all licenses: Risk of accidentally including copyleft dependencies
- Allow only MIT/Apache-2.0: Too restrictive, blocks useful crates

---

## Summary

All decisions use established, well-documented Rust ecosystem tools. No external research or experimentation required—these are industry-standard choices that minimize risk and maximize contributor familiarity.

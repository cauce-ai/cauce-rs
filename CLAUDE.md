# cauce-rs Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-01-06

## Active Technologies
- YAML (GitHub Actions workflow syntax), Rust 1.75+ (target of CI) + GitHub Actions, cargo, rustfmt, clippy, cargo-deny, cargo-tarpaulin/llvm-cov (002-ci-cd-pipeline)
- N/A - Configuration files only (002-ci-cd-pipeline)
- Markdown (documentation only) + None (static documentation files) (003-docs-setup)
- N/A (files only) (003-docs-setup)
- Rust stable (1.75+) + serde, serde_json, thiserror, chrono, uuid, jsonschema (004-cauce-core)
- N/A (library crate, no persistence) (004-cauce-core)
- Rust stable (1.75+) + serde 1.0, serde_json 1.0, thiserror 1.0, chrono 0.4, uuid 1.0 (005-core-types)
- N/A (library crate, in-memory types only) (005-core-types)

- Rust stable (1.75+) + None (configuration files only; tools are external) (001-repo-setup)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust stable (1.75+): Follow standard conventions

## Recent Changes
- 005-core-types: Added Rust stable (1.75+) + serde 1.0, serde_json 1.0, thiserror 1.0, chrono 0.4, uuid 1.0
- 004-cauce-core: Added Rust stable (1.75+) + serde, serde_json, thiserror, chrono, uuid, jsonschema
- 003-docs-setup: Added Markdown (documentation only) + None (static documentation files)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

## Protocol Principles

**Cauce Protocol**: Constitution v1.1.0

Key development requirements:
- **TDD Required**: Tests before code, 95% coverage (Principle XI)
- **Spec-First**: Behavior defined in spec before implementation (Principle I)
- **Schema-Driven**: JSON Schemas for all protocol messages (Principle II)
- **Component Separation**: Adapter/Hub/Agent boundaries (Principle VI)

Full constitution: `.specify/memory/constitution.md`

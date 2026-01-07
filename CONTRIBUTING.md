# Contributing to Cauce-RS

Thank you for your interest in contributing to cauce-rs! This document provides guidelines and information to help you contribute effectively.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Code Standards](#code-standards)
- [Commit Conventions](#commit-conventions)
- [Branch Naming](#branch-naming)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)

## Prerequisites

Before contributing, ensure you have the following installed:

### Required Tools

- **Rust toolchain** (stable): Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **cargo-deny**: License and security auditing
  ```bash
  cargo install cargo-deny
  ```

- **cargo-llvm-cov**: Code coverage reporting
  ```bash
  cargo install cargo-llvm-cov
  ```

### Verify Installation

```bash
rustc --version      # Rust compiler
cargo --version      # Cargo package manager
cargo deny --version # Dependency auditor
cargo llvm-cov --version # Coverage tool
```

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/cauce-ai/cauce-rs.git
cd cauce-rs
```

### Build the Project

```bash
cargo build --workspace
```

### Run Tests

```bash
cargo test --workspace
```

### Run All Checks (Recommended Before PR)

```bash
# Format check
cargo fmt --all --check

# Lint check
cargo clippy --workspace --all-targets -- -D warnings

# Dependency audit
cargo deny check

# Tests with coverage
cargo llvm-cov --workspace --fail-under-lines 95
```

## Code Standards

### Formatting

All code must be formatted with `rustfmt`:

```bash
cargo fmt --all
```

The CI will reject PRs with formatting differences.

### Linting

All code must pass `clippy` with warnings treated as errors:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### Code Coverage

**Per Constitution Principle XI, all code must maintain 95% coverage.**

```bash
# Run tests with coverage
cargo llvm-cov --workspace

# Check coverage threshold
cargo llvm-cov --workspace --fail-under-lines 95
```

The CI enforces this threshold. PRs that reduce coverage below 95% will not be merged.

### Dependencies

All dependencies must:
- Use Apache 2.0 compatible licenses
- Pass security vulnerability checks

```bash
cargo deny check
```

## Commit Conventions

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
type(scope): description

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Formatting, no code change |
| `refactor` | Code restructuring |
| `test` | Adding or updating tests |
| `chore` | Maintenance tasks |
| `ci` | CI/CD changes |

### Examples

```
feat(core): add Signal validation
fix(hub): correct subscription timeout handling
docs: update ARCHITECTURE.md with new crate
test(client-sdk): add WebSocket reconnection tests
ci: add coverage enforcement to workflow
```

### Guidelines

- Use imperative mood ("add" not "added")
- Keep the first line under 72 characters
- Reference issues in the footer: `Fixes #123`

## Branch Naming

This project uses SpecKit for feature management. Branch names follow the pattern:

```
NNN-short-description
```

Where `NNN` is a zero-padded feature number.

### Examples

```
001-repo-setup
002-ci-cd-pipeline
003-docs-setup
004-cauce-core
```

### Creating a Feature Branch

```bash
# From main branch
git checkout main
git pull origin main
git checkout -b NNN-feature-name
```

Or use the SpecKit command to create a properly numbered branch.

## Pull Request Process

### Before Opening a PR

1. **Ensure all checks pass locally**:
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo llvm-cov --workspace --fail-under-lines 95
   cargo deny check
   ```

2. **Rebase on latest main**:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

3. **Push your branch**:
   ```bash
   git push -u origin NNN-feature-name
   ```

### Opening the PR

1. Create a pull request via GitHub
2. Fill in the PR template with:
   - Summary of changes
   - Testing performed
   - Related issues

### Review Process

1. All CI checks must pass
2. At least one approval required
3. Address review feedback promptly

### Merging

PRs are merged using **squash merge** to keep history clean. The merge commit message should follow conventional commits format.

## Issue Reporting

### Bug Reports

Include:
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages or logs

### Feature Requests

Include:
- Use case description
- Proposed solution (optional)
- Alternatives considered

---

## Questions?

If you have questions not covered here, please open an issue or start a discussion.

Thank you for contributing to cauce-rs!

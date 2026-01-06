# CI/CD Pipeline Quickstart

This guide explains how to work with the CI/CD pipeline for cauce-rs.

## Overview

The CI/CD system consists of three GitHub Actions workflows:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push, PR | Quality checks (build, test, lint, format, coverage, audit) |
| `release.yml` | Version tags | Build release binaries for all platforms |
| `docker.yml` | Push to main, tags | Build and publish Docker images |

## For Developers

### Before Pushing Code

Run these checks locally to catch issues before CI:

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Check coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace --fail-under 95

# Dependency audit (requires cargo-deny)
cargo deny check
```

### Installing CI Tools Locally

```bash
# Coverage tool
cargo install cargo-llvm-cov

# Dependency auditor
cargo install cargo-deny

# These are already installed via rustup:
# rustfmt, clippy
```

### Understanding CI Results

When you push code or open a PR, CI runs automatically. Check the **Actions** tab or PR status checks.

**Common failures and fixes**:

| Failure | Cause | Fix |
|---------|-------|-----|
| Format check | Code not formatted | Run `cargo fmt` |
| Clippy | Linting warnings | Fix warnings shown in logs |
| Tests | Test failures | Debug and fix failing tests |
| Coverage | Below 95% | Add more tests |
| Audit | Vulnerability or license issue | Update dependency or add exception |

### Coverage Requirements

Per project constitution, **95% code coverage is mandatory**:

- CI fails if coverage drops below 95%
- Coverage is measured on Linux (results apply to all platforms)
- View coverage report in CI artifacts

To check coverage locally:
```bash
# Quick coverage check
cargo llvm-cov --workspace

# With HTML report
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html

# Fail if below threshold
cargo llvm-cov --workspace --fail-under 95
```

## For Maintainers

### Creating a Release

1. Ensure all tests pass on main
2. Create and push a version tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

3. Release workflow automatically:
   - Builds optimized binaries for all platforms
   - Creates GitHub Release
   - Uploads binaries as release assets

### Release Targets

| Target | Platform | Architecture |
|--------|----------|--------------|
| x86_64-unknown-linux-gnu | Linux | x86_64 |
| x86_64-unknown-linux-musl | Linux (static) | x86_64 |
| aarch64-unknown-linux-gnu | Linux | ARM64 |
| x86_64-apple-darwin | macOS | Intel |
| aarch64-apple-darwin | macOS | Apple Silicon |
| x86_64-pc-windows-msvc | Windows | x86_64 |

### Docker Images

Docker images are built automatically on:
- Push to main branch (tagged as `latest`)
- Version tags (tagged as version, e.g., `v1.0.0`)

Pull images:
```bash
# Latest from main
docker pull ghcr.io/cauce-ai/cauce-rs:latest

# Specific version
docker pull ghcr.io/cauce-ai/cauce-rs:v1.0.0
```

## Workflow Files

All workflows are in `.github/workflows/`:

- `ci.yml` - Main CI pipeline
- `release.yml` - Release automation
- `docker.yml` - Container builds

## Troubleshooting

### CI is slow

- Check if caches are being used (look for "Cache restored" in logs)
- First run after Cargo.lock changes will be slower
- Matrix builds run in parallel, so total time is longest single job

### Coverage tool fails

```bash
# Ensure llvm tools are installed
rustup component add llvm-tools-preview

# Reinstall cargo-llvm-cov
cargo install cargo-llvm-cov --force
```

### cargo-deny fails

```bash
# Update advisory database
cargo deny fetch

# Check what's failing
cargo deny check --show-stats
```

### Platform-specific test failures

- Check CI logs for the specific platform
- Some tests may need platform-specific handling
- Use `#[cfg(target_os = "...")]` for platform-specific code

## CI Philosophy

1. **Fast feedback**: Formatting/linting checks run first and fail fast
2. **Comprehensive**: All platforms tested before merge
3. **Strict**: 95% coverage enforced, no exceptions
4. **Automated**: Releases and Docker builds require no manual steps

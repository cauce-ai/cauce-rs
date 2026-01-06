# Research: CI/CD Pipeline

**Feature**: 002-ci-cd-pipeline
**Date**: 2026-01-06

## Overview

This document captures research findings and decisions for the CI/CD Pipeline implementation.

---

## Decision 1: CI Platform

**Decision**: GitHub Actions

**Rationale**:
- Native integration with GitHub repository
- Free tier includes 2,000 minutes/month for public repos
- Supports Linux, macOS, and Windows runners
- Matrix builds for cross-platform testing
- Built-in caching for dependencies
- First-class support for Rust via actions-rs ecosystem

**Alternatives Considered**:
- **CircleCI**: Good Rust support but adds external dependency
- **Travis CI**: Less active development, pricing changes
- **GitLab CI**: Would require repository migration
- **Self-hosted runners**: Maintenance overhead not justified at this stage

---

## Decision 2: Coverage Tool

**Decision**: cargo-llvm-cov (primary), with cargo-tarpaulin as fallback

**Rationale**:
- cargo-llvm-cov provides accurate coverage using LLVM instrumentation
- Cross-platform support (Linux, macOS, Windows)
- Integrates well with GitHub Actions via Codecov/Coveralls
- Supports workspace coverage aggregation
- Active maintenance and community support

**Alternatives Considered**:
- **cargo-tarpaulin**: Linux-only, may have issues with async code
- **grcov**: Requires nightly Rust for some features
- **kcov**: Linux-only, less Rust-specific

**Configuration**:
```yaml
# Install with: cargo install cargo-llvm-cov
# Run with: cargo llvm-cov --workspace --fail-under 95
```

---

## Decision 3: Dependency Audit Tool

**Decision**: cargo-deny

**Rationale**:
- Already configured in repository (deny.toml exists from 001-repo-setup)
- Checks both licenses and security advisories
- Integrates well with CI
- Single tool for both compliance needs

**Alternatives Considered**:
- **cargo-audit**: Security only, no license checking
- **cargo-license**: Licenses only, no security checking
- Using both would add complexity

---

## Decision 4: Matrix Build Strategy

**Decision**: Build and test on all platforms, run coverage on Linux only

**Rationale**:
- Cross-platform builds verify platform compatibility
- Coverage measurement is consistent when run on single platform
- Linux runners are fastest and cheapest
- macOS ARM64 (M1/M2) testing via macos-latest

**Platform Matrix**:
| Platform | Build | Test | Coverage | Lint |
|----------|-------|------|----------|------|
| ubuntu-latest | ✅ | ✅ | ✅ | ✅ |
| macos-latest | ✅ | ✅ | ❌ | ❌ |
| windows-latest | ✅ | ✅ | ❌ | ❌ |

---

## Decision 5: Release Build Targets

**Decision**: Build release binaries for 6 targets

**Rationale**:
- Cover major deployment platforms
- Use cross-compilation where beneficial
- Static linking for portability

**Targets**:
| Target | Runner | Notes |
|--------|--------|-------|
| x86_64-unknown-linux-gnu | ubuntu-latest | Primary Linux target |
| x86_64-unknown-linux-musl | ubuntu-latest | Static binary, Alpine-compatible |
| aarch64-unknown-linux-gnu | ubuntu-latest | ARM64 Linux (cross-compile) |
| x86_64-apple-darwin | macos-latest | Intel Mac |
| aarch64-apple-darwin | macos-latest | Apple Silicon |
| x86_64-pc-windows-msvc | windows-latest | Windows |

---

## Decision 6: Docker Image Strategy

**Decision**: Multi-stage build with distroless runtime

**Rationale**:
- Multi-stage reduces image size
- Distroless provides minimal attack surface
- No shell access in production containers
- Consistent with security best practices

**Base Images**:
- Builder: `rust:1.75-slim` (or latest stable)
- Runtime: `gcr.io/distroless/cc-debian12` or `debian:bookworm-slim`

---

## Decision 7: Caching Strategy

**Decision**: Cache Cargo registry, index, and target directory

**Rationale**:
- Significantly reduces build times (up to 70% improvement)
- GitHub Actions cache action supports automatic key management
- Target directory caching requires careful invalidation

**Cache Keys**:
```yaml
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
restore-keys: |
  ${{ runner.os }}-cargo-
```

---

## Decision 8: Workflow Triggers

**Decision**: Different triggers for different workflows

**CI Workflow**:
- `push` to any branch
- `pull_request` to main/master

**Release Workflow**:
- `push` tags matching `v*.*.*`

**Docker Workflow**:
- `push` to main branch
- `push` tags matching `v*.*.*`

---

## Decision 9: Coverage Reporting

**Decision**: Fail CI if coverage below 95%, report to PR

**Rationale**:
- Per Constitution Principle XI, 95% coverage is mandatory
- PR comments provide visibility without external service
- Can add Codecov integration later for trends

**Implementation**:
```yaml
- name: Generate coverage report
  run: cargo llvm-cov --workspace --lcov --output-path lcov.info --fail-under 95

- name: Upload coverage to PR
  uses: actions/upload-artifact@v4
  with:
    name: coverage-report
    path: lcov.info
```

---

## Decision 10: Job Dependencies

**Decision**: Parallel jobs with strategic dependencies

**Job Graph**:
```
┌─────────┐    ┌─────────┐    ┌─────────┐
│ Format  │    │  Lint   │    │  Audit  │
└────┬────┘    └────┬────┘    └────┬────┘
     │              │              │
     └──────────────┼──────────────┘
                    │
              ┌─────▼─────┐
              │   Build   │  (matrix: linux, macos, windows)
              └─────┬─────┘
                    │
              ┌─────▼─────┐
              │   Test    │  (matrix: linux, macos, windows)
              └─────┬─────┘
                    │
              ┌─────▼─────┐
              │ Coverage  │  (linux only, with threshold check)
              └───────────┘
```

**Rationale**:
- Fast-fail on formatting/linting saves compute
- Build must succeed before testing
- Coverage runs after tests pass on all platforms

---

## Open Questions (Resolved)

All research questions have been resolved. No outstanding unknowns.

---

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Distroless Container Images](https://github.com/GoogleContainerTools/distroless)

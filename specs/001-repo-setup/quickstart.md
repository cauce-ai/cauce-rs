# Quickstart: cauce-rs Development Environment

This guide helps you set up your local development environment for contributing to cauce-rs.

## Prerequisites

### Required

- **Rust toolchain**: Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Git**: Version 2.9+ (for `core.hooksPath` support)

### Recommended

- **cargo-deny**: Dependency auditing
  ```bash
  cargo install cargo-deny
  ```

- **cargo-tarpaulin**: Code coverage
  ```bash
  cargo install cargo-tarpaulin
  ```

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/cauce-ai/cauce-rs.git
cd cauce-rs
```

### 2. Enable Pre-commit Hooks

```bash
git config core.hooksPath .githooks
```

This configures git to use the project's hooks from `.githooks/` instead of `.git/hooks/`.

### 3. Verify Toolchain

```bash
# Check Rust version (1.75+ required)
rustc --version

# Install components if missing
rustup component add rustfmt clippy
```

### 4. Build the Workspace

```bash
cargo build --workspace
```

### 5. Run Tests

```bash
cargo test --workspace
```

## Development Workflow

### Before Committing

Pre-commit hooks automatically run:
1. `cargo fmt --check` - Formatting verification
2. `cargo clippy -- -D warnings` - Lint checks
3. `cargo test` - Test suite

If any check fails, the commit is rejected with an error message.

### Manual Checks

```bash
# Format code
cargo fmt

# Run lints
cargo clippy

# Run tests with coverage
cargo tarpaulin --workspace

# Audit dependencies
cargo deny check
```

### Fixing Formatting Issues

```bash
# Auto-fix formatting
cargo fmt

# Then retry your commit
git commit
```

### Fixing Lint Issues

```bash
# View lint suggestions
cargo clippy

# Many issues have auto-fix suggestions
cargo clippy --fix
```

## Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest, shared dependencies |
| `rustfmt.toml` | Code formatting rules |
| `clippy.toml` | Lint configuration (if present) |
| `deny.toml` | Dependency license/security policy |
| `.pre-commit-config.yaml` | Hook configuration reference |
| `.githooks/pre-commit` | Pre-commit hook script |

## Troubleshooting

### "Pre-commit hook failed"

The hook will print which check failed:
- **Formatting**: Run `cargo fmt` and retry
- **Clippy**: Review warnings, fix issues, retry
- **Tests**: Fix failing tests, retry

### "cargo-deny not found"

Install it:
```bash
cargo install cargo-deny
```

### "Coverage tool not found"

Install cargo-tarpaulin:
```bash
cargo install cargo-tarpaulin
```

Or use cargo-llvm-cov if you have LLVM tools:
```bash
cargo install cargo-llvm-cov
```

### Hooks Not Running

Verify hooks are enabled:
```bash
git config core.hooksPath
# Should output: .githooks
```

If empty, re-run:
```bash
git config core.hooksPath .githooks
```

## Editor Setup

### VS Code

Recommended extensions:
- rust-analyzer
- Even Better TOML

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### Other Editors

Configure your editor to:
1. Run `cargo fmt` on save
2. Use rust-analyzer for IDE features
3. Show clippy warnings inline

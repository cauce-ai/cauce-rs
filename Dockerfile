# =============================================================================
# Cauce-RS Dockerfile
# =============================================================================
# Multi-stage build for minimal runtime image.
#
# Build stages:
#   1. builder: Compiles the Rust application
#   2. runtime: Minimal Debian-based image with only the binary
#
# Usage:
#   docker build -t cauce-rs .
#   docker run --rm cauce-rs
#
# For more information, see: specs/002-ci-cd-pipeline/quickstart.md
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
# Uses official Rust image to compile the application.
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the application in release mode
RUN cargo build --release --workspace

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
# Uses minimal Debian image for production.
# Note: Using debian:bookworm-slim instead of distroless for easier debugging.
# Switch to gcr.io/distroless/cc-debian12 for production hardening.
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies (if needed)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
# Note: Update this COPY command when specific binaries are available
# COPY --from=builder /app/target/release/cauce-hub /app/
# COPY --from=builder /app/target/release/cauce-adapter /app/

# For now, copy all release binaries (workspace may have multiple)
COPY --from=builder /app/target/release/ /app/target/release/

# Set default command (update when specific binary is available)
# CMD ["/app/cauce-hub"]
CMD ["echo", "Cauce-RS container ready. Update CMD when binaries are available."]

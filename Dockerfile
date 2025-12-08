# ============================================
# WEBRANA CLI - Dockerfile
# Created by: ATLAS (Team Beta)
# ============================================

# Stage 1: Builder
FROM rust:1.82-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies (including cmake for wasmtime)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy all source files
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY config ./config
COPY agents ./agents

# Build the binary
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash webrana

# Copy binary from builder
COPY --from=builder /app/target/release/webrana /usr/local/bin/webrana

# Copy default config
COPY --from=builder /app/config /app/config
COPY --from=builder /app/agents /app/agents

# Set permissions
RUN chmod +x /usr/local/bin/webrana

# Switch to non-root user
USER webrana

# Set environment
ENV RUST_LOG=info
ENV HOME=/home/webrana

# Create config directory
RUN mkdir -p /home/webrana/.config/webrana

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD webrana --version || exit 1

# Default command
ENTRYPOINT ["webrana"]
CMD ["--help"]

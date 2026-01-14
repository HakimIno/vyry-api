# Multi-stage build for Rust API
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Configure Rust for better caching and resource usage
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTFLAGS="-C target-cpu=native"

# Copy only Cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml ./apps/api/
COPY crates/*/Cargo.toml ./crates/
COPY migration/Cargo.toml ./migration/

# Create dummy source files for dependency caching
RUN mkdir -p apps/api/src crates/application/src crates/core/src crates/domain/src \
    crates/infrastructure/src crates/interfaces/src crates/p2p/src migration/src && \
    echo "fn main() {}" > apps/api/src/main.rs && \
    echo "" > crates/application/src/lib.rs && \
    echo "" > crates/core/src/lib.rs && \
    echo "" > crates/domain/src/lib.rs && \
    echo "" > crates/infrastructure/src/lib.rs && \
    echo "" > crates/interfaces/src/lib.rs && \
    echo "" > crates/p2p/src/lib.rs && \
    echo "fn main() {}" > migration/src/main.rs

# Build dependencies only (this layer will be cached)
# Limit parallel jobs to avoid OOM: use 1 job if RAM < 4GB, otherwise 2
RUN cargo build --release --package api --jobs 2 && \
    rm -rf apps/api/src crates/*/src migration/src

# Copy actual source code
COPY . .

# Build the application with limited jobs to save memory
RUN cargo build --release --package api --jobs 2

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api /app/api

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Run the application
CMD ["./api"]

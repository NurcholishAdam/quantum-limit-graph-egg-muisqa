# Quantum LIMIT-Graph egg - Production Dockerfile
# Multi-stage build for minimal image size

# Build stage
FROM rust:1.75-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY crates ./crates
COPY services ./services

# Build release binary
RUN cargo build --release --bin limit-api

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/limit-api /app/limit-api

# Create data directory
RUN mkdir -p /app/data

# Set environment variables
ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0:8080
ENV STORAGE_ROOT=/app/data

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run as non-root user
RUN useradd -m -u 1000 limit && chown -R limit:limit /app
USER limit

# Start the API service
CMD ["/app/limit-api"]

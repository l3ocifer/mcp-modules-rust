# Multi-stage build for MCP Modules Rust
FROM rust:1.75-slim AS builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml ./

# Copy source code
COPY src/ ./src/
COPY examples/ ./examples/
COPY tests/ ./tests/

# Build the release binary
RUN cargo build --release --bin devops-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false mcp

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/devops-mcp /app/devops-mcp

# Copy configuration files if any
COPY config/ ./config/
COPY monitoring/ ./monitoring/

# Create data directory
RUN mkdir -p /app/data && chown mcp:mcp /app/data

# Change ownership
RUN chown -R mcp:mcp /app

# Switch to app user
USER mcp

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["./devops-mcp"]

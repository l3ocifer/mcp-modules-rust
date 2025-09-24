# Multi-stage build for MCP Modules Rust
FROM rust:1.80-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r mcp && useradd -r -g mcp mcp

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/devops-mcp /app/devops-mcp

# Change ownership
RUN chown -R mcp:mcp /app

# Switch to non-root user
USER mcp

# Expose port
EXPOSE 8890

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8890/health || exit 1

# Run the application
CMD ["./devops-mcp"]

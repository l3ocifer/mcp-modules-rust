#!/bin/bash
# Quick Start Script for MCP Modules Rust

set -e

echo "🚀 MCP Modules Rust - Quick Start"
echo "================================="
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command_exists cargo; then
    echo "❌ Rust/Cargo not found!"
    echo "   Please install Rust: https://rustup.rs/"
    exit 1
fi
echo "✅ Rust/Cargo found"

if ! command_exists docker; then
    echo "⚠️  Docker not found - some features will be unavailable"
    echo "   Install Docker: https://docs.docker.com/get-docker/"
else
    echo "✅ Docker found"
fi

if ! command_exists docker-compose && ! command_exists "docker compose"; then
    echo "⚠️  Docker Compose not found"
    echo "   Install Docker Compose: https://docs.docker.com/compose/install/"
else
    echo "✅ Docker Compose found"
fi

# Build the project
echo ""
echo "🔨 Building the project..."
cargo build --release --all-features

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi
echo "✅ Build successful!"

# Start dependencies if Docker is available
if command_exists docker && (command_exists docker-compose || command_exists "docker compose"); then
    echo ""
    echo "🐳 Starting dependencies with Docker Compose..."
    
    # Use docker compose v2 if available, otherwise fall back to docker-compose
    if command_exists "docker compose"; then
        docker compose up -d
    else
        docker-compose up -d
    fi
    
    echo "⏳ Waiting for services to be healthy..."
    sleep 10
    
    echo ""
    echo "📊 Services status:"
    if command_exists "docker compose"; then
        docker compose ps
    else
        docker-compose ps
    fi
fi

# Create environment file
echo ""
echo "📝 Creating environment configuration..."
cat > .env << EOF
# Database Configuration
DATABASE_URL=postgresql://mcp:mcp_password@localhost:5432/mcp_db
MONGODB_URI=mongodb://mcp:mcp_password@localhost:27017/mcp?authSource=admin

# Monitoring
PROMETHEUS_URL=http://localhost:9090
GRAFANA_URL=http://localhost:3000

# Tracing
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=mcp-modules-rust

# Logging
RUST_LOG=devops_mcp=info,tower_http=debug

# MCP Configuration
MCP_TRANSPORT=http
MCP_HTTP_PORT=8080
EOF

echo "✅ Environment configured"

# Run the server
echo ""
echo "🚀 Starting MCP server..."
echo ""
echo "Server will be available at:"
echo "  - HTTP API: http://localhost:8080"
echo "  - Health: http://localhost:8080/health"
echo "  - Metrics: http://localhost:8080/metrics"
echo ""
echo "Monitoring available at:"
echo "  - Prometheus: http://localhost:9090"
echo "  - Grafana: http://localhost:3000 (admin/admin)"
echo "  - Jaeger: http://localhost:16686"
echo ""
echo "Press Ctrl+C to stop the server"
echo "================================="
echo ""

# Run with environment variables
export $(cat .env | grep -v '^#' | xargs)
./target/release/devops-mcp
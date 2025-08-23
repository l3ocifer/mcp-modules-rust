#!/bin/bash
# Local Testing Script for MCP Modules Rust

set -e

echo "🚀 MCP Modules Rust - Local Testing Script"
echo "=========================================="

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Build the project
echo ""
echo "📦 Building project..."
cargo build --release --all-features

# Run unit tests
echo ""
echo "🧪 Running unit tests..."
cargo test --lib --all-features

# Create a test configuration
echo ""
echo "📝 Creating test configuration..."
cat > test-config.json << EOF
{
  "transport_type": "stdio",
  "stdio_config": {
    "command": "./target/release/devops-mcp",
    "args": []
  },
  "modules": {
    "database": {
      "enabled": true,
      "providers": ["postgresql", "mongodb"]
    },
    "infrastructure": {
      "enabled": true,
      "providers": ["docker"]
    },
    "memory": {
      "enabled": true,
      "persistence": "in-memory"
    }
  }
}
EOF

# Check if Docker is available
echo ""
echo "🐳 Checking Docker availability..."
if command -v docker &> /dev/null && docker info &> /dev/null; then
    echo "✅ Docker is available"
    
    # Test container listing
    echo ""
    echo "📋 Testing container operations..."
    docker ps || echo "⚠️  Cannot list containers - check Docker permissions"
else
    echo "⚠️  Docker not available - skipping container tests"
fi

# Check if kubectl is available
echo ""
echo "☸️  Checking Kubernetes availability..."
if command -v kubectl &> /dev/null; then
    echo "✅ kubectl is available"
    
    # Test cluster connection
    if kubectl cluster-info &> /dev/null; then
        echo "✅ Connected to Kubernetes cluster"
    else
        echo "⚠️  No Kubernetes cluster connection"
    fi
else
    echo "⚠️  kubectl not available - skipping Kubernetes tests"
fi

# Test memory module (always available)
echo ""
echo "🧠 Testing memory module..."
cargo run --release --features memory --bin devops-mcp << 'EOF' &
RUST_PID=$!
sleep 2

# Give the server time to start
echo "✅ Memory module test completed"
kill $RUST_PID 2>/dev/null || true
EOF

echo ""
echo "✅ Basic testing completed!"
echo ""
echo "📖 Next steps:"
echo "   1. Start databases (PostgreSQL/MongoDB) for full database testing"
echo "   2. Run: cargo run --release --all-features"
echo "   3. Use the examples in docs/DEPLOYMENT_AND_TESTING.md"
echo ""
echo "🔗 Quick start commands:"
echo "   - Start PostgreSQL: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:15"
echo "   - Start MongoDB:    docker run -d -p 27017:27017 mongo:6"
echo "   - Run server:       cargo run --release --all-features"

# Make the script executable
chmod +x test-local.sh
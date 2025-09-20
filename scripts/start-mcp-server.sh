#!/bin/bash
# Start MCP Server Script
set -e

echo "🚀 Starting MCP Modules Server for Cursor Integration"
echo "===================================================="

# Use clean environment
unset RUSTUP_TOOLCHAIN
unset RUSTUP_HOME
export PATH="/home/l3o/.cargo/bin:/usr/bin:/bin"
export HOME="/home/l3o"

# Get the directory where this script is located, then go to parent directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

# Configuration
export MCP_HTTP_HOST="0.0.0.0"
export MCP_HTTP_PORT="8888"  # Safe port, not conflicting with existing services
export RUST_LOG="devops_mcp=info,tower_http=debug"

# Use existing homelab services instead of creating new ones
echo "📋 Using existing homelab services..."
echo "   ✅ PostgreSQL: neon-postgres-leopaska (port 5432)"
echo "   ✅ Redis: redis-nd-leopaska (port 6379)"

# Connect to existing services - you'll need to create the MCP database
export DATABASE_URL="postgresql://postgres:password@localhost:5432/mcp_db"
export REDIS_URL="redis://localhost:6379"

echo "📝 Note: You may need to create the 'mcp_db' database in your existing PostgreSQL"
echo "   Connect to your postgres and run: CREATE DATABASE mcp_db;"

echo ""
echo "✅ Configuration:"
echo "   - MCP Server: http://localhost:8888"
echo "   - PostgreSQL: localhost:5432 (existing neon-postgres)"
echo "   - Redis: localhost:6379 (existing redis-nd)"
echo ""
echo "🔗 For Cursor configuration, use:"
echo "   Transport: HTTP"
echo "   URL: http://localhost:8888"
echo ""
echo "Press Ctrl+C to stop the server"
echo "================================="
echo ""

# Start the MCP server
./target/release/devops-mcp

#!/bin/bash
# Clean Rust Build Script for MCP Modules
set -e

echo "🔧 Building MCP Modules with clean Rust environment..."

# Clean environment variables that might interfere
unset RUSTUP_TOOLCHAIN
unset RUSTUP_HOME
unset RUST_LOG

# Set clean path with just the essentials
export PATH="/home/l3o/.cargo/bin:/usr/bin:/bin"
export HOME="/home/l3o"

cd /home/l3o/git/l3ocifer/mcp-modules-rust

echo "📋 Rust version check..."
rustc --version
cargo --version

echo "🧹 Cleaning previous build artifacts..."
cargo clean

echo "🔨 Building with all features..."
cargo build --release --all-features --bin devops-mcp

echo "✅ Build completed successfully!"
echo "Binary location: ./target/release/devops-mcp"

# Test the binary
echo "🧪 Testing the binary..."
if [ -f "./target/release/devops-mcp" ]; then
    echo "✅ Binary exists and is ready to use"
    ./target/release/devops-mcp --help || echo "Binary built but needs runtime configuration"
else
    echo "❌ Binary not found"
    exit 1
fi

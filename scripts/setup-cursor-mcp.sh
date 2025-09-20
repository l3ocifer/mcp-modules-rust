#!/bin/bash

# Setup Cursor MCP Integration Script
set -e

echo "🔧 Setting up Cursor MCP Integration..."
echo "======================================"

cd "$(dirname "$0")/.."

# Check if MCP server is running
if ! netstat -tlnp 2>/dev/null | grep -q :8888; then
    echo "❌ MCP server not running on port 8888"
    echo "💡 Start it with: ./scripts/start-mcp-server.sh"
    exit 1
fi

echo "✅ MCP server is running on port 8888"

# Test the stdio bridge
echo "🧪 Testing stdio bridge..."
BRIDGE_TEST=$(echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}' | node scripts/mcp-stdio-bridge.js 2>/dev/null)

if echo "$BRIDGE_TEST" | grep -q '"jsonrpc":"2.0"'; then
    echo "✅ Stdio bridge is working"
else
    echo "❌ Stdio bridge test failed"
    echo "Bridge output: $BRIDGE_TEST"
    exit 1
fi

# Verify Cursor MCP config
CURSOR_CONFIG="$HOME/.cursor/mcp.json"
if [ ! -f "$CURSOR_CONFIG" ]; then
    echo "❌ Cursor MCP config not found at $CURSOR_CONFIG"
    exit 1
fi

echo "✅ Cursor MCP config found"

# Check if our server is configured
if grep -q "devops-mcp-rust" "$CURSOR_CONFIG"; then
    echo "✅ devops-mcp-rust is configured in Cursor"
else
    echo "❌ devops-mcp-rust not found in Cursor config"
    exit 1
fi

echo ""
echo "🎉 Setup Complete!"
echo "=================="
echo ""
echo "✅ MCP HTTP server: http://localhost:8888"
echo "✅ Stdio bridge: $(pwd)/scripts/mcp-stdio-bridge.js"
echo "✅ Cursor config: $CURSOR_CONFIG"
echo ""
echo "📋 Next steps:"
echo "1. Restart Cursor completely (close all windows)"
echo "2. Open Cursor settings > MCP"
echo "3. Verify 'devops-mcp-rust' shows as enabled"
echo "4. Open a new Composer session"
echo "5. Ask: 'What MCP tools are available?'"
echo ""
echo "🔍 If issues persist:"
echo "- Check Cursor developer console (Help > Toggle Developer Tools)"
echo "- Look for MCP-related error messages"
echo "- Ensure Node.js is available in PATH"

# Cursor MCP Setup Instructions

## ✅ MCP Server Status
Your MCP server is **running successfully** on `http://localhost:8888`

## 🔧 Cursor Configuration

### Method 1: HTTP Transport (Recommended)
Since your MCP server is running as an HTTP service, configure Cursor to connect via HTTP:

1. **Open Cursor Settings**
   - Go to `Settings` → `Features` → `MCP`

2. **Add New MCP Server**
   - Click `+ Add New MCP Server`
   - **Name**: `DevOps MCP Rust`
   - **Type**: `SSE` (Server-Sent Events)
   - **URL**: `http://localhost:8888/sse`

### Method 2: Direct HTTP API (Alternative)
If SSE doesn't work, you can use direct HTTP calls:

1. **Server Configuration**:
   ```json
   {
     "name": "devops-mcp-rust",
     "type": "http",
     "url": "http://localhost:8888",
     "timeout": 30000
   }
   ```

## 🧪 Testing the Connection

### Test 1: Health Check
```bash
curl http://localhost:8888/health
```
Expected response: `OK`

### Test 2: Available Tools
```bash
curl -X POST http://localhost:8888 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### Test 3: MCP Initialize
```bash
curl -X POST http://localhost:8888 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": {
        "name": "cursor-test",
        "version": "1.0.0"
      }
    }
  }'
```

## 📋 Available Modules

Your MCP server includes these powerful modules:

### 🏗️ Infrastructure
- **Kubernetes**: Cluster management, pod deployment
- **Docker**: Container lifecycle management  
- **Cloud**: Azure, AWS, GCP integration
- **Cloudflare**: DNS and CDN management

### 🗄️ Database
- **MongoDB**: Document database operations
- **PostgreSQL**: Relational database management
- **Supabase**: Backend-as-a-Service integration

### 🔒 Security
- **Credential Management**: Secure storage
- **Input Validation**: Comprehensive sanitization
- **Rate Limiting**: Governor-based protection
- **Cryptography**: Ring-based operations

### 📊 Office & Productivity
- **PowerPoint**: Create and edit presentations
- **Word**: Document creation and editing
- **Excel**: Workbook and data management

### 🔬 Research & AI
- **Deep Research**: Multi-source information gathering
- **LLM Responses**: Response management and storage
- **Citation Management**: Academic standards

### 💰 Financial
- **Alpaca Trading**: Stock market integration
- **Portfolio Management**: Real-time data

### 🏠 Smart Home
- **Home Assistant**: IoT device management
- **Automation**: Event handling and control

### 🗺️ Maps & Geospatial
- **OpenStreetMap**: Geographic data processing
- **Location Services**: Bounding box queries

### 🧠 Memory Management
- **Knowledge Base**: Advanced memory storage
- **Relationship Tracking**: Graph traversal

## 🚀 Server Management

### Start Server
```bash
cd /home/l3o/git/l3ocifer/mcp-modules-rust
./start-mcp-server.sh
```

### Stop Server
Press `Ctrl+C` in the terminal running the server

### Restart Server
```bash
# Stop with Ctrl+C, then:
./start-mcp-server.sh
```

## 🔧 Configuration

### Environment Variables
The server uses these key configurations:
- **Host**: `0.0.0.0` (all interfaces)
- **Port**: `8888` (no conflicts with your homelab)
- **Database**: Uses your existing PostgreSQL (`neon-postgres-leopaska`)
- **Redis**: Uses your existing Redis (`redis-nd-leopaska`)

### Database Setup
If you need to create the MCP database:
```sql
-- Connect to your PostgreSQL instance
CREATE DATABASE mcp_db;
GRANT ALL PRIVILEGES ON DATABASE mcp_db TO postgres;
```

## 🐛 Troubleshooting

### Server Not Starting
1. Check if port 8888 is free: `netstat -tlnp | grep :8888`
2. Verify binary exists: `ls -la target/release/devops-mcp`
3. Check logs in the terminal where server is running

### Cursor Connection Issues
1. Verify server is running: `curl http://localhost:8888/health`
2. Check Cursor MCP settings are correct
3. Try refreshing Cursor's MCP server list
4. Check Cursor console for error messages

### Database Connection Issues
1. Verify PostgreSQL is running: `docker ps | grep postgres`
2. Test connection: `psql -h localhost -p 5432 -U postgres -d mcp_db`
3. Create database if it doesn't exist

## 📚 Usage Examples

Once connected in Cursor, you can use prompts like:

- "Create a PowerPoint presentation about our project status"
- "Query our PostgreSQL database for user statistics"
- "Deploy a new container to our Kubernetes cluster"
- "Search for recent research papers on AI optimization"
- "Check our Home Assistant devices and turn off lights"
- "Analyze trading opportunities with our Alpaca account"

The MCP server will automatically handle the complex integrations and provide results back to Cursor!

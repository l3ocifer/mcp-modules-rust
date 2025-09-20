#!/bin/bash
# Homelab Setup Script for Ubuntu Desktop
# This script sets up the MCP server for network access in a homelab environment

set -e

echo "🏠 MCP Modules Homelab Setup"
echo "============================"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Get the local IP address
get_local_ip() {
    if command_exists ip; then
        ip route get 1.1.1.1 | awk '{print $7}' | head -1
    elif command_exists hostname; then
        hostname -I | awk '{print $1}'
    else
        echo "127.0.0.1"
    fi
}

LOCAL_IP=$(get_local_ip)
echo "🔍 Detected local IP: $LOCAL_IP"

# Update system packages
echo ""
echo "📦 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install required packages
echo ""
echo "🔧 Installing required packages..."
sudo apt install -y \
    docker.io \
    docker-compose-plugin \
    ufw \
    nginx \
    certbot \
    python3-certbot-nginx

# Add user to docker group
echo ""
echo "👥 Adding user to docker group..."
sudo usermod -aG docker $USER

# Configure firewall
echo ""
echo "🔥 Configuring firewall..."
sudo ufw --force reset
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow ssh

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow MCP services
sudo ufw allow 8080/tcp   # MCP Server
sudo ufw allow 3000/tcp   # Grafana
sudo ufw allow 9090/tcp   # Prometheus
sudo ufw allow 16686/tcp  # Jaeger

# Allow database access from local network (optional)
sudo ufw allow from 192.168.0.0/16 to any port 5432   # PostgreSQL
sudo ufw allow from 192.168.0.0/16 to any port 27017  # MongoDB
sudo ufw allow from 192.168.0.0/16 to any port 6379   # Redis

# Enable firewall
sudo ufw --force enable

# Create data directories
echo ""
echo "📁 Creating data directories..."
mkdir -p data/{mcp,postgres-backups,mongo-backups,letsencrypt}
mkdir -p monitoring/grafana-dashboards

# Create Redis configuration
echo ""
echo "⚙️  Creating Redis configuration..."
cat > data/redis.conf << EOF
# Redis configuration for homelab
bind 0.0.0.0
port 6379
requirepass mcp_redis_password
maxmemory 1gb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
appendonly yes
appendfsync everysec
EOF

# Update Docker Compose with local IP
echo ""
echo "🔄 Updating Docker Compose configuration..."
sed -i "s/YOUR_UBUNTU_IP/$LOCAL_IP/g" docker-compose.homelab.yml

# Create environment file for homelab
echo ""
echo "📝 Creating homelab environment file..."
cat > .env.homelab << EOF
# Homelab Environment Configuration
LOCAL_IP=$LOCAL_IP
DOMAIN=${LOCAL_IP}.nip.io

# Database Configuration
DATABASE_URL=postgresql://mcp:mcp_password@${LOCAL_IP}:5432/mcp_db
MONGODB_URI=mongodb://mcp:mcp_password@${LOCAL_IP}:27017/mcp?authSource=admin
REDIS_URL=redis://:mcp_redis_password@${LOCAL_IP}:6379

# Monitoring
PROMETHEUS_URL=http://${LOCAL_IP}:9090
GRAFANA_URL=http://${LOCAL_IP}:3000

# Tracing
OTEL_EXPORTER_OTLP_ENDPOINT=http://${LOCAL_IP}:4317
OTEL_SERVICE_NAME=mcp-modules-rust

# Logging
RUST_LOG=devops_mcp=info,tower_http=debug

# MCP Configuration
MCP_TRANSPORT=http
MCP_HTTP_HOST=0.0.0.0
MCP_HTTP_PORT=8080
EOF

# Create startup script
echo ""
echo "🚀 Creating startup script..."
cat > start-homelab.sh << 'EOF'
#!/bin/bash
# Start MCP Homelab Services

set -e

echo "🏠 Starting MCP Homelab Services..."
echo "================================="

# Load environment
source .env.homelab

# Start services
docker compose -f docker-compose.homelab.yml up -d

echo ""
echo "⏳ Waiting for services to start..."
sleep 15

echo ""
echo "🎉 MCP Homelab is ready!"
echo ""
echo "📍 Access URLs:"
echo "  - MCP Server:    http://$LOCAL_IP:8080"
echo "  - Health Check:  http://$LOCAL_IP:8080/health"
echo "  - Grafana:       http://$LOCAL_IP:3000 (admin/admin)"
echo "  - Prometheus:    http://$LOCAL_IP:9090"
echo "  - Jaeger:        http://$LOCAL_IP:16686"
echo "  - Traefik:       http://$LOCAL_IP:8081"
echo ""
echo "🔗 From MacBook, use: http://$LOCAL_IP:8080"
echo ""
echo "📊 Service Status:"
docker compose -f docker-compose.homelab.yml ps
EOF

chmod +x start-homelab.sh

# Create stop script
cat > stop-homelab.sh << 'EOF'
#!/bin/bash
# Stop MCP Homelab Services

echo "🛑 Stopping MCP Homelab Services..."
docker compose -f docker-compose.homelab.yml down

echo "✅ Services stopped."
EOF

chmod +x stop-homelab.sh

# Create MacBook client configuration
echo ""
echo "💻 Creating MacBook client configuration..."
cat > macbook-client-config.md << EOF
# MacBook Client Configuration

## Environment Variables for MacBook
Add these to your \`~/.zshrc\` or \`~/.bashrc\`:

\`\`\`bash
# MCP Homelab Configuration
export MCP_SERVER_URL="http://$LOCAL_IP:8080"
export MCP_GRAFANA_URL="http://$LOCAL_IP:3000"
export MCP_PROMETHEUS_URL="http://$LOCAL_IP:9090"
export MCP_JAEGER_URL="http://$LOCAL_IP:16686"
\`\`\`

## Claude Code Configuration
Create \`~/.claude/mcp-homelab.json\`:

\`\`\`json
{
  "transport": {
    "type": "http",
    "url": "http://$LOCAL_IP:8080"
  },
  "timeout": 30000,
  "retries": 3
}
\`\`\`

## Testing Connection from MacBook
\`\`\`bash
# Test basic connectivity
curl http://$LOCAL_IP:8080/health

# Test with client
cargo run --example simple_client
\`\`\`

## SSH Tunnel (if needed for security)
\`\`\`bash
# Create SSH tunnel to Ubuntu desktop
ssh -L 8080:localhost:8080 -L 3000:localhost:3000 user@$LOCAL_IP

# Then access via localhost:8080
\`\`\`
EOF

echo ""
echo "✅ Homelab setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Reboot or log out/in to apply docker group membership"
echo "2. Run: ./start-homelab.sh"
echo "3. Configure your router to allow port access if needed"
echo "4. Copy macbook-client-config.md to your MacBook"
echo ""
echo "🔧 Manual configuration needed:"
echo "- Update YOUR_DOMAIN in docker-compose.homelab.yml if using a domain"
echo "- Configure your router firewall if accessing from outside LAN"
echo "- Set up dynamic DNS if IP changes frequently"
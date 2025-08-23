#!/bin/bash
# Coolify Homelab Deployment Script
# Deploy MCP to Ubuntu Desktop + Raspberry Pi Cluster

set -e

# Configuration
UBUNTU_IP="192.168.1.100"  # Update with your Ubuntu Desktop IP
PI_IPS=("192.168.1.101" "192.168.1.102" "192.168.1.103")  # Update with your Pi IPs
COOLIFY_API_TOKEN=""  # Set your Coolify API token
DOCKER_REGISTRY="your-registry.com"  # Update with your registry

echo "🏠 MCP Coolify Homelab Deployment"
echo "=================================="
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to wait for service
wait_for_service() {
    local url=$1
    local name=$2
    local max_attempts=30
    local attempt=0
    
    echo "⏳ Waiting for $name at $url..."
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo "✅ $name is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 2
    done
    echo "❌ $name failed to start"
    return 1
}

# Phase 1: Prepare Environment
echo "📋 Phase 1: Environment Preparation"
echo "-----------------------------------"

# Update IPs in configuration files
echo "🔄 Updating configuration files with network IPs..."
sed -i.bak "s/YOUR_UBUNTU_IP/$UBUNTU_IP/g" coolify-docker-compose.yml
sed -i.bak "s/YOUR_UBUNTU_IP/$UBUNTU_IP/g" monitoring/prometheus-homelab.yml

# Create data directories
echo "📁 Creating data directories..."
mkdir -p data/{mcp,postgres,mongodb,redis,prometheus,grafana,elasticsearch}
mkdir -p backups/{postgres,mongodb}
mkdir -p config
mkdir -p monitoring/{grafana/dashboards,grafana/dashboard-configs,rules}

# Create necessary config files
echo "⚙️  Creating configuration files..."

# Redis config
cat > config/redis.conf << EOF
bind 0.0.0.0
port 6379
requirepass mcp_redis_secure
maxmemory 2gb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
appendonly yes
appendfsync everysec
tcp-keepalive 300
timeout 300
EOF

# Postgres config
cat > config/postgres.conf << EOF
# PostgreSQL configuration for homelab
max_connections = 200
shared_buffers = 2GB
effective_cache_size = 6GB
maintenance_work_mem = 512MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 10MB
min_wal_size = 1GB
max_wal_size = 4GB
max_worker_processes = 8
max_parallel_workers_per_gather = 4
max_parallel_workers = 8
max_parallel_maintenance_workers = 4
EOF

# Grafana datasources
mkdir -p monitoring/grafana
cat > monitoring/grafana/datasources.yml << EOF
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
  - name: Jaeger
    type: jaeger
    access: proxy
    url: http://jaeger:16686
    editable: true
EOF

echo "✅ Environment preparation complete"

# Phase 2: Deploy Ubuntu Desktop Services
echo ""
echo "🖥️  Phase 2: Ubuntu Desktop Deployment"
echo "--------------------------------------"

echo "🚀 Starting Ubuntu Desktop services..."
docker compose -f coolify-docker-compose.yml up -d

# Wait for services to be ready
wait_for_service "http://$UBUNTU_IP:5432" "PostgreSQL"
wait_for_service "http://$UBUNTU_IP:27017" "MongoDB"
wait_for_service "http://$UBUNTU_IP:6379" "Redis"
wait_for_service "http://$UBUNTU_IP:9090" "Prometheus"
wait_for_service "http://$UBUNTU_IP:3000" "Grafana"
wait_for_service "http://$UBUNTU_IP:8081/health" "MCP Primary Server"

echo "✅ Ubuntu Desktop services are running"

# Phase 3: Build and Push MCP Image
echo ""
echo "📦 Phase 3: Build and Push MCP Image"
echo "------------------------------------"

if [ -n "$DOCKER_REGISTRY" ] && [ "$DOCKER_REGISTRY" != "your-registry.com" ]; then
    echo "🔨 Building MCP image..."
    docker build -t mcp-modules-rust:latest .
    docker tag mcp-modules-rust:latest $DOCKER_REGISTRY/mcp-modules-rust:latest
    
    echo "📤 Pushing to registry..."
    docker push $DOCKER_REGISTRY/mcp-modules-rust:latest
    echo "✅ Image pushed to registry"
else
    echo "⚠️  Skipping registry push - using local images"
    docker build -t mcp-modules-rust:latest .
fi

# Phase 4: Deploy to Raspberry Pis (Manual Coolify Configuration)
echo ""
echo "🍓 Phase 4: Raspberry Pi Deployment Instructions"
echo "------------------------------------------------"

echo "📝 Manual Coolify configuration required:"
echo ""
echo "1. Access Coolify at: http://$UBUNTU_IP:8000"
echo "2. Add each Pi as a server:"
for ip in "${PI_IPS[@]}"; do
    echo "   - Pi at $ip"
done
echo ""
echo "3. Create MCP replica applications on Pis with these settings:"
echo "   - Image: $DOCKER_REGISTRY/mcp-modules-rust:latest"
echo "   - Port: 8080"
echo "   - Environment Variables:"
echo "     MCP_HTTP_HOST=0.0.0.0"
echo "     MCP_HTTP_PORT=8080"
echo "     DATABASE_URL=postgresql://mcp:mcp_secure_password@$UBUNTU_IP:5432/mcp_db"
echo "     MONGODB_URI=mongodb://mcp:mcp_secure_password@$UBUNTU_IP:27017/mcp"
echo "     REDIS_URL=redis://:mcp_redis_secure@$UBUNTU_IP:6379"
echo "     RUST_LOG=devops_mcp=info"
echo "     NODE_ROLE=replica"
echo ""
echo "4. Configure load balancer to include Pi IPs in haproxy.cfg"

# Phase 5: Update Load Balancer Configuration
echo ""
echo "⚖️  Phase 5: Load Balancer Configuration"
echo "----------------------------------------"

echo "📝 Updating HAProxy configuration..."
# Backup original config
cp config/haproxy.cfg config/haproxy.cfg.bak

# Add Pi servers to HAProxy config
for i in "${!PI_IPS[@]}"; do
    pi_ip="${PI_IPS[$i]}"
    pi_num=$((i + 1))
    echo "    server pi$pi_num $pi_ip:8080 check weight 1 maxconn 200" >> config/haproxy.cfg
done

echo "🔄 Reloading HAProxy..."
docker compose -f coolify-docker-compose.yml restart haproxy

echo "✅ Load balancer updated"

# Phase 6: Final Testing
echo ""
echo "🧪 Phase 6: Testing Deployment"
echo "------------------------------"

echo "🔍 Testing services..."

# Test main MCP server
if curl -s "http://$UBUNTU_IP:8081/health" > /dev/null; then
    echo "✅ MCP Primary Server: healthy"
else
    echo "❌ MCP Primary Server: unhealthy"
fi

# Test load balancer
if curl -s "http://$UBUNTU_IP:8080/health" > /dev/null; then
    echo "✅ Load Balancer: healthy"
else
    echo "❌ Load Balancer: unhealthy"
fi

# Test monitoring
if curl -s "http://$UBUNTU_IP:9090/-/healthy" > /dev/null; then
    echo "✅ Prometheus: healthy"
else
    echo "❌ Prometheus: unhealthy"
fi

if curl -s "http://$UBUNTU_IP:3000/api/health" > /dev/null; then
    echo "✅ Grafana: healthy"
else
    echo "❌ Grafana: unhealthy"
fi

# Test Pi replicas
echo ""
echo "🍓 Testing Pi replicas..."
for i in "${!PI_IPS[@]}"; do
    pi_ip="${PI_IPS[$i]}"
    pi_num=$((i + 1))
    if curl -s "http://$pi_ip:8080/health" > /dev/null; then
        echo "✅ Pi $pi_num ($pi_ip): healthy"
    else
        echo "⚠️  Pi $pi_num ($pi_ip): not responding (configure via Coolify)"
    fi
done

# Final Summary
echo ""
echo "🎉 Deployment Summary"
echo "===================="
echo ""
echo "📍 Access URLs:"
echo "  - MCP Load Balancer: http://$UBUNTU_IP:8080"
echo "  - MCP Primary:       http://$UBUNTU_IP:8081"
echo "  - Coolify UI:        http://$UBUNTU_IP:8000"
echo "  - Grafana:           http://$UBUNTU_IP:3000 (admin/admin)"
echo "  - Prometheus:        http://$UBUNTU_IP:9090"
echo "  - Jaeger:            http://$UBUNTU_IP:16686"
echo "  - HAProxy Stats:     http://$UBUNTU_IP:8404/stats"
echo ""
echo "🔗 From MacBook:"
echo "  export MCP_SERVER_URL=\"http://$UBUNTU_IP:8080\""
echo "  curl \$MCP_SERVER_URL/health"
echo ""
echo "📋 Next Steps:"
echo "1. Complete Pi deployments via Coolify UI"
echo "2. Configure monitoring alerts in Grafana"
echo "3. Set up backup scripts for databases"
echo "4. Configure firewall rules if needed"
echo ""
echo "✅ Ubuntu Desktop deployment complete!"
# Coolify Homelab Deployment Guide

## Architecture Overview

**Ubuntu Desktop (Control & Heavy Services)**
- Coolify instance (management UI)
- PostgreSQL, MongoDB (databases need stable storage)
- Prometheus, Grafana (monitoring aggregation)
- Main MCP server instance

**Raspberry Pi Cluster (Distributed Light Services)**
- MCP server replicas (load balanced)
- Redis instances (distributed cache)
- Edge processing services
- Log aggregation nodes

## Setup Strategy

### Phase 1: Ubuntu Desktop Setup

#### 1. Install Coolify on Ubuntu Desktop
```bash
# Install Coolify
curl -fsSL https://cdn.coollabs.io/coolify/install.sh | bash

# Access Coolify at http://ubuntu-desktop-ip:8000
# Complete initial setup
```

#### 2. Configure Ubuntu Desktop Services
Create these in Coolify as "Services":

**PostgreSQL Service**
```yaml
# In Coolify: New Service -> PostgreSQL
Environment Variables:
  POSTGRES_DB: mcp_db
  POSTGRES_USER: mcp
  POSTGRES_PASSWORD: mcp_secure_password
  
Volumes:
  - /data/postgres:/var/lib/postgresql/data

Ports:
  - 5432:5432

Labels:
  coolify.managed: true
  homelab.service: database
```

**MongoDB Service**
```yaml
# In Coolify: New Service -> MongoDB
Environment Variables:
  MONGO_INITDB_ROOT_USERNAME: mcp
  MONGO_INITDB_ROOT_PASSWORD: mcp_secure_password
  MONGO_INITDB_DATABASE: mcp

Volumes:
  - /data/mongodb:/data/db

Ports:
  - 27017:27017
```

**Prometheus Service**
```yaml
# In Coolify: New Application -> Docker Compose
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.external-url=http://YOUR_UBUNTU_IP:9090'
      - '--storage.tsdb.retention.time=30d'
    
volumes:
  prometheus_data:
```

### Phase 2: Raspberry Pi Cluster Setup

#### 1. Install Docker on Each Pi
```bash
# Run on each Pi
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Install Coolify agent (lightweight)
curl -fsSL https://cdn.coollabs.io/coolify/install.sh | bash -s -- --agent

# Register with main Coolify instance
# Get registration command from Coolify UI -> Servers -> Add Server
```

#### 2. Pi Resource Allocation Strategy

**High-Memory Pis (512GB storage)**
- Primary MCP server replicas
- Redis cache clusters
- Log aggregation

**Standard Pis (64-256GB storage)**
- Edge processing services
- Monitoring agents
- Backup services

### Phase 3: Application Deployments

## Coolify Application Configurations

### 1. Main MCP Server (Ubuntu Desktop)
```dockerfile
# Dockerfile for Coolify deployment
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features database

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/devops-mcp /usr/local/bin/
EXPOSE 8080

CMD ["devops-mcp"]
```

**Coolify Configuration:**
```yaml
# Application Type: Docker
Build Pack: Dockerfile
Git Repository: your-mcp-repo

Environment Variables:
  MCP_HTTP_HOST: 0.0.0.0
  MCP_HTTP_PORT: 8080
  DATABASE_URL: postgresql://mcp:mcp_secure_password@ubuntu-ip:5432/mcp_db
  MONGODB_URI: mongodb://mcp:mcp_secure_password@ubuntu-ip:27017/mcp?authSource=admin
  PROMETHEUS_URL: http://ubuntu-ip:9090
  RUST_LOG: devops_mcp=info

Port Mapping:
  8080:8080

Health Check:
  Path: /health
  Port: 8080
  Interval: 30s

Resources:
  Memory: 2GB
  CPU: 2 cores
```

### 2. MCP Replica for Pi Cluster
```yaml
# Deploy to high-memory Pis
Application Type: Docker
Image: your-registry/mcp-modules-rust:latest

Environment Variables:
  MCP_HTTP_HOST: 0.0.0.0
  MCP_HTTP_PORT: 8080
  DATABASE_URL: postgresql://mcp:mcp_secure_password@ubuntu-ip:5432/mcp_db
  MONGODB_URI: mongodb://mcp:mcp_secure_password@ubuntu-ip:27017/mcp?authSource=admin
  REDIS_URL: redis://pi-redis-ip:6379
  RUST_LOG: devops_mcp=info

Port Mapping:
  8080:8080

Resources:
  Memory: 512MB  # Fit within Pi constraints
  CPU: 0.5 cores

Deployment Strategy:
  - Deploy to 3 high-memory Pis
  - Use Coolify's built-in load balancing
```

### 3. Redis Cluster on Pis
```yaml
# Deploy Redis to multiple Pis for HA
Application Type: Docker
Image: redis:7-alpine

Environment Variables:
  REDIS_PASSWORD: redis_secure_password

Command: 
  - redis-server
  - --requirepass redis_secure_password
  - --maxmemory 256mb
  - --maxmemory-policy allkeys-lru

Port Mapping:
  6379:6379

Volumes:
  - redis_data:/data

Resources:
  Memory: 300MB
  CPU: 0.3 cores
```

## Load Balancing & High Availability

### 1. Coolify Load Balancer Configuration
```yaml
# In Coolify: Create Load Balancer
Name: mcp-cluster-lb
Type: HTTP
Algorithm: round_robin

Upstreams:
  - ubuntu-desktop-ip:8080 (weight: 3)  # More powerful
  - pi1-ip:8080 (weight: 1)
  - pi2-ip:8080 (weight: 1)
  - pi3-ip:8080 (weight: 1)

Health Checks:
  Path: /health
  Interval: 10s
  Timeout: 5s
  Retries: 3

SSL: false  # Internal network
```

### 2. HAProxy Configuration (Alternative)
```haproxy
# /etc/haproxy/haproxy.cfg on Ubuntu Desktop
global
    daemon

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend mcp_frontend
    bind *:8080
    default_backend mcp_backend

backend mcp_backend
    balance roundrobin
    option httpchk GET /health
    server ubuntu ubuntu-ip:8081 check weight 3
    server pi1 pi1-ip:8080 check weight 1
    server pi2 pi2-ip:8080 check weight 1
    server pi3 pi3-ip:8080 check weight 1
```

## Monitoring Setup

### 1. Prometheus Configuration for Pi Monitoring
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'mcp-ubuntu'
    static_configs:
      - targets: ['ubuntu-ip:8081']
    metrics_path: '/metrics'

  - job_name: 'mcp-pis'
    static_configs:
      - targets: 
        - 'pi1-ip:8080'
        - 'pi2-ip:8080'
        - 'pi3-ip:8080'

  - job_name: 'node-exporters'
    static_configs:
      - targets:
        - 'ubuntu-ip:9100'
        - 'pi1-ip:9100'
        - 'pi2-ip:9100'
        - 'pi3-ip:9100'
```

### 2. Node Exporter on Each Pi
```yaml
# Coolify Application: Node Exporter
Application Type: Docker
Image: prom/node-exporter:latest

Command:
  - --path.rootfs=/host

Volumes:
  - /:/host:ro,rslave

Port Mapping:
  - 9100:9100

Network Mode: host

Resources:
  Memory: 64MB
  CPU: 0.1 cores
```

## Deployment Scripts

### 1. Pi Cluster Deployment Script
```bash
#!/bin/bash
# deploy-to-pis.sh

PI_IPS=("192.168.1.101" "192.168.1.102" "192.168.1.103")
UBUNTU_IP="192.168.1.100"

echo "🍓 Deploying MCP to Raspberry Pi Cluster"
echo "========================================"

# Build and push image
echo "📦 Building MCP image..."
docker build -t mcp-modules-rust:latest .
docker tag mcp-modules-rust:latest your-registry/mcp-modules-rust:latest
docker push your-registry/mcp-modules-rust:latest

# Deploy via Coolify API
for ip in "${PI_IPS[@]}"; do
    echo "🚀 Deploying to Pi at $ip..."
    
    # Use Coolify API to deploy
    curl -X POST "http://$UBUNTU_IP:8000/api/v1/deploy" \
        -H "Authorization: Bearer $COOLIFY_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"server\": \"$ip\",
            \"application\": \"mcp-replica\",
            \"image\": \"your-registry/mcp-modules-rust:latest\"
        }"
done

echo "✅ Deployment complete!"
```

### 2. Health Check Script
```bash
#!/bin/bash
# check-cluster-health.sh

UBUNTU_IP="192.168.1.100"
PI_IPS=("192.168.1.101" "192.168.1.102" "192.168.1.103")

echo "🏥 Checking MCP Cluster Health"
echo "=============================="

# Check main server
echo "📊 Ubuntu Desktop:"
curl -s "$UBUNTU_IP:8081/health" && echo " ✅ Healthy" || echo " ❌ Down"

# Check Pi replicas
for ip in "${PI_IPS[@]}"; do
    echo "🍓 Pi at $ip:"
    curl -s "$ip:8080/health" && echo " ✅ Healthy" || echo " ❌ Down"
done

# Check load balancer
echo "⚖️  Load Balancer:"
curl -s "$UBUNTU_IP:8080/health" && echo " ✅ Healthy" || echo " ❌ Down"
```

## Benefits of Coolify Approach

1. **Lightweight**: Much less overhead than K8s
2. **Pi-Friendly**: Designed for resource-constrained environments
3. **Easy Management**: Web UI for all deployments
4. **Auto-Healing**: Automatic container restart on failure
5. **Load Balancing**: Built-in load balancer
6. **Monitoring**: Integrated with Prometheus/Grafana
7. **Git Integration**: Auto-deploy from Git repos
8. **Resource Limits**: Prevents Pi resource exhaustion

## MacBook Access

From your MacBook, you'll access:
- **Main MCP**: `http://ubuntu-ip:8080` (load balanced)
- **Coolify UI**: `http://ubuntu-ip:8000` (management)
- **Grafana**: `http://ubuntu-ip:3000` (monitoring)
- **Direct Pi Access**: `http://pi-ip:8080` (if needed)

This setup gives you the best of both worlds: powerful services on Ubuntu Desktop with distributed, resilient replicas on your Pi cluster!
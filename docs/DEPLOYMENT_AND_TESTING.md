# MCP Modules Rust - Deployment and Testing Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Building the Project](#building-the-project)
3. [Configuration](#configuration)
4. [Deployment Options](#deployment-options)
5. [Testing Strategy](#testing-strategy)
6. [Quick Start Examples](#quick-start-examples)

## Prerequisites

### System Requirements
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- 4GB RAM minimum
- 10GB free disk space

### Optional Requirements (for specific modules)
- Docker/Podman (for container module)
- Kubernetes cluster access (for K8s module)
- PostgreSQL 14+ (for persistence)
- MongoDB 5+ (for MongoDB features)

## Building the Project

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-modules-rust.git
cd mcp-modules-rust

# Build with all features
cargo build --release --all-features

# Or build with specific features only
cargo build --release --features "database,infrastructure"

# Run tests
cargo test --all-features
```

### 2. Create Optimized Build

```bash
# Build with maximum optimization
RUSTFLAGS="-C target-cpu=native" cargo build --release --all-features

# Strip debug symbols for smaller binary
strip target/release/devops-mcp
```

## Configuration

### 1. Environment Variables

Create a `.env` file in the project root:

```env
# Database Configuration
DATABASE_URL=postgresql://user:password@localhost/mcp_db
MONGODB_URI=mongodb://localhost:27017/mcp

# Cloud Provider Credentials
AWS_ACCESS_KEY_ID=your-key
AWS_SECRET_ACCESS_KEY=your-secret
AWS_REGION=us-east-1

CLOUDFLARE_API_TOKEN=your-cf-token
CLOUDFLARE_ACCOUNT_ID=your-account-id

# Monitoring
PROMETHEUS_URL=http://localhost:9090
GRAFANA_URL=http://localhost:3000
GRAFANA_API_KEY=your-grafana-key

# Security
MCP_ENCRYPTION_KEY=your-32-byte-base64-key
MCP_RATE_LIMIT=100
MCP_RATE_LIMIT_WINDOW=60

# MCP Transport
MCP_TRANSPORT=stdio  # or http, websocket
MCP_HTTP_PORT=8080
MCP_WS_PORT=8081
```

### 2. Configuration File

Create `config.json`:

```json
{
  "transport_type": "http",
  "http_config": {
    "host": "0.0.0.0",
    "port": 8080,
    "tls": {
      "enabled": false,
      "cert_path": "./certs/server.crt",
      "key_path": "./certs/server.key"
    }
  },
  "modules": {
    "database": {
      "enabled": true,
      "providers": ["postgresql", "mongodb"]
    },
    "infrastructure": {
      "enabled": true,
      "providers": ["kubernetes", "docker"]
    },
    "memory": {
      "enabled": true,
      "persistence": "postgresql",
      "cache_size": 10000
    }
  },
  "security": {
    "input_validation": true,
    "rate_limiting": true,
    "audit_logging": true
  }
}
```

## Deployment Options

### Option 1: Standalone Binary

```bash
# Run with environment variables
./target/release/devops-mcp

# Run with config file
./target/release/devops-mcp --config config.json

# Run with verbose logging
RUST_LOG=devops_mcp=debug ./target/release/devops-mcp
```

### Option 2: Docker Container

Create `Dockerfile`:

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --all-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/devops-mcp /usr/local/bin/
COPY --from=builder /app/config.json /etc/mcp/

EXPOSE 8080 8081
CMD ["devops-mcp", "--config", "/etc/mcp/config.json"]
```

Build and run:

```bash
# Build Docker image
docker build -t mcp-modules-rust:latest .

# Run container
docker run -d \
  --name mcp-server \
  -p 8080:8080 \
  -p 8081:8081 \
  -v $(pwd)/config.json:/etc/mcp/config.json \
  --env-file .env \
  mcp-modules-rust:latest
```

### Option 3: Kubernetes Deployment

Create `k8s-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
  namespace: mcp-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mcp-server
  template:
    metadata:
      labels:
        app: mcp-server
    spec:
      containers:
      - name: mcp-server
        image: mcp-modules-rust:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 8081
          name: websocket
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: mcp-secrets
              key: database-url
        - name: MCP_TRANSPORT
          value: "http"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: mcp-server
  namespace: mcp-system
spec:
  selector:
    app: mcp-server
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: websocket
    port: 8081
    targetPort: 8081
  type: LoadBalancer
```

Deploy:

```bash
kubectl create namespace mcp-system
kubectl create secret generic mcp-secrets --from-env-file=.env -n mcp-system
kubectl apply -f k8s-deployment.yaml
```

### Option 4: Systemd Service

Create `/etc/systemd/system/mcp-server.service`:

```ini
[Unit]
Description=MCP Modules Rust Server
After=network.target

[Service]
Type=simple
User=mcp
Group=mcp
WorkingDirectory=/opt/mcp
ExecStart=/opt/mcp/devops-mcp --config /opt/mcp/config.json
Restart=always
RestartSec=5
Environment="RUST_LOG=devops_mcp=info"
EnvironmentFile=/opt/mcp/.env

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/mcp

[Install]
WantedBy=multi-user.target
```

Install and start:

```bash
sudo useradd -r -s /bin/false mcp
sudo mkdir -p /opt/mcp /var/lib/mcp
sudo cp target/release/devops-mcp /opt/mcp/
sudo cp config.json .env /opt/mcp/
sudo chown -R mcp:mcp /opt/mcp /var/lib/mcp

sudo systemctl daemon-reload
sudo systemctl enable mcp-server
sudo systemctl start mcp-server
```

## Testing Strategy

### 1. Unit Tests

```bash
# Run all unit tests
cargo test --all-features

# Run specific module tests
cargo test --features database database::
cargo test --features infrastructure infrastructure::

# Run with verbose output
cargo test --all-features -- --nocapture
```

### 2. Integration Tests

Create `tests/integration_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use devops_mcp::client::MCPClient;
    use devops_mcp::config::Config;

    #[tokio::test]
    async fn test_full_workflow() {
        // Initialize client
        let config = Config::from_file("test-config.json").unwrap();
        let client = MCPClient::new(config);
        client.initialize().await.unwrap();

        // Test database operations
        let dbs = client.call_tool("list_databases", serde_json::json!({})).await.unwrap();
        assert!(dbs.is_array());

        // Test memory storage
        let store_result = client.call_tool("memory_store", serde_json::json!({
            "id": "test-1",
            "type": "test",
            "title": "Integration Test",
            "content": "Test content"
        })).await.unwrap();
        assert_eq!(store_result["status"], "success");

        // Test infrastructure
        let health = client.call_tool("infrastructure_health", serde_json::json!({})).await.unwrap();
        assert!(health["overall_health"].is_string());
    }
}
```

### 3. Performance Testing

Create `benches/benchmark.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use devops_mcp::memory::MemoryModule;

fn memory_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let memory_module = runtime.block_on(async {
        MemoryModule::new().await.unwrap()
    });

    c.bench_function("memory_store", |b| {
        b.iter(|| {
            runtime.block_on(async {
                memory_module.store_memory("bench", "test", "Benchmark", "Content").await
            })
        });
    });
}

criterion_group!(benches, memory_benchmark);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench --all-features
```

### 4. Load Testing

Using [k6](https://k6.io/):

```javascript
// load-test.js
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};

export default function() {
  let response = http.post('http://localhost:8080/rpc', JSON.stringify({
    jsonrpc: '2.0',
    method: 'tools/list',
    params: {},
    id: 1
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

Run load test:

```bash
k6 run load-test.js
```

## Quick Start Examples

### 1. Test Database Module

```bash
# Start PostgreSQL and MongoDB locally
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:15
docker run -d -p 27017:27017 mongo:6

# Set environment variables
export DATABASE_URL=postgresql://postgres:password@localhost/postgres
export MONGODB_URI=mongodb://localhost:27017

# Run the server
cargo run --features database

# In another terminal, test with curl
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "database/list",
    "params": {},
    "id": 1
  }'
```

### 2. Test Infrastructure Module

```bash
# Ensure Docker is running
docker info

# Run the server
cargo run --features infrastructure

# Test container operations
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "docker/list_containers",
    "params": {},
    "id": 1
  }'
```

### 3. Test Memory Module

```bash
# Run with in-memory storage
cargo run --features memory

# Store a memory
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "memory/store",
    "params": {
      "id": "test-memory-1",
      "type": "note",
      "title": "Test Note",
      "content": "This is a test note"
    },
    "id": 1
  }'

# Search memories
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "memory/search",
    "params": {
      "keyword": "test"
    },
    "id": 2
  }'
```

### 4. Health Check

```bash
# Check server health
curl http://localhost:8080/health

# Check readiness
curl http://localhost:8080/ready

# Get metrics (if monitoring enabled)
curl http://localhost:8080/metrics
```

## Monitoring and Observability

### 1. Enable Prometheus Metrics

```rust
// In your main.rs
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

### 2. Configure Grafana Dashboard

Import the dashboard from `monitoring/grafana-dashboard.json`:

```bash
# Copy to Grafana
curl -X POST http://localhost:3000/api/dashboards/db \
  -H "Authorization: Bearer $GRAFANA_API_KEY" \
  -H "Content-Type: application/json" \
  -d @monitoring/grafana-dashboard.json
```

### 3. Enable Tracing

Set environment variables:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_SERVICE_NAME=mcp-modules-rust
export RUST_LOG=devops_mcp=trace,tower_http=debug
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Check if the server is running: `ps aux | grep devops-mcp`
   - Check logs: `journalctl -u mcp-server -f`
   - Verify ports are not in use: `netstat -tlnp | grep 8080`

2. **Database Connection Failed**
   - Verify database is running
   - Check connection string format
   - Test connection: `psql $DATABASE_URL`

3. **Permission Denied**
   - Check file permissions: `ls -la /opt/mcp/`
   - Verify user has necessary permissions
   - Check SELinux/AppArmor if enabled

4. **High Memory Usage**
   - Adjust cache sizes in configuration
   - Enable memory profiling: `RUST_LOG=devops_mcp::memory=trace`
   - Use `heaptrack` or `valgrind` for detailed analysis

### Debug Mode

Run with maximum debugging:

```bash
RUST_LOG=devops_mcp=trace,hyper=debug,tokio=debug \
RUST_BACKTRACE=full \
./target/release/devops-mcp --config config.json 2>&1 | tee debug.log
```

## Production Checklist

- [ ] Configure TLS certificates
- [ ] Set up database backups
- [ ] Configure monitoring and alerts
- [ ] Set up log aggregation
- [ ] Configure rate limiting
- [ ] Enable audit logging
- [ ] Set up health check monitoring
- [ ] Configure auto-scaling (if using K8s)
- [ ] Set up CI/CD pipeline
- [ ] Document runbooks for common operations
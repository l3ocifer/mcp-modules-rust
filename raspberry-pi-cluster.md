# Raspberry Pi Cluster Integration

## Architecture Overview

The 30-core Raspberry Pi cluster can serve as:
1. **Kubernetes Worker Nodes** - Distributed computing
2. **Edge Processing Nodes** - Local AI inference 
3. **High Availability Replicas** - Backup services
4. **Distributed Storage** - Cluster file system

## Setup Options

### Option 1: Kubernetes Cluster with Ubuntu Desktop as Master

#### Ubuntu Desktop (Master Node)
```bash
# Install kubeadm, kubectl, kubelet
curl -s https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -
echo "deb https://apt.kubernetes.io/ kubernetes-xenial main" | sudo tee /etc/apt/sources.list.d/kubernetes.list
sudo apt update
sudo apt install -y kubelet kubeadm kubectl

# Initialize cluster
sudo kubeadm init --pod-network-cidr=10.244.0.0/16 --apiserver-advertise-address=YOUR_UBUNTU_IP

# Setup kubectl
mkdir -p $HOME/.kube
sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config
sudo chown $(id -u):$(id -g) $HOME/.kube/config

# Install Flannel network
kubectl apply -f https://raw.githubusercontent.com/coreos/flannel/master/Documentation/kube-flannel.yml
```

#### Raspberry Pi Nodes
```bash
# On each Pi (flash with Ubuntu Server 20.04+ ARM64)
# Install Docker and kubeadm
curl -sSL get.docker.com | sh
sudo usermod -aG docker $USER

# Add Kubernetes repo and install
curl -s https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -
echo "deb https://apt.kubernetes.io/ kubernetes-xenial main" | sudo tee /etc/apt/sources.list.d/kubernetes.list
sudo apt update
sudo apt install -y kubelet kubeadm kubectl

# Join cluster (get token from master)
sudo kubeadm join YOUR_UBUNTU_IP:6443 --token TOKEN --discovery-token-ca-cert-hash HASH
```

### Option 2: Docker Swarm (Simpler Alternative)

#### Ubuntu Desktop (Swarm Manager)
```bash
# Initialize swarm
docker swarm init --advertise-addr YOUR_UBUNTU_IP

# Get join token
docker swarm join-token worker
```

#### Raspberry Pi Nodes
```bash
# Join swarm (use token from manager)
docker swarm join --token TOKEN YOUR_UBUNTU_IP:2377
```

## MCP Deployment on Cluster

### Kubernetes Deployment
```yaml
# mcp-cluster-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
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
        env:
        - name: MCP_HTTP_HOST
          value: "0.0.0.0"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: mcp-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      nodeSelector:
        kubernetes.io/arch: arm64  # Deploy to Pi nodes
---
apiVersion: v1
kind: Service
metadata:
  name: mcp-service
spec:
  selector:
    app: mcp-server
  ports:
  - port: 8080
    targetPort: 8080
  type: LoadBalancer
```

### Docker Swarm Stack
```yaml
# docker-stack.yml
version: '3.8'
services:
  mcp-server:
    image: mcp-modules-rust:latest
    ports:
      - "8080:8080"
    environment:
      - MCP_HTTP_HOST=0.0.0.0
      - DATABASE_URL=postgresql://mcp:password@postgres:5432/mcp_db
    deploy:
      replicas: 3
      placement:
        constraints:
          - node.platform.arch == aarch64  # Deploy to Pi nodes
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M
    networks:
      - mcp-network

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: mcp_db
      POSTGRES_USER: mcp
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    deploy:
      placement:
        constraints:
          - node.hostname == ubuntu-desktop  # Keep on main server
    networks:
      - mcp-network

networks:
  mcp-network:
    driver: overlay

volumes:
  postgres_data:
```

## Storage Solutions

### Option 1: GlusterFS Distributed Storage
```bash
# Install on all nodes
sudo apt install -y glusterfs-server

# Create distributed volume
sudo gluster volume create mcp-storage replica 3 \
  pi1:/data/gluster \
  pi2:/data/gluster \
  pi3:/data/gluster \
  force

sudo gluster volume start mcp-storage
```

### Option 2: Longhorn Distributed Block Storage
```bash
# Install Longhorn on Kubernetes
kubectl apply -f https://raw.githubusercontent.com/longhorn/longhorn/v1.5.1/deploy/longhorn.yaml
```

## Monitoring and Observability

### Prometheus Node Exporters on Pi Cluster
```yaml
# node-exporter-daemonset.yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: node-exporter
spec:
  selector:
    matchLabels:
      app: node-exporter
  template:
    metadata:
      labels:
        app: node-exporter
    spec:
      containers:
      - name: node-exporter
        image: prom/node-exporter:latest
        ports:
        - containerPort: 9100
        volumeMounts:
        - name: proc
          mountPath: /host/proc
          readOnly: true
        - name: sys
          mountPath: /host/sys
          readOnly: true
      volumes:
      - name: proc
        hostPath:
          path: /proc
      - name: sys
        hostPath:
          path: /sys
      hostNetwork: true
      hostPID: true
```

## Resource Allocation Strategy

### Ubuntu Desktop (55GB RAM, GPU)
- **Primary Services**: MCP server, databases, monitoring
- **GPU Workloads**: AI/ML inference, video processing
- **Role**: Control plane, heavy compute

### Raspberry Pi Cluster (30 cores, 1GB each)
- **Distributed Services**: Replicated MCP instances
- **Edge Processing**: Local data processing
- **Storage**: Distributed file system
- **Monitoring**: Node metrics collection

## Network Configuration

### VLAN Setup (Recommended)
```bash
# Ubuntu Desktop: VLAN 10 (Services)
# Pi Cluster: VLAN 20 (Compute)
# MacBook: VLAN 10 (Development)

# Configure VLANs on switch/router
# Trunk ports for inter-VLAN communication
```

### Firewall Rules
```bash
# Allow cluster communication
sudo ufw allow from 192.168.10.0/24  # Services VLAN
sudo ufw allow from 192.168.20.0/24  # Compute VLAN

# Kubernetes ports
sudo ufw allow 6443/tcp    # API server
sudo ufw allow 2379:2380/tcp  # etcd
sudo ufw allow 10250/tcp   # kubelet
sudo ufw allow 10251/tcp   # kube-scheduler
sudo ufw allow 10252/tcp   # kube-controller
```

## Deployment Commands

### Deploy to Kubernetes
```bash
# Build and push image
docker build -t mcp-modules-rust:latest .
docker tag mcp-modules-rust:latest your-registry/mcp-modules-rust:latest
docker push your-registry/mcp-modules-rust:latest

# Deploy to cluster
kubectl apply -f mcp-cluster-deployment.yaml
kubectl get pods -o wide  # Check pod placement
```

### Deploy to Docker Swarm
```bash
# Deploy stack
docker stack deploy -c docker-stack.yml mcp

# Check services
docker service ls
docker service ps mcp_mcp-server
```

## Benefits of This Architecture

1. **High Availability**: Service replicas across multiple nodes
2. **Load Distribution**: Spread compute across 30 cores
3. **Edge Processing**: Local processing on Pi nodes
4. **Fault Tolerance**: Services continue if nodes fail
5. **Scalability**: Easy to add more Pi nodes
6. **Cost Effective**: Utilize existing Pi hardware
7. **Development Friendly**: MacBook connects seamlessly

## Monitoring Dashboard

Access cluster metrics at:
- **Grafana**: http://ubuntu-desktop-ip:3000
- **Prometheus**: http://ubuntu-desktop-ip:9090
- **Kubernetes Dashboard**: http://ubuntu-desktop-ip:8001/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/
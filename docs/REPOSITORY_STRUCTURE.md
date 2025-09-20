# Repository Structure

This document outlines the organized structure of the MCP Modules Rust repository.

## 📁 **Directory Structure**

```
mcp-modules-rust/
├── src/                          # 🦀 Rust source code
│   ├── main.rs                   # Main MCP server entry point
│   ├── lib.rs                    # Library exports
│   ├── client.rs                 # MCP client implementation
│   ├── server.rs                 # MCP server core
│   ├── config.rs                 # Configuration management
│   ├── error.rs                  # Error handling
│   ├── ai/                       # AI and LLM integrations
│   ├── analytics/                # Analytics tools (Superset, etc.)
│   ├── auth/                     # Authentication & OAuth
│   ├── cicd/                     # CI/CD pipeline tools
│   ├── cloud/                    # Cloud provider integrations (AWS, Azure, GCP)
│   ├── collaboration/            # Team collaboration tools
│   ├── creation/                 # Content creation tools
│   ├── database/                 # Database integrations (PostgreSQL, MongoDB, Supabase)
│   ├── development/              # Development tools (Flutter, etc.)
│   ├── finance/                  # Financial integrations (Alpaca, etc.)
│   ├── gaming/                   # Gaming platform integrations (Steam)
│   ├── government/               # Government APIs and grants
│   ├── infrastructure/           # Infrastructure management (Docker, K8s, Cloudflare)
│   ├── lifecycle/                # Application lifecycle management
│   ├── maps/                     # Mapping services (OpenStreetMap)
│   ├── memory/                   # Memory and persistence systems
│   ├── monitoring/               # System monitoring tools
│   ├── office/                   # Office automation (Excel, PowerPoint, Word)
│   ├── research/                 # Research and deep research tools
│   ├── security/                 # Security validation and tools
│   ├── smart_home/               # Smart home integrations (Home Assistant)
│   ├── tools/                    # General purpose tools
│   ├── transport/                # MCP transport implementations
│   └── web/                      # Web-related utilities
├── scripts/                      # 📜 Shell scripts and automation
│   ├── build-clean.sh            # Clean build script
│   ├── start-mcp-server.sh       # MCP server startup script
│   ├── test-local.sh             # Local testing script
│   ├── quick-start.sh            # Quick setup script
│   ├── homelab-setup.sh          # Homelab deployment script
│   └── coolify-deploy.sh         # Coolify deployment script
├── deployment/                   # 🚀 Deployment configurations
│   ├── Dockerfile                # Docker build configuration
│   ├── docker-compose.yml        # Basic Docker Compose
│   ├── docker-compose.homelab.yml # Homelab Docker Compose
│   ├── coolify-docker-compose.yml # Coolify deployment config
│   └── monitoring/               # Monitoring configurations
│       ├── prometheus.yml        # Prometheus config
│       ├── prometheus-homelab.yml # Homelab Prometheus config
│       └── grafana-datasources.yml # Grafana datasources
├── configs/                      # ⚙️ Configuration files
│   ├── cursor-mcp-config.json    # Cursor MCP configuration
│   └── haproxy.cfg               # HAProxy configuration
├── docs/                         # 📚 Documentation
│   ├── REPOSITORY_STRUCTURE.md   # This file
│   ├── SETUP_COMPLETE.md         # Setup completion guide
│   ├── GETTING_STARTED.md        # Getting started guide
│   ├── CURSOR_SETUP.md           # Cursor integration guide
│   ├── ARCHITECTURE.md           # Architecture documentation
│   ├── MODULE_GUIDE.md           # Module development guide
│   ├── SECURITY.md               # Security documentation
│   ├── PERFORMANCE_GUIDE.md      # Performance optimization
│   ├── DEPLOYMENT_AND_TESTING.md # Deployment guide
│   ├── PRODUCTION_READINESS_REPORT.md # Production readiness
│   ├── PRODUCTION_READINESS_UPDATE.md # Production updates
│   ├── FINAL_ASSESSMENT.md       # Final assessment
│   ├── coolify-homelab-setup.md  # Coolify setup guide
│   └── raspberry-pi-cluster.md   # Raspberry Pi cluster setup
├── examples/                     # 💡 Usage examples
│   ├── README.md                 # Examples documentation
│   ├── basic_usage.rs            # Basic usage examples
│   ├── office_automation.rs      # Office automation examples
│   └── simple_client.rs          # Simple client implementation
├── tests/                        # 🧪 Test files
│   └── integration_test.rs       # Integration tests
├── target/                       # 🎯 Build artifacts (generated)
│   └── release/
│       └── devops-mcp            # Compiled binary
├── Cargo.toml                    # 📦 Rust package configuration
├── Cargo.lock                    # 🔒 Dependency lock file
└── README.md                     # 📖 Main project README
```

## 🎯 **Quick Navigation**

### **For Users:**
- **Getting Started**: [`docs/GETTING_STARTED.md`](./GETTING_STARTED.md)
- **Cursor Setup**: [`docs/CURSOR_SETUP.md`](./CURSOR_SETUP.md)
- **Setup Complete**: [`docs/SETUP_COMPLETE.md`](./SETUP_COMPLETE.md)

### **For Developers:**
- **Architecture**: [`docs/ARCHITECTURE.md`](./ARCHITECTURE.md)
- **Module Guide**: [`docs/MODULE_GUIDE.md`](./MODULE_GUIDE.md)
- **Examples**: [`examples/`](../examples/)

### **For Deployment:**
- **Scripts**: [`scripts/`](../scripts/)
- **Docker**: [`deployment/`](../deployment/)
- **Configs**: [`configs/`](../configs/)

### **For Operations:**
- **Security**: [`docs/SECURITY.md`](./SECURITY.md)
- **Performance**: [`docs/PERFORMANCE_GUIDE.md`](./PERFORMANCE_GUIDE.md)
- **Monitoring**: [`deployment/monitoring/`](../deployment/monitoring/)

## 🚀 **Quick Commands**

```bash
# Build the project
./scripts/build-clean.sh

# Start MCP server
./scripts/start-mcp-server.sh

# Run tests
./scripts/test-local.sh

# Quick setup
./scripts/quick-start.sh

# Deploy to homelab
./scripts/homelab-setup.sh
```

## 🔧 **Configuration Files**

- **Cursor MCP**: [`configs/cursor-mcp-config.json`](../configs/cursor-mcp-config.json)
- **Docker Compose**: [`deployment/docker-compose*.yml`](../deployment/)
- **Monitoring**: [`deployment/monitoring/`](../deployment/monitoring/)

## 📝 **Notes**

- All scripts are executable and include proper error handling
- Configuration files are environment-specific and documented
- Documentation is comprehensive and up-to-date
- Examples demonstrate real-world usage patterns
- Build artifacts are gitignored but documented

This structure follows Rust conventions and best practices for maintainable, scalable projects.

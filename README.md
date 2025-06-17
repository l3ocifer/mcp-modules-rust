# DevOps MCP

A comprehensive modular Rust implementation of the Model Context Protocol (MCP) designed specifically for DevOps workflows.

## Overview

This SDK provides a modular REST MCP implementation that consolidates all essential DevOps functionality into a single, cohesive interface. It's organized into core modules that cover the entire DevOps lifecycle:

- **Infrastructure**: Manage cloud resources, Kubernetes clusters, and infrastructure as code
- **CI/CD**: Integrate with development workflows, issue tracking, and deployment pipelines
- **Monitoring**: Track application performance, logs, and error reporting
- **Database**: Interact with various database systems and manage data operations
- **Security**: Handle authentication, vulnerability scanning, and security events
- **Collaboration**: Send notifications and integrate with team communication platforms
- **Development**: Mobile and web application development tools
- **Analytics**: Data visualization and business intelligence
- **Gaming**: Game development and platform integration

## Architecture

The DevOps Architect MCP follows a modular architecture with core and extension modules:

```
┌───────────────────────────────────────────────────────────────────────┐
│                      DevOps Architect MCP                              │
├─────────┬─────────┬──────────┬──────────┬──────────┬──────────────────┤
│ Infra   │ CI/CD   │ Monitor  │ Database │ Security │ Collaboration    │
│ Module  │ Module  │ Module   │ Module   │ Module   │ Module           │
├─────────┼─────────┼──────────┼──────────┼──────────┼──────────────────┤
│ Cloud   │ GitHub  │ Metrics  │ SQL      │ Secrets  │ Slack            │
│ Services│ Actions │ & Logs   │ & NoSQL  │ & Auth   │ & Notifications  │
├─────────┼─────────┼──────────┼──────────┼──────────┼──────────────────┤
│ Creation│ Develop │ Analytics│ Gaming   │          │                  │
│ Module  │ Module  │ Module   │ Module   │          │                  │
└─────────┴─────────┴──────────┴──────────┴──────────┴──────────────────┘
```

### Design Principles

1. **Modularity**: Organized into functional modules that can be used independently or together
2. **REST Compliance**: Following REST architectural principles for consistency and interoperability 
3. **Authentication Unification**: Standardized authentication across all modules
4. **Consistent Error Handling**: Uniform error reporting and handling
5. **Extensibility**: Easy to extend with additional modules or capabilities

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
devops-mcp = "0.1.0"
```

## Usage

### Basic Example

```rust
use devops_mcp::{new, Config};
use devops_mcp::config::{
    InfrastructureConfig, InfrastructureProvider, CloudflareConfig,
    CicdConfig, CicdProvider, GitHubConfig,
    CollaborationConfig, CollaborationProvider, SlackConfig
};

// Create configuration
let config = Config::new(
    // Infrastructure configuration
    InfrastructureConfig {
        providers: vec![
            InfrastructureProvider::Cloudflare(CloudflareConfig {
                api_key: "your_api_key".to_string(),
                account_id: "your_account_id".to_string(),
            }),
        ],
    },
    // CI/CD configuration
    CicdConfig {
        providers: vec![
            CicdProvider::GitHub(GitHubConfig {
                token: "your_github_token".to_string(),
                organization: Some("your_organization".to_string()),
            }),
        ],
    },
    // Other configurations...
);

// Create client
let client = new(config);

// Use the client
async fn example_usage(client: &devops_mcp::client::DevOpsArchitectMcp) -> Result<(), Box<dyn std::error::Error>> {
    // Get GitHub repositories
    let repos = client.cicd().github()?.list_repositories().await?;
    
    // Deploy to Cloudflare
    let deployment = client.infrastructure().cloudflare()?.deploy_website("example.com", "/path/to/site").await?;
    
    // Send notification to Slack
    let notification = devops_mcp::collaboration::Notification {
        title: "Deployment Successful".to_string(),
        message: format!("Website deployed to {}", deployment.url),
        level: devops_mcp::collaboration::NotificationLevel::Success,
        source: "DevOps MCP".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: None,
    };
    
    client.collaboration().slack()?.send_notification(&notification).await?;
    
    Ok(())
}
```

### Using the Builder Pattern

```rust
use devops_mcp::client::DevOpsArchitectMcpBuilder;
use devops_mcp::config::{
    InfrastructureConfig, InfrastructureProvider, KubernetesConfig,
    MonitoringConfig, MonitoringProvider, GrafanaConfig,
};

// Create client using builder pattern
let client = DevOpsArchitectMcpBuilder::new()
    .with_infrastructure(InfrastructureConfig {
        providers: vec![
            InfrastructureProvider::Kubernetes(KubernetesConfig {
                kubeconfig: Some("/path/to/kubeconfig".to_string()),
                context: Some("production".to_string()),
            }),
        ],
    })
    .with_monitoring(MonitoringConfig {
        providers: vec![
            MonitoringProvider::Grafana(GrafanaConfig {
                api_key: "your_grafana_api_key".to_string(),
                url: "https://grafana.example.com".to_string(),
            }),
        ],
    })
    .build();
```

### Loading Configuration from File

```rust
use devops_mcp::from_file;

// Load configuration from file
let client = from_file("/path/to/config.json").expect("Failed to load configuration");
```

## Core Modules

### Infrastructure Module

Manage cloud resources, infrastructure provisioning, and configuration.

- **Resource provisioning and management**
- **Infrastructure as Code template management**
- **Deployment automation**
- **Configuration management**
- **Service discovery**

**Supported Providers**: Cloudflare, Vercel, Kubernetes, and more

### CI/CD Module

Manage code repositories, pipelines, and release processes.

- **Repository management**
- **Branch and PR workflows**
- **Pipeline configuration and execution**
- **Issue tracking**
- **Release management**

**Supported Systems**: GitHub, Linear, and more

### Monitoring Module

Manage observability, logging, metrics, and alerting.

- **Log collection and analysis**
- **Metrics gathering and visualization**
- **Dashboard management**
- **Alert configuration**
- **Incident management**

**Supported Systems**: Axiom, Grafana, Raygun, PostHog, Comet Opik

### Database Module

Manage database operations across different database systems.

- **Database provisioning and configuration**
- **Schema management**
- **Query execution**
- **Backup and restore operations**
- **Migration management**

**Supported Systems**: MongoDB, PostgreSQL, Supabase, Neon, ClickHouse

### Security Module

Manage security aspects including secrets, authentication, and compliance.

- **Secret management**
- **Authentication and authorization**
- **Compliance monitoring and reporting**
- **Security scanning**
- **Policy management**

### Collaboration Module

Manage team communication and collaboration tools.

- **Message sending and management**
- **Channel management**
- **Notification configuration**
- **Document sharing and collaboration**

**Supported Systems**: Slack, Resend (email), and more

### Development Module

Tools for mobile and web application development.

- **Flutter application building and testing**
- **App store deployment**
- **Mobile app analytics**

### Analytics Module

Data visualization and business intelligence tools.

- **Apache Superset dashboard management**
- **Chart creation and editing**
- **Data source management**

### Gaming Module

Game development and platform integration.

- **Steam API integration**
- **Game library management**
- **Friend network interaction**

## Configuration

The SDK uses a modular configuration system that allows you to enable only the providers you need:

```json
{
  "infrastructure": {
    "providers": [
      {
        "type": "cloudflare",
        "api_key": "your_api_key",
        "account_id": "your_account_id"
      }
    ]
  },
  "cicd": {
    "providers": [
      {
        "type": "github",
        "token": "your_github_token",
        "organization": "your_organization"
      }
    ]
  },
  "monitoring": {
    "providers": [
      {
        "type": "grafana",
        "api_key": "your_grafana_api_key",
        "url": "https://grafana.example.com"
      }
    ]
  }
}
```

## Error Handling

The SDK provides a comprehensive error handling system:

```rust
match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(devops_mcp::error::Error::Auth(msg)) => println!("Authentication error: {}", msg),
    Err(devops_mcp::error::Error::Service(msg)) => println!("Service error: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## MCP Creation

The SDK includes a Creation module that allows you to create new MCP servers:

```rust
// Create a new MCP server from a template
let server_id = client.creation().create_server_from_template("typescript").await?;

// Execute a tool on the server
let result = client.creation().execute_tool(
    &server_id,
    "echo",
    serde_json::json!({ "message": "Hello, world!" })
).await?;
```

## New Modules

We've extended the DevOps Architect MCP with additional specialized modules:

### Government Module

Access government data and services:

- **Grants**: Search and retrieve information about government grant opportunities
  - Features include searching by keywords, retrieving detailed grant information
  - Supports pagination and filtering

### Memory Module

Long-term knowledge graph storage to maintain context across sessions:

- Store various memory types (Project, Issue, System, Config, Finance, Todo, Knowledge)
- Create relationships between memories (RelatedTo, PartOf, DependsOn, etc.)
- Query and retrieve memories based on type, keyword, or relationships
- In-memory implementation with optional Redis persistence

### Finance Module

Financial trading and market data:

- **Alpaca**: Stock market trading and data
  - Retrieve account information and positions
  - Get real-time quotes and historical price data
  - Place market and limit orders
  - Manage existing orders

### Maps Module

Geographic and spatial data services:

- **OpenStreetMap**: Access geographic data and services
  - Query OSM data using Overpass QL
  - Find points of interest near locations
  - Search for named places
  - Route planning capabilities
  - Map image generation (requires external service)

## Adding New MCP Modules

The DevOps Architect MCP is designed to be extensible, allowing easy integration of new MCP modules. Follow these steps to add a new module:

### Step 1: Clone the repository

Clone the target MCP repository to the `temp_repos` directory for analysis:

```bash
git clone https://github.com/example/target-mcp temp_repos/target-mcp
```

### Step 2: Analyze the repository

Examine the repository to understand:
- The main functionality and API
- Data structures and types
- Configuration requirements
- Dependencies

### Step 3: Create module directory structure

Create appropriate directory structure under `src/`:

```bash
mkdir -p src/your_module_name/your_submodule
```

### Step 4: Implement the module

Create the following files:

1. **Main module file** (`src/your_module_name/mod.rs`):
   - Re-export key types and submodules
   - Provide documentation

2. **Submodule implementation** (`src/your_module_name/your_submodule/mod.rs`):
   - Implement the client and API
   - Define necessary data structures
   - Implement tool definitions for LLM use

### Step 5: Add configuration

Update `src/config.rs` to include configuration for your module:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YourModuleConfig {
    // Configuration parameters
}
```

Add it to the main Config struct:

```rust
pub struct Config {
    // ...
    pub your_module: Option<YourModuleConfig>,
}
```

### Step 6: Add module access in the client

Update `src/client.rs` to provide access to your module:

```rust
impl DevOpsArchitectMcp {
    // ...
    pub fn your_module(&self) -> YourModuleModule {
        YourModuleModule {
            client: self,
            lifecycle: &self.lifecycle,
        }
    }
}

pub struct YourModuleModule<'a> {
    client: &'a DevOpsArchitectMcp,
    lifecycle: &'a LifecycleManager,
}

impl<'a> YourModuleModule<'a> {
    pub fn your_submodule(&self) -> Result<YourSubmoduleClient> {
        // Configure and return the client
    }
}
```

### Step 7: Update lib.rs

Update `src/lib.rs` to expose your module:

```rust
pub mod your_module;
pub use your_module::your_submodule::{YourSubmoduleClient, OtherTypes};
```

### Step 8: Test your implementation

Create tests to verify your implementation works correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_your_module() {
        // Test implementation
    }
}
```

### Step 9: Document your module

Add comprehensive documentation:
- Update the README.md with information about your module
- Add inline documentation using rustdoc comments
- Provide usage examples

Following these steps ensures a consistent approach to extending the DevOps Architect MCP with new capabilities.

## License

MIT

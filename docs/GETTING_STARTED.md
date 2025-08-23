# MCP Modules Rust - Getting Started Guide

## Introduction

Welcome to mcp-modules-rust! This guide will help you get up and running with this comprehensive Model Context Protocol (MCP) implementation for DevOps workflows.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: Version 1.70 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Git**: For cloning the repository
- **OpenSSL**: Required for TLS support (usually pre-installed on most systems)

### Optional Prerequisites

For full functionality, you may also need:

- **Docker**: For container management features
- **Kubernetes CLI (kubectl)**: For Kubernetes operations
- **Cloud CLIs**: AWS CLI, Azure CLI, or gcloud for cloud operations
- **PostgreSQL/MongoDB**: For database features

## Installation

### Method 1: As a Rust Dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
devops-mcp = "0.1.0"
```

For specific features:

```toml
[dependencies]
devops-mcp = { version = "0.1.0", features = ["database", "cloud", "containers"] }
```

### Method 2: Clone and Build

```bash
# Clone the repository
git clone https://github.com/devops-mcp/mcp-modules-rust
cd mcp-modules-rust

# Build the project
cargo build --release

# Run tests
cargo test

# Build with all features
cargo build --release --all-features
```

## Quick Start

### Basic Usage

```rust
use devops_mcp::{new, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default configuration
    let config = Config::default();
    let client = new(config)?;
    
    // Initialize the client
    client.initialize().await?;
    
    // Perform a health check
    let health = client.health_check(None).await?;
    println!("System health: {:?}", health.overall);
    
    Ok(())
}
```

### Using Specific Modules

#### Memory Module Example

```rust
use devops_mcp::{new, Config};
use devops_mcp::memory::{MemoryClient, Memory, MemoryType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let client = new(config)?;
    
    // Create a memory client
    let memory_client = MemoryClient::new(client.lifecycle());
    
    // Store a memory
    let memory = Memory {
        content: "Important project decision: Use Rust for performance".to_string(),
        memory_type: MemoryType::Decision,
        metadata: serde_json::json!({
            "project": "mcp-modules",
            "date": "2024-01-15"
        }),
        ..Default::default()
    };
    
    let memory_id = memory_client.create_memory(memory).await?;
    println!("Created memory with ID: {}", memory_id);
    
    // Search memories
    let results = memory_client.search_memories(
        devops_mcp::memory::MemorySearchParams {
            keyword: Some("Rust".to_string()),
            ..Default::default()
        }
    ).await?;
    
    println!("Found {} memories", results.len());
    
    Ok(())
}
```

#### Security Module Example

```rust
use devops_mcp::security::{SecurityModule, SanitizationOptions, ValidationResult};

fn main() {
    let security = SecurityModule::new();
    
    // Configure validation options
    let options = SanitizationOptions {
        allow_html: false,
        allow_sql: false,
        allow_shell_meta: false,
        max_length: Some(1000),
    };
    
    // Validate user input
    let user_input = "SELECT * FROM users WHERE id = 1";
    
    match security.validate_input(user_input, &options) {
        ValidationResult::Valid => {
            println!("Input is safe to use");
        },
        ValidationResult::Invalid(reason) => {
            println!("Input validation failed: {}", reason);
        },
        ValidationResult::Malicious(reason) => {
            println!("SECURITY ALERT: {}", reason);
        }
    }
}
```

## Configuration

### Configuration File

Create a `config.toml` file:

```toml
[transport]
type = "websocket"
url = "ws://localhost:8080"

[auth]
required = true
token = "your-auth-token"

[security]
rate_limit = 60
input_validation = true

[infrastructure.kubernetes]
enabled = true
kubeconfig = "~/.kube/config"

[database.postgresql]
enabled = true
connection_string = "postgresql://user:pass@localhost/db"
```

Load from file:

```rust
use devops_mcp::from_file;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = from_file("config.toml")?;
    client.initialize().await?;
    Ok(())
}
```

### Environment Variables

Configure via environment variables:

```bash
export MCP_TRANSPORT_TYPE=http
export MCP_TRANSPORT_URL=http://localhost:8080
export MCP_AUTH_TOKEN=your-token
export MCP_SECURITY_RATE_LIMIT=100
```

### Programmatic Configuration

```rust
use devops_mcp::{Config, new};
use devops_mcp::config::{TransportConfig, AuthConfig, SecurityConfig};

let mut config = Config::new();

// Configure transport
config.transport = Some(TransportConfig {
    transport_type: "websocket".to_string(),
    url: Some("ws://localhost:8080".to_string()),
    ..Default::default()
});

// Configure authentication
config.auth = Some(AuthConfig {
    required: true,
    token: Some("your-token".to_string()),
    ..Default::default()
});

// Configure security
config.security = Some(SecurityConfig {
    rate_limit: Some(100),
    input_validation: true,
    ..Default::default()
});

let client = new(config)?;
```

## Common Use Cases

### 1. Infrastructure Management

```rust
use devops_mcp::infrastructure::kubernetes::KubernetesClient;

async fn manage_k8s(client: &KubernetesClient) -> Result<(), Box<dyn std::error::Error>> {
    // List pods in default namespace
    let pods = client.list_pods("default").await?;
    for pod in pods {
        println!("Pod: {}", pod.name);
    }
    
    // Scale a deployment
    client.scale_deployment("my-app", "default", 3).await?;
    
    Ok(())
}
```

### 2. Office Automation

```rust
use devops_mcp::office::powerpoint::{PowerPointClient, Presentation, Slide, SlideLayout};

async fn create_presentation(client: &PowerPointClient) -> Result<(), Box<dyn std::error::Error>> {
    let presentation = Presentation {
        title: "Project Status Update".to_string(),
        slides: vec![
            Slide {
                title: "Overview".to_string(),
                layout: SlideLayout::TitleSlide,
                subtitle: Some("Q4 2024 Progress".to_string()),
                ..Default::default()
            },
            Slide {
                title: "Key Achievements".to_string(),
                layout: SlideLayout::TitleAndContent,
                bullets: Some(vec![
                    "Completed migration to Rust".to_string(),
                    "Improved performance by 50%".to_string(),
                    "Reduced memory usage by 30%".to_string(),
                ]),
                ..Default::default()
            }
        ],
        ..Default::default()
    };
    
    let ppt_id = client.create_presentation(presentation).await?;
    println!("Created presentation: {}", ppt_id);
    
    Ok(())
}
```

### 3. Financial Trading

```rust
use devops_mcp::finance::alpaca::{AlpacaClient, MarketOrderRequest, OrderSide, TimeInForce};

async fn place_trade(client: &AlpacaClient) -> Result<(), Box<dyn std::error::Error>> {
    // Check account balance
    let account = client.get_account().await?;
    println!("Buying power: ${}", account.buying_power);
    
    // Place a market order
    if account.buying_power > 1000.0 {
        let order = MarketOrderRequest {
            symbol: "AAPL".to_string(),
            qty: 10.0,
            side: OrderSide::Buy,
            time_in_force: TimeInForce::Day,
        };
        
        let result = client.place_market_order(order).await?;
        println!("Order placed: {:?}", result);
    }
    
    Ok(())
}
```

### 4. Smart Home Automation

```rust
use devops_mcp::smart_home::home_assistant::{HomeAssistantClient, EntityDomain};

async fn automate_home(client: &HomeAssistantClient) -> Result<(), Box<dyn std::error::Error>> {
    // Get all lights
    let lights = client.get_entities_by_domain(EntityDomain::Light).await?;
    
    // Turn off all lights
    for light in lights {
        if light.state == Some("on".to_string()) {
            client.call_service("light", "turn_off", &light.entity_id, None).await?;
            println!("Turned off: {}", light.entity_id);
        }
    }
    
    // Set thermostat
    client.call_service(
        "climate",
        "set_temperature",
        "climate.living_room",
        Some(serde_json::json!({ "temperature": 72 }))
    ).await?;
    
    Ok(())
}
```

## Error Handling

The library uses a comprehensive error system:

```rust
use devops_mcp::Error;

match some_operation().await {
    Ok(result) => {
        // Handle success
    },
    Err(e) => {
        match e.category() {
            "network" => {
                // Retry logic for network errors
                if e.is_recoverable() {
                    retry_operation().await?;
                }
            },
            "authentication" => {
                // Re-authenticate
                refresh_token().await?;
            },
            "validation" => {
                // Handle invalid input
                eprintln!("Invalid input: {}", e);
            },
            _ => {
                // Generic error handling
                eprintln!("Operation failed: {}", e);
            }
        }
    }
}
```

## Best Practices

### 1. Use Lifecycle Management

Always use the lifecycle manager for coordinating modules:

```rust
let client = new(Config::default())?;
let lifecycle = client.lifecycle();

// Share lifecycle across modules
let memory = MemoryClient::new(&lifecycle);
let office = PowerPointClient::new(&lifecycle);
```

### 2. Handle Errors Gracefully

```rust
use devops_mcp::error::retry_with_backoff;

// Retry operations that might fail
let result = retry_with_backoff(
    || async { fetch_data().await },
    3, // max attempts
    Duration::from_millis(100), // initial delay
).await?;
```

### 3. Validate Input

Always validate user input before processing:

```rust
let security = SecurityModule::new();
if let ValidationResult::Valid = security.validate_input(input, &options) {
    // Safe to process
} else {
    return Err(Error::validation("Invalid input"));
}
```

### 4. Use Appropriate Features

Only enable the features you need:

```toml
[dependencies]
devops-mcp = { 
    version = "0.1.0", 
    features = ["database", "security"],
    default-features = false 
}
```

## Troubleshooting

### Common Issues

1. **Connection Errors**
   ```
   Error: Network error: Connection refused
   ```
   - Check if the MCP server is running
   - Verify the URL in configuration
   - Check firewall settings

2. **Authentication Failures**
   ```
   Error: Authentication error: Invalid token
   ```
   - Verify your authentication token
   - Check token expiration
   - Ensure auth is configured correctly

3. **Module Not Found**
   ```
   Error: Module 'kubernetes' not found
   ```
   - Enable the required feature in Cargo.toml
   - Rebuild with `--all-features` flag

### Debug Logging

Enable debug logging for troubleshooting:

```rust
env_logger::init();
std::env::set_var("RUST_LOG", "devops_mcp=debug");
```

Or set via environment:

```bash
RUST_LOG=devops_mcp=debug cargo run
```

## Next Steps

1. **Explore Modules**: Check the [Module Guide](MODULE_GUIDE.md) for detailed module documentation
2. **Security**: Review the [Security Guide](SECURITY.md) for security best practices
3. **Performance**: See the [Performance Guide](PERFORMANCE_GUIDE.md) for optimization tips
4. **Architecture**: Understand the system design in the [Architecture Guide](ARCHITECTURE.md)

## Getting Help

- **Documentation**: Check the `/docs` directory
- **Issues**: Report bugs on GitHub
- **Examples**: Look in the `/examples` directory (when available)
- **Tests**: Review `/tests` for usage examples

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.
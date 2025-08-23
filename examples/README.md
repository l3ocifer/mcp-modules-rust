# MCP Modules Rust - Examples

This directory contains practical examples demonstrating how to use various modules of the mcp-modules-rust library.

## Available Examples

### Core Examples

- **[basic_usage.rs](basic_usage.rs)** - Basic client setup, security validation, memory management, and error handling
- **[office_automation.rs](office_automation.rs)** - PowerPoint, Word, and Excel automation for document generation

### Quick Start

To run any example:

```bash
# Basic usage example
cargo run --example basic_usage

# Office automation example  
cargo run --example office_automation
```

## Example Categories

### 🔧 Core Functionality
- Client initialization and configuration
- Health checking and lifecycle management
- Error handling patterns
- Security and input validation

### 📊 Office Automation
- PowerPoint presentation creation
- Word document generation
- Excel spreadsheet manipulation
- Automated reporting workflows

### 🏗️ Infrastructure Management
- Kubernetes cluster operations
- Docker container management
- Cloud resource provisioning
- Infrastructure as Code workflows

### 🏠 Smart Home Integration
- Home Assistant entity control
- Automation scenarios
- Device state management
- Service orchestration

### 💰 Financial Trading
- Alpaca API integration
- Portfolio management
- Order execution
- Market data analysis

### 🔍 Research and Analysis
- Deep research capabilities
- Citation management
- Knowledge synthesis
- Academic workflows

### 🧠 Memory Management
- Knowledge base operations
- Memory relationships
- Search and retrieval
- Metadata management

### 🗺️ Geographic Data
- OpenStreetMap integration
- Location services
- Geographic queries
- Spatial analysis

## Running Examples

### Prerequisites

Some examples require additional setup:

1. **Environment Variables**: Set required API keys and credentials
2. **External Services**: Some examples connect to external services
3. **Dependencies**: Ensure all optional features are enabled

### Configuration

Create a `.env` file in the project root:

```bash
# Office automation (if using real Office APIs)
OFFICE_API_KEY=your_office_api_key

# Financial trading (Alpaca)
ALPACA_API_KEY=your_alpaca_api_key
ALPACA_SECRET_KEY=your_alpaca_secret_key
ALPACA_PAPER_TRADING=true

# Smart home (Home Assistant)
HOME_ASSISTANT_URL=http://homeassistant.local:8123
HOME_ASSISTANT_TOKEN=your_home_assistant_token

# Database connections
MONGODB_CONNECTION_STRING=mongodb://localhost:27017
POSTGRESQL_CONNECTION_STRING=postgresql://user:pass@localhost/db

# Infrastructure
KUBECONFIG=~/.kube/config
AWS_PROFILE=default
```

### Example Usage Patterns

#### Error Handling
```rust
match client.some_operation().await {
    Ok(result) => {
        // Handle success
        println!("Operation completed: {:?}", result);
    },
    Err(e) => {
        // Handle different error types
        match e.category() {
            "network" => {
                if e.is_recoverable() {
                    // Retry logic
                    retry_operation().await?;
                }
            },
            "authentication" => {
                // Re-authenticate
                refresh_credentials().await?;
            },
            _ => {
                // Log and continue
                eprintln!("Operation failed: {}", e);
            }
        }
    }
}
```

#### Configuration Patterns
```rust
// From environment variables
let config = Config::from_env()?;

// From configuration file
let config = Config::from_file("config.toml")?;

// Programmatic configuration
let mut config = Config::new();
config.transport = Some(TransportConfig {
    transport_type: "websocket".to_string(),
    url: Some("ws://localhost:8080".to_string()),
    ..Default::default()
});
```

#### Module Integration
```rust
// Share lifecycle across modules
let client = new(config)?;
let lifecycle = client.lifecycle();

let memory = MemoryClient::new(&lifecycle);
let office = PowerPointClient::new(&lifecycle);
let security = SecurityModule::new();

// Coordinate operations
let validated_input = security.validate_input(user_input, &options)?;
let memory_id = memory.create_memory(validated_input).await?;
let presentation = office.create_from_memory(memory_id).await?;
```

## Example Categories Explained

### Core Examples
These examples demonstrate fundamental concepts:
- Client lifecycle management
- Configuration patterns
- Error handling strategies
- Security best practices

### Module-Specific Examples
Each major module has dedicated examples showing:
- Module initialization
- Core functionality
- Integration patterns
- Performance considerations

### Workflow Examples
Real-world scenarios combining multiple modules:
- Automated reporting pipelines
- Infrastructure deployment workflows
- Data analysis and presentation
- Security monitoring and response

## Testing Examples

Run examples in test mode:

```bash
# Set test environment
export MCP_TEST_MODE=true

# Run with mock data
cargo run --example basic_usage --features test-mode
```

## Contributing Examples

When adding new examples:

1. **Follow naming convention**: `module_name_functionality.rs`
2. **Include documentation**: Explain what the example demonstrates
3. **Handle errors gracefully**: Show proper error handling patterns
4. **Use realistic data**: Provide meaningful example data
5. **Add to this README**: Update the examples list

### Example Template

```rust
/// [Module] [Functionality] Example
/// 
/// This example demonstrates how to:
/// 1. [First capability]
/// 2. [Second capability]
/// 3. [Third capability]

use devops_mcp::{new, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("📋 [Module] [Functionality] Example");
    
    // Initialize client
    let config = Config::default();
    let client = new(config)?;
    
    // Example implementation
    // ...
    
    println!("✅ Example completed successfully!");
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **Missing Features**: Enable required features in Cargo.toml
2. **Network Connections**: Check if external services are running
3. **Authentication**: Verify API keys and credentials
4. **Permissions**: Ensure proper file/network permissions

### Debug Mode

Run examples with debug logging:

```bash
RUST_LOG=devops_mcp=debug cargo run --example basic_usage
```

### Mock Mode

Some examples support mock mode for testing without external dependencies:

```bash
MCP_MOCK_MODE=true cargo run --example financial_trading
```

## Next Steps

After running examples:

1. **Read the documentation**: Check `/docs` for detailed guides
2. **Explore modules**: Investigate specific module capabilities
3. **Build your own**: Create custom workflows for your use case
4. **Contribute**: Share your examples with the community
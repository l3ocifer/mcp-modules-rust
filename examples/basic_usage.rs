/// Basic usage example for the MCP Modules Rust library
/// 
/// This example demonstrates how to:
/// 1. Create and initialize an MCP client
/// 2. Perform health checks
/// 3. Use basic functionality from core modules

use devops_mcp::{new, Config, Error};
use devops_mcp::memory::{MemoryClient, Memory, MemoryType};
use devops_mcp::security::{SecurityModule, SanitizationOptions, ValidationResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 MCP Modules Rust - Basic Usage Example");
    
    // Create a client with default configuration
    let config = Config::default();
    let client = new(config)?;
    
    println!("✅ Client created successfully");
    
    // Initialize the client
    match client.initialize().await {
        Ok(_) => println!("✅ Client initialized successfully"),
        Err(e) => {
            // This might fail if no MCP server is running, which is expected in this example
            println!("⚠️  Client initialization failed (expected): {}", e);
            println!("   This is normal if no MCP server is running");
        }
    }
    
    // Perform a health check
    match client.health_check(None).await {
        Ok(health) => {
            println!("✅ Health check completed: {:?}", health.overall);
        },
        Err(e) => {
            println!("⚠️  Health check failed (expected): {}", e);
            println!("   This is normal if no MCP server is running");
        }
    }
    
    // Example 1: Security Module - Input Validation
    println!("\n🔒 Security Module Example");
    
    let security = SecurityModule::new();
    let options = SanitizationOptions::default();
    
    let test_inputs = vec![
        "normal_username_123",          // Valid input
        "'; DROP TABLE users; --",     // SQL injection attempt
        "<script>alert('xss')</script>", // XSS attempt
        "normal input",                 // Valid input
    ];
    
    for input in test_inputs {
        match security.validate_input(input, &options) {
            ValidationResult::Valid => {
                println!("✅ Input '{}' is valid", input);
            },
            ValidationResult::Invalid(reason) => {
                println!("⚠️  Input '{}' is invalid: {}", input, reason);
            },
            ValidationResult::Malicious(reason) => {
                println!("🚨 SECURITY ALERT - Input '{}': {}", input, reason);
            }
        }
    }
    
    // Example 2: Memory Module - Knowledge Management
    println!("\n🧠 Memory Module Example");
    
    let memory_client = MemoryClient::new(client.lifecycle());
    
    // Create some example memories
    let memories = vec![
        Memory {
            content: "Rust provides memory safety without garbage collection".to_string(),
            memory_type: MemoryType::Fact,
            metadata: serde_json::json!({
                "category": "programming",
                "language": "rust",
                "importance": 9
            }),
            ..Default::default()
        },
        Memory {
            content: "Decision: Use actix-web for HTTP server implementation".to_string(),
            memory_type: MemoryType::Decision,
            metadata: serde_json::json!({
                "project": "mcp-modules",
                "date": "2024-01-15",
                "reasoning": "Performance and ecosystem"
            }),
            ..Default::default()
        },
        Memory {
            content: "Always validate user input to prevent security vulnerabilities".to_string(),
            memory_type: MemoryType::Insight,
            metadata: serde_json::json!({
                "category": "security",
                "importance": 10
            }),
            ..Default::default()
        },
    ];
    
    // Store memories
    let mut memory_ids = Vec::new();
    for memory in memories {
        match memory_client.create_memory(memory.clone()).await {
            Ok(id) => {
                println!("✅ Created memory: '{}' (ID: {})", 
                    memory.content.chars().take(50).collect::<String>(), id);
                memory_ids.push(id);
            },
            Err(e) => {
                println!("⚠️  Failed to create memory: {}", e);
            }
        }
    }
    
    // Search memories
    if !memory_ids.is_empty() {
        let search_params = devops_mcp::memory::MemorySearchParams {
            keyword: Some("rust".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        
        match memory_client.search_memories(search_params).await {
            Ok(results) => {
                println!("🔍 Found {} memories containing 'rust':", results.len());
                for memory in results {
                    println!("   - {}", memory.content.chars().take(60).collect::<String>());
                }
            },
            Err(e) => {
                println!("⚠️  Memory search failed: {}", e);
            }
        }
    }
    
    // Example 3: Error Handling Demonstration
    println!("\n⚠️  Error Handling Example");
    
    // Demonstrate different error types
    let errors = vec![
        Error::network("Connection timeout"),
        Error::auth("Invalid API key"),
        Error::validation("Missing required field"),
        Error::internal("Unexpected server error"),
    ];
    
    for error in errors {
        println!("Error category '{}': {} (recoverable: {})", 
            error.category(), error, error.is_recoverable());
    }
    
    // Example 4: Configuration Demonstration
    println!("\n⚙️  Configuration Example");
    
    // Show different ways to configure the client
    let mut custom_config = Config::new();
    
    // Configure transport (this would normally connect to a real server)
    custom_config.transport = Some(devops_mcp::config::TransportConfig {
        transport_type: "http".to_string(),
        url: Some("http://localhost:8080".to_string()),
        ..Default::default()
    });
    
    // Configure security
    custom_config.security = Some(devops_mcp::config::SecurityConfig {
        rate_limit: Some(100),
        input_validation: true,
        ..Default::default()
    });
    
    println!("✅ Custom configuration created:");
    println!("   Transport: HTTP to localhost:8080");
    println!("   Security: Rate limit 100/min, validation enabled");
    
    // Example 5: Tool Discovery
    println!("\n🔧 Tool Discovery Example");
    
    let tools = memory_client.get_tools();
    println!("Memory module provides {} tools:", tools.len());
    for (name, tool) in tools {
        println!("   - {}: {}", name, tool.description);
    }
    
    println!("\n🎉 Basic usage example completed!");
    println!("📚 Check out other examples for specific modules:");
    println!("   - office_automation.rs");
    println!("   - smart_home_control.rs");
    println!("   - financial_trading.rs");
    println!("   - infrastructure_management.rs");
    
    Ok(())
}
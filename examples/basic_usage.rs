/// Basic usage example for the MCP Modules Rust library
/// 
/// This example demonstrates how to:
/// 1. Create and initialize an MCP client
/// 2. Perform health checks
/// 3. Use basic functionality from core modules

use devops_mcp::{Mcp, Config};
use devops_mcp::memory::{MemoryClient, MemoryType};
use devops_mcp::security::{SecurityModule, SanitizationOptions, ValidationResult};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("devops_mcp=debug")
        .init();
    
    println!("🚀 MCP Modules Rust - Basic Usage Example");
    
    // Create a client with default configuration
    let config = Config::default();
    let mut client = Mcp::new(config)?;
    
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
    
    // Get lifecycle for modules that need it
    let lifecycle = match client.lifecycle() {
        Ok(lc) => Some(lc),
        Err(_) => {
            println!("⚠️  Lifecycle not available - some features may be limited");
            None
        }
    };
    
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
    
    // Example 2: Memory Module - Knowledge Management (if lifecycle available)
    if let Some(ref lifecycle) = lifecycle {
        println!("\n🧠 Memory Module Example");
        
        let memory_client = MemoryClient::new(lifecycle);
        
        // Create some example memories
        let memories = vec![
            ("Rust provides memory safety without garbage collection", MemoryType::Knowledge, {
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), serde_json::json!("programming"));
                metadata.insert("language".to_string(), serde_json::json!("rust"));
                metadata.insert("importance".to_string(), serde_json::json!(9));
                metadata
            }),
            ("Decision: Use actix-web for HTTP server implementation", MemoryType::Project, {
                let mut metadata = HashMap::new();
                metadata.insert("project".to_string(), serde_json::json!("mcp-modules"));
                metadata.insert("date".to_string(), serde_json::json!("2024-01-15"));
                metadata.insert("reasoning".to_string(), serde_json::json!("Performance and ecosystem"));
                metadata
            }),
            ("MCP protocol version 2025-06-18 is supported", MemoryType::System, {
                let mut metadata = HashMap::new();
                metadata.insert("protocol".to_string(), serde_json::json!("MCP"));
                metadata.insert("version".to_string(), serde_json::json!("2025-06-18"));
                metadata
            }),
        ];
        
        // Store memories
        for (content, memory_type, metadata) in memories {
            match memory_client.create_memory(
                memory_type.clone(),
                format!("{:?} Memory", memory_type),
                content,
                Some(metadata)
            ).await {
                Ok(id) => println!("✅ Created memory: {}", id),
                Err(e) => println!("⚠️  Failed to create memory: {}", e),
            }
        }
        
        // Search memories
        let search_params = devops_mcp::memory::MemorySearchParams {
            keyword: Some("rust".to_string()),
            memory_type: None,
            metadata_filters: None,
            limit: Some(5),
        };
        
        match memory_client.search_memories(search_params).await {
            Ok(results) => {
                println!("\n🔍 Search results for 'rust':");
                for memory in results.iter() {
                    println!("   - {}: {}", memory.title, &memory.content[..50.min(memory.content.len())]);
                }
            },
            Err(e) => println!("⚠️  Search failed: {}", e),
        }
    }
    
    // Example 3: Tools Module
    println!("\n🛠️  Tools Module Example");
    
    match client.tools() {
        Ok(tool_manager) => {
            let tools = tool_manager.list_tools();
            println!("Available tools: {} tools registered", tools.len());
            for tool in tools.iter().take(5) {
                println!("   - {}: {}", tool.name, tool.description);
            }
        },
        Err(e) => println!("⚠️  Failed to get tools: {}", e),
    }
    
    println!("\n✨ Basic usage example completed!");
    Ok(())
}
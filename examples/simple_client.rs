use devops_mcp::client::MCPClient;
use devops_mcp::config::{Config, StdioConfig};
use devops_mcp::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("devops_mcp=debug")
        .init();

    println!("🚀 MCP Modules Rust - Simple Client Example");
    println!("==========================================");

    // Create configuration
    let config = Config {
        transport_type: "stdio".to_string(),
        stdio_config: Some(StdioConfig {
            command: "./target/release/devops-mcp".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
        }),
        ..Default::default()
    };

    // Create and initialize client
    let client = MCPClient::new(config);
    println!("\n📡 Initializing MCP client...");
    client.initialize().await?;
    println!("✅ Client initialized successfully!");

    // List available tools
    println!("\n🛠️  Available tools:");
    let tools = client.list_tools().await?;
    for tool in tools.iter().take(10) {
        println!("   - {}: {}", tool.name, tool.description);
    }
    if tools.len() > 10 {
        println!("   ... and {} more tools", tools.len() - 10);
    }

    // Test memory module (always available)
    println!("\n🧠 Testing memory module:");
    
    // Store a memory
    let memory_id = "example-memory-1";
    let result = client.call_tool("memory_store", serde_json::json!({
        "memory": {
            "id": memory_id,
            "memory_type": "note",
            "title": "Example Note",
            "content": "This is a test note created by the simple client example.",
            "metadata": {
                "source": "simple_client.rs",
                "category": "example"
            }
        }
    })).await?;
    println!("   ✅ Stored memory: {}", memory_id);

    // Retrieve the memory
    let result = client.call_tool("memory_get", serde_json::json!({
        "id": memory_id
    })).await?;
    if let Some(memory) = result.get("memory") {
        println!("   ✅ Retrieved memory: {}", memory.get("title").unwrap_or(&serde_json::Value::Null));
    }

    // Search memories
    let result = client.call_tool("memory_search", serde_json::json!({
        "params": {
            "keyword": "test"
        }
    })).await?;
    if let Some(memories) = result.get("memories").and_then(|v| v.as_array()) {
        println!("   ✅ Found {} memories matching 'test'", memories.len());
    }

    // Test infrastructure module if Docker is available
    if std::process::Command::new("docker").arg("info").output().is_ok() {
        println!("\n🐳 Testing Docker module:");
        
        let result = client.call_tool("docker_list_containers", serde_json::json!({
            "all": true
        })).await;
        
        match result {
            Ok(containers) => {
                if let Some(container_list) = containers.as_array() {
                    println!("   ✅ Found {} containers", container_list.len());
                    for (i, container) in container_list.iter().take(3).enumerate() {
                        if let Some(name) = container.get("name") {
                            println!("      {}. {}", i + 1, name);
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Docker test failed: {}", e);
            }
        }
    } else {
        println!("\n⚠️  Docker not available - skipping container tests");
    }

    // Clean up
    println!("\n🧹 Cleaning up...");
    client.shutdown().await?;
    println!("✅ Client shutdown successfully!");

    println!("\n🎉 Example completed successfully!");
    Ok(())
}
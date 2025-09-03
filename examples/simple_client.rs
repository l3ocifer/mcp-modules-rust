use devops_mcp::{Mcp, Config};
use devops_mcp::config::TransportConfig;
use devops_mcp::error::Result;
use devops_mcp::memory::{MemoryClient, MemoryType, MemorySearchParams};

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
        transport: Some(TransportConfig {
            transport_type: "stdio".to_string(),
            command: Some("./target/release/devops-mcp".to_string()),
            args: Some(vec![]),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create and initialize client
    let mut client = Mcp::new(config)?;
    println!("\n📡 Initializing MCP client...");
    client.initialize().await?;
    println!("✅ Client initialized successfully!");

    // Get the lifecycle manager
    let lifecycle = client.lifecycle()?;

    // List available tools
    println!("\n🛠️  Available tools:");
    let tool_manager = client.tools()?;
    let tools = tool_manager.list_tools();
    for tool in tools.iter().take(10) {
        println!("   - {}: {}", tool.name, tool.description);
    }
    if tools.len() > 10 {
        println!("   ... and {} more tools", tools.len() - 10);
    }

    // Test memory module (always available)
    println!("\n🧠 Testing memory module:");
    
    // Create memory client
    let memory_client = MemoryClient::new(&lifecycle);
    
    // Store a memory
    let memory_id = memory_client.create_memory(
        MemoryType::Knowledge,
        "Example Note",
        "This is a test note created by the simple client example.",
        Some([
            ("source".to_string(), serde_json::json!("simple_client.rs")),
            ("category".to_string(), serde_json::json!("example"))
        ].into_iter().collect())
    ).await?;
    println!("   ✅ Stored memory with ID: {}", memory_id);

    // Retrieve the memory
    let retrieved = memory_client.get_memory(&memory_id).await?;
    println!("   📖 Retrieved memory: {}", retrieved.title);
    println!("      Content: {}", retrieved.content);

    // Search memories
    let search_params = MemorySearchParams {
        keyword: Some("test".to_string()),
        memory_type: None,
        metadata_filters: None,
        limit: Some(10)
    };
    let search_results = memory_client.search_memories(search_params).await?;
    println!("   🔍 Found {} search results", search_results.len());

    // Test other modules if needed
    println!("\n🐳 Other modules (Docker, Kubernetes, etc.) would be tested here if configured");

    println!("\n✨ All operations completed successfully!");
    Ok(())
}
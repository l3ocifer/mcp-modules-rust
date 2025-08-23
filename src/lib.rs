/// High-Performance Rust Implementation of Model Context Protocol (MCP)
///
/// This crate provides a comprehensive MCP implementation with extensive performance
/// optimizations including zero-copy operations, efficient memory management,
/// and async-first design patterns.
// Core modules with performance optimizations
pub mod client;
pub mod config;
pub mod error;
pub mod lifecycle;
pub mod transport;

// Authentication and security with zero-copy where possible
pub mod auth;
pub mod security;

// Tools and capabilities
pub mod tools;

// Infrastructure and DevOps modules with efficient resource management
pub mod cicd;
pub mod cloud;
pub mod database;
pub mod infrastructure;
pub mod monitoring;

// Collaboration and development
pub mod collaboration;
pub mod creation;
pub mod development;

// Analytics and AI capabilities
pub mod ai;
pub mod analytics;

// Specialized domain modules
pub mod finance;
pub mod gaming;
pub mod government;
pub mod maps;
pub mod memory;
pub mod office;
pub mod research;
pub mod smart_home;
pub mod web;

// Re-export core types for performance-optimized API
pub use client::Mcp;
pub use config::Config;
pub use error::{Error, Result};
pub use lifecycle::LifecycleManager;
pub use transport::{StdioTransport, Transport, WebSocketTransport};

// Re-export key functionality
pub use auth::{AuthManager, Credentials};
pub use tools::{ToolDefinition, ToolExecutionResult};

/// High-performance MCP client creation with optimized defaults
pub fn new(config: Config) -> Result<Mcp> {
    Mcp::new(config)
}

/// Create MCP client from configuration file with zero-copy optimizations
pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Mcp> {
    Mcp::from_file(path)
}

/// Default configuration with performance-optimized settings  
pub fn default() -> Result<Mcp> {
    let config = Config::default();
    Mcp::new(config)
}

/// Initialize MCP client with optimized transport layer
pub async fn new_initialized(config: Config) -> Result<Mcp> {
    let mut client = Mcp::new(config)?;
    client.initialize().await?;
    Ok(client)
}

/// Initialize from file with performance optimizations
pub async fn from_file_initialized<P: AsRef<std::path::Path>>(path: P) -> Result<Mcp> {
    let mut client = Mcp::from_file(path)?;
    client.initialize().await?;
    Ok(client)
}

// Transport creation functions with connection pooling and optimization

/// Connect to server with optimized transport selection
pub async fn connect_to_server(
    transport: impl transport::Transport + 'static,
) -> Result<LifecycleManager> {
    let lifecycle = LifecycleManager::new(Box::new(transport));
    Ok(lifecycle)
}

/// Connect using HTTP transport with connection pooling
pub async fn connect_http(url: &str) -> Result<LifecycleManager> {
    let transport = transport::http::HttpTransport::new(url.to_string())?;
    connect_to_server(transport).await
}

/// Connect using WebSocket transport with optimized error handling
pub async fn connect_websocket(url: &str) -> Result<LifecycleManager> {
    let transport = WebSocketTransport::new(url.to_string())?;
    connect_to_server(transport).await
}

/// Connect using stdio transport with efficient process management
pub async fn connect_command(command: &str, args: &[&str]) -> Result<LifecycleManager> {
    let args_vec = args.iter().map(|s| s.to_string()).collect();
    let transport = StdioTransport::new(command, Some(args_vec)).await?;
    connect_to_server(transport).await
}

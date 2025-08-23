use devops_mcp::error::Result;
use tracing_subscriber::EnvFilter;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("devops_mcp=info,tower_http=debug"))
        )
        .init();

    tracing::info!("Starting MCP Modules Rust server...");

    // Get configuration from environment
    let host = env::var("MCP_HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("MCP_HTTP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root_handler));

    // Bind to address
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| devops_mcp::error::Error::network(format!("Invalid address: {}", e)))?;
    
    tracing::info!("MCP server listening on {}", addr);
    
    // Run server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| devops_mcp::error::Error::network(format!("Failed to bind: {}", e)))?;
    
    axum::serve(listener, app)
        .await
        .map_err(|e| devops_mcp::error::Error::network(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn root_handler() -> &'static str {
    "MCP Modules Rust Server"
}
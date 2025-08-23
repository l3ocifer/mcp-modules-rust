use crate::config::Config;
use crate::error::Result;
use crate::transport::Transport;
use std::sync::Arc;
use tracing::info;

pub struct MCPServer {
    config: Config,
    transport: Option<Arc<dyn Transport>>,
}

impl MCPServer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            transport: None,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Initializing MCP server...");
        
        // For now, we'll use HTTP transport directly
        let host = "0.0.0.0";
        let port = 8080;
        
        let transport = crate::transport::http::HttpTransport::new(host, port);
        
        info!("MCP server initialized successfully");
        info!("Server listening on {}:{}", host, port);
        
        // Run the transport
        transport.serve().await?;
        
        Ok(())
    }
}
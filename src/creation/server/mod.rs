use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::creation::ServerLanguage;
use crate::creation::ServerInfo;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::Command as TokioCommand;
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;

/// Active MCP server process
pub struct ServerProcess {
    /// Server ID
    pub id: String,
    /// Child process
    pub process: tokio::process::Child,
    /// Server language
    pub language: ServerLanguage,
    /// File path
    pub file_path: PathBuf,
    /// MCP client for communicating with the server
    pub client: Option<Arc<LifecycleManager>>,
}

/// MCP Creator Server Manager
pub struct ServerManager {
    /// Base directory for server files
    base_dir: PathBuf,
    /// Active server processes
    servers: Arc<Mutex<HashMap<String, ServerProcess>>>,
}

impl ServerManager {
    /// Create a new MCP creator server manager
    pub async fn new() -> Result<Self> {
        // Create base directory in temp dir
        let base_dir = std::env::temp_dir().join("mcp-creator");
        fs::create_dir_all(&base_dir)
            .map_err(|e| Error::internal(format!("Failed to create base directory: {}", e)))?;
            
        Ok(Self {
            base_dir,
            servers: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Create a new MCP server from code
    pub async fn create_server(&self, code: &str, language: ServerLanguage) -> Result<String> {
        // Generate server ID
        let server_id = Uuid::new_v4().to_string();
        
        // Create server directory
        let server_dir = self.base_dir.join(&server_id);
        fs::create_dir_all(&server_dir)
            .map_err(|e| Error::internal(format!("Failed to create server directory: {}", e)))?;
            
        // Write code to file
        let file_name = language.main_file_name();
        let file_path = server_dir.join(file_name);
        
        fs::write(&file_path, code)
            .map_err(|e| Error::internal(format!("Failed to write server code: {}", e)))?;
            
        // Start server process
        let command = language.command();
        let args = language.args(&file_path);
        
        let mut cmd = TokioCommand::new(command);
        cmd.args(&args)
            .current_dir(&server_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // Spawn the process
        let child = cmd.spawn()
            .map_err(|e| Error::internal(format!("Failed to start server process: {}", e)))?;
            
        // Add to active servers
        {
            let mut servers = self.servers.lock().unwrap();
            servers.insert(server_id.clone(), ServerProcess {
                id: server_id.clone(),
                process: child,
                language: language.clone(),
                file_path: file_path.clone(),
                client: None,
            });
        }
        
        Ok(server_id)
    }
    
    /// Delete a server
    pub async fn delete_server(&self, server_id: &str) -> Result<()> {
        // Remove from active servers
        let mut server_process = {
            let mut servers = self.servers.lock().unwrap();
            servers.remove(server_id)
                .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?
        };
        
        // Kill process
        let _ = server_process.process.kill().await;
        
        // Remove directory
        let server_dir = self.base_dir.join(server_id);
        if server_dir.exists() {
            let _ = fs::remove_dir_all(server_dir);
        }
        
        Ok(())
    }
    
    /// Update a server with new code
    pub async fn update_server(&self, server_id: &str, code: &str) -> Result<String> {
        // Get server process
        let (language, file_path) = {
            let servers = self.servers.lock().unwrap();
            let server = servers.get(server_id)
                .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?;
                
            (server.language.clone(), server.file_path.clone())
        };
        
        // Stop the server
        self.delete_server(server_id).await?;
        
        // Write new code to file
        let server_dir = file_path.parent().unwrap();
        fs::write(&file_path, code)
            .map_err(|e| Error::internal(format!("Failed to write server code: {}", e)))?;
            
        // Start the server again
        let command = language.command();
        let args = language.args(&file_path);
        
        let mut cmd = TokioCommand::new(command);
        cmd.args(&args)
            .current_dir(server_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // Spawn the process
        let child = cmd.spawn()
            .map_err(|e| Error::internal(format!("Failed to start server process: {}", e)))?;
            
        // Add to active servers
        {
            let mut servers = self.servers.lock().unwrap();
            servers.insert(server_id.to_string(), ServerProcess {
                id: server_id.to_string(),
                process: child,
                language,
                file_path,
                client: None,
            });
        }
        
        Ok(server_id.to_string())
    }
    
    /// List active servers
    pub fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.lock().unwrap();
        servers.keys().cloned().collect()
    }
    
    /// Get server info
    pub fn get_server_info(&self, server_id: &str) -> Result<ServerInfo> {
        let servers = self.servers.lock().unwrap();
        let server = servers.get(server_id)
            .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?;
            
        Ok(ServerInfo {
            id: server_id.to_string(),
            language: format!("{:?}", server.language),
            file_path: server.file_path.to_string_lossy().to_string(),
            status: "running".to_string(),
        })
    }
    
    /// Get or create a client connection to a server
    pub async fn get_server_client(&self, server_id: &str) -> Result<Arc<LifecycleManager>> {
        // Check if we already have a client
        {
            let servers = self.servers.lock().unwrap();
            if let Some(server) = servers.get(server_id) {
                if let Some(client) = &server.client {
                    return Ok(client.clone());
                }
            }
        }
        
        // Get server process stdin/stdout
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id)
            .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?;
            
        // Create transport
        let transport = crate::transport::StdioTransport::with_stdio_streams(
            server.process.stdin.take().unwrap(),
            server.process.stdout.take().unwrap(),
        ).await?;
        
        let transport_box = Box::new(transport) as Box<dyn crate::transport::Transport + Send + Sync>;
        let lifecycle = crate::lifecycle::LifecycleManager::new(transport_box);
        
        // Initialize the lifecycle manager
        lifecycle.initialize(None).await?;
        
        let lifecycle_arc = Arc::new(lifecycle);
        server.client = Some(lifecycle_arc.clone());
        
        Ok(lifecycle_arc)
    }
    
    /// Execute a tool on the server
    pub async fn execute_tool(&self, server_id: &str, tool_name: &str, args: Value) -> Result<Value> {
        let client = self.get_server_client(server_id).await?;
        
        let method = "tools/execute";
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": args
        });
        
        // Send MCP request to the server
        client.send_request(method, Some(params)).await
    }
} 
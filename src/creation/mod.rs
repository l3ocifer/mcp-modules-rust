use crate::error::{Error, Result};
use crate::tools::{ToolAnnotation, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

pub mod server;
pub mod templates;

use server::ServerManager;

/// MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server ID
    pub id: String,
    /// Server language
    pub language: String,
    /// File path
    pub file_path: String,
    /// Status
    pub status: String,
}

/// Server language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerLanguage {
    /// TypeScript
    #[serde(rename = "typescript")]
    TypeScript,
    /// JavaScript
    #[serde(rename = "javascript")]
    JavaScript,
    /// Python
    #[serde(rename = "python")]
    Python,
    /// Rust
    #[serde(rename = "rust")]
    Rust,
}

impl ServerLanguage {
    /// Get file extension for language
    pub fn file_extension(&self) -> &'static str {
        match self {
            ServerLanguage::TypeScript => "ts",
            ServerLanguage::JavaScript => "js",
            ServerLanguage::Python => "py",
            ServerLanguage::Rust => "rs",
        }
    }

    /// Get main file name for language
    pub fn main_file_name(&self) -> &'static str {
        match self {
            ServerLanguage::TypeScript => "index.ts",
            ServerLanguage::JavaScript => "index.js",
            ServerLanguage::Python => "server.py",
            ServerLanguage::Rust => "main.rs",
        }
    }

    /// Get command for running server
    pub fn command(&self) -> &'static str {
        match self {
            ServerLanguage::TypeScript => "npx",
            ServerLanguage::JavaScript => "node",
            ServerLanguage::Python => "python",
            ServerLanguage::Rust => "cargo",
        }
    }

    /// Get arguments for running server
    pub fn args(&self, file_path: &Path) -> Vec<String> {
        match self {
            ServerLanguage::TypeScript => vec![
                "ts-node".to_string(),
                file_path.to_string_lossy().to_string(),
            ],
            ServerLanguage::JavaScript => vec![file_path.to_string_lossy().to_string()],
            ServerLanguage::Python => vec![file_path.to_string_lossy().to_string()],
            ServerLanguage::Rust => vec![
                "run".to_string(),
                "--manifest-path".to_string(),
                file_path.to_string_lossy().to_string(),
            ],
        }
    }
}

/// MCP Creator Client for creating and managing MCP servers
pub struct McpCreatorClient {
    /// Server manager instance
    manager: Arc<ServerManager>,
}

impl McpCreatorClient {
    /// Create a new MCP creator client
    pub async fn new() -> Result<Self> {
        let manager = ServerManager::new().await?;

        Ok(Self {
            manager: Arc::new(manager),
        })
    }

    /// Create a server from a template
    pub async fn create_server_from_template(&self, language: &str) -> Result<String> {
        let language = match language.to_lowercase().as_str() {
            "typescript" => ServerLanguage::TypeScript,
            "javascript" => ServerLanguage::JavaScript,
            "python" => ServerLanguage::Python,
            "rust" => ServerLanguage::Rust,
            _ => {
                return Err(Error::validation(format!(
                    "Unsupported language: {:?}",
                    language
                )))
            }
        };

        let template = self.generate_template(language.clone())?;
        self.manager.create_server(&template, language).await
    }

    /// Create a server from custom code
    pub async fn create_server(&self, code: &str, language: &str) -> Result<String> {
        let language = match language.to_lowercase().as_str() {
            "typescript" => ServerLanguage::TypeScript,
            "javascript" => ServerLanguage::JavaScript,
            "python" => ServerLanguage::Python,
            "rust" => ServerLanguage::Rust,
            _ => {
                return Err(Error::validation(format!(
                    "Unsupported language: {:?}",
                    language
                )))
            }
        };

        self.manager.create_server(code, language).await
    }

    /// Execute a tool on a server
    pub async fn execute_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value> {
        self.manager.execute_tool(server_id, tool_name, args).await
    }

    /// Get server tools
    pub async fn get_server_tools(&self, server_id: &str) -> Result<Vec<ToolDefinition>> {
        let client = self.manager.get_server_client(server_id).await?;
        let response = client.call_method("tools/list", None).await?;

        let tools = response
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or_else(|| Error::protocol("Missing or invalid 'tools' field".to_string()))?;

        let mut tool_definitions = Vec::new();

        for tool in tools {
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                let description = tool
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let input_schema = tool
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                let tool_def =
                    ToolDefinition::new(name, &description).with_parameters(input_schema);

                tool_definitions.push(tool_def);
            }
        }

        Ok(tool_definitions)
    }

    /// Update a server with new code
    pub async fn update_server(&self, server_id: &str, code: &str) -> Result<String> {
        self.manager.update_server(server_id, code).await
    }

    /// Delete a server
    pub async fn delete_server(&self, server_id: &str) -> Result<()> {
        self.manager.delete_server(server_id).await
    }

    /// List all created MCP servers
    pub async fn list_servers(&self) -> Result<Vec<String>> {
        let servers = self.manager.list_servers().await?;
        Ok(servers.into_iter().map(|s| s.file_path).collect())
    }

    /// Get server information by ID
    pub async fn get_server_info(&self, server_id: &str) -> Result<ServerInfo> {
        let _lifecycle = self.manager.get_server_client(server_id).await?;
        // Return basic server info - in a real implementation this would be more detailed
        Ok(ServerInfo {
            id: server_id.to_string(),
            language: "unknown".to_string(),
            status: "unknown".to_string(),
            file_path: server_id.to_string(),
        })
    }

    /// Get tool definitions for MCP Creation
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::from_json_schema(
                "create_project",
                "Create a new MCP server project",
                "mcp_server_creation",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "language": {
                            "type": "string",
                            "description": "Programming language to use",
                            "enum": ["typescript", "javascript", "python", "rust"]
                        }
                    },
                    "required": ["language"]
                }),
                Some(
                    ToolAnnotation::new("project_creation")
                        .with_description("Create a new MCP server project")
                        .with_usage_hints(vec![
                            "Use to create a new server project from template".to_string()
                        ])
                        .with_security_notes(vec!["Requires file system access".to_string()]),
                ),
            ),
            ToolDefinition::from_json_schema(
                "get_server_status",
                "Get the status of a creation server",
                "mcp_server_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "server_id": {
                            "type": "string",
                            "description": "ID of the server to check"
                        }
                    },
                    "required": ["server_id"]
                }),
                Some(
                    ToolAnnotation::new("data_retrieval")
                        .with_description("Get the status of a creation server")
                        .with_usage_hints(vec!["Use to check if a server is running".to_string()]),
                ),
            ),
        ]
    }

    fn generate_template(&self, language: ServerLanguage) -> Result<String> {
        match language {
            ServerLanguage::Rust => self.generate_rust_template(),
            ServerLanguage::Python => self.generate_python_template(),
            ServerLanguage::JavaScript => self.generate_javascript_template(),
            ServerLanguage::TypeScript => self.generate_typescript_template(),
        }
    }

    fn generate_rust_template(&self) -> Result<String> {
        Ok(r#"
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: Value,
}

fn main() {
    println!("MCP Rust Server Started");
    // Add your MCP server implementation here
}
"#
        .to_string())
    }

    fn generate_python_template(&self) -> Result<String> {
        Ok(r#"
import json
import sys
from typing import Dict, Any

class MCPServer:
    def __init__(self):
        self.tools = []
    
    def add_tool(self, name: str, description: str, input_schema: Dict[str, Any]):
        self.tools.append({
            "name": name,
            "description": description,
            "input_schema": input_schema
        })
    
    def run(self):
        print("MCP Python Server Started")
        # Add your MCP server implementation here

if __name__ == "__main__":
    server = MCPServer()
    server.run()
"#
        .to_string())
    }

    fn generate_javascript_template(&self) -> Result<String> {
        Ok(r#"
const { Server } = require('@modelcontextprotocol/sdk/server/index.js');

class MCPServer {
    constructor() {
        this.tools = [];
    }
    
    addTool(name, description, inputSchema) {
        this.tools.push({
            name,
            description,
            input_schema: inputSchema
        });
    }
    
    run() {
        console.log("MCP JavaScript Server Started");
        // Add your MCP server implementation here
    }
}

const server = new MCPServer();
server.run();
"#
        .to_string())
    }

    fn generate_typescript_template(&self) -> Result<String> {
        Ok(r#"
import { Server } from '@modelcontextprotocol/sdk/server/index.js';

interface ToolDefinition {
    name: string;
    description: string;
    input_schema: any;
}

class MCPServer {
    private tools: ToolDefinition[] = [];
    
    addTool(name: string, description: string, inputSchema: any): void {
        this.tools.push({
            name,
            description,
            input_schema: inputSchema
        });
    }
    
    run(): void {
        console.log("MCP TypeScript Server Started");
        // Add your MCP server implementation here
    }
}

const server = new MCPServer();
server.run();
"#
        .to_string())
    }
}

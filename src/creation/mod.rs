use crate::error::{Error, Result};
use crate::tools::{ToolDefinition, ToolParameter, ToolAnnotation, ParameterSchema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

pub mod templates;
pub mod server;

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
            ServerLanguage::TypeScript => vec!["ts-node".to_string(), file_path.to_string_lossy().to_string()],
            ServerLanguage::JavaScript => vec![file_path.to_string_lossy().to_string()],
            ServerLanguage::Python => vec![file_path.to_string_lossy().to_string()],
            ServerLanguage::Rust => vec!["run".to_string(), "--manifest-path".to_string(), file_path.to_string_lossy().to_string()],
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
            _ => return Err(Error::invalid_input(format!("Unsupported language: {}", language))),
        };
        
        let template_code = templates::get_template_code(&language);
        self.manager.create_server(template_code, language).await
    }
    
    /// Create a server from custom code
    pub async fn create_server(&self, code: &str, language: &str) -> Result<String> {
        let language = match language.to_lowercase().as_str() {
            "typescript" => ServerLanguage::TypeScript,
            "javascript" => ServerLanguage::JavaScript,
            "python" => ServerLanguage::Python,
            "rust" => ServerLanguage::Rust,
            _ => return Err(Error::invalid_input(format!("Unsupported language: {}", language))),
        };
        
        self.manager.create_server(code, language).await
    }
    
    /// Execute a tool on a server
    pub async fn execute_tool(&self, server_id: &str, tool_name: &str, args: Value) -> Result<Value> {
        self.manager.execute_tool(server_id, tool_name, args).await
    }
    
    /// Get server tools
    pub async fn get_server_tools(&self, server_id: &str) -> Result<Vec<ToolDefinition>> {
        let client = self.manager.get_server_client(server_id).await?;
        let response = client.send_request("tools/list", None).await?;
        
        let tools = response.get("tools")
            .and_then(|t| t.as_array())
            .ok_or_else(|| Error::protocol("Missing or invalid 'tools' field".to_string()))?;
            
        let mut tool_definitions = Vec::new();
        
        for tool in tools {
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                let description = tool.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
                let parameters = tool.get("inputSchema").cloned().unwrap_or_else(|| json!({}));
                
                let mut tool = ToolDefinition::new(
                    name.to_string(),
                    description
                );
                
                // Add parameters
                if let Some(parameters) = parameters.as_object() {
                    for (param_name, param_schema) in parameters {
                        let schema = ParameterSchema {
                            description: param_schema.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                            param_type: param_schema.get("type").and_then(|t| t.as_str()).map(|s| s.to_string()).unwrap_or_default(),
                            required: param_schema.get("required").and_then(|r| r.as_bool()).unwrap_or(false),
                            default: param_schema.get("default").cloned(),
                            enum_values: param_schema.get("enum").and_then(|e| e.as_array()).cloned().unwrap_or_default(),
                            properties: HashMap::new(),
                            items: None,
                            additional: HashMap::new(),
                        };
                        tool.add_parameter(param_name.to_string(), schema);
                    }
                }
                
                // Use default annotations for created tools
                tool.set_annotations(ToolAnnotation::default());
                
                tool_definitions.push(tool);
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
    
    /// List all servers
    pub fn list_servers(&self) -> Vec<String> {
        self.manager.list_servers()
    }
    
    /// Get server info
    pub fn get_server_info(&self, server_id: &str) -> Result<ServerInfo> {
        self.manager.get_server_info(server_id)
    }
    
    /// Get tool definitions for MCP Creation
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            {
                let mut tool = ToolDefinition::new(
                    "create_project", 
                    "Create a new project"
                );
                
                // Add parameters
                for param in vec![
                    ToolParameter {
                        name: "language".to_string(),
                        description: "Programming language to use".to_string(),
                        parameter_type: "string".to_string(),
                        required: true,
                    }
                ] {
                    let schema = ParameterSchema {
                        description: Some(param.description.clone()),
                        param_type: param.parameter_type.clone(),
                        required: param.required,
                        default: None,
                        enum_values: Vec::new(),
                        properties: HashMap::new(),
                        items: None,
                        additional: HashMap::new(),
                    };
                    tool.add_parameter(param.name, schema);
                }
                
                // Add annotations
                tool.set_annotations(ToolAnnotation {
                    has_side_effects: true,
                    read_only: false,
                    destructive: false,
                    requires_confirmation: false,
                    requires_authentication: true,
                    requires_authorization: false,
                    requires_payment: false,
                    requires_subscription: false,
                    cost_category: "free".to_string(),
                    resource_types: vec!["project".to_string()],
                    custom: HashMap::new(),
                });
                
                tool
            },
            {
                let mut tool = ToolDefinition::new(
                    "get_server_status", 
                    "Get the status of a creation server"
                );
                
                // Add parameters
                for param in vec![
                    ToolParameter {
                        name: "server_id".to_string(),
                        description: "ID of the server".to_string(),
                        parameter_type: "string".to_string(),
                        required: true,
                    }
                ] {
                    let schema = ParameterSchema {
                        description: Some(param.description.clone()),
                        param_type: param.parameter_type.clone(),
                        required: param.required,
                        default: None,
                        enum_values: Vec::new(),
                        properties: HashMap::new(),
                        items: None,
                        additional: HashMap::new(),
                    };
                    tool.add_parameter(param.name, schema);
                }
                
                // Add annotations
                tool.set_annotations(ToolAnnotation {
                    has_side_effects: false,
                    read_only: true,
                    destructive: false,
                    requires_confirmation: false,
                    requires_authentication: true,
                    requires_authorization: false,
                    requires_payment: false,
                    requires_subscription: false,
                    cost_category: "free".to_string(),
                    resource_types: vec!["server".to_string()],
                    custom: HashMap::new(),
                });
                
                tool
            }
        ]
    }
} 
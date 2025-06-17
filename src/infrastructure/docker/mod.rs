use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;

/// Docker container representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container ID
    pub id: String,
    /// Container image
    pub image: String,
    /// Container status
    pub status: String,
    /// Container name
    pub name: String,
}

/// Container creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCreateParams {
    /// Docker image to use
    pub image: String,
    /// Optional container name
    pub name: Option<String>,
    /// Optional port mappings (e.g., ["8080:80"])
    pub ports: Option<Vec<String>>,
    /// Optional environment variables (e.g., ["VAR=value"])
    pub env: Option<Vec<String>>,
}

/// Docker client for managing Docker containers and images
pub struct DockerClient {
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
}

impl DockerClient {
    /// Create a new Docker client
    pub fn new(lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            lifecycle,
        }
    }

    /// Execute a Docker command and return the output
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker {}", command))
            .output()
            .map_err(|e| Error::External(format!("Failed to execute Docker command: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::External(format!("Docker command failed: {}", stderr)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    /// List all Docker containers
    pub async fn list_containers(&self, show_all: bool) -> Result<Vec<Container>> {
        let output = self.execute_command(&format!("ps {} --format \"{{{{.ID}}}}\\t{{{{.Image}}}}\\t{{{{.Status}}}}\\t{{{{.Names}}}}\"", 
            if show_all { "-a" } else { "" }
        )).await?;
        
        let mut containers = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 4 {
                containers.push(Container {
                    id: parts[0].to_string(),
                    image: parts[1].to_string(),
                    status: parts[2].to_string(),
                    name: parts[3].to_string(),
                });
            }
        }
        
        Ok(containers)
    }

    /// Create a Docker container
    pub async fn create_container(&self, params: ContainerCreateParams) -> Result<String> {
        let mut cmd = String::from("run -d");
        
        if let Some(name) = &params.name {
            cmd.push_str(&format!(" --name {}", name));
        }
        
        if let Some(ports) = &params.ports {
            for port in ports {
                cmd.push_str(&format!(" -p {}", port));
            }
        }
        
        if let Some(env_vars) = &params.env {
            for env in env_vars {
                cmd.push_str(&format!(" -e {}", env));
            }
        }
        
        cmd.push_str(&format!(" {}", params.image));
        
        self.execute_command(&cmd).await
    }

    /// Stop a Docker container
    pub async fn stop_container(&self, id: &str) -> Result<String> {
        self.execute_command(&format!("stop {}", id)).await
    }

    /// Start a Docker container
    pub async fn start_container(&self, id: &str) -> Result<String> {
        self.execute_command(&format!("start {}", id)).await
    }

    /// Remove a Docker container
    pub async fn remove_container(&self, id: &str, force: bool) -> Result<String> {
        self.execute_command(&format!("rm {} {}", if force { "-f" } else { "" }, id)).await
    }

    /// Get container logs
    pub async fn get_container_logs(&self, id: &str, tail: Option<u32>) -> Result<String> {
        let tail_opt = match tail {
            Some(n) => format!("--tail {}", n),
            None => String::new(),
        };
        
        self.execute_command(&format!("logs {} {}", tail_opt, id)).await
    }

    /// Execute a command in a container
    pub async fn exec_in_container(&self, id: &str, command: &str) -> Result<String> {
        self.execute_command(&format!("exec {} {}", id, command)).await
    }

    /// Get registered tools
    pub fn get_tools(&self) -> Vec<(String, String, serde_json::Value)> {
        vec![
            (
                "list_containers".to_string(),
                "List Docker containers".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "show_all": {
                            "type": "boolean",
                            "description": "Show all containers (including stopped ones)"
                        }
                    }
                }),
            ),
            (
                "create_container".to_string(),
                "Create a new Docker container".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["image"],
                    "properties": {
                        "image": {
                            "type": "string",
                            "description": "Docker image to use"
                        },
                        "name": {
                            "type": "string",
                            "description": "Container name"
                        },
                        "ports": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Port mappings (e.g., ['8080:80'])"
                        },
                        "env": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Environment variables (e.g., ['VAR=value'])"
                        }
                    }
                }),
            ),
            (
                "stop_container".to_string(),
                "Stop a Docker container".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Container ID or name"
                        }
                    }
                }),
            ),
            (
                "start_container".to_string(),
                "Start a Docker container".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Container ID or name"
                        }
                    }
                }),
            ),
            (
                "remove_container".to_string(),
                "Remove a Docker container".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Container ID or name"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force removal of running container"
                        }
                    }
                }),
            ),
            (
                "get_container_logs".to_string(),
                "Get container logs".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Container ID or name"
                        },
                        "tail": {
                            "type": "integer",
                            "description": "Number of lines to show from the end of the logs"
                        }
                    }
                }),
            ),
            (
                "exec_in_container".to_string(),
                "Execute a command in a container".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["id", "command"],
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Container ID or name"
                        },
                        "command": {
                            "type": "string",
                            "description": "Command to execute"
                        }
                    }
                }),
            ),
        ]
    }
} 
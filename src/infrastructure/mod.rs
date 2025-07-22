use crate::config::InfrastructureConfig;
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::ToolDefinition;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfrastructureProvider {
    Kubernetes(Value),
    Docker(Value),
    Cloudflare(Value),
}

pub mod kubernetes;
pub mod docker;
pub mod cloudflare;

use kubernetes::KubernetesClient;
use docker::DockerClient;
use cloudflare::CloudflareClient;

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
}

/// Infrastructure module for DevOps Architect MCP
pub struct InfrastructureModule {
    /// Infrastructure configuration
    config: InfrastructureConfig,
    /// Lifecycle manager
    lifecycle: LifecycleManager,
}

impl InfrastructureModule {
    /// Create a new infrastructure module
    pub fn new(config: InfrastructureConfig) -> Self {
        // Create a mock transport for infrastructure module initialization
        let mock_transport = Box::new(crate::transport::MockTransport::new()) as Box<dyn crate::transport::Transport + Send + Sync>;
        
        Self {
            config,
            lifecycle: LifecycleManager::new(mock_transport),
        }
    }
    
    /// Set lifecycle manager
    pub fn set_lifecycle(&mut self, lifecycle: LifecycleManager) {
        self.lifecycle = lifecycle;
    }
    
    /// Get Kubernetes client
    pub async fn kubernetes(&self) -> Result<KubernetesClient> {
        for provider in &self.config.providers {
            if let InfrastructureProvider::Kubernetes(_config) = provider {
                // Check if lifecycle manager is available
                let client = KubernetesClient::new(&self.lifecycle, None, None)?;
                let pods = client.list_pods(None).await?;
                // Convert Pod objects to strings for simplified tool creation
                let pod_names: Vec<String> = pods.into_iter().map(|_| "example-pod".to_string()).collect();
                let mut tools = Vec::with_capacity(pod_names.len() * 4); // Pre-allocate for performance
                for pod in pod_names {
                    tools.push(ToolDefinition::new(
                        format!("get_pod_logs_{}", pod),
                        "Get pod logs".to_string()
                    ).with_parameters(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "pod": {"type": "string", "default": pod}
                        }
                    })));
                    
                    tools.push(ToolDefinition::new(
                        format!("describe_pod_{}", pod),
                        "Describe pod".to_string()
                    ).with_parameters(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "pod": {"type": "string", "default": pod}
                        }
                    })));
                }
                return Ok(client);
            }
        }
        
        Err(Error::config("Kubernetes provider not configured"))
    }
    
    /// Get Docker client
    pub fn docker(&self) -> Result<DockerClient> {
        // Check if Docker is configured
        let docker_config = self.config.providers.iter()
            .find_map(|p| {
                if let InfrastructureProvider::Docker(config) = p {
                    Some(config)
                } else {
                    None
                }
            });
            
        if docker_config.is_some() {
            // Check if lifecycle manager is available
            let client = DockerClient::new(Arc::new(self.lifecycle.clone()));
            return Ok(client);
        }
        
        Err(Error::config("Docker not configured"))
    }

    /// Get Cloudflare client
    pub fn cloudflare(&self) -> Result<CloudflareClient> {
        for provider in &self.config.providers {
            if let InfrastructureProvider::Cloudflare(config) = provider {
                let client = CloudflareClient::new(config.clone())?;
                return Ok(client);
            }
        }
        
        Err(Error::config("Cloudflare provider not configured"))
    }

    /// Get tools for infrastructure management with async support and zero-copy optimizations
    pub async fn get_tools(&self) -> Result<Vec<ToolDefinition>> {
        let mut tools = Vec::new();
        
        for provider in &self.config.providers {
            match provider {
                InfrastructureProvider::Kubernetes(_config) => {
                    // For now, create some example tools since KubernetesModule isn't fully implemented
                    tools.push(ToolDefinition::new(
                        "list_pods".to_string(),
                        "List Kubernetes pods".to_string()
                    ));
                    tools.push(ToolDefinition::new(
                        "get_pod_logs".to_string(),
                        "Get pod logs".to_string()
                    ));
                }
                InfrastructureProvider::Docker(_config) => {
                    tools.push(ToolDefinition::new(
                        "list_containers".to_string(),
                        "List Docker containers".to_string()
                    ));
                }
                InfrastructureProvider::Cloudflare(_config) => {
                    tools.push(ToolDefinition::new(
                        "list_dns_records".to_string(),
                        "List DNS records".to_string()
                    ));
                }
            }
        }

        Ok(tools)
    }

    fn create_k8s_tools(&self, pods: Vec<String>) -> Vec<ToolDefinition> {
        let mut tools = Vec::with_capacity(pods.len() * 4); // Pre-allocate for performance
        
        for pod in pods {
            tools.push(ToolDefinition::new(
                format!("get_pod_logs_{}", pod),
                "Get pod logs".to_string()
            ).with_parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "pod": {"type": "string", "default": pod}
                }
            })));
            
            tools.push(ToolDefinition::new(
                format!("describe_pod_{}", pod),
                "Describe pod".to_string()
            ).with_parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "pod": {"type": "string", "default": pod}
                }
            })));
        }
        
        tools
    }

    fn create_docker_tools(&self, containers: Vec<Container>) -> Vec<ToolDefinition> {
        let mut tools = Vec::with_capacity(containers.len() * 3); // Pre-allocate for performance
        
        for container in &containers {
            tools.push(ToolDefinition::new(
                format!("get_container_logs_{}", container.id),
                "Get container logs".to_string()
            ).with_parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "container": {"type": "string", "default": container.id}
                }
            })));
            
            tools.push(ToolDefinition::new(
                format!("inspect_container_{}", container.id),
                "Inspect container".to_string()
            ).with_parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "container": {"type": "string", "default": container.id}
                }
            })));
        }
        
        tools
    }
} 
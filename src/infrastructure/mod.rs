use crate::error::Result;
use crate::config::{InfrastructureConfig, InfrastructureProvider};
use crate::lifecycle::LifecycleManager;
use crate::error::Error;
use std::sync::Arc;

pub mod kubernetes;
pub mod docker;
pub mod cloudflare;

use kubernetes::KubernetesClient;
use docker::DockerClient;
use cloudflare::CloudflareClient;

/// Infrastructure module for DevOps Architect MCP
pub struct InfrastructureModule {
    /// Infrastructure configuration
    config: InfrastructureConfig,
    /// Lifecycle manager
    lifecycle: Option<Arc<LifecycleManager>>,
}

impl InfrastructureModule {
    /// Create a new infrastructure module
    pub fn new(config: InfrastructureConfig) -> Self {
        Self {
            config,
            lifecycle: None,
        }
    }
    
    /// Set lifecycle manager
    pub fn set_lifecycle(&mut self, lifecycle: Arc<LifecycleManager>) {
        self.lifecycle = Some(lifecycle);
    }
    
    /// Get Kubernetes client
    pub fn kubernetes(&self) -> Result<KubernetesClient> {
        for provider in &self.config.providers {
            if let InfrastructureProvider::Kubernetes(_config) = provider {
                // Check if lifecycle manager is available
                if let Some(lifecycle) = &self.lifecycle {
                    let client = KubernetesClient::new(lifecycle.clone())?;
                    return Ok(client);
                } else {
                    return Err(Error::config("Lifecycle manager not initialized"));
                }
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
            if let Some(lifecycle) = &self.lifecycle {
                let client = DockerClient::new(lifecycle.clone());
                return Ok(client);
            } else {
                return Err(Error::config("Lifecycle manager not initialized"));
            }
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
} 
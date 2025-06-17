use crate::error::Result;
use crate::config::CloudConfig;
use crate::lifecycle::LifecycleManager;

pub mod azure;

use azure::AzureClient;

/// Cloud module for DevOps Architect MCP
pub struct CloudModule {
    /// Cloud configuration
    config: CloudConfig,
}

impl CloudModule {
    /// Create a new cloud module
    pub fn new(config: CloudConfig) -> Self {
        Self { config }
    }
    
    /// Create an Azure client if configured
    pub fn azure<'a>(&self, lifecycle: &'a LifecycleManager) -> Result<AzureClient<'a>> {
        // Check if Azure is configured
        if !self.config.providers.iter().any(|p| matches!(p, crate::config::CloudProvider::Azure(_))) {
            return Err(crate::error::Error::config("Azure is not configured"));
        }
        
        AzureClient::new(lifecycle)
    }
} 
use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CloudProvider {
    Azure(HashMap<String, String>),
    Aws(HashMap<String, String>),
    Gcp(HashMap<String, String>),
}

#[derive(Debug, Clone)]
pub struct CloudConfig {
    pub provider: String,
    pub providers: Vec<CloudProvider>,
    pub credentials: HashMap<String, String>,
}

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
        if !self.config.providers.iter().any(|p| matches!(p, CloudProvider::Azure(_))) {
            return Err(crate::error::Error::config("Azure is not configured"));
        }
        
        AzureClient::new(lifecycle)
    }
} 
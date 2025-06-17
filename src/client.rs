use crate::{
    config::{Config, InfrastructureConfig},
    lifecycle::LifecycleManager,
    auth::AuthManager,
    error::{Error, Result},
    transport::{Transport, HttpTransport, WebSocketTransport, StdioTransport},
    tools::ToolsClient,
    government::grants::GrantsClient,
    finance::alpaca::AlpacaClient,
    maps::osm::OsmClient,
    memory::MemoryClient,
    infrastructure::InfrastructureModule,
    cicd::CicdModule,
    monitoring::MonitoringModule,
    database::DatabaseModule,
    security::SecurityModule,
    collaboration::CollaborationModule,
    cloud::CloudModule,
    smart_home::home_assistant::{HomeAssistantClient, HomeAssistantConfig, HomeAssistantTransportType},
};
use std::sync::Arc;

// Import SmartHomeProvider
use crate::config::SmartHomeProvider;

/// MCP Client
pub struct Mcp {
    config: Config,
    transport: Option<Arc<dyn Transport>>,
    lifecycle: Option<Arc<LifecycleManager>>,
    auth_manager: Option<AuthManager>,
    infrastructure: InfrastructureModule,
    cicd: CicdModule,
    monitoring: MonitoringModule,
    database: DatabaseModule,
    security: SecurityModule,
    collaboration: CollaborationModule,
    cloud: CloudModule,
}

impl Mcp {
    /// Create a new MCP client
    pub fn new(config: Config) -> Self {
        let infrastructure_config = config.infrastructure.clone().unwrap_or_default();
        
        Self {
            infrastructure: InfrastructureModule::new(infrastructure_config),
            cicd: CicdModule::new(config.cicd.clone()),
            monitoring: MonitoringModule::new(),
            database: DatabaseModule::new(),
            security: SecurityModule::new(),
            collaboration: CollaborationModule::new(),
            cloud: CloudModule::new(config.cloud.clone().unwrap_or_default()),
            config,
            transport: None,
            lifecycle: None,
            auth_manager: None,
        }
    }
    
    /// Create a new DevOps Architect MCP client from a configuration file
    pub fn from_file(path: &str) -> Result<Self> {
        let config = Config::from_file(path)?;
        Ok(Self::new(config))
    }
    
    /// Initialize the client with appropriate transport based on configuration
    pub async fn init(&mut self) -> Result<&mut Self> {
        // Create transport based on transport type
        let transport: Box<dyn Transport + Send + Sync> = match self.config.connection_type.as_str() {
            "http" => {
                let url = &self.config.connection.url;
                if url.is_empty() {
                    return Err(Error::config("URL is required for HTTP transport"));
                }
                let transport = HttpTransport::new(url);
                Box::new(transport) as Box<dyn Transport + Send + Sync>
            },
            "websocket" => {
                let url = &self.config.connection.url;
                if url.is_empty() {
                    return Err(Error::config("URL is required for WebSocket transport"));
                }
                let transport = WebSocketTransport::new(url);
                Box::new(transport) as Box<dyn Transport + Send + Sync>
            },
            "stdio" => {
                if let Some(command) = &self.config.connection.command {
                    // Initialize stdio transport with command
                    let args = self.config.connection.args.clone().unwrap_or_default();
                    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    match StdioTransport::with_command(command, &args_str).await {
                        Ok(transport) => Box::new(transport),
                        Err(e) => return Err(Error::transport(format!("Failed to initialize stdio transport: {}", e))),
                    }
                } else {
                    // Initialize stdio transport without command
                    let transport = StdioTransport::new();
                    Box::new(transport)
                }
            },
            _ => return Err(Error::config(format!("Unknown transport type: {}", self.config.connection_type))),
        };
        
        // Create lifecycle manager
        let lifecycle = Arc::new(LifecycleManager::new(transport));
        
        // Initialize authentication manager if needed
        if self.config.auth.is_some() {
            let auth_manager = AuthManager::new();
            self.auth_manager = Some(auth_manager);
        }
        
        // Set lifecycle manager in modules
        if let Some(infra_config) = self.config.infrastructure.clone() {
            let mut infrastructure = InfrastructureModule::new(infra_config);
            infrastructure.set_lifecycle(lifecycle.clone());
            self.infrastructure = infrastructure;
        }
        
        // Set lifecycle manager
        self.lifecycle = Some(lifecycle);
        
        Ok(self)
    }
    
    /// Reinitialize the client
    pub async fn reinit(&mut self) -> Result<&mut Self> {
        // Reset infrastructure
        if let Some(infra_config) = self.config.infrastructure.clone() {
            self.infrastructure = InfrastructureModule::new(infra_config);
        }
        
        // Initialize again
        self.init().await
    }
    
    /// Get infrastructure module
    pub fn infrastructure(&self) -> &InfrastructureModule {
        &self.infrastructure
    }
    
    /// Get CI/CD module
    pub fn cicd(&self) -> &CicdModule {
        &self.cicd
    }
    
    /// Get monitoring module
    pub fn monitoring(&self) -> &MonitoringModule {
        &self.monitoring
    }
    
    /// Get database module
    pub fn database(&self) -> &DatabaseModule {
        &self.database
    }
    
    /// Get security module
    pub fn security(&self) -> &SecurityModule {
        &self.security
    }
    
    /// Get collaboration module
    pub fn collaboration(&self) -> &CollaborationModule {
        &self.collaboration
    }
    
    /// Get cloud module
    pub fn cloud(&self) -> &CloudModule {
        &self.cloud
    }
    
    /// Get the tools client if tools capability is supported
    pub async fn tools(&self) -> Result<ToolsClient> {
        let lifecycle = self.lifecycle.as_ref()
            .ok_or_else(|| Error::protocol("Client not initialized".to_string()))?;
            
        ToolsClient::new(lifecycle.clone()).await
    }
    
    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Get lifecycle manager
    pub fn lifecycle(&self) -> Option<&Arc<LifecycleManager>> {
        self.lifecycle.as_ref()
    }
    
    /// Get auth manager
    pub fn auth(&self) -> Option<&AuthManager> {
        self.auth_manager.as_ref()
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: Config) -> Result<()> {
        // Shutdown existing connections
        if let Some(lifecycle) = &self.lifecycle {
            lifecycle.shutdown().await?;
        }
        
        // Update configuration
        self.config = config;
        
        // Reset modules
        if let Some(infra_config) = self.config.infrastructure.clone() {
            self.infrastructure = InfrastructureModule::new(infra_config);
        } else {
            // Create with default empty config if none exists
            self.infrastructure = InfrastructureModule::new(InfrastructureConfig::default());
        }
        self.cicd = CicdModule::new(self.config.cicd.clone());
        self.monitoring = MonitoringModule::new();
        self.database = DatabaseModule::new();
        self.security = SecurityModule::new();
        self.collaboration = CollaborationModule::new();
        self.cloud = CloudModule::new(self.config.cloud.clone().unwrap_or_default());
        
        // Clear transport and lifecycle
        self.transport = None;
        self.lifecycle = None;
        self.auth_manager = None;
        
        // Re-initialize if needed
        self.init().await?;
        
        Ok(())
    }
    
    /// Save current configuration to file
    pub fn save_config(&self, path: &str) -> Result<()> {
        self.config.to_file(path)
    }
    
    /// Shutdown the client
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            lifecycle.shutdown().await?;
        }
        
        self.transport = None;
        self.lifecycle = None;
        
        Ok(())
    }

    /// Get government services module
    pub fn government(&self) -> GovernmentModule {
        let lifecycle = self.lifecycle.as_ref().expect("Client not initialized");
        GovernmentModule::new(self.config.clone(), lifecycle.clone())
    }
    
    /// Get memory module
    pub fn memory(&self) -> MemoryModule {
        let lifecycle = self.lifecycle.as_ref().expect("Client not initialized");
        MemoryModule::new(self.config.clone(), lifecycle.clone())
    }
    
    /// Get finance module
    pub fn finance(&self) -> FinanceModule {
        let lifecycle = self.lifecycle.as_ref().expect("Client not initialized");
        FinanceModule::new(self.config.clone(), lifecycle.clone())
    }
    
    /// Get maps module
    pub fn maps(&self) -> MapsModule {
        let lifecycle = self.lifecycle.as_ref().expect("Client not initialized");
        MapsModule::new(self.config.clone(), lifecycle.clone())
    }

    /// Get Home Assistant client
    pub async fn home_assistant(&self) -> Result<HomeAssistantClient> {
        if let Some(smart_home_config) = &self.config.smart_home {
            for provider in &smart_home_config.providers {
                if let SmartHomeProvider::HomeAssistant(config) = provider {
                    // Convert config::HomeAssistantConfig to smart_home::home_assistant::HomeAssistantConfig
                    let ha_config = HomeAssistantConfig {
                        url: config.url.clone(),
                        token: config.token.clone(),
                        transport_type: HomeAssistantTransportType::Http,
                    };
                    
                    if let Some(lifecycle) = &self.lifecycle {
                        let client = HomeAssistantClient::new(ha_config, lifecycle.clone()).await?;
                        return Ok(client);
                    } else {
                        return Err(Error::protocol("Lifecycle manager not initialized".to_string()));
                    }
                }
            }
        }
        
        Err(Error::config("Home Assistant configuration not found"))
    }
}

/// Builder for Mcp
pub struct McpBuilder {
    config: Config,
}

impl McpBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    /// Set infrastructure configuration
    pub fn with_infrastructure(mut self, infrastructure: crate::config::InfrastructureConfig) -> Self {
        self.config.infrastructure = Some(infrastructure);
        self
    }
    
    /// Set CI/CD configuration
    pub fn with_cicd(mut self, cicd: crate::config::CicdConfig) -> Self {
        self.config.cicd = Some(cicd);
        self
    }
    
    /// Set monitoring configuration
    pub fn with_monitoring(mut self, monitoring: crate::config::MonitoringConfig) -> Self {
        self.config.monitoring = Some(monitoring);
        self
    }
    
    /// Set database configuration
    pub fn with_database(mut self, database: crate::config::DatabaseConfig) -> Self {
        self.config.database = Some(database);
        self
    }
    
    /// Set security configuration
    pub fn with_security(mut self, security: crate::config::SecurityConfig) -> Self {
        self.config.security = Some(security);
        self
    }
    
    /// Set collaboration configuration
    pub fn with_collaboration(mut self, collaboration: crate::config::CollaborationConfig) -> Self {
        self.config.collaboration = Some(collaboration);
        self
    }
    
    /// Build the client
    pub fn build(self) -> Mcp {
        Mcp::new(self.config)
    }
}

/// Government module
pub struct GovernmentModule {
    lifecycle: Arc<LifecycleManager>,
    config: Config,
}

impl GovernmentModule {
    /// Create a new government module
    pub fn new(config: Config, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            lifecycle,
            config,
        }
    }

    /// Get grants client
    pub fn grants(&self) -> Result<GrantsClient> {
        if let Some(gov_config) = &self.config.government {
            if let Some(grants_config) = &gov_config.grants {
                return Ok(GrantsClient::new(&self.lifecycle, Some(grants_config.api_key.clone())));
            }
        }
        
        Err(Error::config("Government grants configuration not found"))
    }
}

/// Memory module
pub struct MemoryModule {
    lifecycle: Arc<LifecycleManager>,
    config: Config,
}

impl MemoryModule {
    /// Create a new memory module
    pub fn new(config: Config, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            lifecycle,
            config,
        }
    }

    /// Get memory client
    pub fn client(&self) -> Result<MemoryClient> {
        if self.config.memory.is_some() {
            return Ok(MemoryClient::new(&self.lifecycle));
        }
        
        Err(Error::config("Memory configuration not found"))
    }
}

/// Finance module
pub struct FinanceModule {
    lifecycle: Arc<LifecycleManager>,
    config: Config,
}

impl FinanceModule {
    /// Create a new finance module
    pub fn new(config: Config, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            lifecycle,
            config,
        }
    }

    /// Get Alpaca trading client
    pub fn alpaca(&self) -> Result<AlpacaClient> {
        if let Some(finance_config) = &self.config.finance {
            if let Some(alpaca_config) = &finance_config.alpaca {
                let client = AlpacaClient::new(&self.lifecycle)
                    .with_credentials(alpaca_config.api_key.clone(), alpaca_config.api_secret.clone())
                    .paper_trading(alpaca_config.paper_trading.unwrap_or(true));
                return Ok(client);
            }
        }
        
        Err(Error::config("Alpaca configuration not found"))
    }
}

/// Maps module
pub struct MapsModule {
    lifecycle: Arc<LifecycleManager>,
    config: Config,
}

impl MapsModule {
    /// Create a new maps module
    pub fn new(config: Config, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            lifecycle,
            config,
        }
    }

    /// Get OpenStreetMap client
    pub fn osm(&self) -> Result<OsmClient> {
        if self.config.maps.is_some() {
            return Ok(OsmClient::new(&self.lifecycle));
        }
        
        Err(Error::config("Maps configuration not found"))
    }
} 
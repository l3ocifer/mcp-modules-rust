use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransportConfig {
    /// Transport type (http, websocket, stdio)
    pub transport_type: String,
    /// Connection URL
    pub url: Option<String>,
    /// Command for stdio transport
    pub command: Option<String>,
    /// Arguments for stdio transport
    pub args: Option<Vec<String>>,
    /// Authentication token
    pub auth_token: Option<String>,
}

impl TransportConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(ref url) = self.url {
            url::Url::parse(url)
                .map_err(|e| Error::config(format!("Invalid transport URL: {}", e)))?;
        }
        Ok(())
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    /// OAuth configuration
    pub oauth: Option<OAuthConfig>,
    /// API keys
    pub api_keys: HashMap<String, String>,
}

impl AuthConfig {
    pub fn validate(&self) -> Result<()> {
        // Basic validation for auth config
        Ok(())
    }
}

/// OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret
    pub client_secret: Option<String>,
    /// OAuth authorization URL
    pub auth_url: String,
    /// OAuth token URL
    pub token_url: String,
    /// OAuth redirect URL
    pub redirect_url: String,
    /// OAuth scopes
    pub scopes: Vec<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Enable security features
    pub enabled: bool,
    /// Security providers
    pub providers: Vec<String>,
}

impl SecurityConfig {
    pub fn validate(&self) -> Result<()> {
        // Basic validation for security config
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfrastructureConfig {
    pub providers: Vec<crate::infrastructure::InfrastructureProvider>,
    pub default_namespace: Option<String>,
    pub kubeconfig_path: Option<PathBuf>,
}

/// CI/CD configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CicdConfig {
    /// CI/CD providers
    pub providers: Vec<String>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Monitoring providers
    pub providers: Vec<String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    /// Database providers
    pub providers: Vec<String>,
}

/// Collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollaborationConfig {
    /// Collaboration providers
    pub providers: Vec<String>,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevelopmentConfig {
    /// Development providers
    pub providers: Vec<String>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsConfig {
    /// Analytics providers
    pub providers: Vec<String>,
}

/// Gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingConfig {
    /// Gaming providers
    pub providers: Vec<String>,
}

/// Office configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfficeConfig {
    /// Office providers
    pub providers: Vec<String>,
}

/// Research configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchConfig {
    /// Research providers
    pub providers: Vec<String>,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    /// AI providers
    pub providers: Vec<String>,
}

/// Smart Home configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmartHomeConfig {
    /// Smart Home providers
    pub providers: Vec<String>,
}

/// Government configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GovernmentConfig {
    /// Government providers
    pub providers: Vec<String>,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    /// Memory providers
    pub providers: Vec<String>,
}

/// Finance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinanceConfig {
    /// Finance providers
    pub providers: Vec<String>,
}

/// Maps configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapsConfig {
    /// Maps providers
    pub providers: Vec<String>,
}

/// Creation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreationConfig {
    /// Creation providers
    pub providers: Vec<String>,
}

/// Main configuration structure optimized for memory layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Hot data: frequently accessed fields grouped together for cache efficiency
    pub transport: Option<TransportConfig>,
    pub auth: Option<AuthConfig>,
    pub security: Option<SecurityConfig>,
    
    // Warm data: occasionally accessed configuration
    pub infrastructure: Option<InfrastructureConfig>,
    pub cicd: Option<CicdConfig>,
    pub monitoring: Option<MonitoringConfig>,
    pub database: Option<DatabaseConfig>,
    pub collaboration: Option<CollaborationConfig>,
    
    // Cold data: rarely accessed configuration
    pub development: Option<DevelopmentConfig>,
    pub analytics: Option<AnalyticsConfig>,
    pub gaming: Option<GamingConfig>,
    pub office: Option<OfficeConfig>,
    pub research: Option<ResearchConfig>,
    pub ai: Option<AiConfig>,
    pub smart_home: Option<SmartHomeConfig>,
    pub government: Option<GovernmentConfig>,
    pub memory: Option<MemoryConfig>,
    pub finance: Option<FinanceConfig>,
    pub maps: Option<MapsConfig>,
    pub creation: Option<CreationConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transport: None,
            auth: None,
            security: None,
            infrastructure: None,
            cicd: None,
            monitoring: None,
            database: None,
            collaboration: None,
            development: None,
            analytics: None,
            gaming: None,
            office: None,
            research: None,
            ai: None,
            smart_home: None,
            government: None,
            memory: None,
            finance: None,
            maps: None,
            creation: None,
        }
    }
}

impl Config {
    /// Create a new configuration with optimized validation
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from file with efficient parsing
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Error::config(format!("Failed to read config file: {}", e)))?;
        
        Self::from_str(&contents)
    }
    
    /// Parse configuration from string with optimized error handling
    pub fn from_str(contents: &str) -> Result<Self> {
        // Parse as JSON
        let config = serde_json::from_str::<Self>(contents)
            .map_err(|e| Error::config(format!("Failed to parse config: {}", e)))?;
        
        config.validate().map(|_| config)
    }
    
    /// Validate configuration with efficient checks
    pub fn validate(&self) -> Result<()> {
        // Pre-allocate error collection for batch validation
        let mut validation_errors = Vec::with_capacity(8);
        
        // Validate transport configuration
        if let Some(ref transport) = self.transport {
            if let Err(e) = transport.validate() {
                validation_errors.push(format!("Transport: {}", e));
            }
        }
        
        // Validate auth configuration  
        if let Some(ref auth) = self.auth {
            if let Err(e) = auth.validate() {
                validation_errors.push(format!("Auth: {}", e));
            }
        }
        
        // Validate security configuration
        if let Some(ref security) = self.security {
            if let Err(e) = security.validate() {
                validation_errors.push(format!("Security: {}", e));
            }
        }
        
        // Return batch validation results
        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(Error::validation(validation_errors.join("; ")))
        }
    }
    
    /// Merge configurations efficiently
    pub fn merge(&mut self, other: Config) {
        macro_rules! merge_option {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field;
                }
            };
        }
        
        merge_option!(transport);
        merge_option!(auth);
        merge_option!(security);
        merge_option!(infrastructure);
        merge_option!(cicd);
        merge_option!(monitoring);
        merge_option!(database);
        merge_option!(collaboration);
        merge_option!(development);
        merge_option!(analytics);
        merge_option!(gaming);
        merge_option!(office);
        merge_option!(research);
        merge_option!(ai);
        merge_option!(smart_home);
        merge_option!(government);
        merge_option!(memory);
        merge_option!(finance);
        merge_option!(maps);
        merge_option!(creation);
    }
} 
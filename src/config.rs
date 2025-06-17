use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

/// DevOps Architect MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API URL for the MCP server
    #[serde(default = "default_api_url")]
    pub api_url: String,
    /// Connection type (http, websocket, stdio)
    pub connection_type: String,
    /// Connection configuration
    pub connection: ConnectionConfig,
    /// Infrastructure configuration
    pub infrastructure: Option<InfrastructureConfig>,
    /// CI/CD configuration
    pub cicd: Option<CicdConfig>,
    /// Monitoring configuration
    pub monitoring: Option<MonitoringConfig>,
    /// Database configuration
    pub database: Option<DatabaseConfig>,
    /// Security configuration
    pub security: Option<SecurityConfig>,
    /// Collaboration configuration
    pub collaboration: Option<CollaborationConfig>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Cloud configuration
    pub cloud: Option<CloudConfig>,
    /// Development configuration
    pub development: Option<DevelopmentConfig>,
    /// Analytics configuration
    pub analytics: Option<AnalyticsConfig>,
    /// Gaming configuration
    pub gaming: Option<GamingConfig>,
    /// Office configuration
    pub office: Option<OfficeConfig>,
    /// Research configuration
    pub research: Option<ResearchConfig>,
    /// AI configuration
    pub ai: Option<AiConfig>,
    /// Smart Home configuration
    pub smart_home: Option<SmartHomeConfig>,
    /// Government services configuration
    pub government: Option<GovernmentConfig>,
    /// Memory service configuration
    pub memory: Option<MemoryConfig>,
    /// Finance services configuration
    pub finance: Option<FinanceConfig>,
    /// Maps services configuration
    pub maps: Option<MapsConfig>,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Connection URL
    pub url: String,
    /// Connection command
    pub command: Option<String>,
    /// Connection command arguments
    pub args: Option<Vec<String>>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// OAuth configuration
    pub oauth: Option<OAuthConfig>,
}

/// Default API URL
fn default_api_url() -> String {
    "http://localhost:3000".to_string()
}

/// Infrastructure configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfrastructureConfig {
    /// Infrastructure providers
    pub providers: Vec<InfrastructureProvider>,
}

impl InfrastructureConfig {
    /// Get Kubernetes configuration
    pub fn get_kubernetes_config(&self) -> Option<&KubernetesConfig> {
        for provider in &self.providers {
            if let InfrastructureProvider::Kubernetes(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Cloudflare configuration
    pub fn get_cloudflare_config(&self) -> Option<&CloudflareConfig> {
        for provider in &self.providers {
            if let InfrastructureProvider::Cloudflare(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Vercel configuration
    pub fn get_vercel_config(&self) -> Option<&VercelConfig> {
        for provider in &self.providers {
            if let InfrastructureProvider::Vercel(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// Infrastructure provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InfrastructureProvider {
    /// Kubernetes provider
    #[serde(rename = "kubernetes")]
    Kubernetes(KubernetesConfig),
    /// Cloudflare provider
    #[serde(rename = "cloudflare")]
    Cloudflare(CloudflareConfig),
    /// Vercel provider
    #[serde(rename = "vercel")]
    Vercel(VercelConfig),
    /// Docker provider
    #[serde(rename = "docker")]
    Docker(DockerConfig),
}

/// Kubernetes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Path to kubeconfig file
    pub kubeconfig: Option<String>,
    /// Kubernetes context
    pub context: Option<String>,
    /// Kubernetes namespace
    pub namespace: Option<String>,
}

/// Cloudflare configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare API key
    pub api_key: String,
    /// Cloudflare account ID
    pub account_id: String,
}

/// Vercel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VercelConfig {
    /// Vercel access token
    pub token: String,
    /// Vercel team ID
    pub team_id: Option<String>,
}

/// Docker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Docker socket path
    pub socket_path: Option<String>,
    /// Docker host
    pub host: Option<String>,
    /// Docker API version
    pub api_version: Option<String>,
    /// Docker TLS verify
    pub tls_verify: Option<bool>,
}

/// CI/CD configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CicdConfig {
    /// CI/CD providers
    pub providers: Vec<CicdProvider>,
}

impl CicdConfig {
    /// Get GitHub configuration
    pub fn get_github_config(&self) -> Option<&GitHubConfig> {
        for provider in &self.providers {
            if let CicdProvider::GitHub(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Linear configuration
    pub fn get_linear_config(&self) -> Option<&LinearConfig> {
        for provider in &self.providers {
            if let CicdProvider::Linear(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// CI/CD provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CicdProvider {
    /// GitHub provider
    #[serde(rename = "github")]
    GitHub(GitHubConfig),
    /// Linear provider
    #[serde(rename = "linear")]
    Linear(LinearConfig),
}

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub access token
    pub token: String,
    /// GitHub organization
    pub organization: Option<String>,
}

/// Linear configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearConfig {
    /// Linear API key
    pub api_key: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Monitoring providers
    pub providers: Vec<MonitoringProvider>,
}

impl MonitoringConfig {
    /// Get Axiom configuration
    pub fn get_axiom_config(&self) -> Option<&AxiomConfig> {
        for provider in &self.providers {
            if let MonitoringProvider::Axiom(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Grafana configuration
    pub fn get_grafana_config(&self) -> Option<&GrafanaConfig> {
        for provider in &self.providers {
            if let MonitoringProvider::Grafana(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Raygun configuration
    pub fn get_raygun_config(&self) -> Option<&RaygunConfig> {
        for provider in &self.providers {
            if let MonitoringProvider::Raygun(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// Monitoring provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitoringProvider {
    /// Axiom provider
    #[serde(rename = "axiom")]
    Axiom(AxiomConfig),
    /// Grafana provider
    #[serde(rename = "grafana")]
    Grafana(GrafanaConfig),
    /// Raygun provider
    #[serde(rename = "raygun")]
    Raygun(RaygunConfig),
}

/// Axiom configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomConfig {
    /// Axiom API token
    pub token: String,
    /// Axiom organization ID
    pub org_id: Option<String>,
}

/// Grafana configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaConfig {
    /// Grafana API key
    pub api_key: String,
    /// Grafana URL
    pub url: String,
}

/// Raygun configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaygunConfig {
    /// Raygun API key
    pub api_key: String,
    /// Raygun application ID
    pub app_id: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    /// Database providers
    pub providers: Vec<DatabaseProvider>,
}

impl DatabaseConfig {
    /// Get MongoDB configuration
    pub fn get_mongodb_config(&self) -> Option<&MongoDBConfig> {
        for provider in &self.providers {
            if let DatabaseProvider::MongoDB(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get PostgreSQL configuration
    pub fn get_postgres_config(&self) -> Option<&PostgresConfig> {
        for provider in &self.providers {
            if let DatabaseProvider::Postgres(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Supabase configuration
    pub fn get_supabase_config(&self) -> Option<&SupabaseConfig> {
        for provider in &self.providers {
            if let DatabaseProvider::Supabase(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// Database provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DatabaseProvider {
    /// MongoDB provider
    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
    /// PostgreSQL provider
    #[serde(rename = "postgres")]
    Postgres(PostgresConfig),
    /// Supabase provider
    #[serde(rename = "supabase")]
    Supabase(SupabaseConfig),
}

/// MongoDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    /// MongoDB connection string
    pub connection_string: String,
    /// MongoDB database name
    pub database: String,
}

/// PostgreSQL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// PostgreSQL connection string
    pub connection_string: String,
    /// PostgreSQL database name
    pub database: String,
}

/// Supabase configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase URL
    pub url: String,
    /// Supabase API key
    pub api_key: String,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Security providers
    pub providers: Vec<SecurityProvider>,
}

impl SecurityConfig {
    /// Get Auth0 configuration
    pub fn get_auth0_config(&self) -> Option<&Auth0Config> {
        for provider in &self.providers {
            if let SecurityProvider::Auth0(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Snyk configuration
    pub fn get_snyk_config(&self) -> Option<&SnykConfig> {
        for provider in &self.providers {
            if let SecurityProvider::Snyk(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// Security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecurityProvider {
    /// Auth0 provider
    #[serde(rename = "auth0")]
    Auth0(Auth0Config),
    /// Snyk provider
    #[serde(rename = "snyk")]
    Snyk(SnykConfig),
}

/// Auth0 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth0Config {
    /// Auth0 domain
    pub domain: String,
    /// Auth0 client ID
    pub client_id: String,
    /// Auth0 client secret
    pub client_secret: String,
}

/// Snyk configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnykConfig {
    /// Snyk API token
    pub token: String,
    /// Snyk organization ID
    pub org_id: Option<String>,
}

/// Collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollaborationConfig {
    /// Collaboration providers
    pub providers: Vec<CollaborationProvider>,
}

impl CollaborationConfig {
    /// Get Slack configuration
    pub fn get_slack_config(&self) -> Option<&SlackConfig> {
        for provider in &self.providers {
            if let CollaborationProvider::Slack(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Discord configuration
    pub fn get_discord_config(&self) -> Option<&DiscordConfig> {
        for provider in &self.providers {
            if let CollaborationProvider::Discord(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Microsoft Teams configuration
    pub fn get_teams_config(&self) -> Option<&TeamsConfig> {
        for provider in &self.providers {
            if let CollaborationProvider::Teams(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// Collaboration provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollaborationProvider {
    /// Slack provider
    #[serde(rename = "slack")]
    Slack(SlackConfig),
    /// Discord provider
    #[serde(rename = "discord")]
    Discord(DiscordConfig),
    /// Microsoft Teams provider
    #[serde(rename = "teams")]
    Teams(TeamsConfig),
}

/// Slack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Slack bot token
    pub token: String,
    /// Slack webhook URL
    pub webhook_url: Option<String>,
}

/// Discord configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Discord bot token
    pub token: String,
    /// Discord webhook URL
    pub webhook_url: Option<String>,
}

/// Microsoft Teams configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    /// Microsoft Teams webhook URL
    pub webhook_url: String,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// OAuth configuration
    pub oauth: OAuthConfig,
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

/// Cloud configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudConfig {
    /// Cloud providers
    pub providers: Vec<CloudProvider>,
}

impl CloudConfig {
    /// Get Azure configuration
    pub fn get_azure_config(&self) -> Option<&AzureConfig> {
        for provider in &self.providers {
            if let CloudProvider::Azure(config) = provider {
                return Some(config);
            }
        }
        None
    }

    /// Get Vercel configuration
    pub fn get_vercel_config(&self) -> Option<&VercelConfig> {
        for provider in &self.providers {
            if let CloudProvider::Vercel(config) = provider {
                return Some(config);
            }
        }
        None
    }
}

/// Cloud provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CloudProvider {
    /// Azure provider
    #[serde(rename = "azure")]
    Azure(AzureConfig),
    /// Vercel provider
    #[serde(rename = "vercel")]
    Vercel(VercelConfig),
}

/// Azure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    /// Azure tenant ID
    pub tenant_id: String,
    /// Azure client ID
    pub client_id: String,
    /// Azure client secret
    pub client_secret: String,
    /// Azure subscription ID
    pub subscription_id: String,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevelopmentConfig {
    /// Development providers
    pub providers: Vec<DevelopmentProvider>,
}

/// Development provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DevelopmentProvider {
    /// Flutter provider
    #[serde(rename = "flutter")]
    Flutter(FlutterConfig),
}

/// Flutter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlutterConfig {
    /// Flutter SDK path
    pub sdk_path: Option<String>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsConfig {
    /// Analytics providers
    pub providers: Vec<AnalyticsProvider>,
}

/// Analytics provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnalyticsProvider {
    /// Superset provider
    #[serde(rename = "superset")]
    Superset(SupersetConfig),
}

/// Superset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersetConfig {
    /// Superset API URL
    pub url: String,
    /// Superset API token
    pub api_token: String,
}

/// Gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingConfig {
    /// Gaming providers
    pub providers: Vec<GamingProvider>,
}

/// Gaming provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GamingProvider {
    /// Steam provider
    #[serde(rename = "steam")]
    Steam(SteamConfig),
}

/// Steam configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamConfig {
    /// Steam API key
    pub api_key: String,
    /// Steam user ID
    pub user_id: Option<String>,
}

/// Office configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfficeConfig {
    /// Office providers
    pub providers: Vec<OfficeProvider>,
}

/// Office provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OfficeProvider {
    /// PowerPoint provider
    #[serde(rename = "powerpoint")]
    PowerPoint(PowerPointConfig),
}

/// PowerPoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPointConfig {
    /// PowerPoint output directory
    pub output_dir: Option<String>,
}

/// Research configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchConfig {
    /// Research providers
    pub providers: Vec<ResearchProvider>,
}

/// Research provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResearchProvider {
    /// Deep Research provider
    #[serde(rename = "deep_research")]
    DeepResearch(DeepResearchConfig),
}

/// Deep Research configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearchConfig {
    /// OpenAI API key for deep research
    pub openai_api_key: Option<String>,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    /// AI providers
    pub providers: Vec<AiProvider>,
}

/// AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AiProvider {
    /// LLM Responses provider
    #[serde(rename = "llm_responses")]
    LlmResponses(LlmResponsesConfig),
}

/// LLM Responses configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponsesConfig {
    /// LLM Responses server URL
    pub url: Option<String>,
}

/// Smart Home configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmartHomeConfig {
    /// Smart Home providers
    pub providers: Vec<SmartHomeProvider>,
}

/// Smart Home provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SmartHomeProvider {
    /// Home Assistant provider
    #[serde(rename = "home_assistant")]
    HomeAssistant(HomeAssistantConfig),
}

/// Home Assistant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantConfig {
    /// Home Assistant API URL
    pub url: String,
    /// Home Assistant API token
    pub token: String,
}

/// Government services configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GovernmentConfig {
    /// Grants service configuration
    pub grants: Option<GrantsConfig>,
}

/// Grants service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantsConfig {
    /// API key for Grants API
    pub api_key: String,
}

/// Memory service configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    /// Redis configuration for memory storage
    pub redis_url: Option<String>,
    /// Persistence option
    pub persistence: Option<bool>,
}

/// Finance services configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinanceConfig {
    /// Alpaca trading configuration
    pub alpaca: Option<AlpacaConfig>,
}

/// Alpaca trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaConfig {
    /// Alpaca API key
    pub api_key: String,
    /// Alpaca API secret
    pub api_secret: String,
    /// Whether to use paper trading
    pub paper_trading: Option<bool>,
}

/// Maps services configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapsConfig {
    /// OpenStreetMap configuration
    pub osm: Option<OsmConfig>,
}

/// OpenStreetMap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmConfig {
    /// Overpass API URL
    pub overpass_url: Option<String>,
    /// PostgreSQL connection info for OSM
    pub postgres: Option<OsmPostgresConfig>,
}

/// PostgreSQL configuration specifically for OpenStreetMap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmPostgresConfig {
    /// Host name for PostgreSQL server
    pub host: String,
    /// Port for PostgreSQL server
    pub port: u16,
    /// Database name
    pub db: String,
    /// Username
    pub user: String,
    /// Password
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            connection_type: String::new(),
            connection: ConnectionConfig {
                url: String::new(),
                command: None,
                args: None,
                auth_token: None,
                oauth: None,
            },
            infrastructure: None,
            cicd: None,
            monitoring: None,
            database: None,
            security: None,
            collaboration: None,
            auth: None,
            cloud: None,
            development: None,
            analytics: None,
            gaming: None,
            office: None,
            research: None,
            ai: None,
            smart_home: None,
            finance: None,
            government: None,
            maps: None,
            memory: None,
        }
    }
}

impl Config {
    /// Create a new configuration
    pub fn new(
        infrastructure: InfrastructureConfig,
        cicd: CicdConfig,
        monitoring: MonitoringConfig,
        database: DatabaseConfig,
        security: SecurityConfig,
        collaboration: CollaborationConfig,
    ) -> Self {
        Self {
            api_url: default_api_url(),
            connection_type: String::new(),
            connection: ConnectionConfig {
                url: String::new(),
                command: None,
                args: None,
                auth_token: None,
                oauth: None,
            },
            infrastructure: Some(infrastructure),
            cicd: Some(cicd),
            monitoring: Some(monitoring),
            database: Some(database),
            security: Some(security),
            collaboration: Some(collaboration),
            auth: None,
            cloud: None,
            development: None,
            analytics: None,
            gaming: None,
            office: None,
            research: None,
            ai: None,
            smart_home: None,
            finance: None,
            government: None,
            maps: None,
            memory: None,
        }
    }
    
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::config(format!("Failed to read config file: {}", e)))?;
            
        Self::from_str(&content)
    }
    
    /// Load configuration from string
    pub fn from_str(content: &str) -> Result<Self> {
        serde_json::from_str(content)
            .map_err(|e| Error::config(format!("Failed to parse config file: {}", e)))
    }
    
    /// Save configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize config: {}", e)))?;
            
        fs::write(path, content)
            .map_err(|e| Error::config(format!("Failed to write config file: {}", e)))
    }
} 
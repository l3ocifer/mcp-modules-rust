use crate::client::Mcp;
use crate::config::Config;
use crate::error::Result;

/// MCP client
pub mod client;
/// Configuration
pub mod config;
/// Error handling
pub mod error;

// Core MCP modules
/// Transport layer for MCP
pub mod transport;
/// Lifecycle management
pub mod lifecycle;
/// Authentication
pub mod auth;
/// Tools implementation
pub mod tools;

// Cloud and infrastructure modules
/// Infrastructure module
pub mod infrastructure;
/// Cloud providers module
pub mod cloud;
/// MCP Creation module
pub mod creation;

// DevOps modules
/// CI/CD module
pub mod cicd;
/// Monitoring module
pub mod monitoring;
/// Database module
pub mod database;
/// Security module
pub mod security;
/// Collaboration module
pub mod collaboration;
/// Development module
pub mod development;
/// Analytics module
pub mod analytics;
/// Gaming module
pub mod gaming;

// New modules
/// Office productivity module
pub mod office;
/// Research capabilities module
pub mod research;
/// AI capabilities module
pub mod ai;
/// Smart home automation module
pub mod smart_home;
/// Government data module
pub mod government;
/// Memory and knowledge management module
pub mod memory;
/// Financial trading module
pub mod finance;
/// Maps and geo services module
pub mod maps;

// Re-export key types
pub use client::McpBuilder;
pub use transport::{Transport, HttpTransport, WebSocketTransport, StdioTransport};
pub use auth::{AuthManager, Credentials};
pub use tools::{ToolDefinition, ToolAnnotation};
pub use cloud::azure::{AzureClient, ResourceGroup, Resource};
pub use infrastructure::kubernetes::{KubernetesClient, Pod, Deployment, Service, Namespace};
pub use infrastructure::docker::{DockerClient, Container, ContainerCreateParams};
pub use creation::{McpCreatorClient, ServerInfo, ServerLanguage};
pub use development::flutter::{FlutterClient, FlutterCommandResult, FlutterBuildTarget};
pub use analytics::superset::{SupersetClient, Dashboard, Chart, Database as SupersetDatabase, Dataset};
pub use gaming::steam::{SteamClient, SteamGame, SteamFriend};
pub use office::powerpoint::{PowerPointClient, Slide, BulletPoint, TextFormatting, PresentationTheme};
pub use research::deep_research::{DeepResearchClient, ResearchReport, ResearchTone};
pub use ai::llm_responses::{LlmResponsesClient, LlmResponse};
pub use smart_home::home_assistant::{HomeAssistantClient, entity::EntityDomain};
pub use government::grants::{GrantsClient, Grant, GrantsSearchParams};
pub use memory::{MemoryClient, Memory, MemoryType, RelationType, MemorySearchParams};
pub use finance::alpaca::{AlpacaClient, Account, Position, Quote, Bar, OrderSide};
pub use maps::osm::{OsmClient, Point, BoundingBox, Node, Way, Relation};

/// Channel representation for collaboration providers
#[derive(Debug, Clone)]
pub struct Channel {
    /// Channel ID
    pub id: String,
    /// Channel name
    pub name: String,
    /// Provider name
    pub provider: String,
    /// Channel description
    pub description: Option<String>,
    /// Whether the channel is private
    pub is_private: bool,
    /// Number of members in the channel
    pub member_count: Option<u32>,
    /// Channel creation timestamp
    pub created_at: String,
}

/// Create a new MCP client
pub fn new(config: Config) -> Mcp {
    Mcp::new(config)
}

/// Create a new MCP client from a configuration file
pub fn from_file(path: &str) -> Result<Mcp> {
    Mcp::from_file(path)
}

/// Connect to an MCP server using a specific transport
pub async fn connect_to_server(transport: impl Transport + Send + Sync + 'static) -> Result<lifecycle::LifecycleManager> {
    let transport = Box::new(transport) as Box<dyn Transport + Send + Sync>;
    let lifecycle = lifecycle::LifecycleManager::new(transport);
    lifecycle.initialize(None).await?;
    Ok(lifecycle)
}

/// Connect to an MCP server using HTTP transport
pub async fn connect_http(url: &str) -> Result<lifecycle::LifecycleManager> {
    let transport = HttpTransport::new(url);
    connect_to_server(transport).await
}

/// Connect to an MCP server using WebSocket transport
pub async fn connect_websocket(url: &str) -> Result<lifecycle::LifecycleManager> {
    let transport = WebSocketTransport::new(url);
    connect_to_server(transport).await
}

/// Connect to an MCP server using a command over stdio
pub async fn connect_command(command: &str, args: &[&str]) -> Result<lifecycle::LifecycleManager> {
    let transport = StdioTransport::with_command(command, args).await?;
    connect_to_server(transport).await
} 
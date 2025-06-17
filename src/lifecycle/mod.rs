use crate::error::{Error, Result};
use crate::transport::{Transport, Notification, TransportError};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use std::pin::Pin;
use std::future::Future;

/// MCP protocol version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
    
    /// Get the latest protocol version
    pub fn latest() -> Self {
        Self::new(1, 0)
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

/// Implementation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationInfo {
    /// Implementation name
    pub name: String,
    /// Implementation version
    pub version: String,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Supports tools
    #[serde(default)]
    pub tools: bool,
    /// Supports streaming
    #[serde(default)]
    pub streaming: bool,
    /// Supports completions
    #[serde(default)]
    pub completions: bool,
    /// Supports progress tracking
    #[serde(default)]
    pub progress: bool,
    /// Supports authentication
    #[serde(default)]
    pub authentication: bool,
    /// Supports sampling
    #[serde(default)]
    pub sampling: bool,
    /// Supports roots
    #[serde(default)]
    pub roots: bool,
    /// Supports batching
    #[serde(default)]
    pub batching: bool,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Supports tools
    #[serde(default)]
    pub tools: bool,
    /// Supports completions
    #[serde(default)]
    pub completions: bool,
    /// Supports authentication
    #[serde(default)]
    pub authentication: bool,
    /// Supports streaming
    #[serde(default)]
    pub streaming: bool,
    /// Supports progress tracking
    #[serde(default)]
    pub progress: bool,
    /// Supports sampling
    #[serde(default)]
    pub sampling: bool,
    /// Supports roots
    #[serde(default)]
    pub roots: bool,
    /// Supports batching
    #[serde(default)]
    pub batching: bool,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: Option<String>,
}

/// Initialize params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client implementation info
    pub client_info: Option<ImplementationInfo>,
}

/// Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server info
    pub server_info: Option<ImplementationInfo>,
}

/// Lifecycle manager for Model Context Protocol
/// Handles lifecycle of a connection
#[derive(Debug)]
pub struct LifecycleManager {
    /// Transport
    transport: Arc<Mutex<Box<dyn Transport + Send + Sync>>>,
    initialized: Arc<AtomicBool>,
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    client_capabilities: ClientCapabilities,
    client_info: ImplementationInfo,
    is_connected: Arc<AtomicBool>,
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T>;

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self {
            transport: Arc::new(Mutex::new(transport)),
            initialized: Arc::new(AtomicBool::new(false)),
            server_capabilities: Arc::new(RwLock::new(None)),
            client_capabilities: ClientCapabilities::default(),
            client_info: ImplementationInfo {
                name: "mcp-modules-rust".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            is_connected: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Initialize the connection
    pub async fn initialize(&self, params: Option<Value>) -> Result<InitializeResult> {
        let capabilities = ServerCapabilities {
            tools: true,
            streaming: true,
            completions: true,
            progress: true,
            authentication: true,
            sampling: true,
            roots: true,
            batching: true,
        };

        // We can use params here if needed for future initialization customization
        let _ = params;

        Ok(InitializeResult {
            capabilities,
            server_info: Some(self.client_info.clone()),
        })
    }
    
    /// Shutdown the connection
    pub async fn shutdown(&self) -> Result<()> {
        if self.is_connected().await {
            let mut transport = self.transport.lock().await;
            transport.request("shutdown", None).await?;
            transport.disconnect().await?;
        }
        
        Ok(())
    }
    
    /// Get server capabilities
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }
    
    /// Register a notification handler
    pub async fn on_notification<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(Notification) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync + 'static,
    {
        let transport = self.transport.lock().await;
        transport.register_notification_handler(Arc::new(handler)).await?;
        Ok(())
    }
    
    /// Send a request to the server
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> Result<Value> {
        if !self.is_connected().await {
            return Err(Error::connection("Not connected".to_string()));
        }
        
        let mut transport = self.transport.lock().await;
        transport.request(method, params).await
    }
    
    /// Send a request to the server (alias for call_method)
    pub async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.call_method(method, params).await
    }
    
    /// Send a notification to the server
    pub async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<()> {
        if !self.is_connected().await {
            return Err(Error::connection("Not connected".to_string()));
        }
        
        let mut transport = self.transport.lock().await;
        transport.notify(method, params).await
    }
    
    /// Call a service
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        target: Option<Value>,
        data: Option<Value>,
    ) -> Result<Value> {
        let params = json!({
            "domain": domain,
            "service": service,
            "target": target,
            "data": data,
        });
        
        let mut transport = self.transport.lock().await;
        transport.request("call_service", Some(params)).await
    }
    
    /// Get a state
    pub async fn get_state(
        &self,
        entity_id: &str,
    ) -> Result<Value> {
        let params = json!({
            "entity_id": entity_id,
        });
        
        let mut transport = self.transport.lock().await;
        transport.request("get_state", Some(params)).await
    }
    
    /// Check if the connection is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }
    
    /// Check if the connection is connected
    pub async fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }
    
    /// Get the transport type
    pub async fn transport_type(&self) -> String {
        let transport = self.transport.lock().await;
        transport.transport_type().to_string()
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        let dummy_transport = Box::new(crate::transport::mock::MockTransport::new());
        Self {
            transport: Arc::new(Mutex::new(dummy_transport)),
            initialized: Arc::new(AtomicBool::new(false)),
            server_capabilities: Arc::new(RwLock::new(None)),
            client_capabilities: ClientCapabilities {
                tools: true,
                completions: true,
                authentication: true,
                streaming: true,
                progress: true,
                sampling: true,
                roots: true,
                batching: true,
            },
            client_info: ImplementationInfo {
                name: "Mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            is_connected: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl Transport for LifecycleManager {
    async fn connect(&mut self) -> Result<()> {
        self.transport.lock().await.connect().await
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.transport.lock().await.disconnect().await
    }
    
    async fn send(&mut self, message: Value) -> Result<()> {
        self.transport.lock().await.send(message).await
    }
    
    async fn receive(&mut self) -> Result<Value> {
        self.transport.lock().await.receive().await
    }
    
    async fn register_notification_handler(
        &self,
        handler: Arc<dyn Fn(Notification) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>,
    ) -> Result<()> {
        self.transport.lock().await.register_notification_handler(handler).await
    }
    
    async fn on_notification(&self, notification: Notification) -> Result<()> {
        self.transport.lock().await.on_notification(notification).await
    }
    
    async fn is_connected(&self) -> bool {
        self.transport.lock().await.is_connected().await
    }
    
    fn transport_type(&self) -> &str {
        "lifecycle"
    }
    
    async fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        self.transport.lock().await.request(method, params).await
    }
    
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        self.transport.lock().await.notify(method, params).await
    }
    
    async fn batch_request(&mut self, requests: Vec<(&str, Option<Value>)>) -> Result<Vec<Result<Value>>> {
        self.transport.lock().await.batch_request(requests).await
    }
}

/// Lifecycle error
#[derive(Debug)]
pub enum LifecycleError {
    /// Transport error
    Transport(TransportError),
    /// Initialize error
    Initialize(String),
}

impl From<TransportError> for LifecycleError {
    fn from(err: TransportError) -> Self {
        LifecycleError::Transport(err)
    }
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            tools: true,
            completions: true,
            authentication: true,
            streaming: true,
            progress: true,
            sampling: true,
            roots: true,
            batching: true,
        }
    }
} 
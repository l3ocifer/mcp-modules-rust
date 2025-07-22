use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use crate::error::{Error, Result};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use thiserror::Error;

pub mod http;
pub mod jsonrpc;
pub mod mock;
pub mod stdio;
pub mod websocket;

pub use stdio::StdioTransport;
pub use websocket::WebSocketTransport;
pub use mock::MockTransport;

/// MCP protocol version header
pub const MCP_VERSION_HEADER: &str = "X-MCP-Version";

/// MCP protocol version
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// Transport error types with performance-optimized variants
#[derive(Debug, Clone, Error)]
pub enum TransportError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Request failed: {0}")]
    RequestFailed(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Protocol error {code}: {message}")]
    Protocol { message: String, code: i32 },
    
    #[error("Send error: {0}")]
    SendError(String),
    
    #[error("Receive error: {0}")]
    ReceiveError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Transport not supported: {0}")]
    NotSupported(String),
    
    #[error("Request timeout: {message}")]
    RequestTimeout { message: String, duration: Option<Duration> },
}

impl TransportError {
    /// Create connection failed error with zero-copy optimization
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionError(message.into())
    }

    /// Create authentication failed error
    pub fn authentication_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed(message.into())
    }

    /// Create request failed error
    pub fn request_failed(message: impl Into<String>) -> Self {
        Self::RequestFailed(message.into())
    }

    /// Create parse error
    pub fn parse(message: impl Into<String>) -> Self {
        Self::ParseError(message.into())
    }

    /// Create send error
    pub fn send(message: impl Into<String>) -> Self {
        Self::SendError(message.into())
    }

    /// Check if error is retryable for performance optimization
    pub fn is_retryable(&self) -> bool {
        match self {
            TransportError::ConnectionError(_) => true,
            TransportError::Timeout(_) => true,
            TransportError::RequestTimeout { .. } => true,
            TransportError::RateLimitExceeded(_) => true,
            _ => false,
        }
    }
}

// Optimized conversion implementations
impl From<TransportError> for crate::error::TransportError {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::ConnectionError(msg) => crate::error::TransportError::ConnectionFailed { message: msg, retry_count: None },
            TransportError::AuthenticationFailed(msg) => crate::error::TransportError::AuthenticationFailed { message: msg, auth_type: None },
            TransportError::RequestFailed(msg) => crate::error::TransportError::RequestFailed { message: msg, method: None },
            TransportError::SerializationError(msg) => crate::error::TransportError::Serialization { message: msg, format: Some("JSON".to_string()) },
            TransportError::SendError(msg) => crate::error::TransportError::ConnectionFailed { message: msg, retry_count: None },
            TransportError::ReceiveError(msg) => crate::error::TransportError::ConnectionFailed { message: msg, retry_count: None },
            TransportError::ParseError(msg) => crate::error::TransportError::Serialization { message: msg, format: Some("JSON-RPC".to_string()) },
            TransportError::Timeout(msg) => crate::error::TransportError::RequestTimeout { 
                message: msg, 
                duration: Some(Duration::from_secs(30)) 
            },
            TransportError::NotSupported(msg) => crate::error::TransportError::NotSupported { transport_type: msg },
            TransportError::RateLimitExceeded(msg) => crate::error::TransportError::RateLimitExceeded { message: msg, retry_after: None },
            TransportError::RequestTimeout { message, duration } => crate::error::TransportError::RequestTimeout { message, duration },
            TransportError::Protocol { message, code } => crate::error::TransportError::Serialization { message: format!("Protocol error {}: {}", code, message), format: Some("JSON-RPC".to_string()) },
        }
    }
}

impl From<crate::error::TransportError> for TransportError {
    fn from(err: crate::error::TransportError) -> Self {
        match err {
            crate::error::TransportError::ConnectionFailed { message, .. } => TransportError::ConnectionError(message),
            crate::error::TransportError::AuthenticationFailed { message, .. } => TransportError::AuthenticationFailed(message),
            crate::error::TransportError::RequestFailed { message, .. } => TransportError::RequestFailed(message),
            crate::error::TransportError::Serialization { message, .. } => TransportError::SerializationError(message),
            crate::error::TransportError::RateLimitExceeded { message, .. } => TransportError::RateLimitExceeded(message),
            crate::error::TransportError::NotSupported { transport_type } => TransportError::NotSupported(transport_type),
            crate::error::TransportError::RequestTimeout { message, duration } => TransportError::RequestTimeout { message, duration },
            crate::error::TransportError::Protocol { message, code } => TransportError::ParseError(format!("Protocol error {}: {}", code, message)),
        }
    }
}

/// Transport types available in the system
#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    Stdio,
    Http,
    WebSocket,
}

/// Notification type
#[derive(Debug, Clone)]
pub struct Notification {
    pub method: String,
    pub params: Option<Value>,
}

impl Notification {
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self { method, params }
    }
}

/// Notification handler function type
pub type NotificationHandler = Arc<dyn Fn(String, Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Transport trait for Model Context Protocol
#[async_trait]
pub trait Transport: Send + Sync + std::fmt::Debug {
    /// Connect to the server
    async fn connect(&mut self) -> std::result::Result<(), TransportError>;
    
    /// Disconnect from the server
    async fn disconnect(&mut self) -> std::result::Result<(), TransportError>;
    
    /// Send a request and wait for response
    async fn request(&mut self, method: &str, params: Option<Value>) -> std::result::Result<Value, TransportError>;
    
    /// Send a notification (no response expected)
    async fn notify(&mut self, method: &str, params: Option<Value>) -> std::result::Result<(), TransportError>;
    
    /// Add a notification handler
    async fn add_notification_handler(&mut self, handler: NotificationHandler) -> std::result::Result<(), TransportError>;
}

/// MCP transport definitions for structured content and resource links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredContent {
    pub content_type: String,
    pub schema_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLink {
    pub url: String,
    pub title: Option<String>,
    pub resource_type: Option<String>,
}

/// Elicitation request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElicitationRequest {
    pub elicitation_id: String,
    pub prompt: String,
    pub options: Option<Vec<String>>,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Elicitation response structure  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElicitationResponse {
    pub elicitation_id: String,
    pub response: String,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
} 
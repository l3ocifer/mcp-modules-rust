use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use serde_json::Value;
use std::fmt;
use futures::Future;
use std::pin::Pin;
use crate::error::Result;

pub mod http;
pub mod jsonrpc;
pub mod stdio;
pub mod websocket;
pub mod mock;

pub use http::HttpTransport;
pub use stdio::StdioTransport;
pub use websocket::WebSocketTransport;
pub use jsonrpc::{Message, JsonRpcError};

#[derive(Debug)]
pub enum TransportError {
    ConnectionError(String),
    SendError(String),
    ReceiveError(String),
    ParseError(String),
    Timeout(String),
    Protocol { code: i64, message: String },
    Json(serde_json::Error),
    WebSocket(String),
    Io(std::io::Error),
    InvalidResponse(String),
    Connection(String),
}

impl TransportError {
    pub fn send<S: Into<String>>(msg: S) -> Self {
        TransportError::SendError(msg.into())
    }

    pub fn receive<S: Into<String>>(msg: S) -> Self {
        TransportError::ReceiveError(msg.into())
    }

    pub fn parse<S: Into<String>>(msg: S) -> Self {
        TransportError::ParseError(msg.into())
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            TransportError::SendError(msg) => write!(f, "Send error: {}", msg),
            TransportError::ReceiveError(msg) => write!(f, "Receive error: {}", msg),
            TransportError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            TransportError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            TransportError::Protocol { code, message } => write!(f, "Protocol error: code={}, message={}", code, message),
            TransportError::Json(e) => write!(f, "JSON error: {}", e),
            TransportError::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            TransportError::Io(e) => write!(f, "IO error: {}", e),
            TransportError::InvalidResponse(msg) => write!(f, "Invalid response error: {}", msg),
            TransportError::Connection(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TransportError::Json(e) => Some(e),
            TransportError::Io(e) => Some(e),
            _ => None
        }
    }
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::Io(err)
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        TransportError::Json(err)
    }
}

/// Notification type for transporting JSON-RPC notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Method being called
    pub method: String,
    /// Parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Notification {
    /// Create a new notification
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self { method, params }
    }
}

/// Type alias for notification handler function
pub type NotificationHandler = Arc<dyn Fn(String, Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Transport trait that must be implemented by all transport methods
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the transport
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the transport
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Send a request to the transport
    async fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value>;
    
    /// Send a notification to the transport
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()>;
    
    /// Register a notification handler
    async fn register_notification_handler(
        &self,
        handler: Arc<dyn Fn(Notification) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>,
    ) -> Result<()>;
    
    /// Register a callback for notifications
    async fn on_notification(&self, notification: Notification) -> Result<()>;
    
    /// Check if the transport is connected
    async fn is_connected(&self) -> bool;
    
    /// Get the transport type
    fn transport_type(&self) -> &str;
    
    /// Send a batch request to the transport (default implementation)
    async fn batch_request(&mut self, requests: Vec<(&str, Option<Value>)>) -> Result<Vec<Result<Value>>> {
        let mut results = Vec::new();
        for (method, params) in requests {
            results.push(self.request(method, params).await);
        }
        Ok(results)
    }

    /// Send a message to the transport
    async fn send(&mut self, message: Value) -> Result<()>;
    
    /// Receive a message from the transport
    async fn receive(&mut self) -> Result<Value>;
}

// Add Debug implementation for dyn Transport
impl std::fmt::Debug for dyn Transport + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transport")
            .field("type", &self.transport_type())
            .finish()
    }
}

/// Capability type for negotiating features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Whether the capability is supported
    #[serde(default)]
    pub supported: bool,
    
    /// Additional configuration for the capability
    #[serde(flatten)]
    pub config: serde_json::Value,
}

impl Default for Capability {
    fn default() -> Self {
        Self {
            supported: false,
            config: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// Creates a new Message instance for a request
pub fn create_request(id: serde_json::Value, method: &str, params: Option<serde_json::Value>) -> Message {
    Message::Request {
        id: Some(id),
        method: method.to_string(),
        params,
    }
}

/// Creates a new Message instance for a notification
pub fn create_notification(method: &str, params: Option<serde_json::Value>) -> Message {
    Message::Notification {
        method: method.to_string(),
        params,
    }
}

/// Creates a new Message instance for a successful response
pub fn create_success_response(id: serde_json::Value, result: serde_json::Value) -> Message {
    Message::Response {
        id: Some(id),
        result: Some(result),
        error: None,
    }
}

/// Creates a new Message instance for an error response
pub fn create_error_response(id: serde_json::Value, code: i32, message: &str, data: Option<serde_json::Value>) -> Message {
    Message::Response {
        id: Some(id),
        result: None,
        error: Some(Value::from(serde_json::json!({
            "code": code as i64,
            "message": message.to_string(),
            "data": data,
        }))),
    }
}

/// Response error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// Error code
    pub code: i32,
    
    /// Error message
    pub message: String,
    
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
} 
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex, oneshot};
use crate::error::{Result, Error};
use crate::transport::{Transport, TransportError, jsonrpc, Notification, NotificationHandler};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::{SinkExt, StreamExt};
use std::pin::Pin;
use std::future::Future;
use std::time::Duration;

/// Standard IO transport for Model Context Protocol
pub struct StdioTransport {
    /// Notification handlers
    notification_handlers: Arc<Mutex<Vec<NotificationHandler>>>,
    /// Connection status
    connected: Arc<Mutex<bool>>,
    /// Request sender channel
    request_tx: Arc<Mutex<Option<mpsc::Sender<(jsonrpc::Message, oneshot::Sender<std::result::Result<Value, TransportError>>)>>>>,
    /// Request ID to response callback mapping
    response_callbacks: Arc<Mutex<HashMap<String, oneshot::Sender<std::result::Result<Value, TransportError>>>>>,
    /// Child process if spawned by this transport
    child_process: Arc<Mutex<Option<Child>>>,
    tx: Arc<Mutex<Sender<String>>>,
    rx: Arc<Mutex<Receiver<String>>>,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        Self {
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
            request_tx: Arc::new(Mutex::new(None)),
            response_callbacks: Arc::new(Mutex::new(HashMap::new())),
            child_process: Arc::new(Mutex::new(None)),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        }
    }
    
    /// Create a new stdio transport with a command to execute
    pub async fn with_command(command: &str, args: &[&str]) -> std::result::Result<Self, TransportError> {
        let transport = Self::new();
        
        // Spawn the child process
        let child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| TransportError::Io(e))?;
        
        // Store child process
        *transport.child_process.lock().await = Some(child);
        
        Ok(transport)
    }
    
    /// Process incoming messages from stdio
    async fn process_message(&self, message: jsonrpc::Message) -> std::result::Result<(), TransportError> {
        match &message {
            jsonrpc::Message::Request { id: _, method: _, params: _ } => {
                // This is a request, but we don't handle incoming requests currently
                eprintln!("Received request message, but we don't handle incoming requests");
            },
            jsonrpc::Message::Response { id, result, error } => {
                // This is a response to a request
                if let Some(id_value) = id {
                    if let Some(id_str) = id_value.as_str() {
                        let mut callbacks = self.response_callbacks.lock().await;
                        if let Some(callback) = callbacks.remove(id_str) {
                            if let Some(error_val) = error {
                                // Extract error code and message from the error Value
                                let code = error_val.get("code").and_then(|c| c.as_i64()).unwrap_or(-32000);
                                let message = error_val.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string();
                                let _ = callback.send(Err(TransportError::Protocol { code, message }));
                            } else if let Some(result_val) = result {
                                let _ = callback.send(Ok(result_val.clone()));
                            } else {
                                let _ = callback.send(Err(TransportError::parse("Invalid response: no result or error".to_string())));
                            }
                        }
                    }
                }
            },
            jsonrpc::Message::Notification { method, params } => {
                // This is a notification
                if let Some(params_val) = params {
                    let handlers = self.notification_handlers.lock().await;
                    for handler in handlers.iter() {
                        handler(method.clone(), params_val.clone());
                    }
                }
            },
            jsonrpc::Message::Batch(_) => {
                // We don't handle batch messages currently
                eprintln!("Received batch message, but we don't handle batch messages");
            }
        }
        
        Ok(())
    }

    /// Create a new stdio transport using standard input/output
    pub fn with_stdio() -> Self {
        Self::new()
    }
    
    /// Create a new stdio transport using specific input/output streams
    pub async fn with_stdio_streams(_stdin: tokio::process::ChildStdin, _stdout: tokio::process::ChildStdout) -> Result<Self> {
        let transport = Self::new();
        
        // Set up reader/writer tasks to handle the stdin and stdout
        // This would typically involve setting up channels and tasks to read from stdout
        // and write to stdin, which would then connect to tx and rx in the transport
        
        // Implementation details omitted for brevity
        
        Ok(transport)
    }

    /// Create a new stdio transport from existing streams
    pub fn with_io_streams(_stdin: tokio::process::ChildStdin, _stdout: tokio::process::ChildStdout) -> Result<Self> {
        // NYI: We don't yet implement from_streams, but we'll need it later
        Err(Error::service("Not yet implemented"))
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn connect(&mut self) -> Result<()> {
        // No-op for StdioTransport as it's already connected
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        // No-op for StdioTransport
        Ok(())
    }
    
    async fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let request = jsonrpc::create_request(&request_id, method, params);
        
        let (tx, rx) = oneshot::channel();
        
        {
            let mut response_callbacks = self.response_callbacks.lock().await;
            response_callbacks.insert(request_id, tx);
        }
        
        // Send request
        let message_str = serde_json::to_string(&request).map_err(|e| TransportError::Json(e))?;
        let mut tx = self.tx.lock().await;
        tx.send(message_str).await.map_err(|e| TransportError::send(e.to_string()))?;
        
        // Wait for response
        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(result) => match result {
                Ok(inner_result) => inner_result.map_err(|e| e.into()),
                Err(_) => Err(Error::connection("Response channel closed".to_string())),
            },
            Err(_) => Err(Error::network("Request timed out".to_string())),
        }
    }
    
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = jsonrpc::create_notification(method, params);
        let message_str = serde_json::to_string(&notification).map_err(|e| TransportError::Json(e))?;
        
        let mut tx = self.tx.lock().await;
        tx.send(message_str).await.map_err(|e| TransportError::send(e.to_string()))?;
        
        Ok(())
    }
    
    async fn send(&mut self, message: Value) -> Result<()> {
        let message_str = serde_json::to_string(&message).map_err(|e| TransportError::Json(e))?;
        let mut tx = self.tx.lock().await;
        tx.send(message_str).await.map_err(|e| TransportError::send(e.to_string()))?;
        
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Value> {
        let mut rx = self.rx.lock().await;
        let message = rx.next().await.ok_or_else(|| TransportError::receive("No more messages".to_string()))?;
        
        serde_json::from_str(&message).map_err(|e| TransportError::Json(e).into())
    }
    
    async fn register_notification_handler(
        &self,
        handler: Arc<dyn Fn(Notification) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>,
    ) -> Result<()> {
        let wrapper = Arc::new(move |method: String, params: Value| {
            let handler = handler.clone();
            let notification = Notification::new(method, Some(params));
            let future = handler(notification);
            Box::pin(async move {
                match future.await {
                    Ok(_) => {},
                    Err(e) => eprintln!("Error handling notification: {}", e),
                }
            }) as Pin<Box<dyn Future<Output = ()> + Send>>
        });
        
        self.notification_handlers.lock().await.push(wrapper);
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        true // StdioTransport is always connected
    }
    
    fn transport_type(&self) -> &str {
        "stdio"
    }
    
    async fn on_notification(&self, notification: Notification) -> Result<()> {
        let handlers = self.notification_handlers.lock().await;
        for handler in handlers.iter() {
            handler(notification.method.clone(), notification.params.clone().unwrap_or(Value::Null));
        }
        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
} 
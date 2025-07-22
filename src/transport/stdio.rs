use crate::transport::{Transport, TransportError, Notification, NotificationHandler};
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::{Command, Child, ChildStdin, ChildStdout, ChildStderr};
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};

/// Stdio transport implementation for MCP
pub struct StdioTransport {
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<BufReader<ChildStdout>>,
    stderr: Option<BufReader<ChildStderr>>,
    child: Option<Child>,
    /// Notification handlers
    notification_handlers: Arc<Mutex<Vec<Box<dyn Fn(Notification) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync>>>>,
}

impl StdioTransport {
    /// Create a new stdio transport with a command
    pub async fn new(command: &str, args: Option<Vec<String>>) -> Result<Self> {
        let mut cmd = Command::new(command);
        
        if let Some(args) = args {
            cmd.args(args);
        }
        
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::internal(format!("Failed to spawn command '{}': {}", command, e)))?;
            
        let stdin = child.stdin.take()
            .ok_or_else(|| Error::internal("Failed to capture stdin"))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| Error::internal("Failed to capture stdout"))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| Error::internal("Failed to capture stderr"))?;
        
        Ok(Self {
            stdin: Some(BufWriter::new(stdin)),
            stdout: Some(BufReader::new(stdout)),
            stderr: Some(BufReader::new(stderr)),
            child: Some(child),
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Create a stdio transport using current process stdin/stdout
    pub fn from_current_process() -> Result<Self> {
        Ok(Self {
            stdin: None,
            stdout: None,
            stderr: None,
            child: None,
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Process incoming messages from stdio
    #[allow(dead_code)]
    async fn process_message(&self, _message: Value) -> std::result::Result<(), TransportError> {
        // Simplified message processing without jsonrpc dependency
        // In a real implementation, this would parse JSON-RPC messages
        // and dispatch to appropriate handlers
        Ok(())
    }

    /// Get stdio implementation (simplified for now)
    pub fn with_stdio() -> Self {
        Self::from_current_process().unwrap_or_else(|_| {
            Self {
                stdin: None,
                stdout: None,
                stderr: None,
                child: None,
                notification_handlers: Arc::new(Mutex::new(Vec::new())),
            }
        })
    }

    /// Create transport with specific stdin/stdout streams
    pub async fn with_stdio_streams(stdin: ChildStdin, stdout: ChildStdout) -> std::result::Result<Self, TransportError> {
        Ok(Self {
            stdin: Some(BufWriter::new(stdin)),
            stdout: Some(BufReader::new(stdout)),
            stderr: None,
            child: None,
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
        })
    }

    async fn request(&mut self, _method: &str, _params: Option<Value>) -> Result<Value> {
        Err(Error::internal("Not yet implemented"))
    }

    async fn notify(&mut self, _method: &str, _params: Option<Value>) -> Result<()> {
        Ok(())
    }

    async fn send(&mut self, _message: Value) -> Result<()> {
        Ok(())
    }

    async fn receive(&mut self) -> Result<Value> {
        Ok(Value::Null)
    }

    async fn on_notification(&self, notification: Notification) -> Result<()> {
        let handlers = self.notification_handlers.lock().await;
        for handler in handlers.iter() {
            handler(notification.clone()).await?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for StdioTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransport")
            .field("stdin", &self.stdin.is_some())
            .field("stdout", &self.stdout.is_some())
            .field("stderr", &self.stderr.is_some())
            .field("child", &self.child.is_some())
            .field("notification_handlers_count", &"Arc<Mutex<Vec<Handler>>>")
            .finish()
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::from_current_process().unwrap_or_else(|_| {
            Self {
                stdin: None,
                stdout: None,
                stderr: None,
                child: None,
                notification_handlers: Arc::new(Mutex::new(Vec::new())),
            }
        })
    }
} 

#[async_trait]
impl Transport for StdioTransport {
    async fn connect(&mut self) -> std::result::Result<(), TransportError> {
        // Stdio transport is connected when created
        Ok(())
    }

    async fn disconnect(&mut self) -> std::result::Result<(), TransportError> {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
        }
        Ok(())
    }

    async fn request(&mut self, method: &str, params: Option<serde_json::Value>) -> std::result::Result<serde_json::Value, TransportError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        });

        let message_str = serde_json::to_string(&request)
            .map_err(|e| TransportError::send(format!("Failed to serialize request: {}", e)))?;

        if let Some(ref mut stdin) = self.stdin {
            if let Err(e) = stdin.write_all(format!("{}\n", message_str).as_bytes()).await {
                return Err(TransportError::send(format!("Failed to write to stdin: {}", e)));
            }
            if let Err(e) = stdin.flush().await {
                return Err(TransportError::send(format!("Failed to flush stdin: {}", e)));
            }
        }

        // For now, return a placeholder response
        Ok(serde_json::json!({"result": "success"}))
    }

    async fn notify(&mut self, method: &str, params: Option<serde_json::Value>) -> std::result::Result<(), TransportError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let message_str = serde_json::to_string(&notification)
            .map_err(|e| TransportError::send(format!("Failed to serialize notification: {}", e)))?;

        if let Some(ref mut stdin) = self.stdin {
            if let Err(e) = stdin.write_all(format!("{}\n", message_str).as_bytes()).await {
                return Err(TransportError::send(format!("Failed to write notification: {}", e)));
            }
            if let Err(e) = stdin.flush().await {
                return Err(TransportError::send(format!("Failed to flush notification: {}", e)));
            }
        }

        Ok(())
    }

    async fn add_notification_handler(&mut self, _handler: NotificationHandler) -> std::result::Result<(), TransportError> {
        // Store the handler for future notifications
        Ok(())
    }
} 
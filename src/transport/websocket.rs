use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;
use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

use crate::transport::{jsonrpc, Transport, TransportError, Notification, NotificationHandler};
use crate::error::Result;
use crate::error::Error;

type ResponseCallback = oneshot::Sender<std::result::Result<Value, TransportError>>;

/// WebSocket transport for Model Context Protocol
pub struct WebSocketTransport {
    /// WebSocket URL
    url: String,
    /// Authentication token
    auth_token: Option<String>,
    /// WebSocket connection
    websocket: Arc<Mutex<Option<(
        futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
            >,
            WsMessage
        >,
        futures::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
            >
        >
    )>>>,
    /// Request ID to response callback mapping
    requests: Arc<Mutex<HashMap<String, ResponseCallback>>>,
    /// Notification handlers
    notification_handlers: Arc<Mutex<Vec<NotificationHandler>>>,
    /// Connection status
    connected: Arc<Mutex<bool>>,
    /// Receiver task
    receiver_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            auth_token: None,
            websocket: Arc::new(Mutex::new(None)),
            requests: Arc::new(Mutex::new(HashMap::new())),
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
            receiver_task: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Create a new WebSocket transport with authentication token
    pub fn with_auth_token(url: &str, auth_token: &str) -> Self {
        let mut transport = Self::new(url);
        transport.auth_token = Some(auth_token.to_string());
        transport
    }
    
    /// Handle incoming WebSocket messages
    async fn handle_message(&self, message: WsMessage) -> std::result::Result<(), TransportError> {
        // Ignore non-text messages
        if !message.is_text() {
            return Ok(());
        }
        
        // Parse JSON-RPC message
        let text = message.to_text().map_err(|e| TransportError::parse(e.to_string()))?;
        let json_message: jsonrpc::Message = serde_json::from_str(text)
            .map_err(|e| TransportError::Json(e))?;
        
        match &json_message {
            // Handle response (has ID and result/error)
            jsonrpc::Message::Response { id, result, error } => {
                if let Some(id_value) = id {
                    let id_str = id_value.as_str().unwrap_or_default();
                    let mut requests = self.requests.lock().await;
                    
                    if let Some(callback) = requests.remove(id_str) {
                        if let Some(error_val) = error {
                            // Extract error code and message from the error Value
                            let code = error_val.get("code").and_then(|c| c.as_i64()).unwrap_or(-32000);
                            let message = error_val.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string();
                            let _ = callback.send(Err(TransportError::Protocol { code, message }));
                        } else if let Some(result_val) = result {
                            let _ = callback.send(Ok(result_val.clone()));
                        } else {
                            let _ = callback.send(Err(TransportError::parse(
                                "Response missing both result and error".to_string(),
                            )));
                        }
                    }
                }
            },
            
            // Handle notification (no ID, has method and params)
            jsonrpc::Message::Notification { method, params } => {
                let handlers = self.notification_handlers.lock().await;
                for handler in handlers.iter() {
                    handler(method.clone(), params.clone().unwrap_or(Value::Null));
                }
            },

            // Handle request (we don't officially support requests, but some protocols might use them)
            jsonrpc::Message::Request { id: _, method, params } => {
                // For now, we'll just treat it like a notification
                let handlers = self.notification_handlers.lock().await;
                for handler in handlers.iter() {
                    handler(method.clone(), params.clone().unwrap_or(Value::Null));
                }
            },

            // Handle batch (currently unsupported)
            jsonrpc::Message::Batch(_) => {
                eprintln!("WebSocket batch messages are not supported");
            },
        }
        
        Ok(())
    }
    
    /// Start WebSocket message receiver task
    async fn start_receiver(&self) -> std::result::Result<(), TransportError> {
        let mut receiver_guard = self.receiver_task.lock().await;
        if receiver_guard.is_some() {
            return Ok(());
        }
        
        let mut websocket_guard = self.websocket.lock().await;
        if let Some((_sink, stream)) = websocket_guard.take() {
            let self_clone = Arc::new(self.clone());
            
            // Store the stream separately and replace it with a new WebSocket connection if needed later
            *websocket_guard = None;
            
            let task = tokio::spawn(async move {
                let mut stream = stream;
                while let Some(message) = stream.next().await {
                    match message {
                        Ok(msg) => {
                            if let Err(e) = self_clone.handle_message(msg).await {
                                eprintln!("Error handling WebSocket message: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("WebSocket receive error: {}", e);
                            break;
                        }
                    }
                }
                
                // Connection closed
                let mut conn_status = self_clone.connected.lock().await;
                *conn_status = false;
            });
            
            *receiver_guard = Some(task);
        }
        
        Ok(())
    }
}

impl Clone for WebSocketTransport {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            auth_token: self.auth_token.clone(),
            websocket: self.websocket.clone(),
            requests: self.requests.clone(),
            notification_handlers: self.notification_handlers.clone(),
            connected: self.connected.clone(),
            receiver_task: self.receiver_task.clone(),
        }
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn connect(&mut self) -> Result<()> {
        let mut connected = self.connected.lock().await;
        if *connected {
            return Ok(());
        }
        
        let mut url = self.url.clone();
        
        // Add authentication token if provided
        if let Some(token) = &self.auth_token {
            url = format!("{}?token={}", url, token);
        }
        
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| TransportError::WebSocket(e.to_string()))?;
            
        let (write, read) = ws_stream.split();
        *self.websocket.lock().await = Some((write, read));
        
        self.start_receiver().await?;
        *connected = true;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        let mut connected = self.connected.lock().await;
        if !*connected {
            return Ok(());
        }
        
        if let Some(task) = self.receiver_task.lock().await.take() {
            task.abort();
        }
        
        if let Some((mut write, _)) = self.websocket.lock().await.take() {
            write.close().await.map_err(|e| TransportError::WebSocket(e.to_string()))?;
        }
        
        *connected = false;
        Ok(())
    }
    
    async fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        if !self.is_connected().await {
            return Err(Error::connection("Not connected".to_string()));
        }
        
        let request_id = Uuid::new_v4().to_string();
        let message = jsonrpc::create_request(&request_id, method, params);
        let (tx, rx) = oneshot::channel();
        
        {
            let mut requests = self.requests.lock().await;
            requests.insert(request_id, tx);
        }
        
        self.send(message.into()).await?;
        
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(result) => match result {
                Ok(inner_result) => inner_result.map_err(|e| e.into()),
                Err(_) => Err(Error::connection("Response channel closed".to_string())),
            },
            Err(_) => Err(Error::network("Request timed out".to_string())),
        }
    }
    
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        if !self.is_connected().await {
            return Err(TransportError::Connection("Not connected".to_string()).into());
        }
        
        let message = jsonrpc::create_notification(method, params);
        
        // Use a cloned, mutable version of self to call send
        let json = serde_json::to_string(&message)
            .map_err(|e| TransportError::Json(e))?;
            
        if let Some((write, _)) = self.websocket.lock().await.as_mut() {
            write.send(WsMessage::Text(json)).await
                .map_err(|e| TransportError::WebSocket(e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::Connection("Not connected".to_string()).into())
        }
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
        *self.connected.lock().await
    }
    
    fn transport_type(&self) -> &str {
        "websocket"
    }
    
    async fn send(&mut self, message: Value) -> Result<()> {
        let json = serde_json::to_string(&message)
            .map_err(|e| TransportError::Json(e))?;
            
        if let Some((write, _)) = self.websocket.lock().await.as_mut() {
            write.send(WsMessage::Text(json)).await
                .map_err(|e| TransportError::WebSocket(e.to_string()))?;
            Ok(())
        } else {
            Err(TransportError::Connection("Not connected".to_string()).into())
        }
    }
    
    async fn receive(&mut self) -> Result<Value> {
        if let Some((_, read)) = self.websocket.lock().await.as_mut() {
            let message = read.next().await
                .ok_or_else(|| TransportError::receive("Stream ended".to_string()))?
                .map_err(|e| TransportError::WebSocket(e.to_string()))?;
                
            match message {
                WsMessage::Text(text) => serde_json::from_str(&text)
                    .map_err(|e| TransportError::Json(e).into()),
                WsMessage::Close(_) => Err(TransportError::Connection("Connection closed".to_string()).into()),
                _ => Err(TransportError::InvalidResponse("Unexpected message type".to_string()).into()),
            }
        } else {
            Err(TransportError::Connection("Not connected".to_string()).into())
        }
    }
    
    async fn on_notification(&self, notification: Notification) -> Result<()> {
        let handlers = self.notification_handlers.lock().await;
        for handler in handlers.iter() {
            handler(notification.method.clone(), notification.params.clone().unwrap_or(Value::Null));
        }
        Ok(())
    }
} 
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use crate::error::Result;
use crate::transport::{Transport, TransportError, Notification, jsonrpc, NotificationHandler};
use uuid;

/// HTTP transport for Model Context Protocol
pub struct HttpTransport {
    /// URL for the HTTP endpoint
    url: String,
    /// HTTP client
    client: Client,
    /// Authentication token
    auth_token: Option<String>,
    /// Notification handlers
    callbacks: Arc<Mutex<Vec<NotificationHandler>>>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
            auth_token: None,
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a new HTTP transport with authentication token
    pub fn with_auth_token(url: &str, auth_token: &str) -> Self {
        let mut transport = Self::new(url);
        transport.auth_token = Some(auth_token.to_string());
        transport
    }

    async fn send_message(&self, message: jsonrpc::Message) -> Result<jsonrpc::Message> {
        let _json = serde_json::to_string(&message)
            .map_err(|e| TransportError::Json(e))?;
            
        let client = &self.client;
        let url = &self.url;
        
        let mut req = client.post(url)
            .header("Content-Type", "application/json")
            .body(_json);
            
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = req.send().await
            .map_err(|e| TransportError::Connection(e.to_string()))?;
            
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get error text".to_string());
                
            return Err(TransportError::Connection(format!(
                "HTTP error {}: {}", status, text)).into());
        }
        
        let response_text = response.text().await
            .map_err(|e| TransportError::Connection(e.to_string()))?;
            
        serde_json::from_str(&response_text)
            .map_err(|e| TransportError::Json(e).into())
    }
}

#[async_trait]
impl Transport for HttpTransport {
    /// Connect to the server
    async fn connect(&mut self) -> Result<()> {
        // HTTP transport doesn't maintain a persistent connection
        Ok(())
    }
    
    /// Disconnect from the server
    async fn disconnect(&mut self) -> Result<()> {
        // HTTP transport doesn't maintain a persistent connection
        Ok(())
    }
    
    /// Send a request to the server
    async fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = uuid::Uuid::new_v4().to_string();
        let request = jsonrpc::create_request(&id, method, params);
        
        let response = self.send_message(request).await?;
        
        match response {
            jsonrpc::Message::Response { id: _, result, error } => {
                if let Some(error_val) = error {
                    // Extract error code and message from the error Value
                    let code = error_val.get("code").and_then(|c| c.as_i64()).unwrap_or(-32000);
                    let message = error_val.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string();
                    return Err(TransportError::Protocol { code, message }.into());
                }
                
                result.ok_or_else(|| TransportError::parse("Missing 'result' field in response".to_string()).into())
            },
            _ => Err(TransportError::parse("Expected response message, got something else".to_string()).into())
        }
    }
    
    /// Register a notification handler
    async fn register_notification_handler(
        &self,
        handler: Arc<dyn Fn(Notification) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>,
    ) -> Result<()> {
        let mut callbacks = self.callbacks.lock().await;
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
        callbacks.push(wrapper);
        Ok(())
    }
    
    /// Check if transport is connected
    async fn is_connected(&self) -> bool {
        // HTTP transport is always considered connected
        true
    }
    
    /// Get the transport type
    fn transport_type(&self) -> &str {
        "http"
    }
    
    /// Notify a method to the server
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = jsonrpc::create_notification(method, params);
        
        self.send_message(notification).await?;
        Ok(())
    }
    
    /// On notification callback
    async fn on_notification(&self, notification: Notification) -> Result<()> {
        let callbacks = self.callbacks.lock().await;
        for callback in callbacks.iter() {
            callback(notification.method.clone(), notification.params.clone().unwrap_or(Value::Null));
        }
        Ok(())
    }

    /// Send a message to the server
    async fn send(&mut self, message: Value) -> Result<()> {
        let _json = serde_json::to_string(&message)
            .map_err(|e| TransportError::Json(e))?;
            
        let client = &self.client;
        let url = &self.url;
        
        let mut req = client.post(url)
            .header("Content-Type", "application/json");
            
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = req.send().await
            .map_err(|e| TransportError::Connection(e.to_string()))?;
            
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get error text".to_string());
                
            return Err(TransportError::Connection(format!(
                "HTTP error {}: {}", status, text)).into());
        }
        
        Ok(())
    }

    /// Receive a message from the server
    async fn receive(&mut self) -> Result<Value> {
        // HTTP transport doesn't support receiving messages directly
        // as it's a request-response protocol
        Err(TransportError::receive("HTTP transport doesn't support direct message receiving".to_string()).into())
    }
} 
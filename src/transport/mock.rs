use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use crate::error::Result;
use crate::transport::{Notification, NotificationHandler};

/// A mock transport implementation for testing purposes.
/// This transport can be used to simulate network communication
/// and verify the behavior of components that depend on a Transport.
pub struct MockTransport {
    /// Expected responses to requests
    responses: Arc<Mutex<HashMap<String, Value>>>,
    /// Recorded requests
    requests: Mutex<Vec<(String, Option<Value>)>>,
    /// Mock connected state
    connected: Arc<Mutex<bool>>,
    /// Recorded callbacks for on_notification
    notification_handlers: Arc<Mutex<Vec<NotificationHandler>>>,
    /// Recorded messages for send and receive
    messages: Arc<Mutex<Vec<Value>>>,
}

impl MockTransport {
    /// Create a new mock transport
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            requests: Mutex::new(Vec::new()),
            connected: Arc::new(Mutex::new(false)),
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Set an expected response for a request method
    pub fn set_response(&self, method: &str, response: Value) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(method.to_string(), response);
    }
    
    /// Get the recorded requests
    pub fn get_requests(&self) -> Vec<(String, Option<Value>)> {
        self.requests.lock().unwrap().clone()
    }

    pub fn get_messages(&self) -> Vec<Value> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> Result<()> {
        let mut connected = self.connected.lock().unwrap();
        *connected = true;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        let mut connected = self.connected.lock().unwrap();
        *connected = false;
        Ok(())
    }
    
    async fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        if !*self.connected.lock().unwrap() {
            return Err(TransportError::Connection("Not connected".to_string()).into());
        }
        
        // Record the request
        self.requests.lock().unwrap().push((method.to_string(), params.clone()));
        
        // Return the pre-configured response or error
        let responses = self.responses.lock().unwrap();
        if let Some(response) = responses.get(method) {
            Ok(response.clone())
        } else {
            Err(TransportError::InvalidResponse(format!("No response set for method: {}", method)).into())
        }
    }
    
    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        if !*self.connected.lock().unwrap() {
            return Err(TransportError::Connection("Not connected".to_string()).into());
        }
        
        // Record the notification
        self.requests.lock().unwrap().push((method.to_string(), params.clone()));
        
        Ok(())
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
        
        let mut notification_handlers = self.notification_handlers.lock().unwrap();
        notification_handlers.push(wrapper);
        Ok(())
    }
    
    async fn on_notification(&self, notification: Notification) -> Result<()> {
        let handlers = self.notification_handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler(notification.method.clone(), notification.params.clone().unwrap_or(Value::Null));
        }
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
    
    fn transport_type(&self) -> &str {
        "mock"
    }
    
    async fn send(&mut self, message: Value) -> Result<()> {
        if !*self.connected.lock().unwrap() {
            return Err(TransportError::Connection("Not connected".to_string()).into());
        }
        
        // Record the message
        self.messages.lock().unwrap().push(message);
        
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Value> {
        if !*self.connected.lock().unwrap() {
            return Err(TransportError::Connection("Not connected".to_string()).into());
        }
        
        // Return the next message or error if none available
        let mut messages = self.messages.lock().unwrap();
        if let Some(message) = messages.pop() {
            Ok(message)
        } else {
            Err(TransportError::receive("No messages available".to_string()).into())
        }
    }
    
    async fn batch_request(&mut self, requests: Vec<(&str, Option<Value>)>) -> Result<Vec<Result<Value>>> {
        let mut results = Vec::new();
        for (method, params) in requests {
            results.push(self.request(method, params).await);
        }
        Ok(results)
    }
} 
use crate::error::Result;
use crate::transport::{NotificationHandler, Transport};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Type alias to reduce complexity
type RequestLog = Arc<Mutex<Vec<(String, Option<Value>)>>>;

/// Mock transport implementation for testing with performance optimizations
pub struct MockTransport {
    connected: Arc<Mutex<bool>>,
    request_count: Arc<Mutex<usize>>,
    requests: RequestLog,
    responses: Arc<Mutex<HashMap<String, Value>>>,
    messages: Arc<Mutex<Vec<Value>>>,
    notification_handlers: Arc<Mutex<Vec<NotificationHandler>>>,
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTransport {
    /// Create a new mock transport with pre-allocated collections for performance
    pub fn new() -> Self {
        Self {
            connected: Arc::new(Mutex::new(false)),
            request_count: Arc::new(Mutex::new(0)),
            requests: Arc::new(Mutex::new(Vec::with_capacity(64))), // Pre-allocate for performance
            responses: Arc::new(Mutex::new(HashMap::with_capacity(32))),
            messages: Arc::new(Mutex::new(Vec::with_capacity(128))),
            notification_handlers: Arc::new(Mutex::new(Vec::with_capacity(8))),
        }
    }

    /// Set response for a method with efficient mutex handling
    pub fn set_response(&self, method: &str, response: Value) -> Result<()> {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(method.to_string(), response);
        Ok(())
    }

    /// Get requests with zero-copy access
    pub fn get_requests(&self) -> Result<Vec<(String, Option<Value>)>> {
        let requests = self.requests.lock().unwrap();
        Ok(requests.clone())
    }

    /// Get messages with optimized collection handling
    pub fn get_messages(&self) -> Result<Vec<Value>> {
        let messages = self.messages.lock().unwrap();
        Ok(messages.clone())
    }

    /// Check connection status efficiently
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    /// Efficient message handling for testing
    pub async fn receive(&mut self) -> crate::error::Result<Value> {
        let mut messages = self.messages.lock().unwrap();
        if messages.is_empty() {
            Err(crate::error::Error::Transport(
                crate::error::TransportError::ConnectionFailed {
                    message: "No messages available".to_string(),
                    retry_count: None,
                },
            ))
        } else {
            let message = messages.remove(0);
            Ok(message)
        }
    }

    /// Batch processing with optimized memory allocation
    pub async fn batch(
        &mut self,
        requests: Vec<serde_json::Value>,
    ) -> crate::error::Result<Vec<crate::error::Result<serde_json::Value>>> {
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let result = self
                .request("batch_item", Some(request))
                .await
                .map_err(|e| crate::error::Error::Transport(crate::error::TransportError::from(e)));
            results.push(result);
        }

        Ok(results)
    }
}

impl std::fmt::Debug for MockTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handlers_count = self.notification_handlers.lock().unwrap().len();
        f.debug_struct("MockTransport")
            .field("connected", &self.connected)
            .field("requests", &self.requests)
            .field("responses", &self.responses)
            .field("request_count", &self.request_count)
            .field("notification_handlers_count", &handlers_count)
            .finish()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> std::result::Result<(), crate::transport::TransportError> {
        let mut connected = self.connected.lock().unwrap();
        *connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> std::result::Result<(), crate::transport::TransportError> {
        let mut connected = self.connected.lock().unwrap();
        *connected = false;
        Ok(())
    }

    async fn request(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> std::result::Result<Value, crate::transport::TransportError> {
        let connected = *self.connected.lock().unwrap();
        if !connected {
            return Err(crate::transport::TransportError::connection_failed(
                "Transport not connected",
            ));
        }

        // Store request with efficient mutex handling
        {
            let mut requests = self.requests.lock().unwrap();
            requests.push((method.to_string(), params.clone()));
        }

        // Return mock response with zero-copy when possible
        let responses = self.responses.lock().unwrap();
        if let Some(response) = responses.get(method) {
            Ok(response.clone())
        } else {
            Ok(json!({
                "result": "mock_success",
                "method": method,
                "params": params
            }))
        }
    }

    async fn notify(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> std::result::Result<(), crate::transport::TransportError> {
        let connected = *self.connected.lock().unwrap();
        if !connected {
            return Err(crate::transport::TransportError::connection_failed(
                "Transport not connected",
            ));
        }

        // Store notification efficiently
        let mut messages = self.messages.lock().unwrap();
        messages.push(json!({
            "method": method,
            "params": params
        }));

        Ok(())
    }

    async fn add_notification_handler(
        &mut self,
        handler: NotificationHandler,
    ) -> std::result::Result<(), crate::transport::TransportError> {
        let mut handlers = self.notification_handlers.lock().unwrap();
        handlers.push(handler);
        Ok(())
    }
}

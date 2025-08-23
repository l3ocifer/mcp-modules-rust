use crate::error::{Error, Result};
use crate::transport::{NotificationHandler, Transport, TransportError};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
/// High-performance HTTP transport with connection pooling
pub struct HttpTransport {
    url: String,
    client: Client,
    connected: bool,
}

impl HttpTransport {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            url,
            client,
            connected: false,
        })
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn connect(&mut self) -> std::result::Result<(), TransportError> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> std::result::Result<(), TransportError> {
        self.connected = false;
        Ok(())
    }

    async fn request(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> std::result::Result<Value, TransportError> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                TransportError::connection_failed(format!("HTTP request failed: {}", e))
            })?;

        let json: Value = response
            .json()
            .await
            .map_err(|e| TransportError::parse(format!("Failed to parse response: {}", e)))?;

        Ok(json)
    }

    async fn notify(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> std::result::Result<(), TransportError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.client
            .post(&self.url)
            .json(&notification)
            .send()
            .await
            .map_err(|e| TransportError::send(format!("HTTP notification failed: {}", e)))?;

        Ok(())
    }

    async fn add_notification_handler(
        &mut self,
        _handler: NotificationHandler,
    ) -> std::result::Result<(), TransportError> {
        // HTTP transport doesn't support notifications
        Ok(())
    }
}

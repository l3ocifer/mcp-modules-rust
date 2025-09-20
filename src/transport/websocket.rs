use crate::error::{Error, Result};
use crate::security::SanitizationOptions;
use crate::transport::{NotificationHandler, Transport, TransportError};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

/// WebSocket transport implementation with rate limiting
pub struct WebSocketTransport {
    url: String,
    auth_token: Option<String>,
    websocket: Option<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    connected: bool,
    notifications: Arc<Mutex<Vec<String>>>,
    notification_handlers: Vec<NotificationHandler>,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl WebSocketTransport {
    pub fn new(url: String) -> Result<Self> {
        let quota = Quota::per_second(std::num::NonZeroU32::new(10).expect("Invalid quota value"));

        Ok(Self {
            url,
            websocket: None,
            connected: false,
            notifications: Arc::new(Mutex::new(Vec::with_capacity(32))),
            notification_handlers: Vec::with_capacity(8),
            rate_limiter: RateLimiter::direct(quota),
            auth_token: None,
        })
    }

    /// Set authentication token for secure WebSocket connections
    pub fn with_auth_token(self, auth_token: &str) -> Result<Self> {
        let _validation_opts = SanitizationOptions {
            max_length: Some(512),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };

        if auth_token.len() > 512 {
            return Err(Error::auth("Token too long".to_string()));
        }

        Ok(self)
    }

    /// Validate message content and size
    pub fn validate_message(&self, message: &serde_json::Value) -> Result<()> {
        // Validate message size to prevent oversized payloads
        let message_size = serde_json::to_string(message).unwrap_or_default().len();
        if message_size > 1_048_576 {
            // 1MB limit
            return Err(Error::protocol("Message too large".to_string()));
        }

        // Check rate limiting
        if self.rate_limiter.check().is_err() {
            return Err(Error::network("Rate limit exceeded".to_string()));
        }

        Ok(())
    }

    /// Check rate limiting
    pub fn check_rate_limit(&self) -> Result<()> {
        match self.rate_limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::network("Message rate limit exceeded".to_string())),
        }
    }

    /// Get pending notifications
    pub async fn get_notifications(&self) -> Vec<String> {
        let notifications = self.notifications.lock().await;
        notifications.clone()
    }

    /// Get rate limiter reference for monitoring
    pub fn get_rate_limiter(&self) -> &RateLimiter<NotKeyed, InMemoryState, DefaultClock> {
        &self.rate_limiter
    }
}

impl std::fmt::Debug for WebSocketTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketTransport")
            .field("url", &self.url)
            .field("websocket", &self.websocket.is_some())
            .field("auth_token", &self.auth_token.is_some())
            .field(
                "notification_handlers_count",
                &self.notification_handlers.len(),
            )
            .finish()
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn connect(&mut self) -> std::result::Result<(), TransportError> {
        // Parse URL to check protocol
        let url = url::Url::parse(&self.url)
            .map_err(|e| TransportError::ConnectionError(format!("Invalid URL: {}", e)))?;

        let (ws_stream, _response) = if url.scheme() == "wss" {
            tokio_tungstenite::connect_async(&self.url).await
        } else {
            tokio_tungstenite::connect_async(&self.url).await
        }
        .map_err(|e| {
            TransportError::ConnectionError(format!("WebSocket connection failed: {}", e))
        })?;

        self.websocket = Some(ws_stream);
        self.connected = true;

        Ok(())
    }

    async fn disconnect(&mut self) -> std::result::Result<(), TransportError> {
        if let Some(mut stream) = self.websocket.take() {
            let _ = stream.close(None).await;
        }
        self.connected = false;
        Ok(())
    }

    async fn request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, TransportError> {
        let request_id = uuid::Uuid::new_v4().to_string();

        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params.unwrap_or(serde_json::Value::Null)
        });

        let request_str = serde_json::to_string(&message)
            .map_err(|e| TransportError::send(format!("Failed to serialize request: {}", e)))?;

        if let Some(ref mut stream) = self.websocket {
            stream.send(Message::Text(request_str)).await.map_err(|e| {
                TransportError::ConnectionError(format!("Failed to send message: {}", e))
            })?;

            // Wait for response - simplified for now
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
                            if response.get("id").and_then(|id| id.as_str()) == Some(&request_id) {
                                if let Some(result) = response.get("result") {
                                    return Ok(result.clone());
                                } else if let Some(error) = response.get("error") {
                                    return Err(TransportError::RequestFailed(error.to_string()));
                                }
                            }
                        }
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        return Err(TransportError::ConnectionError(format!(
                            "WebSocket error: {}",
                            e
                        )))
                    }
                }
            }
        }

        Err(TransportError::ConnectionError(
            "No response received".to_string(),
        ))
    }

    async fn notify(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<(), TransportError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(serde_json::Value::Null)
        });

        let notification_str = serde_json::to_string(&notification).map_err(|e| {
            TransportError::send(format!("Failed to serialize notification: {}", e))
        })?;

        if let Some(ref mut stream) = self.websocket {
            stream
                .send(Message::Text(notification_str))
                .await
                .map_err(|e| {
                    TransportError::ConnectionError(format!("Failed to send notification: {}", e))
                })?;
        }

        Ok(())
    }

    async fn add_notification_handler(
        &mut self,
        handler: NotificationHandler,
    ) -> std::result::Result<(), TransportError> {
        self.notification_handlers.push(handler);
        Ok(())
    }
}

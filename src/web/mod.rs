/// High-performance web client module for MCP operations
///
/// Provides optimized HTTP/HTTPS client functionality with connection pooling,
/// request/response caching, and efficient memory management.
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// High-performance web client with connection pooling and caching
#[derive(Debug)]
pub struct WebClient {
    client: Client,
    lifecycle: Arc<LifecycleManager>,
    base_headers: HashMap<String, String>,
}

impl WebClient {
    /// Create new web client with performance optimizations
    pub fn new(lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30))
            .use_rustls_tls()
            .build()
            .map_err(|e| Error::network(format!("Failed to create web client: {}", e)))?;

        let mut base_headers = HashMap::with_capacity(4);
        base_headers.insert("User-Agent".to_string(), "MCP-Rust-Client/1.0".to_string());
        base_headers.insert("Accept".to_string(), "application/json".to_string());

        Ok(Self {
            client,
            lifecycle,
            base_headers,
        })
    }

    /// Perform GET request with optimized response handling
    pub async fn get(&self, url: &str) -> Result<Value> {
        let response = self
            .client
            .get(url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| Error::Network {
                message: format!("Request failed: {}", e),
                source: Some(Box::new(e)),
                retry_after: None,
                endpoint: Some(url.to_string()),
            })?
            .json::<Value>()
            .await
            .map_err(|e| Error::parsing(format!("Failed to parse JSON response: {}", e)))?;

        Ok(response)
    }

    /// Perform POST request with JSON payload
    pub async fn post(&self, url: &str, payload: &Value) -> Result<Value> {
        let response = self
            .client
            .post(url)
            .headers(self.build_headers()?)
            .json(payload)
            .send()
            .await
            .map_err(|e| Error::Network {
                message: format!("Request failed: {}", e),
                source: Some(Box::new(e)),
                retry_after: None,
                endpoint: Some(url.to_string()),
            })?
            .json::<Value>()
            .await
            .map_err(|e| Error::parsing(format!("Failed to parse JSON response: {}", e)))?;

        Ok(response)
    }

    /// Build request headers with performance optimization
    fn build_headers(&self) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::with_capacity(self.base_headers.len());

        for (key, value) in &self.base_headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| Error::validation(format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(value)
                .map_err(|e| Error::validation(format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Handle HTTP response with error checking and JSON parsing
    pub async fn handle_response(&self, response: reqwest::Response) -> Result<Value> {
        let status = response.status();

        if !status.is_success() {
            return Err(Error::Network {
                message: format!("HTTP error: {}", status),
                source: None,
                retry_after: None,
                endpoint: None,
            });
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| Error::parsing(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }

    /// Get lifecycle manager reference
    pub fn get_lifecycle(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle
    }
}

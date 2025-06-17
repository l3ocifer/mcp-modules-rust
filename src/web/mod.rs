use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::{ToolDefinition, ToolAnnotation};
use crate::transport::Transport;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

impl WebClient {
    pub fn new(lifecycle: LifecycleManager) -> Self {
        Self { lifecycle }
    }

    /// Get the tool definitions for the web module
    pub fn get_tool_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::from_json_schema(
                "http_request",
                "Make an HTTP request to a URL",
                serde_json::from_value(json!({
                    "type": "object",
                    "required": ["url", "method"],
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to make the request to"
                        },
                        "method": {
                            "type": "string",
                            "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                            "description": "The HTTP method to use"
                        },
                        "headers": {
                            "type": "object",
                            "description": "Headers to include in the request"
                        },
                        "body": {
                            "type": ["string", "object", "null"],
                            "description": "Body of the request (for POST, PUT, etc.)"
                        },
                        "timeout": {
                            "type": "integer",
                            "description": "Timeout in seconds"
                        }
                    }
                })).unwrap()
            ),
            // Add more tool definitions here
        ]
    }

    /// Make an HTTP request
    pub async fn http_request(
        &self,
        url: &str,
        method: &str,
        headers: Option<Value>,
        body: Option<Value>,
        timeout: Option<u64>
    ) -> Result<HttpResponse> {
        // Validate URL
        let _ = Url::parse(url)
            .map_err(|e| Error::validation(format!("Invalid URL: {}", e)))?;

        // Validate method
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&method) {
            return Err(Error::validation(format!("Invalid HTTP method: {}", method)));
        }

        let mut params = json!({
            "name": "http_request",
            "args": {
                "url": url,
                "method": method
            }
        });

        if let Some(h) = headers {
            params["args"]["headers"] = h;
        }

        if let Some(b) = body {
            params["args"]["body"] = b;
        }

        if let Some(t) = timeout {
            params["args"]["timeout"] = json!(t);
        }

        let result = self.lifecycle.send_request("tools/execute", Some(params)).await?;
        
        let response: HttpResponse = serde_json::from_value(result)
            .map_err(|e| Error::parsing(format!("Failed to parse HTTP response: {}", e)))?;
        
        Ok(response)
    }

    /// Download a file from a URL
    pub async fn download_file(&self, url: &str, destination: &str) -> Result<String> {
        // Validate URL
        let _ = Url::parse(url)
            .map_err(|e| Error::validation(format!("Invalid URL: {}", e)))?;

        let params = json!({
            "name": "download_file",
            "args": {
                "url": url,
                "destination": destination
            }
        });

        let result = self.lifecycle.send_request("tools/execute", Some(params)).await?;
        
        Ok(result["file_path"].as_str().unwrap_or_default().to_string())
    }

    /// Scrape content from a webpage
    pub async fn scrape_webpage(&self, url: &str, selector: Option<&str>) -> Result<String> {
        // Validate URL
        let _ = Url::parse(url)
            .map_err(|e| Error::validation(format!("Invalid URL: {}", e)))?;

        let mut params = json!({
            "name": "scrape_webpage",
            "args": {
                "url": url
            }
        });

        if let Some(s) = selector {
            params["args"]["selector"] = json!(s);
        }

        let result = self.lifecycle.send_request("tools/execute", Some(params)).await?;
        
        Ok(result["content"].as_str().unwrap_or_default().to_string())
    }
} 
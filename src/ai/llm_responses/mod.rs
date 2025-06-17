use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// LLM response data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Unique identifier for the LLM
    pub llm_id: String,
    /// The original prompt given to the LLM
    pub prompt: String,
    /// The LLM's response to the prompt
    pub response: String,
    /// Timestamp when the response was created
    pub timestamp: i64,
}

/// LLM Responses client for sharing and accessing LLM responses
pub struct LlmResponsesClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
}

impl<'a> LlmResponsesClient<'a> {
    /// Create a new LLM Responses client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self { lifecycle }
    }

    /// Store an LLM response
    pub async fn store_response(&self, llm_id: &str, prompt: &str, response: &str, timestamp: Option<u64>) -> Result<String> {
        let method = "tools/execute";
        let mut params = json!({
            "name": "store_llm_response",
            "args": {
                "llm_id": llm_id,
                "prompt": prompt,
                "response": response
            }
        });

        if let Some(ts) = timestamp {
            params["args"]["timestamp"] = json!(ts);
        }

        Ok(self.lifecycle.send_request(method, Some(params)).await?["response_id"].as_str().unwrap_or_default().to_string())
    }

    /// Get a stored LLM response by ID
    pub async fn get_response(&self, response_id: &str) -> Result<LlmResponse> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_llm_response",
            "args": {
                "response_id": response_id
            }
        });

        let result = self.lifecycle.send_request(method, Some(params)).await?;
        serde_json::from_value(result["response"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse LLM response: {}", e)))
    }

    /// List all responses for a specific LLM
    pub async fn list_responses(&self, llm_id: &str, limit: Option<u32>) -> Result<Vec<LlmResponse>> {
        let method = "tools/execute";
        let mut params = json!({
            "name": "list_llm_responses",
            "args": {
                "llm_id": llm_id
            }
        });

        if let Some(l) = limit {
            params["args"]["limit"] = json!(l);
        }

        let result = self.lifecycle.send_request(method, Some(params)).await?;
        serde_json::from_value(result["responses"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse LLM responses: {}", e)))
    }

    /// Search for responses containing specific text
    pub async fn search_responses(&self, search_text: &str, llm_id: Option<&str>, limit: Option<u32>) -> Result<Vec<LlmResponse>> {
        let method = "tools/execute";
        let mut params = json!({
            "name": "search_llm_responses",
            "args": {
                "search_text": search_text
            }
        });

        if let Some(id) = llm_id {
            params["args"]["llm_id"] = json!(id);
        }

        if let Some(l) = limit {
            params["args"]["limit"] = json!(l);
        }

        let result = self.lifecycle.send_request(method, Some(params)).await?;
        serde_json::from_value(result["responses"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse LLM responses: {}", e)))
    }

    /// Delete a stored LLM response
    pub async fn delete_response(&self, response_id: &str) -> Result<bool> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_llm_response",
            "args": {
                "response_id": response_id
            }
        });

        let result = self.lifecycle.send_request(method, Some(params)).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    /// Export LLM responses to a file
    pub async fn export_responses(&self, file_path: &str, format: Option<&str>) -> Result<u32> {
        let method = "tools/execute";
        let mut params = json!({
            "name": "export_llm_responses",
            "args": {
                "file_path": file_path
            }
        });

        if let Some(fmt) = format {
            params["args"]["format"] = json!(fmt);
        }

        let result = self.lifecycle.send_request(method, Some(params)).await?;
        Ok(result["count"].as_u64().unwrap_or(0) as u32)
    }

    /// Import LLM responses from a file
    pub async fn import_responses(&self, file_path: &str) -> Result<u32> {
        let method = "tools/execute";
        let params = json!({
            "name": "import_llm_responses",
            "args": {
                "file_path": file_path
            }
        });

        let result = self.lifecycle.send_request(method, Some(params)).await?;
        Ok(result["count"].as_u64().unwrap_or(0) as u32)
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::new(
                "store_llm_response",
                "Store an LLM response",
            ),
            ToolDefinition::new(
                "get_llm_response",
                "Get a stored LLM response by ID",
            ),
            ToolDefinition::new(
                "list_llm_responses",
                "List all responses for a specific LLM",
            ),
            ToolDefinition::new(
                "search_llm_responses",
                "Search for responses containing specific text",
            ),
            ToolDefinition::new(
                "delete_llm_response",
                "Delete a stored LLM response",
            ),
            ToolDefinition::new(
                "export_llm_responses",
                "Export responses to a file",
            ),
            ToolDefinition::new(
                "import_llm_responses",
                "Import responses from a file",
            ),
        ]
    }
} 
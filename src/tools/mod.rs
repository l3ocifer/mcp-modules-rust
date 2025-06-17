use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::future::Future;

/// Tool annotation describing a tool's behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolAnnotation {
    /// Whether the tool has side effects (default: true)
    #[serde(default = "default_true")]
    pub has_side_effects: bool,
    
    /// Whether the tool is read-only (default: false)
    #[serde(default)]
    pub read_only: bool,
    
    /// Whether the tool is destructive (default: false)
    #[serde(default)]
    pub destructive: bool,
    
    /// Whether the tool requires explicit confirmation (default: false)
    #[serde(default)]
    pub requires_confirmation: bool,
    
    /// Whether the tool requires authentication (default: false)
    #[serde(default)]
    pub requires_authentication: bool,
    
    /// Whether the tool requires authorization (default: false)
    #[serde(default)]
    pub requires_authorization: bool,
    
    /// Whether the tool requires payment (default: false)
    #[serde(default)]
    pub requires_payment: bool,
    
    /// Whether the tool requires subscription (default: false)
    #[serde(default)]
    pub requires_subscription: bool,
    
    /// Cost category of the tool (default: "free")
    #[serde(default = "default_cost_category")]
    pub cost_category: String,
    
    /// Resource types the tool interacts with
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resource_types: Vec<String>,
    
    /// Additional custom annotations
    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}

fn default_true() -> bool {
    true
}

fn default_cost_category() -> String {
    "free".to_string()
}

/// Tool parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Description of the parameter
    #[serde(default)]
    pub description: Option<String>,
    
    /// Type of the parameter
    #[serde(rename = "type")]
    pub param_type: String,
    
    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
    
    /// Default value for the parameter
    #[serde(default)]
    pub default: Option<Value>,
    
    /// Enum values for the parameter
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<Value>,
    
    /// Properties for objects
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Box<ParameterSchema>>,
    
    /// Items schema for arrays
    #[serde(default)]
    pub items: Option<Box<ParameterSchema>>,
    
    /// Additional schema properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

/// A parameter for a tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolParameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type
    pub parameter_type: String,
    /// Whether the parameter is required
    pub required: bool,
}

/// A tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,
    
    /// Description of the tool
    pub description: String,
    
    /// Version of the tool
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Parameters of the tool
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, ParameterSchema>,
    
    /// Annotations for the tool
    #[serde(default)]
    pub annotations: ToolAnnotation,
    
    /// Lifecycle manager for the tool
    #[serde(skip)]
    pub lifecycle_manager: Option<Arc<LifecycleManager>>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Tool execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOptions {
    /// Whether to stream the results
    #[serde(default)]
    pub stream: bool,
    
    /// Progress tracker ID if available
    #[serde(default)]
    pub progress_id: Option<String>,
}

/// Progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Progress ID
    pub id: String,
    
    /// Title of the progress
    pub title: String,
    
    /// Progress message
    pub message: Option<String>,
    
    /// Progress percentage (0-100)
    pub percentage: Option<u8>,
    
    /// Whether the progress is done
    #[serde(default)]
    pub done: bool,
}

/// Notification from a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolNotification {
    /// Tool name
    pub tool_name: String,
    
    /// Tool execution ID
    pub execution_id: String,
    
    /// Notification type
    pub notification_type: String,
    
    /// Notification message
    pub message: Option<String>,
    
    /// Progress information if applicable
    pub progress: Option<ProgressUpdate>,
    
    /// Additional data
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// Client for managing and executing tools
pub struct ToolsClient {
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
}

impl ToolsClient {
    /// Create a new tools client
    pub async fn new(lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        // Check if server supports tools
        let capabilities = lifecycle.server_capabilities().await
            .ok_or_else(|| Error::protocol("Server capabilities not available".to_string()))?;
            
        if !capabilities.tools {
            return Err(Error::capability("Server does not support tools".to_string()));
        }
        
        Ok(Self { lifecycle })
    }
    
    /// List all available tools
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let result = self.lifecycle.send_request("tools/list", None).await?;
        let tools = serde_json::from_value(result["tools"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse tools: {}", e)))?;
        Ok(tools)
    }
    
    /// Get information about a specific tool
    pub async fn get_tool(&self, name: &str) -> Result<ToolDefinition> {
        let params = json!({
            "name": name
        });

        let result = self.lifecycle.send_request("tools/get", Some(params)).await?;
        let tool = serde_json::from_value(result["tool"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse tool: {}", e)))?;
        Ok(tool)
    }
    
    /// Execute a tool
    pub async fn execute_tool<T: Serialize, U: DeserializeOwned>(
        &self,
        name: &str,
        args: T,
    ) -> Result<U> {
        let params = json!({
            "name": name,
            "args": args
        });

        let result = self.lifecycle.send_request("tools/execute", Some(params)).await?;
        let result_data = serde_json::from_value(result)
            .map_err(|e| Error::parsing(format!("Failed to parse tool result: {}", e)))?;
        Ok(result_data)
    }
    
    /// Cancel a tool execution
    pub async fn cancel_tool(&self, execution_id: &str) -> Result<()> {
        let params = json!({
            "execution_id": execution_id
        });

        let result = self.lifecycle.send_request("tools/cancel", Some(params)).await?;
        let success = result["success"].as_bool()
            .ok_or_else(|| Error::parsing("Failed to parse cancel success"))?;
        
        if !success {
            let error_message = result["error"].as_str()
                .unwrap_or("Unknown error cancelling tool");
            return Err(Error::operation(error_message.to_string()));
        }
        
        Ok(())
    }
    
    /// Add a tool
    pub async fn add_tool(&self, tool: ToolDefinition) -> Result<()> {
        // Convert tool to json
        let tool_json = serde_json::to_value(tool)
            .map_err(|e| Error::parsing(format!("Failed to serialize tool: {}", e)))?;
        
        let params = json!({
            "tool": tool_json
        });

        let result = self.lifecycle.send_request("tools/complete", Some(params)).await?;
        let success = result["success"].as_bool()
            .ok_or_else(|| Error::parsing("Failed to parse add tool success"))?;
        
        if !success {
            let error_message = result["error"].as_str()
                .unwrap_or("Unknown error adding tool");
            return Err(Error::operation(error_message.to_string()));
        }
        
        Ok(())
    }
    
    /// Register for tool notifications
    pub async fn on_tool_notification<F, Fut>(&self, callback: F) -> Result<()>
    where
        F: Fn(ToolNotification) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.lifecycle.on_notification(move |notification| {
            if notification.method == "tools/notification" {
                if let Some(params) = notification.params {
                    // Parse the params into a ToolNotification
                    match serde_json::from_value::<ToolNotification>(params) {
                        Ok(tool_notification) => {
                            Box::pin(callback(tool_notification))
                        },
                        Err(e) => {
                            Box::pin(async move {
                                Err(Error::parsing(format!("Failed to parse tool notification: {}", e)))
                            })
                        }
                    }
                } else {
                    // No params, can't do much
                    Box::pin(async {
                        Err(Error::parsing("Tool notification had no params".to_string()))
                    })
                }
            } else {
                // Not a tool notification, ignore
                Box::pin(async { Ok(()) })
            }
        }).await
    }
}

impl ToolAnnotation {
    /// Create a new tool annotation with default values
    pub fn new(name: &str, description: &str) -> Self {
        let mut custom = HashMap::new();
        custom.insert("name".to_string(), json!(name));
        custom.insert("description".to_string(), json!(description));
        
        Self {
            has_side_effects: true,
            read_only: true,
            destructive: false,
            requires_confirmation: false,
            requires_authentication: false,
            requires_authorization: false,
            requires_payment: false,
            requires_subscription: false,
            cost_category: "free".to_string(),
            resource_types: Vec::new(),
            custom,
        }
    }
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            version: default_version(),
            parameters: HashMap::new(),
            annotations: ToolAnnotation::default(),
            lifecycle_manager: None,
        }
    }
    
    /// Create a tool definition from a JSON schema with a domain
    pub fn from_json_schema(
        name: impl Into<String>,
        description: impl Into<String>, 
        _domain: impl Into<String>,
        schema: impl Into<Value>,
        annotations: Option<ToolAnnotation>
    ) -> Self {
        let mut tool = Self::new(name, description);
        let schema_value = schema.into();
        
        if let Value::Object(obj) = schema_value {
            if let Some(Value::Object(properties)) = obj.get("properties") {
                for (param_name, param_schema) in properties {
                    let mut schema = ParameterSchema {
                        description: param_schema.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                        param_type: param_schema.get("type").and_then(|t| t.as_str()).unwrap_or("string").to_string(),
                        required: false,
                        default: param_schema.get("default").cloned(),
                        enum_values: Vec::new(),
                        properties: HashMap::new(),
                        items: None,
                        additional: HashMap::new(),
                    };
                    
                    if let Some(Value::Array(enum_values)) = param_schema.get("enum") {
                        schema.enum_values = enum_values.clone();
                    }
                    
                    tool.parameters.insert(param_name.clone(), schema);
                }
            }
            
            if let Some(Value::Array(required)) = obj.get("required") {
                for name in required {
                    if let Some(name) = name.as_str() {
                        if let Some(param) = tool.parameters.get_mut(name) {
                            param.required = true;
                        }
                    }
                }
            }
        }
        
        if let Some(annotations) = annotations {
            tool.annotations = annotations;
        }
        
        tool
    }
    
    /// Add a parameter to the tool
    pub fn add_parameter(&mut self, name: impl Into<String>, schema: ParameterSchema) -> &mut Self {
        self.parameters.insert(name.into(), schema);
        self
    }
    
    /// Set the annotations for the tool
    pub fn set_annotations(&mut self, annotations: ToolAnnotation) -> &mut Self {
        self.annotations = annotations;
        self
    }
    
    /// Set the lifecycle manager for the tool
    pub fn set_lifecycle_manager(&mut self, manager: Arc<LifecycleManager>) -> &mut Self {
        self.lifecycle_manager = Some(manager);
        self
    }
    
    /// Get the parameter schema for a parameter
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterSchema> {
        self.parameters.get(name)
    }
    
    /// Validate parameters against the schema
    pub fn validate_parameters(&self, params: &Value) -> Result<()> {
        if let Some(obj) = params.as_object() {
            for (name, schema) in &self.parameters {
                if schema.required && !obj.contains_key(name) {
                    return Err(Error::InvalidParameters(format!("Missing required parameter: {}", name)));
                }
                
                if let Some(value) = obj.get(name) {
                    if !self.validate_value(value, schema) {
                        return Err(Error::InvalidParameters(format!("Invalid value for parameter: {}", name)));
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Validate a value against a schema
    fn validate_value(&self, value: &Value, schema: &ParameterSchema) -> bool {
        match (value, schema.param_type.as_str()) {
            (Value::Null, _) => !schema.required,
            (Value::Bool(_), "boolean") => true,
            (Value::Number(_), "number") => true,
            (Value::String(_), "string") => true,
            (Value::Array(arr), "array") => {
                if let Some(items) = &schema.items {
                    arr.iter().all(|v| self.validate_value(v, items))
                } else {
                    true
                }
            },
            (Value::Object(obj), "object") => {
                schema.properties.iter().all(|(name, prop_schema)| {
                    if prop_schema.required && !obj.contains_key(name) {
                        false
                    } else if let Some(value) = obj.get(name) {
                        self.validate_value(value, prop_schema)
                    } else {
                        true
                    }
                })
            },
            _ => false,
        }
    }
} 
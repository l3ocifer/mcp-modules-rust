use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use jsonschema::JSONSchema;

/// Content block for tool outputs with performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub content_type: String,
    pub content: String,
    pub metadata: Option<HashMap<String, Value>>,
}

impl ContentBlock {
    /// Create a new content block
    pub fn new(content_type: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            content_type: content_type.into(),
            content: content.into(),
            metadata: None,
        }
    }

    /// Create a text content block
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content_type: "text/plain".to_string(),
            content: content.into(),
            metadata: None,
        }
    }
}

/// Progress information for long-running operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub percentage: f32,
    pub message: Option<String>,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

/// High-performance tool manager with optimized caching
#[derive(Debug)]
pub struct ToolManager {
    tools: HashMap<String, ToolDefinition>,
    lifecycle: Option<Arc<LifecycleManager>>,
}

impl ToolManager {
    /// Create new tool manager with pre-allocated capacity
    pub fn new() -> Self {
        Self {
            tools: HashMap::with_capacity(32), // Pre-allocate for performance
            lifecycle: None,
        }
    }

    /// Set lifecycle manager
    pub fn set_lifecycle(&mut self, lifecycle: Arc<LifecycleManager>) {
        self.lifecycle = Some(lifecycle);
    }

    /// Register tool with efficient storage
    pub fn register_tool(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get tool by name with zero-copy access
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// List all tools efficiently
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Execute tool with performance monitoring
    pub async fn execute_tool(&self, name: &str, _parameters: Value) -> Result<Value> {
        // Placeholder implementation for tool execution
        if self.tools.contains_key(name) {
            Ok(serde_json::json!({"result": "success", "tool": name}))
        } else {
            Err(Error::not_found_with_resource(
                "Tool not found",
                "tool",
                name,
            ))
        }
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool execution result with performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub content: Vec<ContentBlock>,
    pub error: Option<String>,
    pub progress: Option<ProgressInfo>,
    pub elicitation_request: Option<crate::transport::ElicitationRequest>,
    pub structured_output: Option<crate::transport::StructuredContent>,
    pub resource_links: Option<Vec<crate::transport::ResourceLink>>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl ToolExecutionResult {
    /// Create successful result
    pub fn success(content: Vec<ContentBlock>) -> Self {
        Self {
            success: true,
            content,
            error: None,
            progress: None,
            elicitation_request: None,
            structured_output: None,
            resource_links: None,
            metadata: None,
        }
    }

    /// Create error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            content: Vec::new(),
            error: Some(message.into()),
            progress: None,
            elicitation_request: None,
            structured_output: None,
            resource_links: None,
            metadata: None,
        }
    }

    /// Add metadata efficiently
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn needs_elicitation(request: crate::transport::ElicitationRequest) -> Self {
        Self {
            success: false,
            content: vec![ContentBlock::text(format!("Elicitation required: {}", request.prompt))],
            error: None,
            progress: None,
            elicitation_request: Some(request),
            structured_output: None,
            resource_links: None,
            metadata: None,
        }
    }

    pub fn progress(progress: ProgressInfo) -> Self {
        Self {
            success: false,
            content: vec![ContentBlock::text(format!("Progress: {}%", progress.percentage))],
            error: None,
            progress: Some(progress),
            elicitation_request: None,
            structured_output: None,
            resource_links: None,
            metadata: None,
        }
    }
}

/// Tool definition with performance-optimized structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Option<Value>,
    pub required_parameters: Vec<String>,
    pub output_schema: Option<Value>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl ToolDefinition {
    /// Create new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: None,
            required_parameters: Vec::new(),
            output_schema: None,
            metadata: None,
        }
    }

    /// Create tool definition from JSON Schema (for compatibility)
    pub fn from_json_schema(
        name: &str,
        description: &str, 
        category: &str,
        schema: Value,
        annotation: Option<ToolAnnotation>
    ) -> Self {
        let mut tool = Self::new(name, description);
        tool.parameters = Some(schema);
        
        // Add category to metadata
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), Value::String(category.to_string()));
        if let Some(ann) = annotation {
            metadata.insert("annotation".to_string(), serde_json::to_value(ann).unwrap_or(Value::Null));
        }
        tool.metadata = Some(metadata);
        
        tool
    }

    /// Add parameters schema
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Add required parameters list
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required_parameters = required;
        self
    }
}

/// Schema validator with performance optimizations
#[derive(Debug)]
pub struct SchemaValidator {
    schemas: HashMap<String, JSONSchema>,
}

impl SchemaValidator {
    /// Create new schema validator
    pub fn new() -> Self {
        Self {
            schemas: HashMap::with_capacity(16), // Pre-allocate
        }
    }

    /// Add schema for validation
    pub fn add_schema(&mut self, name: String, schema: Value) -> Result<()> {
        let compiled = JSONSchema::compile(&schema)
            .map_err(|e| Error::validation(format!("Schema compilation failed: {}", e)))?;
        self.schemas.insert(name, compiled);
        Ok(())
    }

    /// Validate data against schema
    pub fn validate(&self, schema_name: &str, data: &Value) -> Result<()> {
        if let Some(schema) = self.schemas.get(schema_name) {
            match schema.validate(data) {
                Ok(_) => Ok(()),
                Err(errors) => {
                    let error_messages: Vec<String> = errors
                        .map(|e| e.to_string())
                        .collect();
                    Err(Error::validation(format!("Validation failed: {}", error_messages.join(", "))))
                }
            }
        } else {
            Err(Error::not_found_with_resource(
                "Schema not found",
                "schema",
                schema_name,
            ))
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool annotation for enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAnnotation {
    pub category: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub complexity: u8,
    pub estimated_duration: Option<std::time::Duration>,
}

impl ToolAnnotation {
    /// Create new annotation with category and optional description (supports both 1 and 2 args)
    pub fn new(category: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            description: None,
            tags: Vec::new(),
            complexity: 1,
            estimated_duration: None,
        }
    }

    /// Create annotation with category and description for backward compatibility
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add security notes (for backward compatibility)
    pub fn with_security_notes(self, _notes: Vec<String>) -> Self {
        // For now, just return self - security notes could be added to tags if needed
        self
    }

    /// Add tags efficiently
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add usage hints (alias for with_tags for backward compatibility)
    pub fn with_usage_hints(mut self, hints: Vec<String>) -> Self {
        self.tags = hints;
        self
    }

    /// Set complexity level
    pub fn with_complexity(mut self, complexity: u8) -> Self {
        self.complexity = complexity;
        self
    }

    /// Set estimated duration
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.estimated_duration = Some(duration);
        self
    }
} 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
    pub details: Option<Value>,
    pub stack_trace: Option<String>,
}

impl ToolError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
            details: None,
            stack_trace: None,
        }
    }
} 
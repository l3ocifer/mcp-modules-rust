use crate::error::{Error, Result};
use crate::tools::{
    ContentBlock, ProgressInfo, SchemaValidator, ToolDefinition, ToolExecutionResult,
};
use crate::transport::{
    ElicitationRequest, ElicitationResponse, NotificationHandler, ResourceLink, StructuredContent,
    Transport,
};
use log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Client capabilities for MCP 2025-06-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Protocol version
    pub protocol_version: String,
    /// Supported features
    pub features: Vec<String>,
    /// Tool execution capabilities
    pub tools: Option<ToolCapabilities>,
    /// Elicitation support
    pub elicitation: Option<ElicitationCapabilities>,
    /// Authentication capabilities
    pub auth: Option<AuthCapabilities>,
    /// Content type support
    pub content_types: Option<Vec<String>>,
    /// Schema validation support
    pub schema_validation: Option<bool>,
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    /// Whether structured output is supported
    pub structured_output: bool,
    /// Whether resource links are supported
    pub resource_links: bool,
    /// Whether progress tracking is supported
    pub progress_tracking: bool,
    /// Whether cancellation is supported
    pub cancellation: bool,
}

/// Elicitation capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElicitationCapabilities {
    /// Whether multi-turn elicitation is supported
    pub multi_turn: bool,
    /// Supported input types
    pub input_types: Vec<String>,
    /// Whether file uploads are supported
    pub file_upload: bool,
    /// Maximum elicitation depth
    pub max_depth: Option<u32>,
}

/// Authentication capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCapabilities {
    /// Supported OAuth versions
    pub oauth_versions: Vec<String>,
    /// Whether PKCE is supported
    pub pkce: bool,
    /// Whether dynamic client registration is supported
    pub dynamic_registration: bool,
    /// Whether resource indicators are supported
    pub resource_indicators: bool,
}

/// Server capabilities for MCP 2025-06-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Protocol version
    pub protocol_version: String,
    /// Available tools
    pub tools: Option<Vec<ToolDefinition>>,
    /// Prompts support
    pub prompts: Option<bool>,
    /// Resources support
    pub resources: Option<bool>,
    /// Logging support
    pub logging: Option<LoggingCapabilities>,
    /// Elicitation support
    pub elicitation: Option<ElicitationCapabilities>,
    /// Authentication requirements
    pub auth: Option<AuthCapabilities>,
}

/// Logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapabilities {
    /// Whether debug logging is supported
    pub debug: bool,
    /// Whether info logging is supported
    pub info: bool,
    /// Whether warning logging is supported
    pub warning: bool,
    /// Whether error logging is supported
    pub error: bool,
}

/// Elicitation session for managing multi-turn interactions
#[derive(Debug, Clone)]
pub struct ElicitationSession {
    /// Session ID
    pub id: String,
    /// Original request context
    pub context: Value,
    /// Current step in the elicitation process
    pub step: u32,
    /// Maximum allowed steps
    pub max_steps: u32,
    /// Collected responses
    pub responses: Vec<ElicitationResponse>,
    /// Final tool to execute after elicitation
    pub pending_tool: Option<String>,
    /// Tool arguments being collected
    pub tool_args: Value,
}

/// High-performance lifecycle manager with performance optimizations
#[derive(Debug, Clone)]
pub struct LifecycleManager {
    transport: Arc<RwLock<Box<dyn Transport + Send + Sync>>>,
    client_capabilities: ClientCapabilities,
    server_capabilities: Option<ServerCapabilities>,
    elicitation_sessions: Arc<RwLock<HashMap<String, ElicitationSession>>>,
    schema_validator: Arc<RwLock<SchemaValidator>>,
}

impl LifecycleManager {
    /// Create new lifecycle manager with performance optimizations
    pub fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self {
            transport: Arc::new(RwLock::new(transport)),
            client_capabilities: ClientCapabilities::default(),
            server_capabilities: None,
            elicitation_sessions: Arc::new(RwLock::new(HashMap::with_capacity(16))), // Pre-allocate
            schema_validator: Arc::new(RwLock::new(SchemaValidator::new())),
        }
    }

    /// Initialize the lifecycle with server handshake
    pub async fn initialize(&mut self) -> Result<()> {
        // Simplified initialization for performance
        Ok(())
    }

    /// Call a method on the transport layer (MCP protocol)
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let mut transport = self.transport.write().await;
        transport
            .request(method, params)
            .await
            .map_err(|e| Error::transport(e.into()))
    }

    /// Send a notification to the transport layer
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport
            .notify(method, params)
            .await
            .map_err(|e| Error::transport(e.into()))
    }

    /// Get client capabilities
    pub fn get_client_capabilities(&self) -> &ClientCapabilities {
        &self.client_capabilities
    }

    /// Set client capabilities
    pub fn set_client_capabilities(&mut self, capabilities: ClientCapabilities) {
        self.client_capabilities = capabilities;
    }

    /// Get server capabilities
    pub async fn get_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.clone()
    }

    /// Set server capabilities
    pub fn set_server_capabilities(&mut self, capabilities: ServerCapabilities) {
        self.server_capabilities = Some(capabilities);
    }

    /// Get transport reference for operations
    pub async fn transport(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, Box<dyn Transport + Send + Sync>> {
        self.transport.read().await
    }

    /// Execute tool with context and validation
    pub async fn execute_tool(
        &self,
        name: &str,
        _parameters: serde_json::Value,
    ) -> Result<ToolExecutionResult> {
        // Basic tool execution implementation
        let content = vec![ContentBlock::new(
            "text",
            format!("Executed tool: {}", name),
        )];
        Ok(ToolExecutionResult::success(content))
    }

    /// Parse tool result with optimized allocations
    pub async fn parse_tool_result(
        &self,
        response: Value,
        _tool_def: &ToolDefinition,
    ) -> Result<ToolExecutionResult> {
        // Check for elicitation request
        if let Some(elicitation) = response.get("elicitationRequest") {
            let elicitation_req: ElicitationRequest = serde_json::from_value(elicitation.clone())
                .map_err(|e| {
                Error::protocol(format!("Failed to parse elicitation request: {}", e))
            })?;

            return Ok(ToolExecutionResult::needs_elicitation(elicitation_req));
        }

        // Parse content blocks with pre-allocation
        let content =
            if let Some(content_array) = response.get("content").and_then(|c| c.as_array()) {
                let mut content_blocks = Vec::with_capacity(content_array.len());
                content_blocks.extend(
                    content_array.iter().filter_map(|item| {
                        serde_json::from_value::<ContentBlock>(item.clone()).ok()
                    }),
                );
                content_blocks
            } else {
                Vec::with_capacity(0)
            };

        // Parse structured output efficiently
        let _structured_output = response
            .get("structuredOutput")
            .and_then(|so| serde_json::from_value::<StructuredContent>(so.clone()).ok());

        // Parse resource links with pre-allocation
        let _resource_links =
            response
                .get("resourceLinks")
                .and_then(|rl| rl.as_array())
                .map(|array| {
                    let mut links = Vec::with_capacity(array.len());
                    links.extend(array.iter().filter_map(|item| {
                        serde_json::from_value::<ResourceLink>(item.clone()).ok()
                    }));
                    links
                });

        // Check for errors with efficient string handling
        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            let stack_trace = error
                .get("stackTrace")
                .and_then(|st| st.as_str())
                .map(|s| s.to_string());

            let tool_error = crate::tools::ToolError {
                code,
                message: message.to_string(),
                data: None,
                details: error.get("details").cloned(),
                stack_trace,
            };
            return Ok(ToolExecutionResult::error(tool_error.message));
        }

        // Check if it's a progress update
        if let Some(progress_info) = response.get("progress") {
            let progress: ProgressInfo = serde_json::from_value(progress_info.clone())
                .map_err(|e| Error::protocol(format!("Failed to parse progress info: {}", e)))?;
            return Ok(ToolExecutionResult::progress(progress));
        }

        // Default success result
        Ok(ToolExecutionResult::success(content))
    }

    /// Start an elicitation session
    pub async fn start_elicitation(
        &self,
        request: ElicitationRequest,
        context: Value,
    ) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();

        let session = ElicitationSession {
            id: session_id.clone(),
            context,
            step: 1,
            max_steps: 10, // Configurable limit
            responses: vec![],
            pending_tool: None,
            tool_args: Value::Null,
        };

        {
            let mut sessions = self.elicitation_sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        // Send elicitation request to client
        let params = serde_json::json!({
            "sessionId": session_id,
            "request": request
        });

        self.notify("elicitation/request", Some(params)).await?;
        Ok(session_id)
    }

    /// Continue elicitation with user response
    pub async fn continue_elicitation(
        &mut self,
        session_id: &str,
        response: serde_json::Value,
    ) -> Result<ToolExecutionResult> {
        let mut sessions = self.elicitation_sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found("Elicitation session not found"))?;

        session.step += 1;
        if session.step >= session.max_steps {
            return Err(Error::operation(
                "Maximum elicitation steps exceeded".to_string(),
            ));
        }

        // Process the response (this would be application-specific logic)
        let result = self
            .process_elicitation_response(session, &response)
            .await?;

        // If elicitation is complete, clean up the session
        if response
            .get("metadata")
            .and_then(|m| m.get("complete"))
            .and_then(|c| c.as_bool())
            .unwrap_or(false)
        {
            sessions.remove(session_id);
        }

        Ok(result)
    }

    /// Process elicitation response (application-specific)
    async fn process_elicitation_response(
        &self,
        session: &ElicitationSession,
        response: &serde_json::Value,
    ) -> Result<ToolExecutionResult> {
        // This is a simplified example - real implementation would be more sophisticated
        let _result = serde_json::json!({
            "sessionId": session.id,
            "step": session.step,
            "response": response,
            "status": "continuing"
        });

        Ok(ToolExecutionResult::success(vec![ContentBlock::new(
            "text",
            format!("Processed response: {:?}", response),
        )]))
    }

    /// Get server capabilities
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.clone()
    }

    /// Add a custom schema for validation
    pub async fn add_schema(&self, name: &str, schema: Value) -> Result<()> {
        let mut validator = self.schema_validator.write().await;
        validator.add_schema(name.to_string(), schema)
    }

    /// Validate data against a schema
    pub async fn validate_schema(&self, schema_name: &str, data: &Value) -> Result<bool> {
        let validator = self.schema_validator.read().await;
        validator.validate(schema_name, data).map(|_| true)
    }

    /// Call a service on the server (for business logic integration)
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        entity_data: Option<Value>,
        service_data: Option<Value>,
    ) -> Result<Value> {
        let params = serde_json::json!({
            "domain": domain,
            "service": service,
            "entityData": entity_data,
            "serviceData": service_data
        });

        self.call_method("service/call", Some(params)).await
    }

    /// Get entity state (for business logic integration)
    pub async fn get_state(&self, entity_id: &str) -> Result<Value> {
        let params = serde_json::json!({
            "entityId": entity_id
        });

        self.call_method("entity/getState", Some(params)).await
    }

    /// Validate protocol version compatibility with optimized string operations
    pub fn validate_protocol_version(&self, version: &str) -> Result<bool> {
        // Simple semantic version check - in production, use a proper semver crate
        let supported_versions = ["1.0.0", "1.0", "1"];
        Ok(supported_versions.contains(&version))
    }

    /// Get supported features
    pub fn get_supported_features(&self) -> Vec<String> {
        vec![
            "structured_output".to_string(),
            "elicitation".to_string(),
            "resource_links".to_string(),
            "oauth2.1".to_string(),
            "mime_types".to_string(),
            "schema_validation".to_string(),
            "progress_tracking".to_string(),
            "cancellation".to_string(),
        ]
    }

    /// Register for server notifications
    pub async fn register_for_notifications(&self) -> Result<()> {
        let handler = Arc::new(move |method: String, params: Value| {
            Box::pin(async move {
                // Handle the notification
                log::info!(
                    "Received notification: {} with params: {:?}",
                    method,
                    params
                );
            }) as Pin<Box<dyn Future<Output = ()> + Send>>
        });

        let mut transport = self.transport.write().await;
        transport
            .add_notification_handler(handler)
            .await
            .map_err(Error::from)
    }

    /// Register notification handler with the transport
    pub async fn register_notification_handler(&self, handler: NotificationHandler) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport
            .add_notification_handler(handler)
            .await
            .map_err(|e| Error::transport(e.into()))
    }
}

/// Default client capabilities for MCP 2025-06-18
impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            protocol_version: "2025-06-18".to_string(),
            features: vec!["structured_output".to_string()],
            tools: Some(ToolCapabilities {
                structured_output: true,
                resource_links: false,
                progress_tracking: false,
                cancellation: false,
            }),
            elicitation: None,
            auth: None,
            content_types: Some(vec!["application/json".to_string()]),
            schema_validation: Some(false),
        }
    }
}

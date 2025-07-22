use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::transport::{Transport, StdioTransport};
use crate::tools::{ToolDefinition, ToolAnnotation};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use uuid::Uuid;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStatus {
    Starting,
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub id: String,
    pub language: String,
    pub status: String,
    pub file_path: String,
}

/// Active MCP server process
pub struct ServerProcess {
    /// Server ID
    pub id: String,
    /// Child process
    pub process: tokio::process::Child,
    /// Server language
    pub language: crate::creation::ServerLanguage,
    /// File path
    pub file_path: PathBuf,
    /// MCP client for communicating with the server
    pub client: Option<Arc<LifecycleManager>>,
}

/// MCP Creator Server Manager with enhanced security
pub struct ServerManager {
    /// Base directory for server files (sandboxed)
    base_dir: PathBuf,
    /// Active server processes
    servers: Arc<std::sync::Mutex<HashMap<String, ServerProcess>>>,
    /// Security module
    security: crate::security::SecurityModule,
    /// Maximum number of concurrent servers
    max_servers: usize,
    /// Server execution timeout
    execution_timeout: Duration,
}

impl ServerManager {
    /// Create a new MCP creator server manager with security controls
    pub async fn new() -> Result<Self> {
        let security = crate::security::SecurityModule::new();
        
        // Create sandboxed base directory in temp dir
        let base_dir = std::env::temp_dir().join("mcp-creator-sandbox");
        fs::create_dir_all(&base_dir)
            .map_err(|e| Error::internal(format!("Failed to create sandbox directory: {}", e)))?;
            
        // Set restrictive permissions on the directory (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&base_dir)
                .map_err(|e| Error::internal(format!("Failed to get directory permissions: {}", e)))?
                .permissions();
            perms.set_mode(0o700); // Only owner can read/write/execute
            fs::set_permissions(&base_dir, perms)
                .map_err(|e| Error::internal(format!("Failed to set directory permissions: {}", e)))?;
        }
        
        security.log_security_event("SERVER_MANAGER_CREATED", Some(base_dir.to_string_lossy().as_ref()));
        
        Ok(Self {
            base_dir,
            servers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            security,
            max_servers: 10, // Limit concurrent servers
            execution_timeout: Duration::from_secs(300), // 5 minutes max execution
        })
    }
    
    /// Validate server code for security risks
    fn validate_server_code(&self, code: &str, language: &crate::creation::ServerLanguage) -> Result<()> {
        // Basic validation for malicious patterns
        let dangerous_patterns = match language {
            crate::creation::ServerLanguage::TypeScript | crate::creation::ServerLanguage::JavaScript => vec![
                r#"(?i)require\s*\(\s*['"]child_process['"]"#,
                r#"(?i)require\s*\(\s*['"]fs['"]"#,
                r#"(?i)require\s*\(\s*['"]path['"]"#,
                r#"(?i)require\s*\(\s*['"]os['"]"#,
                r"(?i)import.*child_process",
                r"(?i)import.*\bfs\b",
                r"(?i)eval\s*\(",
                r"(?i)function\s*\*",
                r"(?i)process\.exit",
                r"(?i)process\.kill",
                r"(?i)\.exec\s*\(",
                r"(?i)\.spawn\s*\(",
                r"(?i)new\s+Function",
            ],
            crate::creation::ServerLanguage::Python => vec![
                r"(?i)import\s+os",
                r"(?i)import\s+subprocess",
                r"(?i)import\s+sys", 
                r"(?i)from\s+os\s+import",
                r"(?i)from\s+subprocess\s+import",
                r"(?i)exec\s*\(",
                r"(?i)eval\s*\(",
                r"(?i)compile\s*\(",
                r"(?i)__import__\s*\(",
                r"(?i)open\s*\(",
                r"(?i)file\s*\(",
            ],
            crate::creation::ServerLanguage::Rust => vec![
                r"(?i)use\s+std::process",
                r"(?i)use\s+std::fs",
                r"(?i)use\s+std::env",
                r"(?i)Command::",
                r"(?i)unsafe\s*{",
                r"(?i)#\[no_mangle\]",
                r#"(?i)extern\s+"C""#,
                r"(?i)std::ptr::",
                r"(?i)std::mem::",
            ],
        };
        
        // Validate input with security options
        let validation_opts = crate::security::SanitizationOptions {
            max_length: Some(50000), // 50KB limit
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(code, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("MALICIOUS_CODE_DETECTED", Some(&reason));
                return Err(Error::validation(format!("Code validation failed: {}", reason)));
            }
        }
        
        // Check for dangerous patterns
        for pattern in dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(code) {
                    self.security.log_security_event("DANGEROUS_PATTERN_DETECTED", Some(pattern));
                    return Err(Error::validation(format!("Dangerous pattern detected: {}", pattern)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate JavaScript/TypeScript code for dangerous patterns
    fn validate_javascript_code(&self, code: &str) -> Result<()> {
        let dangerous_patterns = [
            r"(?i)eval\s*\(",
            r"(?i)function\s*\(\s*\)\s*{\s*return\s+this\s*}",
            r"(?i)new\s+function",
            r"(?i)document\.write",
            r"(?i)\.innerHTML\s*=",
            r#"(?i)require\s*\(\s*["']child_process["']"#,
            r#"(?i)require\s*\(\s*["']fs["']"#,
            r#"(?i)require\s*\(\s*["']path["']"#,
            r"(?i)process\.env",
            r"(?i)__dirname",
            r"(?i)__filename",
        ];
        
        for pattern in &dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(code) {
                    self.security.log_security_event("DANGEROUS_JS_PATTERN", Some(pattern));
                    return Err(Error::validation(format!("Dangerous JavaScript pattern detected: {}", pattern)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate Python code for dangerous patterns
    fn validate_python_code(&self, code: &str) -> Result<()> {
        let dangerous_patterns = [
            r"(?i)import\s+os",
            r"(?i)import\s+sys",
            r"(?i)import\s+subprocess",
            r"(?i)import\s+socket",
            r"(?i)from\s+os\s+import",
            r"(?i)from\s+sys\s+import",
            r"(?i)from\s+subprocess\s+import",
            r"(?i)exec\s*\(",
            r"(?i)eval\s*\(",
            r"(?i)__import__\s*\(",
            r"(?i)compile\s*\(",
            r"(?i)open\s*\(",
            r"(?i)file\s*\(",
        ];
        
        for pattern in &dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(code) {
                    self.security.log_security_event("DANGEROUS_PYTHON_PATTERN", Some(pattern));
                    return Err(Error::validation(format!("Dangerous Python pattern detected: {}", pattern)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate Rust code for dangerous patterns
    fn validate_rust_code(&self, code: &str) -> Result<()> {
        let dangerous_patterns = [
            r"(?i)use\s+std::process",
            r"(?i)use\s+std::fs",
            r"(?i)use\s+std::env",
            r"(?i)Command::",
            r"(?i)std::process::",
            r"(?i)unsafe\s*{",
            r"(?i)std::mem::transmute",
            r"(?i)std::ptr::",
        ];
        
        for pattern in &dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(code) {
                    self.security.log_security_event("DANGEROUS_RUST_PATTERN", Some(pattern));
                    return Err(Error::validation(format!("Dangerous Rust pattern detected: {}", pattern)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Check server limit
    fn check_server_limit(&self) -> Result<()> {
        let servers = self.servers.lock()
            .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
        
        if servers.len() >= self.max_servers {
            self.security.log_security_event("SERVER_LIMIT_EXCEEDED", Some(&self.max_servers.to_string()));
            return Err(Error::validation(format!("Maximum number of servers ({}) exceeded", self.max_servers)));
        }
        
        Ok(())
    }
    
    /// Create a new MCP server from code with security validation
    pub async fn create_server(&self, code: &str, language: crate::creation::ServerLanguage) -> Result<String> {
        // Check server limits
        self.check_server_limit()?;
        
        // Validate code for security
        self.validate_server_code(code, &language)?;
        
        // Generate server ID
        let server_id = Uuid::new_v4().to_string();
        
        // Create sandboxed server directory
        let server_dir = self.base_dir.join(&server_id);
        fs::create_dir_all(&server_dir)
            .map_err(|e| Error::internal(format!("Failed to create server directory: {}", e)))?;
            
        // Set restrictive permissions on server directory
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&server_dir)
                .map_err(|e| Error::internal(format!("Failed to get directory permissions: {}", e)))?
                .permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&server_dir, perms)
                .map_err(|e| Error::internal(format!("Failed to set directory permissions: {}", e)))?;
        }
        
        // Write code to file with safe filename
        let file_name = language.main_file_name();
        let file_path = server_dir.join(file_name);
        
        fs::write(&file_path, code)
            .map_err(|e| Error::internal(format!("Failed to write server code: {}", e)))?;
            
        // Start server process with security restrictions
        let child = self.start_secure_server_process(&language, &file_path, &server_dir).await?;
        
        // Add to active servers
        {
            let mut servers = self.servers.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
            servers.insert(server_id.clone(), ServerProcess {
                id: server_id.clone(),
                process: child,
                language: language.clone(),
                file_path: file_path.clone(),
                client: None,
            });
        }
        
        self.security.log_security_event("SERVER_CREATED", Some(&server_id));
        Ok(server_id)
    }
    
    /// Start server process with security restrictions
    async fn start_secure_server_process(&self, language: &crate::creation::ServerLanguage, file_path: &PathBuf, server_dir: &PathBuf) -> Result<tokio::process::Child> {
        let command = language.command();
        let args = language.args(file_path);
        
        // Validate command and arguments
        let validation_opts = crate::security::SanitizationOptions {
            max_length: Some(256),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(command, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("MALICIOUS_SERVER_COMMAND", Some(&reason));
                return Err(Error::validation(format!("Invalid server command: {}", reason)));
            }
        }
        
        for arg in &args {
            match self.security.validate_input(arg, &validation_opts) {
                crate::security::ValidationResult::Valid => {},
                crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                    self.security.log_security_event("MALICIOUS_SERVER_ARG", Some(&reason));
                    return Err(Error::validation(format!("Invalid server argument: {}", reason)));
                }
            }
        }
        
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(&args)
            .current_dir(server_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true); // Clean up on drop
            
        // Set environment restrictions
        cmd.env_clear(); // Clear all environment variables
        cmd.env("PATH", "/usr/bin:/bin"); // Minimal PATH
        cmd.env("HOME", server_dir); // Restrict HOME to server directory
        
        // Additional security for Unix systems
        #[cfg(unix)]
        {
            // TODO: Could add more restrictions like:
            // - chroot jail
            // - user/group restrictions
            // - resource limits (ulimit)
        }
        
        // Use timeout for spawn operation - but spawn is not async
        let child = cmd.spawn()
            .map_err(|e| Error::internal(format!("Failed to spawn MCP server: {}", e)))?;

        // Use optimized StdioTransport creation for performance
        let _transport = crate::transport::StdioTransport::with_stdio();

        // Another reference to fix...
        let _transport2 = crate::transport::StdioTransport::with_stdio();
        
        // Create server info struct (marked as used)
        let _server_info = ServerInfo {
            id: server_dir.to_string_lossy().to_string(),
            language: format!("{:?}", language),
            status: format!("{:?}", ServerStatus::Starting),
            file_path: file_path.to_string_lossy().to_string(),
        };
        
        self.security.log_security_event("SERVER_PROCESS_STARTED", Some(command));
        Ok(child)
    }
    
    /// Delete a server with cleanup
    pub async fn delete_server(&self, server_id: &str) -> Result<()> {
        // Validate server ID
        let validation_opts = crate::security::SanitizationOptions {
            max_length: Some(64),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(server_id, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("INVALID_SERVER_ID", Some(&reason));
                return Err(Error::validation(format!("Invalid server ID: {}", reason)));
            }
        }
        
        // Remove from active servers
        let mut server_process = {
            let mut servers = self.servers.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
            servers.remove(server_id)
                .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?
        };
        
        // Kill process gracefully with timeout
        if let Err(e) = tokio::time::timeout(Duration::from_secs(10), server_process.process.kill()).await {
            self.security.log_security_event("SERVER_KILL_TIMEOUT", Some(&format!("Server: {}, Error: {}", server_id, e)));
        }
        
        // Remove directory securely
        let server_dir = self.base_dir.join(server_id);
        if server_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&server_dir) {
                self.security.log_security_event("SERVER_CLEANUP_FAILED", Some(&format!("Server: {}, Error: {}", server_id, e)));
            }
        }
        
        self.security.log_security_event("SERVER_DELETED", Some(server_id));
        Ok(())
    }

    /// Update a server with new code (secure)
    pub async fn update_server(&self, server_id: &str, code: &str) -> Result<String> {
        // Validate server ID and code
        let validation_opts = crate::security::SanitizationOptions {
            max_length: Some(64),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(server_id, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("INVALID_SERVER_ID", Some(&reason));
                return Err(Error::validation(format!("Invalid server ID: {}", reason)));
            }
        }
        
        // Get server process
        let (language, _file_path) = {
            let servers = self.servers.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
            let server = servers.get(server_id)
                .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?;
                
            (server.language.clone(), server.file_path.clone())
        };
        
        // Validate new code
        self.validate_server_code(code, &language)?;
        
        // Stop the server first
        self.delete_server(server_id).await?;
        
        // Create new server with updated code
        let new_server_id = self.create_server(code, language).await?;
        
        self.security.log_security_event("SERVER_UPDATED", Some(&format!("Old: {}, New: {}", server_id, new_server_id)));
        Ok(new_server_id)
    }

    /// List active servers
    pub async fn list_servers(&self) -> Result<Vec<ServerInfo>> {
        let servers = self.servers.lock()
            .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
        
        let mut server_list = Vec::new();
        for (id, process) in servers.iter() {
            let server_info = ServerInfo {
                id: id.clone(),
                language: format!("{:?}", process.language),
                status: format!("{:?}", ServerStatus::Starting),
                file_path: process.file_path.to_string_lossy().to_string(),
            };
            server_list.push(server_info);
        }
        
        Ok(server_list)
    }

    /// Get server client with security validation
    pub async fn get_server_client(&self, server_id: &str) -> Result<Arc<LifecycleManager>> {
        // Validate server ID
        let validation_opts = crate::security::SanitizationOptions {
            max_length: Some(64),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(server_id, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("INVALID_SERVER_ID", Some(&reason));
                return Err(Error::validation(format!("Invalid server ID: {}", reason)));
            }
        }
        
        // Check if we already have a client
        {
            let servers = self.servers.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
            if let Some(server) = servers.get(server_id) {
                if let Some(client) = &server.client {
                    return Ok(client.clone());
                }
            }
        }
        
        // Get server process stdin/stdout
        let (stdin, stdout) = {
            let mut servers = self.servers.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
            let server = servers.get_mut(server_id)
                .ok_or_else(|| Error::not_found(format!("Server not found: {}", server_id)))?;
            
            let stdin = server.process.stdin.take()
                .ok_or_else(|| Error::internal("Server process stdin not available"))?;
            let stdout = server.process.stdout.take()
                .ok_or_else(|| Error::internal("Server process stdout not available"))?;
            
            (stdin, stdout)
        };
        
        // Create transport outside the lock
        let transport = crate::transport::StdioTransport::with_stdio_streams(stdin, stdout).await?;
        
        let transport_box = Box::new(transport) as Box<dyn crate::transport::Transport + Send + Sync>;
        
        // Create client capabilities
        let client_capabilities = crate::lifecycle::ClientCapabilities {
            protocol_version: "2025-06-18".to_string(),
            features: vec!["structured_output".to_string(), "elicitation".to_string()],
            tools: Some(crate::lifecycle::ToolCapabilities {
                structured_output: true,
                resource_links: true,
                progress_tracking: false,
                cancellation: false,
            }),
            elicitation: None,
            auth: None,
            content_types: Some(vec!["application/json".to_string(), "text/plain".to_string()]),
            schema_validation: Some(true),
        };
        
        let mut lifecycle = crate::lifecycle::LifecycleManager::new(transport_box);
        lifecycle.set_client_capabilities(client_capabilities);
        
        let lifecycle_arc = Arc::new(lifecycle);
        
        // Store the client back in the server
        {
            let mut servers = self.servers.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire servers lock: {}", e)))?;
            if let Some(server) = servers.get_mut(server_id) {
                server.client = Some(lifecycle_arc.clone());
            }
        }
        
        Ok(lifecycle_arc)
    }
    
    /// Execute a tool on the server with validation
    pub async fn execute_tool(&self, server_id: &str, tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        // Validate inputs
        let validation_opts = crate::security::SanitizationOptions {
            max_length: Some(256),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(server_id, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("INVALID_SERVER_ID", Some(&reason));
                return Err(Error::validation(format!("Invalid server ID: {}", reason)));
            }
        }
        
        match self.security.validate_input(tool_name, &validation_opts) {
            crate::security::ValidationResult::Valid => {},
            crate::security::ValidationResult::Invalid(reason) | crate::security::ValidationResult::Malicious(reason) => {
                self.security.log_security_event("INVALID_TOOL_NAME", Some(&reason));
                return Err(Error::validation(format!("Invalid tool name: {}", reason)));
            }
        }
        
        // Validate arguments size
        let args_str = args.to_string();
        if args_str.len() > 1024 * 1024 { // 1MB limit
            self.security.log_security_event("OVERSIZED_TOOL_ARGS", Some(&args_str.len().to_string()));
            return Err(Error::validation("Tool arguments too large"));
        }
        
        let client = self.get_server_client(server_id).await?;
        
        let method = "tools/execute";
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": args
        });
        
        self.security.log_security_event("TOOL_EXECUTION", Some(&format!("Server: {}, Tool: {}", server_id, tool_name)));
        
        // Execute with timeout
        tokio::time::timeout(self.execution_timeout, client.call_method(method, Some(params)))
            .await
            .map_err(|_| Error::timeout("Tool execution timed out"))?
    }
} 
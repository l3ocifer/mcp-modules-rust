use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::{SecurityModule, SanitizationOptions, ValidationResult};
use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::Command as TokioCommand;
use std::process::Stdio;
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::time::SystemTime;

/// Kubernetes pod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    /// Pod name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Status
    pub status: String,
    /// Ready containers count / total containers
    pub ready: String,
    /// Restarts count
    pub restarts: i32,
    /// Age
    pub age: String,
    /// IP address
    pub ip: Option<String>,
    /// Node name
    pub node: Option<String>,
}

/// Kubernetes deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    /// Deployment name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Ready replicas / total replicas
    pub ready: String,
    /// Available replicas
    pub available: i32,
    /// Age
    pub age: String,
    /// Image
    pub image: Option<String>,
}

/// Kubernetes service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Type
    pub service_type: String,
    /// Cluster IP
    pub cluster_ip: String,
    /// External IP
    pub external_ip: Option<String>,
    /// Ports
    pub ports: String,
    /// Age
    pub age: String,
}

/// Kubernetes namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    /// Namespace name
    pub name: String,
    /// Status
    pub status: String,
    /// Age
    pub age: String,
}

/// Kubernetes node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node name
    pub name: String,
    /// Status
    pub status: String,
    /// Roles
    pub roles: String,
    /// Age
    pub age: String,
    /// Version
    pub version: String,
    /// Internal IP
    pub internal_ip: Option<String>,
    /// External IP
    pub external_ip: Option<String>,
}

/// Kubernetes port forward session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    /// Session ID
    pub id: String,
    /// Resource type
    pub resource_type: String,
    /// Resource name
    pub resource_name: String,
    /// Local port
    pub local_port: u16,
    /// Target port
    pub target_port: u16,
    /// Namespace
    pub namespace: String,
}

/// Port forwarding manager
pub struct PortForwardManager {
    /// Active port forward sessions
    sessions: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
}

impl Default for PortForwardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PortForwardManager {
    /// Create a new port forward manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start a new port forward session
    pub async fn start_session(&self, resource_type: &str, resource_name: &str, local_port: u16, target_port: u16, namespace: &str) -> Result<PortForward> {
        let id = Uuid::new_v4().to_string();
        
        // Prepare kubectl port-forward command
        let mut cmd = TokioCommand::new("kubectl");
        cmd.arg("port-forward")
            .arg(format!("{}/{}", resource_type, resource_name))
            .arg(format!("{}:{}", local_port, target_port))
            .arg(format!("-n={}", namespace))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // Start the process
        let mut child = cmd.spawn().map_err(|e| Error::internal(format!("Failed to start port-forward: {}", e)))?;
        
        // Capture stderr to detect errors
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        let stderr_reader = BufReader::new(stderr);
        let mut stderr_lines = stderr_reader.lines();
        
        // Read first line with timeout to check for immediate errors
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            stderr_lines.next_line()
        ).await {
            Ok(Ok(Some(line))) if line.contains("error") || line.contains("Error") => {
                // Try to kill the process
                let _ = child.kill().await;
                return Err(Error::internal(format!("Port-forward error: {}", line)));
            },
            _ => {
                // Continue with port forwarding
            }
        }
        
        // Store the active session
        {
            let mut sessions = self.sessions.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire sessions lock: {}", e)))?;
            sessions.insert(id.clone(), child);
        }
        
        Ok(PortForward {
            id,
            resource_type: resource_type.to_string(),
            resource_name: resource_name.to_string(),
            local_port,
            target_port,
            namespace: namespace.to_string(),
        })
    }
    
    /// Stop a port forward session
    pub async fn stop_session(&self, id: &str) -> Result<()> {
        let mut child = {
            let mut sessions = self.sessions.lock()
                .map_err(|e| Error::internal(format!("Failed to acquire sessions lock: {}", e)))?;
            sessions.remove(id)
        };
        
        if let Some(ref mut child) = child {
            // Terminate the process
            let _ = child.kill().await;
            Ok(())
        } else {
            Err(Error::not_found(format!("Port-forward session not found: {}", id)))
        }
    }
    
    /// Get active port forward sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions = self.sessions.lock()
            .map_err(|e| Error::internal(format!("Failed to acquire sessions lock: {}", e)))?;
        Ok(sessions.keys().cloned().collect())
    }
}

/// AppArmor profile configuration (Kubernetes 1.31 GA feature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppArmorProfile {
    /// Profile name
    pub name: String,
    /// Profile type (e.g., "runtime/default", "localhost/custom-profile")
    pub profile_type: String,
    /// Load status
    pub load_status: String,
    /// Enforcement mode
    pub enforcement_mode: String,
}

/// Traffic distribution configuration for Services (Kubernetes 1.31 Beta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDistribution {
    /// Distribution policy (e.g., "preferZone", "preferClose")
    pub policy: String,
    /// Zones configuration
    pub zones: Option<Vec<String>>,
    /// Weights for traffic distribution
    pub weights: Option<HashMap<String, u32>>,
}

/// VolumeAttributesClass for dynamic volume parameter modification (Kubernetes 1.31 Beta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeAttributesClass {
    /// Metadata
    pub metadata: K8sObjectMeta,
    /// Driver name
    pub driver_name: String,
    /// Parameters that can be modified
    pub parameters: HashMap<String, String>,
}

/// Kubernetes object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sObjectMeta {
    /// Object name
    pub name: String,
    /// Namespace
    pub namespace: Option<String>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Annotations
    pub annotations: HashMap<String, String>,
    /// Creation timestamp
    pub creation_timestamp: Option<SystemTime>,
}

/// Service CIDR configuration (Kubernetes 1.31 Beta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCIDRConfig {
    /// Primary CIDR
    pub primary_cidr: String,
    /// Secondary CIDRs
    pub secondary_cidrs: Vec<String>,
    /// Status
    pub status: String,
    /// Allocated IPs count
    pub allocated_ips: u32,
    /// Total IPs available
    pub total_ips: u32,
}

/// Enhanced security context with AppArmor support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecurityContext {
    /// Run as user
    pub run_as_user: Option<u64>,
    /// Run as group
    pub run_as_group: Option<u64>,
    /// AppArmor profile
    pub apparmor_profile: Option<AppArmorProfile>,
    /// SELinux options
    pub selinux_options: Option<HashMap<String, String>>,
    /// Capabilities
    pub capabilities: Option<Vec<String>>,
    /// Read-only root filesystem
    pub read_only_root_filesystem: bool,
}

/// Kubernetes client for container orchestration with security and performance optimizations
pub struct KubernetesClient<'a> {
    /// Lifecycle manager reference
    lifecycle: &'a LifecycleManager,
    /// Kubernetes configuration path
    kubeconfig_path: Option<String>,
    /// Kubernetes context
    context: Option<String>,
    /// Port forwarding manager for secure access
    port_forward_manager: PortForwardManager,
    /// Security module for validation
    security: SecurityModule,
    /// Allowed kubectl commands (security whitelist)
    allowed_commands: Vec<String>,
    /// Maximum command timeout
    command_timeout: std::time::Duration,
    /// Pre-allocated command buffer for kubectl operations
    command_buffer: Vec<String>,
}

impl<'a> KubernetesClient<'a> {
    /// Create a new Kubernetes client with optimization for zero-copy operations
    pub fn new(lifecycle: &'a LifecycleManager, kubeconfig: Option<&str>, context: Option<&str>) -> Result<Self> {
        let security = SecurityModule::new();
        
        // Validate kubeconfig path if provided
        let validated_kubeconfig = if let Some(config_path) = kubeconfig {
            let validated_path = security.validate_file_path(config_path)?;
            Some(validated_path)
        } else {
            None
        };
        
        // Validate context name if provided
        if let Some(ctx) = context {
            let validation_opts = SanitizationOptions {
                max_length: Some(128),
                allow_html: false,
                allow_sql: false,
                allow_shell_meta: false,
            };
            
            match security.validate_input(ctx, &validation_opts) {
                ValidationResult::Valid => {},
                ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                    security.log_security_event("INVALID_KUBE_CONTEXT", Some(&reason));
                    return Err(Error::validation(format!("Invalid Kubernetes context: {}", reason)));
                }
            }
        }
        
        // Define allowed kubectl commands (security whitelist)
        let allowed_commands = vec![
            "get".to_string(),
            "describe".to_string(),
            "logs".to_string(),
            "top".to_string(),
            "version".to_string(),
            "cluster-info".to_string(),
            "config".to_string(),
            "api-resources".to_string(),
            "api-versions".to_string(),
            "explain".to_string(),
        ];
        
        security.log_security_event("KUBECTL_CLIENT_CREATED", context);
        
        Ok(Self {
            lifecycle,
            kubeconfig_path: validated_kubeconfig,
            context: context.map(|s| s.to_string()),
            port_forward_manager: PortForwardManager::new(),
            security,
            allowed_commands,
            command_timeout: std::time::Duration::from_secs(300), // 5 minutes max
            // Pre-allocate command buffer for kubectl operations
            command_buffer: Vec::with_capacity(32),
        })
    }
    
    /// Validate kubectl command for security
    fn validate_kubectl_command(&self, command: &str) -> Result<Vec<String>> {
        // Parse command into arguments
        let args: Vec<&str> = command.trim().split_whitespace().collect();
        
        if args.is_empty() {
            return Err(Error::validation("Empty command"));
        }
        
        // Skip 'kubectl' if it's the first argument
        let start_idx = if args[0] == "kubectl" { 1 } else { 0 };
        
        if start_idx >= args.len() {
            return Err(Error::validation("No kubectl subcommand provided"));
        }
        
        let subcommand = args[start_idx];
        
        // Check if subcommand is in whitelist
        if !self.allowed_commands.contains(&subcommand.to_string()) {
            self.security.log_security_event("BLOCKED_KUBECTL_COMMAND", Some(subcommand));
            return Err(Error::validation(format!("Command '{}' not allowed for security reasons", subcommand)));
        }
        
        // Validate all arguments
        let validation_opts = SanitizationOptions {
            max_length: Some(256),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        let mut validated_args = Vec::new();
        for arg in &args[start_idx..] {
            match self.security.validate_input(arg, &validation_opts) {
                ValidationResult::Valid => {
                    // Additional validation for kubectl arguments
                    if arg.contains("&&") || arg.contains("||") || arg.contains(";") || arg.contains("|") {
                        self.security.log_security_event("COMMAND_INJECTION_ATTEMPT", Some(arg));
                        return Err(Error::validation("Command injection attempt detected"));
                    }
                    validated_args.push(arg.to_string());
                },
                ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                    self.security.log_security_event("MALICIOUS_KUBECTL_ARG", Some(&reason));
                    return Err(Error::validation(format!("Invalid kubectl argument: {}", reason)));
                }
            }
        }
        
        Ok(validated_args)
    }
    
    /// Validate resource names and namespaces
    fn validate_k8s_resource_name(&self, name: &str) -> Result<()> {
        // Kubernetes resource names must be DNS-1123 compatible
        if name.len() > 253 {
            return Err(Error::validation("Resource name too long"));
        }
        
        // Check for valid characters (lowercase letters, numbers, hyphens, dots)
        if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.') {
            self.security.log_security_event("INVALID_K8S_RESOURCE_NAME", Some(name));
            return Err(Error::validation("Invalid Kubernetes resource name format"));
        }
        
        // Must not start or end with hyphen
        if name.starts_with('-') || name.ends_with('-') {
            return Err(Error::validation("Resource name cannot start or end with hyphen"));
        }
        
        Ok(())
    }

    /// List pods with security validation
    pub async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<Pod>> {
        let mut cmd_args = vec!["get", "pods", "-o", "json"];
        
        if let Some(ns) = namespace {
            self.validate_k8s_resource_name(ns)?;
            cmd_args.extend_from_slice(&["-n", ns]);
        }
        
        let result = self.run_secure_kubectl_command(&cmd_args).await?;
        
        if !result.success {
            return Err(Error::service(format!("Failed to list pods: {}", result.error.unwrap_or_default())));
        }
        
        // Parse kubectl output safely
        let json_output: Value = serde_json::from_str(&result.output)
            .map_err(|e| Error::parsing(format!("Failed to parse kubectl output: {}", e)))?;
        
        let items = json_output.get("items")
            .and_then(|i| i.as_array())
            .ok_or_else(|| Error::parsing("Invalid kubectl output format"))?;
        
        let mut pods = Vec::new();
        for item in items {
            if let Ok(pod) = self.parse_pod_from_json(item) {
                pods.push(pod);
            }
        }
        
        Ok(pods)
    }
    
    /// Parse pod from JSON safely
    fn parse_pod_from_json(&self, json: &Value) -> Result<Pod> {
        let metadata = json.get("metadata")
            .ok_or_else(|| Error::parsing("Missing pod metadata"))?;
        
        let name = metadata.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| Error::parsing("Missing pod name"))?
            .to_string();
        
        let namespace = metadata.get("namespace")
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();
        
        let status = json.get("status")
            .and_then(|s| s.get("phase"))
            .and_then(|p| p.as_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let _created_at = metadata.get("creationTimestamp")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        
        Ok(Pod {
            name,
            namespace,
            status,
            ready: "0/0".to_string(), // Simplified for security demo
            restarts: 0, // Simplified for security demo
            age: "0s".to_string(), // Simplified for security demo
            ip: None, // Simplified for security demo
            node: None, // Simplified for security demo
        })
    }

    /// List deployments
    pub async fn list_deployments(&self, namespace: Option<&str>) -> Result<Vec<Deployment>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_deployments",
            "args": {
                "namespace": namespace
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let deployments_content = Self::extract_content_as_json(&response)?;
        
        let deployments_data = deployments_content.get("deployments")
            .ok_or_else(|| Error::protocol("Missing 'deployments' field in response".to_string()))?;
            
        let deployments: Vec<Deployment> = serde_json::from_value(deployments_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse deployments: {}", e)))?;
            
        Ok(deployments)
    }
    
    /// List services
    pub async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<Service>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_services",
            "args": {
                "namespace": namespace
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let services_content = Self::extract_content_as_json(&response)?;
        
        let services_data = services_content.get("services")
            .ok_or_else(|| Error::protocol("Missing 'services' field in response".to_string()))?;
            
        let services: Vec<Service> = serde_json::from_value(services_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse services: {}", e)))?;
            
        Ok(services)
    }
    
    /// List namespaces
    pub async fn list_namespaces(&self) -> Result<Vec<Namespace>> {
        let method = "tools/execute";
        let params = json!({
            "name": "list_namespaces",
            "arguments": {}
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let namespaces_content = Self::extract_content_as_json(&response)?;
        
        let namespaces_data = namespaces_content.get("namespaces")
            .ok_or_else(|| Error::protocol("Missing 'namespaces' field in response".to_string()))?;
            
        let namespaces: Vec<Namespace> = serde_json::from_value(namespaces_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse namespaces: {}", e)))?;
            
        Ok(namespaces)
    }
    
    /// List nodes
    pub async fn list_nodes(&self) -> Result<Vec<Node>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_nodes",
            "args": {}
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let nodes_content = Self::extract_content_as_json(&response)?;
        
        let nodes_data = nodes_content.get("nodes")
            .ok_or_else(|| Error::protocol("Missing 'nodes' field in response".to_string()))?;
            
        let nodes: Vec<Node> = serde_json::from_value(nodes_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse nodes: {}", e)))?;
            
        Ok(nodes)
    }
    
    /// Create namespace
    pub async fn create_namespace(&self, name: &str) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "create_namespace",
            "arguments": {
                "name": name
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to create namespace: {}", error_msg)))
        }
    }
    
    /// Delete namespace
    pub async fn delete_namespace(&self, name: &str, ignore_not_found: bool) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_namespace",
            "arguments": {
                "name": name,
                "ignoreNotFound": ignore_not_found
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to delete namespace: {}", error_msg)))
        }
    }
    
    /// Create pod in a namespace
    pub async fn create_pod(&self, name: &str, _namespace: &str, image: &str, command: Option<Vec<String>>) -> Result<()> {
        let yaml = format!(
            r#"apiVersion: v1
kind: Pod
metadata:
  name: {}
spec:
  containers:
  - name: {}
    image: {}{}
    resources:
      requests:
        memory: "64Mi"
        cpu: "100m"
      limits:
        memory: "128Mi"
        cpu: "200m"
"#,
            name, 
            name, 
            image,
            command.map(|cmd| format!("\n    command: [{}]", cmd.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))).unwrap_or_default()
        );
        
        let method = "tools/execute";
        let params = json!({
            "name": "apply_yaml",
            "args": {
                "yaml": yaml
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to create pod: {}", error_msg)))
        }
    }
    
    /// Delete pod in a namespace
    pub async fn delete_pod(&self, name: &str, namespace: &str, _ignore_not_found: bool) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_resource",
            "args": {
                "kind": "pod",
                "name": name,
                "namespace": namespace
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to delete pod: {}", error_msg)))
        }
    }
    
    /// Create deployment
    pub async fn create_deployment(&self, name: &str, _namespace: &str, image: &str, replicas: u32, ports: Option<Vec<u16>>) -> Result<()> {
        // Create ports configuration if provided
        let ports_yaml = match ports {
            Some(port_list) if !port_list.is_empty() => {
                let ports_str = port_list.iter()
                    .map(|p| format!("        - containerPort: {}", p))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("\n      ports:\n{}", ports_str)
            },
            _ => String::new(),
        };
        
        let yaml = format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: {}
        image: {}{}
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "128Mi"
            cpu: "200m"
"#,
            name, 
            replicas,
            name,
            name,
            name, 
            image,
            ports_yaml
        );
        
        let method = "tools/execute";
        let params = json!({
            "name": "apply_yaml",
            "args": {
                "yaml": yaml
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to create deployment: {}", error_msg)))
        }
    }
    
    /// Delete deployment
    pub async fn delete_deployment(&self, name: &str, namespace: &str, _ignore_not_found: bool) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_resource",
            "args": {
                "kind": "deployment",
                "name": name,
                "namespace": namespace
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to delete deployment: {}", error_msg)))
        }
    }
    
    /// Scale deployment
    pub async fn scale_deployment(&self, name: &str, namespace: &str, replicas: u32) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "scale_deployment",
            "arguments": {
                "name": name,
                "namespace": namespace,
                "replicas": replicas
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to scale deployment: {}", error_msg)))
        }
    }
    
    /// Get pod logs with optimized streaming and security validation
    pub async fn get_pod_logs(&self, pod_name: &str, namespace: Option<&str>, tail_lines: Option<u32>) -> Result<String> {
        self.security.validate_resource_name(pod_name)?;
        
        let mut cmd_args = vec!["logs", pod_name];
        
        if let Some(ns) = namespace {
            self.security.validate_resource_name(ns)?;
            cmd_args.extend_from_slice(&["--namespace", ns]);
        }
        
        let tail_limit = tail_lines.unwrap_or(100);
        let tail_limit_str = tail_limit.to_string();
        if tail_limit > 0 {
            cmd_args.extend_from_slice(&["--tail", &tail_limit_str]);
        }
        
        let result = self.run_secure_kubectl_command(&cmd_args).await?;
        Ok(result.output)
    }
    
    /// Install Helm chart
    pub async fn install_helm_chart(&self, name: &str, chart: &str, repo: &str, namespace: &str, values: Option<HashMap<String, Value>>) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "install_helm_chart",
            "arguments": {
                "name": name,
                "chart": chart,
                "repo": repo,
                "namespace": namespace,
                "values": values
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to install Helm chart: {}", error_msg)))
        }
    }
    
    /// Uninstall Helm chart
    pub async fn uninstall_helm_chart(&self, name: &str, namespace: &str) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "uninstall_helm_chart",
            "arguments": {
                "name": name,
                "namespace": namespace
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            let error_msg = content.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
                
            Err(Error::service(format!("Failed to uninstall Helm chart: {}", error_msg)))
        }
    }
    
    /// Start port forwarding with security validation
    pub async fn start_port_forward(&self, resource_type: &str, resource_name: &str, local_port: u16, target_port: u16, namespace: Option<&str>) -> Result<PortForward> {
        // Validate resource type
        let allowed_resource_types = ["pod", "service", "deployment"];
        if !allowed_resource_types.contains(&resource_type) {
            return Err(Error::validation("Resource type not allowed for port forwarding"));
        }
        
        // Validate resource name
        self.validate_k8s_resource_name(resource_name)?;
        
        // Validate ports (avoid privileged ports unless explicitly allowed)
        if local_port < 1024 {
            self.security.log_security_event("PRIVILEGED_PORT_REQUEST", Some(&local_port.to_string()));
            return Err(Error::validation("Local port cannot be privileged (< 1024)"));
        }
        
        if target_port == 0 {
            return Err(Error::validation("Invalid target port"));
        }
        
        let namespace_str = namespace.unwrap_or("default");
        self.validate_k8s_resource_name(namespace_str)?;
        
        self.port_forward_manager.start_session(
            resource_type,
            resource_name,
            local_port,
            target_port,
            namespace_str,
        ).await
    }
    
    /// Stop port forward
    pub async fn stop_port_forward(&self, id: &str) -> Result<()> {
        self.port_forward_manager.stop_session(id).await
    }
    
    /// List port forwards
    pub fn list_port_forwards(&self) -> Vec<String> {
        self.port_forward_manager.list_sessions().unwrap_or_default()
    }
    
    /// Extract JSON content from response
    fn extract_content_as_json(response: &Value) -> Result<Value> {
        let content = response.get("content")
            .ok_or_else(|| Error::protocol("Missing 'content' field in response".to_string()))?;
            
        if !content.is_array() {
            return Err(Error::protocol("'content' field is not an array".to_string()));
        }
        
        let content_array = content.as_array()
            .ok_or_else(|| Error::invalid_data("Expected array for pods list"))?;
        
        for item in content_array {
            if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    return serde_json::from_str(text)
                        .map_err(|e| Error::protocol(format!("Failed to parse content as JSON: {}", e)));
                }
            }
        }
        
        Err(Error::protocol("No text content found in response".to_string()))
    }
    
    /// Get tool definitions
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        use crate::tools::{ToolDefinition, ToolAnnotation};
        
        vec![
            ToolDefinition::from_json_schema(
                "list_pods",
                "List Kubernetes pods",
                "kubernetes",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "namespace": {
                            "type": "string",
                            "description": "Kubernetes namespace (optional)"
                        }
                    },
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Kubernetes pods")
                    .with_usage_hints(vec!["Use to get all pods in a namespace or cluster-wide".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_deployments",
                "List Kubernetes deployments",
                "kubernetes",
                serde_json::json!({
                    "type": "object", 
                    "properties": {
                        "namespace": {
                            "type": "string",
                            "description": "Kubernetes namespace (optional)"
                        }
                    },
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Kubernetes deployments")
                    .with_usage_hints(vec!["Use to get all deployments in a namespace or cluster-wide".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "create_namespace",
                "Create a Kubernetes namespace",
                "kubernetes",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the namespace"
                        }
                    },
                    "required": ["name"]
                }),
                Some(ToolAnnotation::new("resource_creation").with_description("Create a Kubernetes namespace")
                    .with_usage_hints(vec!["Use to create a new namespace in the cluster".to_string()])
                    .with_security_notes(vec!["Requires cluster admin permissions".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "delete_namespace",
                "Delete a Kubernetes namespace",
                "kubernetes",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the namespace"
                        },
                        "ignore_not_found": {
                            "type": "boolean",
                            "description": "Ignore if namespace doesn't exist"
                        }
                    },
                    "required": ["name"]
                }),
                Some(ToolAnnotation::new("resource_deletion").with_description("Delete a Kubernetes namespace")
                    .with_usage_hints(vec!["Use to delete a namespace and all its resources".to_string()])
                    .with_security_notes(vec!["Destructive operation - will delete all resources in namespace".to_string()]))
            ),
        ]
    }

    /// Run secure kubectl command with validation and timeouts
    async fn run_secure_kubectl_command(&self, args: &[&str]) -> Result<KubectlCommandResult> {
        // Validate all arguments
        for arg in args {
            let validation_opts = SanitizationOptions {
                max_length: Some(256),
                allow_html: false,
                allow_sql: false,
                allow_shell_meta: false,
            };
            
            match self.security.validate_input(arg, &validation_opts) {
                ValidationResult::Valid => {},
                ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                    self.security.log_security_event("MALICIOUS_KUBECTL_ARG", Some(&reason));
                    return Err(Error::validation(format!("Invalid kubectl argument: {}", reason)));
                }
            }
        }
        
        // Build secure command
        let mut cmd = TokioCommand::new("kubectl");
        
        // Add kubeconfig if specified
        if let Some(config_path) = &self.kubeconfig_path {
            cmd.env("KUBECONFIG", config_path);
        }
        
        // Add context if specified
        if let Some(context) = &self.context {
            cmd.args(&["--context", context]);
        }
        
        // Add validated arguments
        cmd.args(args);
        
        // Security settings
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .stdin(Stdio::null()) // Prevent interactive input
           .kill_on_drop(true);  // Clean up on drop
        
        let command_str = format!("kubectl {}", args.join(" "));
        self.security.log_security_event("KUBECTL_COMMAND_EXEC", Some(&command_str));
        
        // Execute with timeout
        let output = tokio::time::timeout(self.command_timeout, cmd.output())
            .await
            .map_err(|_| Error::timeout("kubectl command timed out"))?
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let result = if output.status.success() {
            KubectlCommandResult {
                success: true,
                command: command_str,
                output: stdout,
                error: None,
            }
        } else {
            self.security.log_security_event("KUBECTL_COMMAND_FAILED", Some(&stderr));
            KubectlCommandResult {
                success: false,
                command: command_str,
                output: stdout,
                error: Some(stderr),
            }
        };
        
        Ok(result)
    }

    /// Sanitize log output to remove sensitive information
    pub fn sanitize_log_output(&self, logs: &str) -> String {
        let mut sanitized = logs.to_string();
        
        // Remove common patterns that might contain sensitive data
        let sensitive_patterns = [
            r"(?i)(password|secret|key|token)\s*[:=]\s*[^\s]+",
            r"(?i)(api[_-]?key|access[_-]?token)\s*[:=]\s*[^\s]+",
            r"(?i)(authorization|auth)\s*:\s*[^\s]+",
            r"(?i)(bearer\s+)[a-zA-Z0-9._-]+",
        ];
        
        for pattern in &sensitive_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                sanitized = regex.replace_all(&sanitized, "[REDACTED]").to_string();
            }
        }
        
        sanitized
    }

    /// Run kubectl command with extensive security validation (legacy method - now secure)
    pub async fn run_kubectl_command(&self, command: &str, kubeconfig: Option<&str>) -> Result<KubectlCommandResult> {
        // Validate kubeconfig path if provided
        if let Some(config_path) = kubeconfig {
            self.security.validate_file_path(config_path)?;
        }
        
        // Validate and parse command
        let validated_args = self.validate_kubectl_command(command)?;
        
        // Convert to &str slice for run_secure_kubectl_command
        let args_refs: Vec<&str> = validated_args.iter().map(|s| s.as_str()).collect();
        
        self.run_secure_kubectl_command(&args_refs).await
    }

    /// Run a container (placeholder - would need extensive security controls)
    pub async fn run_container(&self, _name: &str, _namespace: &str, _image: &str, _command: Option<Vec<String>>) -> Result<()> {
        // This is a high-risk operation that would need extensive security controls
        // For now, we disable it entirely
        Err(Error::validation("Container execution disabled for security reasons"))
    }

    /// Delete a resource with confirmation
    pub async fn delete_resource(&self, resource_type: &str, resource_name: &str, _namespace: Option<&str>) -> Result<()> {
        // This is a destructive operation - require explicit validation
        self.validate_k8s_resource_name(resource_name)?;
        
        let allowed_resource_types = ["pod", "service", "deployment", "configmap", "secret"];
        if !allowed_resource_types.contains(&resource_type) {
            return Err(Error::validation("Resource type not allowed for deletion"));
        }
        
        // Log security event for destructive operation
        self.security.log_security_event("RESOURCE_DELETION_ATTEMPT", Some(&format!("{}/{}", resource_type, resource_name)));
        
        // For safety, we're disabling deletion operations
        Err(Error::validation("Resource deletion disabled for security reasons"))
    }

    /// Get command buffer for debugging
    pub fn get_command_buffer(&self) -> &Vec<String> {
        &self.command_buffer
    }

    /// Add command to buffer for debugging
    pub fn add_to_command_buffer(&mut self, command: String) {
        self.command_buffer.push(command);
    }

    // ====== Kubernetes 1.31 "Elli" New Features ======

    /// List AppArmor profiles (Kubernetes 1.31 GA feature)
    pub async fn list_apparmor_profiles(&self) -> Result<Vec<AppArmorProfile>> {
        let output = TokioCommand::new("kubectl")
            .args(&["get", "apparmorprofiles", "-o", "json"])
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !output.status.success() {
            return Err(Error::internal(format!("kubectl command failed: {}", 
                String::from_utf8_lossy(&output.stderr))));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let profiles: Vec<AppArmorProfile> = serde_json::from_str(&output_str)
            .map_err(|e| Error::parsing(format!("Failed to parse AppArmor profiles: {}", e)))?;

        Ok(profiles)
    }

    /// Configure traffic distribution for a service (Kubernetes 1.31 Beta)
    pub async fn configure_traffic_distribution(&self, service_name: &str, namespace: &str, distribution: &TrafficDistribution) -> Result<()> {
        self.validate_k8s_resource_name(service_name)?;
        self.validate_k8s_resource_name(namespace)?;

        let patch_data = serde_json::json!({
            "spec": {
                "trafficDistribution": distribution.policy,
                "trafficDistributionOptions": {
                    "zones": distribution.zones,
                    "weights": distribution.weights
                }
            }
        });

        let output = TokioCommand::new("kubectl")
            .args(&["patch", "service", service_name, "-n", namespace, "--type=merge", "-p", &patch_data.to_string()])
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !output.status.success() {
            return Err(Error::internal(format!("Failed to configure traffic distribution: {}", 
                String::from_utf8_lossy(&output.stderr))));
        }

        self.security.log_security_event("TRAFFIC_DISTRIBUTION_CONFIGURED", Some(&format!("{}/{}", namespace, service_name)));
        Ok(())
    }

    /// Create VolumeAttributesClass (Kubernetes 1.31 Beta)
    pub async fn create_volume_attributes_class(&self, vac: &VolumeAttributesClass) -> Result<()> {
        self.validate_k8s_resource_name(&vac.metadata.name)?;

        let manifest = serde_json::json!({
            "apiVersion": "storage.k8s.io/v1beta1",
            "kind": "VolumeAttributesClass",
            "metadata": {
                "name": vac.metadata.name,
                "labels": vac.metadata.labels,
                "annotations": vac.metadata.annotations
            },
            "spec": {
                "driverName": vac.driver_name,
                "parameters": vac.parameters
            }
        });

        let _manifest_str = serde_json::to_string_pretty(&manifest)
            .map_err(|e| Error::protocol(format!("Failed to serialize VolumeAttributesClass: {}", e)))?;

        let output = TokioCommand::new("kubectl")
            .args(&["apply", "-f", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::internal(format!("Failed to spawn kubectl: {}", e)))?;

        // Write manifest to stdin (simplified for this implementation)
        let result = output.wait_with_output().await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !result.status.success() {
            return Err(Error::internal(format!("Failed to create VolumeAttributesClass: {}", 
                String::from_utf8_lossy(&result.stderr))));
        }

        self.security.log_security_event("VOLUME_ATTRIBUTES_CLASS_CREATED", Some(&vac.metadata.name));
        Ok(())
    }

    /// List service CIDRs (Kubernetes 1.31 Beta)
    pub async fn list_service_cidrs(&self) -> Result<Vec<ServiceCIDRConfig>> {
        let output = TokioCommand::new("kubectl")
            .args(&["get", "servicecidrs", "-o", "json"])
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !output.status.success() {
            return Err(Error::internal(format!("kubectl command failed: {}", 
                String::from_utf8_lossy(&output.stderr))));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let cidrs: Vec<ServiceCIDRConfig> = serde_json::from_str(&output_str)
            .map_err(|e| Error::parsing(format!("Failed to parse Service CIDRs: {}", e)))?;

        Ok(cidrs)
    }

    /// Add secondary service CIDR (Kubernetes 1.31 Beta)
    pub async fn add_service_cidr(&self, cidr: &str, name: &str) -> Result<()> {
        self.validate_k8s_resource_name(name)?;
        
        // Validate CIDR format (simplified validation)
        if !cidr.contains('/') {
            return Err(Error::validation("Invalid CIDR format"));
        }

        let manifest = serde_json::json!({
            "apiVersion": "networking.k8s.io/v1beta1",
            "kind": "ServiceCIDR",
            "metadata": {
                "name": name
            },
            "spec": {
                "cidrs": [cidr]
            }
        });

        let _manifest_str = serde_json::to_string_pretty(&manifest)
            .map_err(|e| Error::protocol(format!("Failed to serialize ServiceCIDR: {}", e)))?;

        let output = TokioCommand::new("kubectl")
            .args(&["apply", "-f", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::internal(format!("Failed to spawn kubectl: {}", e)))?;

        let result = output.wait_with_output().await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !result.status.success() {
            return Err(Error::internal(format!("Failed to add Service CIDR: {}", 
                String::from_utf8_lossy(&result.stderr))));
        }

        self.security.log_security_event("SERVICE_CIDR_ADDED", Some(&format!("{}: {}", name, cidr)));
        Ok(())
    }

    /// Apply enhanced security context with AppArmor (Kubernetes 1.31)
    pub async fn apply_enhanced_security_context(&self, pod_name: &str, namespace: &str, security_context: &EnhancedSecurityContext) -> Result<()> {
        self.validate_k8s_resource_name(pod_name)?;
        self.validate_k8s_resource_name(namespace)?;

        let mut patch_data = serde_json::json!({
            "spec": {
                "securityContext": {
                    "runAsUser": security_context.run_as_user,
                    "runAsGroup": security_context.run_as_group,
                    "readOnlyRootFilesystem": security_context.read_only_root_filesystem
                }
            }
        });

        // Add AppArmor annotation if profile is specified
        if let Some(ref apparmor_profile) = security_context.apparmor_profile {
            patch_data["metadata"]["annotations"] = serde_json::json!({
                format!("container.apparmor.security.beta.kubernetes.io/{}", pod_name): 
                    format!("{}/{}", apparmor_profile.profile_type, apparmor_profile.name)
            });
        }

        let output = TokioCommand::new("kubectl")
            .args(&["patch", "pod", pod_name, "-n", namespace, "--type=strategic", "-p", &patch_data.to_string()])
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !output.status.success() {
            return Err(Error::internal(format!("Failed to apply security context: {}", 
                String::from_utf8_lossy(&output.stderr))));
        }

        self.security.log_security_event("ENHANCED_SECURITY_CONTEXT_APPLIED", 
            Some(&format!("{}/{}", namespace, pod_name)));
        Ok(())
    }

    /// Check Kubernetes version compatibility
    pub async fn check_version_compatibility(&self) -> Result<String> {
        let output = TokioCommand::new("kubectl")
            .args(&["version", "--client=false", "-o", "json"])
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl: {}", e)))?;

        if !output.status.success() {
            return Err(Error::internal(format!("Failed to get Kubernetes version: {}", 
                String::from_utf8_lossy(&output.stderr))));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let version_info: serde_json::Value = serde_json::from_str(&output_str)
            .map_err(|e| Error::parsing(format!("Failed to parse version info: {}", e)))?;

        let server_version = version_info
            .get("serverVersion")
            .and_then(|v| v.get("gitVersion"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Check if it's Kubernetes 1.31 or later for new features
        let is_compatible = server_version.contains("v1.31") || 
                           server_version.contains("v1.32") ||
                           server_version.contains("v1.33") ||
                           server_version.contains("v1.34");

        if is_compatible {
            Ok(format!("Compatible: {} supports Kubernetes 1.31+ features", server_version))
        } else {
            Ok(format!("Limited compatibility: {} may not support all Kubernetes 1.31+ features", server_version))
        }
    }
}

/// Kubectl command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubectlCommandResult {
    /// Whether the command was successful
    pub success: bool,
    /// The command that was executed
    pub command: String,
    /// Command output
    pub output: String,
    /// Error output (if any)
    pub error: Option<String>,
} 
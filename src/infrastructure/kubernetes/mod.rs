use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::{ToolDefinition, ToolAnnotation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};

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
            let mut sessions = self.sessions.lock().unwrap();
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
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(mut child) = sessions.remove(id) {
            // Terminate the process
            let _ = child.kill().await;
            Ok(())
        } else {
            Err(Error::not_found(format!("Port-forward session not found: {}", id)))
        }
    }
    
    /// Get active port forward sessions
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }
}

/// Kubernetes client for MCP
pub struct KubernetesClient {
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Port forward manager
    port_forward_manager: PortForwardManager,
    /// Namespace
    namespace: Option<String>,
}

impl KubernetesClient {
    /// Create a new Kubernetes client
    pub fn new(lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        // Check if kubectl is available
        Self::check_kubectl()?;
        
        Ok(Self {
            lifecycle,
            port_forward_manager: PortForwardManager::new(),
            namespace: None,
        })
    }
    
    /// Check if kubectl is available
    fn check_kubectl() -> Result<()> {
        match Command::new("kubectl").arg("version").stdout(Stdio::null()).status() {
            Ok(status) if status.success() => Ok(()),
            _ => Err(Error::config("kubectl not found or not properly configured".to_string())),
        }
    }
    
    /// List pods
    pub async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<Pod>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_pods",
            "args": {
                "namespace": namespace
            }
        });
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
        let pods_content = Self::extract_content_as_json(&response)?;
        
        let pods_data = pods_content.get("pods")
            .ok_or_else(|| Error::protocol("Missing 'pods' field in response".to_string()))?;
            
        let pods: Vec<Pod> = serde_json::from_value(pods_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse pods: {}", e)))?;
            
        Ok(pods)
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
    
    /// Get pod logs
    pub async fn get_pod_logs(&self, name: &str, namespace: &str, container: Option<&str>, tail: Option<u32>) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_pod_logs",
            "args": {
                "namespace": namespace,
                "pod_name": name,
                "container": container,
                "tail": tail
            }
        });
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
        let content = Self::extract_content_as_json(&response)?;
        
        let logs = content.get("logs")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        Ok(logs.to_string())
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
        
        let response = self.lifecycle.send_request(method, Some(params)).await?;
        
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
    
    /// Start port forward
    pub async fn start_port_forward(&self, resource_type: &str, resource_name: &str, local_port: u16, target_port: u16, namespace: &str) -> Result<PortForward> {
        self.port_forward_manager.start_session(resource_type, resource_name, local_port, target_port, namespace).await
    }
    
    /// Stop port forward
    pub async fn stop_port_forward(&self, id: &str) -> Result<()> {
        self.port_forward_manager.stop_session(id).await
    }
    
    /// List port forwards
    pub fn list_port_forwards(&self) -> Vec<String> {
        self.port_forward_manager.list_sessions()
    }
    
    /// Extract JSON content from response
    fn extract_content_as_json(response: &Value) -> Result<Value> {
        let content = response.get("content")
            .ok_or_else(|| Error::protocol("Missing 'content' field in response".to_string()))?;
            
        if !content.is_array() {
            return Err(Error::protocol("'content' field is not an array".to_string()));
        }
        
        let content_array = content.as_array().unwrap();
        
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
        use crate::tools::ParameterSchema;
        use std::collections::HashMap;
        
        let mut namespace_param = HashMap::new();
        namespace_param.insert("namespace".to_string(), ParameterSchema {
            description: Some("Kubernetes namespace".to_string()),
            param_type: "string".to_string(),
            required: false,
            default: None,
            enum_values: Vec::new(),
            properties: HashMap::new(),
            items: None,
            additional: HashMap::new(),
        });

        let mut name_param = HashMap::new();
        name_param.insert("name".to_string(), ParameterSchema {
            description: Some("Name of the namespace".to_string()),
            param_type: "string".to_string(),
            required: true,
            default: None,
            enum_values: Vec::new(),
            properties: HashMap::new(),
            items: None,
            additional: HashMap::new(),
        });

        let mut delete_params = HashMap::new();
        delete_params.insert("name".to_string(), ParameterSchema {
            description: Some("Name of the namespace".to_string()),
            param_type: "string".to_string(),
            required: true,
            default: None,
            enum_values: Vec::new(),
            properties: HashMap::new(),
            items: None,
            additional: HashMap::new(),
        });
        delete_params.insert("ignoreNotFound".to_string(), ParameterSchema {
            description: Some("Ignore if namespace doesn't exist".to_string()),
            param_type: "boolean".to_string(),
            required: false,
            default: None,
            enum_values: Vec::new(),
            properties: HashMap::new(),
            items: None,
            additional: HashMap::new(),
        });

        vec![
            ToolDefinition {
                name: "list_pods".to_string(),
                description: "List Kubernetes pods".to_string(),
                version: "1.0.0".to_string(),
                parameters: namespace_param.clone(),
                annotations: ToolAnnotation {
                    read_only: true,
                    has_side_effects: false,
                    destructive: false,
                    requires_confirmation: false,
                    ..Default::default()
                },
                lifecycle_manager: Some(self.lifecycle.clone()),
            },
            ToolDefinition {
                name: "list_deployments".to_string(),
                description: "List Kubernetes deployments".to_string(),
                version: "1.0.0".to_string(),
                parameters: namespace_param,
                annotations: ToolAnnotation {
                    read_only: true,
                    has_side_effects: false,
                    destructive: false,
                    requires_confirmation: false,
                    ..Default::default()
                },
                lifecycle_manager: Some(self.lifecycle.clone()),
            },
            ToolDefinition {
                name: "create_namespace".to_string(),
                description: "Create a Kubernetes namespace".to_string(),
                version: "1.0.0".to_string(),
                parameters: name_param,
                annotations: ToolAnnotation {
                    read_only: false,
                    has_side_effects: true,
                    destructive: false,
                    requires_confirmation: true,
                    ..Default::default()
                },
                lifecycle_manager: Some(self.lifecycle.clone()),
            },
            ToolDefinition {
                name: "delete_namespace".to_string(),
                description: "Delete a Kubernetes namespace".to_string(),
                version: "1.0.0".to_string(),
                parameters: delete_params,
                annotations: ToolAnnotation {
                    read_only: false,
                    has_side_effects: true,
                    destructive: true,
                    requires_confirmation: true,
                    ..Default::default()
                },
                lifecycle_manager: Some(self.lifecycle.clone()),
            },
            // Add more tool definitions as needed
        ]
    }

    /// Execute kubectl command
    pub async fn run_kubectl_command(&self, command: &str, kubeconfig: Option<&str>) -> Result<KubectlCommandResult> {
        let command_with_prefix = if command.trim().starts_with("kubectl") {
            command.to_string()
        } else {
            format!("kubectl {}", command)
        };
        
        let mut cmd = TokioCommand::new("sh");
        cmd.arg("-c");
        
        // Add kubeconfig if provided
        let full_command = if let Some(config_path) = kubeconfig {
            format!("KUBECONFIG={} {}", config_path, command_with_prefix)
        } else {
            command_with_prefix
        };
        
        cmd.arg(&full_command);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        log::info!("Executing kubectl command: {}", full_command);
        
        let output = cmd.output().await
            .map_err(|e| Error::internal(format!("Failed to execute kubectl command: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let result = if output.status.success() {
            KubectlCommandResult {
                success: true,
                command: command.to_string(),
                output: stdout,
                error: None,
            }
        } else {
            KubectlCommandResult {
                success: false,
                command: command.to_string(),
                output: stdout,
                error: Some(stderr),
            }
        };
        
        Ok(result)
    }

    /// Run a container
    pub async fn run_container(&self, _name: &str, _namespace: &str, _image: &str, _command: Option<Vec<String>>) -> Result<()> {
        // Implementation pending
        Ok(())
    }

    /// Delete a resource
    pub async fn delete_resource(&self, _kind: &str, _name: &str, _namespace: &str, _ignore_not_found: bool) -> Result<()> {
        // Implementation pending
        Ok(())
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
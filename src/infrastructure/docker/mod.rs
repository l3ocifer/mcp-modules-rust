use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::process::Command;

/// Container runtime type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Containerd,
}

impl std::fmt::Display for ContainerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerRuntime::Docker => write!(f, "docker"),
            ContainerRuntime::Podman => write!(f, "podman"),
            ContainerRuntime::Containerd => write!(f, "containerd"),
        }
    }
}

impl ContainerRuntime {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerRuntime::Docker => "docker",
            ContainerRuntime::Podman => "podman",
            ContainerRuntime::Containerd => "nerdctl",
        }
    }
}

/// Container representation with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container ID
    pub id: String,
    /// Container image
    pub image: String,
    /// Container status
    pub status: String,
    /// Container name
    pub name: String,
    /// Runtime used
    pub runtime: ContainerRuntime,
    /// Created timestamp
    pub created: Option<SystemTime>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Resource usage
    pub resources: Option<ResourceUsage>,
    /// Security context
    pub security_context: Option<SecurityContext>,
    /// Is rootless container
    pub rootless: bool,
    /// Pod information (for Podman)
    pub pod: Option<String>,
}

/// Port mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Host port
    pub host_port: u16,
    /// Container port
    pub container_port: u16,
    /// Protocol (tcp/udp)
    pub protocol: String,
    /// Host IP to bind to
    pub host_ip: Option<String>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// Network I/O
    pub network_io: NetworkIO,
    /// Block I/O
    pub block_io: BlockIO,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
}

/// Block I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockIO {
    /// Bytes read
    pub read_bytes: u64,
    /// Bytes written
    pub write_bytes: u64,
}

/// Security context for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Run as user ID
    pub run_as_user: Option<u32>,
    /// Run as group ID
    pub run_as_group: Option<u32>,
    /// Read-only root filesystem
    pub read_only_root_filesystem: bool,
    /// Security capabilities to add
    pub capabilities_add: Vec<String>,
    /// Security capabilities to drop
    pub capabilities_drop: Vec<String>,
    /// SELinux label
    pub selinux_label: Option<String>,
    /// AppArmor profile
    pub apparmor_profile: Option<String>,
}

/// Container creation parameters with enhanced security and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCreateParams {
    /// Container image to use
    pub image: String,
    /// Optional container name
    pub name: Option<String>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Resource limits
    pub resources: Option<ResourceLimits>,
    /// Security context
    pub security_context: Option<SecurityContext>,
    /// Network configuration
    pub network: Option<NetworkConfig>,
    /// Restart policy
    pub restart_policy: RestartPolicy,
    /// Health check configuration
    pub health_check: Option<HealthCheck>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// User-defined networks
    pub networks: Vec<String>,
    /// Init process
    pub init: bool,
    /// Runtime to use
    pub runtime: ContainerRuntime,
    /// Enable rootless mode
    pub rootless: bool,
    /// Pod name (for Podman)
    pub pod: Option<String>,
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Source path on host
    pub source: String,
    /// Target path in container
    pub target: String,
    /// Mount type (bind, volume, tmpfs)
    pub mount_type: String,
    /// Read-only mount
    pub read_only: bool,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (cores)
    pub cpu: Option<f64>,
    /// Memory limit in bytes
    pub memory: Option<u64>,
    /// Memory swap limit in bytes
    pub memory_swap: Option<u64>,
    /// PID limit
    pub pids_limit: Option<u32>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network mode (bridge, host, none, container:name)
    pub mode: String,
    /// DNS servers
    pub dns: Vec<String>,
    /// DNS search domains
    pub dns_search: Vec<String>,
    /// Hostname
    pub hostname: Option<String>,
}

/// Restart policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    No,
    Always,
    OnFailure { max_retry_count: u32 },
    UnlessStopped,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Health check command
    pub test: Vec<String>,
    /// Interval between checks
    pub interval_seconds: u64,
    /// Timeout for each check
    pub timeout_seconds: u64,
    /// Start period before first check
    pub start_period_seconds: u64,
    /// Number of retries before marking unhealthy
    pub retries: u32,
}

/// Podman-specific pod configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodConfig {
    /// Pod name
    pub name: String,
    /// Pod labels
    pub labels: HashMap<String, String>,
    /// Pod annotations
    pub annotations: HashMap<String, String>,
    /// Shared namespaces
    pub shared_namespaces: Vec<String>,
    /// Pod network configuration
    pub network: Option<NetworkConfig>,
    /// Infrastructure container image
    pub infra_image: Option<String>,
}

/// Modern container client supporting Docker, Podman, and containerd
pub struct ContainerClient {
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module for validation
    security: SecurityModule,
    /// Default runtime
    default_runtime: ContainerRuntime,
    /// Available runtimes
    available_runtimes: Vec<ContainerRuntime>,
}

impl ContainerClient {
    /// Create a new container client with runtime detection
    pub async fn new(lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        let mut available_runtimes = Vec::new();
        let mut default_runtime = ContainerRuntime::Docker;

        // Detect available runtimes
        for runtime in &[
            ContainerRuntime::Podman,
            ContainerRuntime::Docker,
            ContainerRuntime::Containerd,
        ] {
            if Self::is_runtime_available(runtime).await {
                available_runtimes.push(runtime.clone());
                if runtime == &ContainerRuntime::Podman {
                    default_runtime = ContainerRuntime::Podman; // Prefer Podman for security
                }
            }
        }

        if available_runtimes.is_empty() {
            return Err(Error::config("No container runtime available"));
        }

        Ok(Self {
            lifecycle,
            security: SecurityModule::new(),
            default_runtime,
            available_runtimes,
        })
    }

    /// Check if a runtime is available
    async fn is_runtime_available(runtime: &ContainerRuntime) -> bool {
        Command::new(runtime.as_str())
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Execute a container runtime command
    async fn run_runtime_command(
        &self,
        runtime: &ContainerRuntime,
        args: &[&str],
    ) -> Result<String> {
        let output = Command::new(runtime.as_str())
            .args(args)
            .output()
            .await
            .map_err(|e| {
                Error::internal(format!(
                    "Failed to execute {} command: {}",
                    runtime.as_str(),
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::internal(format!(
                "{} command failed: {}",
                runtime.as_str(),
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// List containers with enhanced metadata
    pub async fn list_containers(
        &self,
        runtime: Option<ContainerRuntime>,
        show_all: bool,
    ) -> Result<Vec<Container>> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());

        let format = match runtime {
            ContainerRuntime::Podman => {
                "table {{.ID}}\t{{.Image}}\t{{.Status}}\t{{.Names}}\t{{.Created}}\t{{.Ports}}\t{{.Pod}}"
            },
            _ => {
                "table {{.ID}}\t{{.Image}}\t{{.Status}}\t{{.Names}}\t{{.CreatedAt}}\t{{.Ports}}"
            }
        };

        let mut args = vec!["ps", "--format", format];
        if show_all {
            args.push("-a");
        }

        let output = self.run_runtime_command(&runtime, &args).await?;
        let mut containers = Vec::new();

        for line in output.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 6 {
                let ports = self
                    .parse_ports(parts.get(5).unwrap_or(&""))
                    .unwrap_or_default();

                containers.push(Container {
                    id: parts[0].to_string(),
                    image: parts[1].to_string(),
                    status: parts[2].to_string(),
                    name: parts[3].to_string(),
                    runtime: runtime.clone(),
                    created: None, // Would need to parse timestamp
                    ports,
                    resources: None, // Would need separate stats call
                    security_context: None,
                    rootless: runtime == ContainerRuntime::Podman, // Podman defaults to rootless
                    pod: if runtime == ContainerRuntime::Podman {
                        parts.get(6).map(|s| s.to_string())
                    } else {
                        None
                    },
                });
            }
        }

        Ok(containers)
    }

    /// Parse port mappings from container list output
    fn parse_ports(&self, ports_str: &str) -> Result<Vec<PortMapping>> {
        let mut ports = Vec::new();
        if ports_str.is_empty() {
            return Ok(ports);
        }

        // Parse format like "0.0.0.0:8080->80/tcp"
        for port_mapping in ports_str.split(',') {
            let port_mapping = port_mapping.trim();
            if let Some(arrow_pos) = port_mapping.find("->") {
                let host_part = &port_mapping[..arrow_pos];
                let container_part = &port_mapping[arrow_pos + 2..];

                let host_port = host_part
                    .split(':')
                    .last()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(0);

                let (container_port, protocol) = if let Some(slash_pos) = container_part.find('/') {
                    let port = container_part[..slash_pos].parse::<u16>().unwrap_or(0);
                    let proto = container_part[slash_pos + 1..].to_string();
                    (port, proto)
                } else {
                    (
                        container_part.parse::<u16>().unwrap_or(0),
                        "tcp".to_string(),
                    )
                };

                let host_ip = if host_part.contains(':') {
                    Some(host_part.split(':').next().unwrap_or("0.0.0.0").to_string())
                } else {
                    None
                };

                ports.push(PortMapping {
                    host_port,
                    container_port,
                    protocol,
                    host_ip,
                });
            }
        }

        Ok(ports)
    }

    /// Create a container with enhanced security and modern features
    pub async fn create_container(&self, params: ContainerCreateParams) -> Result<String> {
        // Validate image name for security
        self.security.validate_input(
            &params.image,
            &crate::security::SanitizationOptions {
                max_length: Some(256),
                allow_html: false,
                allow_sql: false,
                allow_shell_meta: false,
            },
        );

        let runtime = &params.runtime;
        let mut args = vec!["run", "-d"];

        // Container name
        if let Some(ref name) = params.name {
            args.extend_from_slice(&["--name", name]);
        }

        // Port mappings
        let mut port_strings = Vec::new();
        for port in &params.ports {
            let port_str = if let Some(ref host_ip) = port.host_ip {
                format!(
                    "{}:{}:{}/{}",
                    host_ip, port.host_port, port.container_port, port.protocol
                )
            } else {
                format!(
                    "{}:{}/{}",
                    port.host_port, port.container_port, port.protocol
                )
            };
            port_strings.push(port_str);
        }
        for port_str in &port_strings {
            args.extend_from_slice(&["-p", port_str]);
        }

        // Environment variables
        let mut env_strings = Vec::new();
        for (key, value) in &params.env {
            let env_str = format!("{}={}", key, value);
            env_strings.push(env_str);
        }
        for env_str in &env_strings {
            args.extend_from_slice(&["-e", env_str]);
        }

        // Volume mounts
        let mut volume_strings = Vec::new();
        for volume in &params.volumes {
            let volume_str = format!(
                "{}:{}:{}",
                volume.source,
                volume.target,
                if volume.read_only { "ro" } else { "rw" }
            );
            volume_strings.push(volume_str);
        }
        for volume_str in &volume_strings {
            args.extend_from_slice(&["-v", volume_str]);
        }

        // Resource limits
        let mut resource_strings = Vec::new();
        if let Some(ref resources) = params.resources {
            if let Some(cpu) = resources.cpu {
                resource_strings.push(("--cpus".to_string(), cpu.to_string()));
            }
            if let Some(memory) = resources.memory {
                resource_strings.push(("-m".to_string(), memory.to_string()));
            }
            if let Some(pids_limit) = resources.pids_limit {
                resource_strings.push(("--pids-limit".to_string(), pids_limit.to_string()));
            }
        }
        for (flag, value) in &resource_strings {
            args.extend_from_slice(&[flag, value]);
        }

        // Security context
        let mut security_strings = Vec::new();
        if let Some(ref security) = params.security_context {
            if let Some(user) = security.run_as_user {
                security_strings.push(("--user".to_string(), user.to_string()));
            }
            if security.read_only_root_filesystem {
                args.push("--read-only");
            }
            for cap in &security.capabilities_add {
                security_strings.push(("--cap-add".to_string(), cap.clone()));
            }
            for cap in &security.capabilities_drop {
                security_strings.push(("--cap-drop".to_string(), cap.clone()));
            }
            if let Some(ref profile) = security.apparmor_profile {
                security_strings.push((
                    "--security-opt".to_string(),
                    format!("apparmor={}", profile),
                ));
            }
        }
        for (flag, value) in &security_strings {
            args.extend_from_slice(&[flag, value]);
        }

        // Network configuration
        if let Some(ref network) = params.network {
            args.extend_from_slice(&["--network", &network.mode]);
            for dns in &network.dns {
                args.extend_from_slice(&["--dns", dns]);
            }
            if let Some(ref hostname) = network.hostname {
                args.extend_from_slice(&["--hostname", hostname]);
            }
        }

        // Restart policy - create owned strings for lifetime
        let restart_policy_str;
        match params.restart_policy {
            RestartPolicy::No => {
                args.extend_from_slice(&["--restart", "no"]);
            }
            RestartPolicy::Always => {
                args.extend_from_slice(&["--restart", "always"]);
            }
            RestartPolicy::OnFailure { max_retry_count } => {
                restart_policy_str = format!("on-failure:{}", max_retry_count);
                args.extend_from_slice(&["--restart", &restart_policy_str]);
            }
            RestartPolicy::UnlessStopped => {
                args.extend_from_slice(&["--restart", "unless-stopped"]);
            }
        }

        // Health check - create owned strings for lifetime
        let mut health_strings = Vec::new();
        if let Some(ref health) = params.health_check {
            let health_cmd = health.test.join(" ");
            health_strings.push(("--health-cmd".to_string(), health_cmd));
            health_strings.push((
                "--health-interval".to_string(),
                format!("{}s", health.interval_seconds),
            ));
            health_strings.push((
                "--health-timeout".to_string(),
                format!("{}s", health.timeout_seconds),
            ));
            health_strings.push(("--health-retries".to_string(), health.retries.to_string()));

            for (flag, value) in &health_strings {
                args.extend_from_slice(&[flag, value]);
            }
        }

        // Labels - create owned strings for lifetime
        let mut label_strings = Vec::new();
        for (key, value) in &params.labels {
            label_strings.push(format!("{}={}", key, value));
        }
        for label_str in &label_strings {
            args.extend_from_slice(&["--label", label_str]);
        }

        // Podman-specific: Pod assignment
        if runtime == &ContainerRuntime::Podman {
            if let Some(ref pod) = params.pod {
                args.extend_from_slice(&["--pod", pod]);
            }
        }

        // Init process
        if params.init {
            args.push("--init");
        }

        // Rootless mode (Podman default)
        if params.rootless && runtime == &ContainerRuntime::Docker {
            // Docker rootless mode would need special setup
            self.security
                .log_security_event("ROOTLESS_REQUEST", Some("Rootless requested for Docker"));
        }

        // Image
        args.push(&params.image);

        let output = self.run_runtime_command(runtime, &args).await?;
        self.security.log_security_event(
            "CONTAINER_CREATED",
            Some(&format!(
                "Runtime: {}, Image: {}",
                runtime.as_str(),
                params.image
            )),
        );

        Ok(output.trim().to_string())
    }

    /// Create a Podman pod
    pub async fn create_pod(&self, config: PodConfig) -> Result<String> {
        if !self.available_runtimes.contains(&ContainerRuntime::Podman) {
            return Err(Error::config("Podman not available for pod creation"));
        }

        let mut args = vec!["pod", "create", "--name", &config.name];

        // Labels - create owned strings for lifetime
        let mut label_strings = Vec::new();
        for (key, value) in &config.labels {
            label_strings.push(format!("{}={}", key, value));
        }
        for label_str in &label_strings {
            args.extend_from_slice(&["--label", label_str]);
        }

        // Network configuration
        if let Some(ref network) = config.network {
            args.extend_from_slice(&["--network", &network.mode]);
            for dns in &network.dns {
                args.extend_from_slice(&["--dns", dns]);
            }
        }

        // Infrastructure image
        if let Some(ref infra_image) = config.infra_image {
            args.extend_from_slice(&["--infra-image", infra_image]);
        }

        let output = self
            .run_runtime_command(&ContainerRuntime::Podman, &args)
            .await?;
        self.security
            .log_security_event("POD_CREATED", Some(&config.name));

        Ok(output.trim().to_string())
    }

    /// Get container statistics
    pub async fn get_container_stats(
        &self,
        container_id: &str,
        runtime: Option<ContainerRuntime>,
    ) -> Result<ResourceUsage> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());
        let output = self
            .run_runtime_command(
                &runtime,
                &["stats", "--no-stream", "--format", "json", container_id],
            )
            .await?;

        // Parse JSON output (format varies by runtime)
        let stats: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse stats: {}", e)))?;

        Ok(ResourceUsage {
            cpu_percent: stats
                .get("CPUPerc")
                .and_then(|v| v.as_str())
                .and_then(|s| s.trim_end_matches('%').parse().ok())
                .unwrap_or(0.0),
            memory_usage: stats
                .get("MemUsage")
                .and_then(|v| v.as_str())
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0),
            memory_limit: stats
                .get("MemUsage")
                .and_then(|v| v.as_str())
                .and_then(|s| s.split('/').nth(1))
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0),
            network_io: NetworkIO {
                rx_bytes: 0,
                tx_bytes: 0,
            }, // Would need detailed parsing
            block_io: BlockIO {
                read_bytes: 0,
                write_bytes: 0,
            }, // Would need detailed parsing
        })
    }

    /// Stop a container
    pub async fn stop_container(
        &self,
        id: &str,
        runtime: Option<ContainerRuntime>,
    ) -> Result<String> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());
        let output = self.run_runtime_command(&runtime, &["stop", id]).await?;
        self.security
            .log_security_event("CONTAINER_STOPPED", Some(id));
        Ok(output)
    }

    /// Start a container
    pub async fn start_container(
        &self,
        id: &str,
        runtime: Option<ContainerRuntime>,
    ) -> Result<String> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());
        let output = self.run_runtime_command(&runtime, &["start", id]).await?;
        self.security
            .log_security_event("CONTAINER_STARTED", Some(id));
        Ok(output)
    }

    /// Remove a container
    pub async fn remove_container(
        &self,
        id: &str,
        force: bool,
        runtime: Option<ContainerRuntime>,
    ) -> Result<String> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());
        let mut args = vec!["rm"];
        if force {
            args.push("-f");
        }
        args.push(id);

        let output = self.run_runtime_command(&runtime, &args).await?;
        self.security
            .log_security_event("CONTAINER_REMOVED", Some(id));
        Ok(output)
    }

    /// Get container logs with enhanced options
    pub async fn get_container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        follow: bool,
        timestamps: bool,
        runtime: Option<ContainerRuntime>,
    ) -> Result<String> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());
        let mut args = vec!["logs"];

        let tail_str;
        if let Some(n) = tail {
            tail_str = n.to_string();
            args.extend_from_slice(&["--tail", &tail_str]);
        }
        if follow {
            args.push("-f");
        }
        if timestamps {
            args.push("-t");
        }
        args.push(id);

        self.run_runtime_command(&runtime, &args).await
    }

    /// Execute command in container with enhanced security
    pub async fn exec_in_container(
        &self,
        id: &str,
        command: &[&str],
        interactive: bool,
        runtime: Option<ContainerRuntime>,
    ) -> Result<String> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());
        let mut args = vec!["exec"];

        if interactive {
            args.extend_from_slice(&["-i", "-t"]);
        }
        args.push(id);
        args.extend_from_slice(command);

        self.security.log_security_event(
            "CONTAINER_EXEC",
            Some(&format!("{}: {}", id, command.join(" "))),
        );
        self.run_runtime_command(&runtime, &args).await
    }

    /// Check security posture of a container
    pub async fn security_scan(
        &self,
        container_id: &str,
        runtime: Option<ContainerRuntime>,
    ) -> Result<SecurityScanResult> {
        let runtime = runtime.unwrap_or(self.default_runtime.clone());

        // Get container inspect information
        let inspect_output = self
            .run_runtime_command(&runtime, &["inspect", container_id])
            .await?;
        let inspect_data: serde_json::Value = serde_json::from_str(&inspect_output)
            .map_err(|e| Error::parsing(format!("Failed to parse inspect data: {}", e)))?;

        let mut issues = Vec::new();
        let mut score = 100;

        // Check if running as root
        if let Some(config) = inspect_data.get(0).and_then(|c| c.get("Config")) {
            if let Some(user) = config.get("User").and_then(|u| u.as_str()) {
                if user.is_empty() || user == "root" || user == "0" {
                    issues.push("Container running as root user".to_string());
                    score -= 20;
                }
            }
        }

        // Check for privileged mode
        if let Some(host_config) = inspect_data.get(0).and_then(|c| c.get("HostConfig")) {
            if let Some(privileged) = host_config.get("Privileged").and_then(|p| p.as_bool()) {
                if privileged {
                    issues.push("Container running in privileged mode".to_string());
                    score -= 30;
                }
            }

            // Check for host network
            if let Some(network_mode) = host_config.get("NetworkMode").and_then(|n| n.as_str()) {
                if network_mode == "host" {
                    issues.push("Container using host network".to_string());
                    score -= 15;
                }
            }
        }

        Ok(SecurityScanResult {
            container_id: container_id.to_string(),
            score,
            issues,
            runtime: runtime.clone(),
            scan_time: SystemTime::now(),
        })
    }

    /// Get available runtimes
    pub fn get_available_runtimes(&self) -> &[ContainerRuntime] {
        &self.available_runtimes
    }

    /// Get default runtime
    pub fn get_default_runtime(&self) -> &ContainerRuntime {
        &self.default_runtime
    }

    /// Get lifecycle manager reference
    pub fn get_lifecycle(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle
    }

    /// Check container runtime health
    pub async fn health_check(&self) -> Result<bool> {
        // Try to run a simple command to check if runtime is healthy
        match self.run_runtime_command(&self.default_runtime, &["version"]).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Deploy a container resource
    pub async fn deploy_resource(&self, resource: crate::infrastructure::ResourceSpec) -> Result<crate::infrastructure::ResourceResult> {
        use crate::infrastructure::ResourceResult;
        
        // Create container from resource specification
        let params = ContainerCreateParams {
            image: resource.spec.get("image")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::validation("Missing 'image' in resource spec"))?
                .to_string(),
            name: Some(resource.name.clone()),
            env: resource.spec.get("environment")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect())
                .unwrap_or_default(),
            ports: resource.spec.get("ports")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| {
                        // Parse port mapping format "host:container"
                        let parts: Vec<&str> = s.split(':').collect();
                        if parts.len() == 2 {
                            if let (Ok(host), Ok(container)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                                Some(PortMapping {
                                    host_port: host,
                                    container_port: container,
                                    protocol: "tcp".to_string(),
                                    host_ip: None,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect())
                .unwrap_or_default(),
            volumes: resource.spec.get("volumes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| {
                        // Parse volume format "source:target"
                        let parts: Vec<&str> = s.split(':').collect();
                        if parts.len() == 2 {
                            Some(VolumeMount {
                                source: parts[0].to_string(),
                                target: parts[1].to_string(),
                                mount_type: "bind".to_string(),
                                read_only: false,
                            })
                        } else {
                            None
                        }
                    })
                    .collect())
                .unwrap_or_default(),
            network: resource.spec.get("network")
                .and_then(|v| v.as_object())
                .map(|obj| NetworkConfig {
                    mode: obj.get("mode").and_then(|v| v.as_str()).unwrap_or("bridge").to_string(),
                    dns: obj.get("dns")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect())
                        .unwrap_or_default(),
                    dns_search: obj.get("dns_search")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect())
                        .unwrap_or_default(),
                    hostname: obj.get("hostname").and_then(|v| v.as_str()).map(|s| s.to_string()),
                }),
            restart_policy: match resource.spec.get("restart_policy").and_then(|v| v.as_str()) {
                Some("always") => RestartPolicy::Always,
                Some("unless-stopped") => RestartPolicy::UnlessStopped,
                Some("on-failure") => RestartPolicy::OnFailure { max_retry_count: 3 },
                _ => RestartPolicy::No,
            },
            resources: None,
            security_context: None,
            health_check: None,
            labels: HashMap::new(),
            networks: Vec::new(),
            init: false,
            runtime: self.default_runtime.clone(),
            rootless: false,
            pod: None,
        };
        
        match self.create_container(params).await {
            Ok(container_id) => Ok(ResourceResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                status: "success".to_string(),
                message: Some(format!("Container {} created with ID: {}", resource.name, container_id)),
            }),
            Err(e) => Ok(ResourceResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                status: "failed".to_string(),
                message: Some(format!("Failed to create container: {}", e)),
            }),
        }
    }

    /// Scale container resources (not directly supported, returns error)
    pub async fn scale_resource(&self, target: crate::infrastructure::ScalingTarget) -> Result<crate::infrastructure::ScalingTargetResult> {
        use crate::infrastructure::ScalingTargetResult;
        
        // Docker doesn't support direct scaling like Kubernetes
        // This would need to be implemented with Docker Swarm or similar
        Ok(ScalingTargetResult {
            resource_id: target.resource_name.clone(),
            previous_count: target.current_count,
            new_count: target.current_count,
            status: "unsupported".to_string(),
        })
    }

    /// Get container runtime metrics
    pub async fn get_metrics(&self) -> Result<serde_json::Value> {
        // Get container stats
        let output = self.run_runtime_command(&self.default_runtime, &["stats", "--no-stream", "--format", "json"]).await?;
        
        let containers: Vec<serde_json::Value> = output
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        
        Ok(serde_json::json!({
            "containers": containers,
            "runtime": self.default_runtime.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// Security scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    /// Container ID
    pub container_id: String,
    /// Security score (0-100)
    pub score: u32,
    /// Security issues found
    pub issues: Vec<String>,
    /// Runtime used
    pub runtime: ContainerRuntime,
    /// Scan timestamp
    pub scan_time: SystemTime,
}

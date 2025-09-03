use crate::config::InfrastructureConfig;
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfrastructureProvider {
    Kubernetes(Value),
    Docker(Value),
    Cloudflare(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureStatus {
    pub providers: Vec<ProviderStatus>,
    pub overall_health: HealthStatus,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub provider: String,
    pub resource_type: String,
    pub name: String,
    pub spec: Value,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    pub resources: Vec<ResourceSpec>,
    pub strategy: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResult {
    pub name: String,
    pub resource_type: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub resources: Vec<ResourceResult>,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTarget {
    pub provider: String,
    pub resource_type: String,
    pub resource_name: String,
    pub namespace: Option<String>,
    pub current_count: u32,
    pub target_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingSpec {
    pub targets: Vec<ScalingTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTargetResult {
    pub resource_id: String,
    pub previous_count: u32,
    pub new_count: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingResult {
    pub scaling_id: String,
    pub results: Vec<ScalingTargetResult>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub storage_usage: f64,
    pub network_io: f64,
    pub provider_metrics: HashMap<String, Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub mod cloudflare;
pub mod docker;
pub mod kubernetes;

use cloudflare::CloudflareClient;
use docker::ContainerClient;
use kubernetes::KubernetesClient;

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
}

/// Infrastructure module for DevOps Architect MCP with production-ready features
pub struct InfrastructureModule {
    /// Infrastructure configuration
    config: InfrastructureConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Cached providers for performance
    _cached_providers: std::sync::RwLock<std::collections::HashMap<String, String>>,
}

impl InfrastructureModule {
    /// Create a new infrastructure module
    pub fn new(config: InfrastructureConfig) -> Self {
        // Create a mock transport for infrastructure module initialization
        let mock_transport = Box::new(crate::transport::MockTransport::new())
            as Box<dyn crate::transport::Transport + Send + Sync>;

        Self {
            config,
            lifecycle: Arc::new(LifecycleManager::new(mock_transport)),
            _cached_providers: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Create with existing lifecycle manager (preferred)
    pub fn with_lifecycle(config: InfrastructureConfig, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            config,
            lifecycle,
            _cached_providers: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Set lifecycle manager
    pub fn set_lifecycle(&mut self, lifecycle: LifecycleManager) {
        self.lifecycle = Arc::new(lifecycle);
    }

    /// Get infrastructure status across all providers
    pub async fn get_status(&self) -> Result<InfrastructureStatus> {
        let mut status = InfrastructureStatus {
            providers: Vec::new(),
            overall_health: HealthStatus::Unknown,
            last_updated: chrono::Utc::now(),
        };

        let mut healthy_providers = 0;
        let mut total_providers = 0;

        // Check Kubernetes status
        if let Ok(k8s_client) = self.kubernetes().await {
            total_providers += 1;
            let k8s_status = match k8s_client.health_check().await {
                Ok(true) => {
                    healthy_providers += 1;
                    HealthStatus::Healthy
                },
                Ok(false) => HealthStatus::Degraded,
                Err(_) => HealthStatus::Unhealthy,
            };
            
            status.providers.push(ProviderStatus {
                name: "kubernetes".to_string(),
                status: k8s_status,
                last_check: chrono::Utc::now(),
                metadata: serde_json::json!({
                    "type": "orchestration",
                    "version": "1.31+"
                }),
            });
        }

        // Check Docker status
        if let Ok(docker_client) = self.docker().await {
            total_providers += 1;
            let docker_status = match docker_client.health_check().await {
                Ok(true) => {
                    healthy_providers += 1;
                    HealthStatus::Healthy
                },
                Ok(false) => HealthStatus::Degraded,
                Err(_) => HealthStatus::Unhealthy,
            };
            
            status.providers.push(ProviderStatus {
                name: "docker".to_string(),
                status: docker_status,
                last_check: chrono::Utc::now(),
                metadata: serde_json::json!({
                    "type": "containerization"
                }),
            });
        }

        // Check Cloudflare status
        if let Ok(cf_client) = self.cloudflare() {
            total_providers += 1;
            let cf_status = match cf_client.health_check().await {
                Ok(true) => {
                    healthy_providers += 1;
                    HealthStatus::Healthy
                },
                Ok(false) => HealthStatus::Degraded,
                Err(_) => HealthStatus::Unhealthy,
            };
            
            status.providers.push(ProviderStatus {
                name: "cloudflare".to_string(),
                status: cf_status,
                last_check: chrono::Utc::now(),
                metadata: serde_json::json!({
                    "type": "cdn_dns"
                }),
            });
        }

        // Determine overall health
        status.overall_health = if total_providers == 0 {
            HealthStatus::Unknown
        } else if healthy_providers == total_providers {
            HealthStatus::Healthy
        } else if healthy_providers > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        Ok(status)
    }

    /// Deploy infrastructure resources across providers
    pub async fn deploy_resources(&self, deployment_spec: DeploymentSpec) -> Result<DeploymentResult> {
        let mut results = Vec::new();

        for resource in deployment_spec.resources {
            match resource.provider.as_str() {
                "kubernetes" => {
                    let k8s_client = self.kubernetes().await?;
                    let result = k8s_client.deploy_resource(resource).await?;
                    results.push(result);
                },
                "docker" => {
                    let docker_client = self.docker().await?;
                    let result = docker_client.deploy_resource(resource).await?;
                    results.push(result);
                },
                "cloudflare" => {
                    let cf_client = self.cloudflare()?;
                    let result = cf_client.deploy_resource(resource).await?;
                    results.push(result);
                },
                _ => {
                    return Err(Error::validation(format!("Unsupported provider: {}", resource.provider)));
                }
            }
        }

        let status = if results.iter().all(|r| r.status == "success") {
            "completed".to_string()
        } else {
            "partial".to_string()
        };
        
        Ok(DeploymentResult {
            deployment_id: uuid::Uuid::new_v4().to_string(),
            resources: results,
            status,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Scale infrastructure resources
    pub async fn scale_resources(&self, scaling_spec: ScalingSpec) -> Result<ScalingResult> {
        let mut results = Vec::new();

        for target in scaling_spec.targets {
            match target.provider.as_str() {
                "kubernetes" => {
                    let k8s_client = self.kubernetes().await?;
                    let result = k8s_client.scale_resource(target).await?;
                    results.push(result);
                },
                "docker" => {
                    let docker_client = self.docker().await?;
                    let result = docker_client.scale_resource(target).await?;
                    results.push(result);
                },
                _ => {
                    return Err(Error::validation(format!("Scaling not supported for provider: {}", target.provider)));
                }
            }
        }

        Ok(ScalingResult {
            scaling_id: uuid::Uuid::new_v4().to_string(),
            results,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get resource metrics across all providers
    pub async fn get_metrics(&self) -> Result<InfrastructureMetrics> {
        let mut metrics = InfrastructureMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            storage_usage: 0.0,
            network_io: 0.0,
            provider_metrics: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        // Collect Kubernetes metrics
        if let Ok(k8s_client) = self.kubernetes().await {
            if let Ok(k8s_metrics) = k8s_client.get_metrics().await {
                // Parse CPU and memory from k8s metrics if available
                // For now, just store the raw metrics
                metrics.provider_metrics.insert("kubernetes".to_string(), k8s_metrics);
            }
        }

        // Collect Docker metrics
        if let Ok(docker_client) = self.docker().await {
            if let Ok(docker_metrics) = docker_client.get_metrics().await {
                // Parse CPU and memory from docker metrics if available
                // For now, just store the raw metrics
                metrics.provider_metrics.insert("docker".to_string(), docker_metrics);
            }
        }

        // Collect Cloudflare metrics
        if let Ok(cf_client) = self.cloudflare() {
            if let Ok(cf_metrics) = cf_client.get_metrics().await {
                metrics.provider_metrics.insert("cloudflare".to_string(), cf_metrics);
            }
        }

        Ok(metrics)
    }

    /// Get Kubernetes client
    pub async fn kubernetes(&self) -> Result<KubernetesClient> {
        for provider in &self.config.providers {
            if let InfrastructureProvider::Kubernetes(_config) = provider {
                // Create Kubernetes client with lifecycle manager
                let client = KubernetesClient::new(&self.lifecycle, None, None)?;
                return Ok(client);
            }
        }

        Err(Error::config("Kubernetes provider not configured"))
    }

    /// Get Docker client
    pub async fn docker(&self) -> Result<ContainerClient> {
        // Check if Docker is configured
        let docker_config = self.config.providers.iter().find_map(|p| {
            if let InfrastructureProvider::Docker(config) = p {
                Some(config)
            } else {
                None
            }
        });

        if docker_config.is_some() {
            // Check if lifecycle manager is available
            let client = ContainerClient::new(self.lifecycle.clone()).await?;
            return Ok(client);
        }

        Err(Error::config("Docker not configured"))
    }

    /// Get Cloudflare client
    pub fn cloudflare(&self) -> Result<CloudflareClient> {
        for provider in &self.config.providers {
            if let InfrastructureProvider::Cloudflare(config) = provider {
                let cloudflare_config: cloudflare::CloudflareConfig =
                    serde_json::from_value(config.clone()).map_err(|e| {
                        Error::config(format!("Invalid Cloudflare configuration: {}", e))
                    })?;
                let client = CloudflareClient::new(cloudflare_config)?;
                return Ok(client);
            }
        }

        Err(Error::config("Cloudflare provider not configured"))
    }

    /// Get tools for infrastructure management with async support and zero-copy optimizations
    pub async fn get_tools(&self) -> Result<Vec<ToolDefinition>> {
        let mut tools = Vec::new();

        for provider in &self.config.providers {
            match provider {
                InfrastructureProvider::Kubernetes(_config) => {
                    // For now, create some example tools since KubernetesModule isn't fully implemented
                    tools.push(ToolDefinition::new(
                        "list_pods".to_string(),
                        "List Kubernetes pods".to_string(),
                    ));
                    tools.push(ToolDefinition::new(
                        "get_pod_logs".to_string(),
                        "Get pod logs".to_string(),
                    ));
                }
                InfrastructureProvider::Docker(_config) => {
                    tools.push(ToolDefinition::new(
                        "list_containers".to_string(),
                        "List Docker containers".to_string(),
                    ));
                }
                InfrastructureProvider::Cloudflare(_config) => {
                    tools.push(ToolDefinition::new(
                        "list_dns_records".to_string(),
                        "List DNS records".to_string(),
                    ));
                }
            }
        }

        Ok(tools)
    }

    pub fn create_k8s_tools(&self, pods: Vec<String>) -> Vec<ToolDefinition> {
        let mut tools = Vec::with_capacity(pods.len() * 4); // Pre-allocate for performance

        for pod in pods {
            tools.push(
                ToolDefinition::new(format!("get_pod_logs_{}", pod), "Get pod logs".to_string())
                    .with_parameters(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "pod": {"type": "string", "default": pod}
                        }
                    })),
            );

            tools.push(
                ToolDefinition::new(format!("describe_pod_{}", pod), "Describe pod".to_string())
                    .with_parameters(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "pod": {"type": "string", "default": pod}
                        }
                    })),
            );
        }

        tools
    }

    pub fn create_docker_tools(&self, containers: Vec<Container>) -> Vec<ToolDefinition> {
        let mut tools = Vec::with_capacity(containers.len() * 3); // Pre-allocate for performance

        for container in &containers {
            tools.push(
                ToolDefinition::new(
                    format!("get_container_logs_{}", container.id),
                    "Get container logs".to_string(),
                )
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "container": {"type": "string", "default": container.id}
                    }
                })),
            );

            tools.push(
                ToolDefinition::new(
                    format!("inspect_container_{}", container.id),
                    "Inspect container".to_string(),
                )
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "container": {"type": "string", "default": container.id}
                    }
                })),
            );
        }

        tools
    }
}

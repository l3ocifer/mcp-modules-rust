use crate::config::CicdConfig;
/// Enhanced CI/CD module with GitOps and IaC support
///
/// Provides unified access to:
/// - CI/CD Platforms (GitHub Actions, GitLab CI, Jenkins)
/// - GitOps Tools (ArgoCD, Flux, Helm)
/// - Infrastructure as Code (Terraform, Pulumi, CDK)
/// - Version Control Integration
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;

/// Enhanced CI/CD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCicdConfig {
    /// Original config for compatibility
    pub legacy: CicdConfig,
    /// GitHub Actions configuration
    pub github_actions: Option<GitHubActionsConfig>,
    /// GitLab CI configuration
    pub gitlab_ci: Option<GitLabCiConfig>,
    /// Jenkins configuration
    pub jenkins: Option<JenkinsConfig>,
    /// Terraform configuration
    pub terraform: Option<TerraformConfig>,
    /// Helm configuration
    pub helm: Option<HelmConfig>,
    /// ArgoCD configuration
    pub argocd: Option<ArgoCDConfig>,
    /// Flux configuration
    pub flux: Option<FluxConfig>,
}

/// GitHub Actions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubActionsConfig {
    /// GitHub token
    pub token: String,
    /// Default owner
    pub owner: String,
    /// Default repository
    pub repo: String,
}

/// GitLab CI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabCiConfig {
    /// GitLab URL
    pub url: String,
    /// Private token
    pub token: String,
    /// Default project ID
    pub project_id: Option<String>,
}

/// Jenkins configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsConfig {
    /// Jenkins URL
    pub url: String,
    /// Username
    pub username: String,
    /// API token
    pub api_token: String,
}

/// Terraform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformConfig {
    /// Version
    pub version: String,
    /// Working directory
    pub working_dir: String,
    /// Backend configuration
    pub backend: HashMap<String, Value>,
    /// Variables
    pub variables: HashMap<String, Value>,
    /// Workspace
    pub workspace: Option<String>,
}

/// Helm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmConfig {
    /// Helm version
    pub version: String,
    /// Kubeconfig path
    pub kubeconfig: Option<String>,
    /// Namespace
    pub namespace: Option<String>,
    /// Repository configurations
    pub repositories: Vec<HelmRepository>,
}

/// Helm repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRepository {
    /// Name
    pub name: String,
    /// URL
    pub url: String,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
}

/// ArgoCD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgoCDConfig {
    /// Server URL
    pub server: String,
    /// Auth token
    pub auth_token: Option<String>,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// Insecure
    pub insecure: bool,
    /// gRPC web
    pub grpc_web: bool,
}

/// Flux configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxConfig {
    /// Version
    pub version: String,
    /// Namespace
    pub namespace: String,
    /// Components
    pub components: Vec<String>,
}

/// CI/CD module implementation
#[derive(Clone)]
pub struct CicdModule {
    config: EnhancedCicdConfig,
    lifecycle: Arc<LifecycleManager>,
    security: SecurityModule,
}

impl CicdModule {
    /// Create a new enhanced CI/CD module
    pub fn new(config: EnhancedCicdConfig, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
        }
    }

    /// Create from legacy config for compatibility
    pub fn from_legacy(config: Option<CicdConfig>, lifecycle: Arc<LifecycleManager>) -> Self {
        let legacy_config = config.unwrap_or_else(|| CicdConfig {
            providers: Vec::new(),
        });

        Self {
            config: EnhancedCicdConfig {
                legacy: legacy_config,
                github_actions: None,
                gitlab_ci: None,
                jenkins: None,
                terraform: None,
                helm: None,
                argocd: None,
                flux: None,
            },
            lifecycle,
            security: SecurityModule::new(),
        }
    }

    /// Check if CI/CD capabilities are available
    pub async fn check_available(&self) -> Result<bool> {
        // Check for available tools
        let mut available = false;

        // Check GitHub CLI
        if self.config.github_actions.is_some() {
            if let Ok(output) = Command::new("gh").arg("--version").output().await {
                available = available || output.status.success();
            }
        }

        // Check Terraform
        if self.config.terraform.is_some() {
            if let Ok(output) = Command::new("terraform").arg("--version").output().await {
                available = available || output.status.success();
            }
        }

        // Check Helm
        if self.config.helm.is_some() {
            if let Ok(output) = Command::new("helm").arg("version").output().await {
                available = available || output.status.success();
            }
        }

        // Check ArgoCD CLI
        if self.config.argocd.is_some() {
            if let Ok(output) = Command::new("argocd").arg("version").output().await {
                available = available || output.status.success();
            }
        }

        Ok(available)
    }

    // GitHub Actions operations

    /// List GitHub Actions workflows
    pub async fn list_workflows(
        &self,
        owner: Option<&str>,
        repo: Option<&str>,
    ) -> Result<Vec<Workflow>> {
        let gh_config = self
            .config
            .github_actions
            .as_ref()
            .ok_or_else(|| Error::config("GitHub Actions not configured"))?;

        let owner = owner.unwrap_or(&gh_config.owner);
        let repo = repo.unwrap_or(&gh_config.repo);

        let output = Command::new("gh")
            .args([
                "workflow",
                "list",
                "--repo",
                &format!("{}/{}", owner, repo),
                "--json",
                "id,name,state,path",
            ])
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to list workflows: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Failed to list workflows: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let workflows: Vec<Workflow> = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::parsing(format!("Failed to parse workflows: {}", e)))?;

        Ok(workflows)
    }

    /// Trigger a workflow
    pub async fn trigger_workflow(
        &self,
        workflow_id: &str,
        inputs: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let gh_config = self
            .config
            .github_actions
            .as_ref()
            .ok_or_else(|| Error::config("GitHub Actions not configured"))?;

        let repo_str = format!("{}/{}", gh_config.owner, gh_config.repo);
        let mut args = vec!["workflow", "run", workflow_id, "--repo", &repo_str];

        let mut input_strings = Vec::new();
        if let Some(inputs) = inputs {
            for (key, value) in inputs {
                input_strings.push(format!("{}={}", key, value));
            }
        }

        for input_str in &input_strings {
            args.push("-f");
            args.push(input_str);
        }

        let output = Command::new("gh")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to trigger workflow: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Failed to trigger workflow: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    // Terraform operations

    /// Initialize Terraform
    pub async fn terraform_init(&self) -> Result<()> {
        let tf_config = self
            .config
            .terraform
            .as_ref()
            .ok_or_else(|| Error::config("Terraform not configured"))?;

        let output = Command::new("terraform")
            .current_dir(&tf_config.working_dir)
            .arg("init")
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to init Terraform: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Terraform init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Plan Terraform changes
    pub async fn terraform_plan(&self, out_file: Option<&str>) -> Result<String> {
        let tf_config = self
            .config
            .terraform
            .as_ref()
            .ok_or_else(|| Error::config("Terraform not configured"))?;

        let mut args = vec!["plan"];
        if let Some(out) = out_file {
            args.push("-out");
            args.push(out);
        }

        let output = Command::new("terraform")
            .current_dir(&tf_config.working_dir)
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to plan Terraform: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Terraform plan failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Apply Terraform changes
    pub async fn terraform_apply(&self, auto_approve: bool) -> Result<String> {
        let tf_config = self
            .config
            .terraform
            .as_ref()
            .ok_or_else(|| Error::config("Terraform not configured"))?;

        let mut args = vec!["apply"];
        if auto_approve {
            args.push("-auto-approve");
        }

        let output = Command::new("terraform")
            .current_dir(&tf_config.working_dir)
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to apply Terraform: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Terraform apply failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    // Helm operations

    /// List Helm releases
    pub async fn helm_list(&self, all_namespaces: bool) -> Result<Vec<HelmRelease>> {
        let helm_config = self
            .config
            .helm
            .as_ref()
            .ok_or_else(|| Error::config("Helm not configured"))?;

        let mut args = vec!["list", "--output", "json"];

        if all_namespaces {
            args.push("--all-namespaces");
        } else if let Some(ref ns) = helm_config.namespace {
            args.push("-n");
            args.push(ns);
        }

        let output = Command::new("helm")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to list Helm releases: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Failed to list Helm releases: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let releases: Vec<HelmRelease> = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::parsing(format!("Failed to parse Helm releases: {}", e)))?;

        Ok(releases)
    }

    /// Install Helm chart
    pub async fn helm_install(
        &self,
        release_name: &str,
        chart: &str,
        values: Option<HashMap<String, String>>,
        namespace: Option<&str>,
    ) -> Result<()> {
        let helm_config = self
            .config
            .helm
            .as_ref()
            .ok_or_else(|| Error::config("Helm not configured"))?;

        let mut args = vec!["install", release_name, chart];

        let ns = namespace.or(helm_config.namespace.as_deref());
        if let Some(ns) = ns {
            args.push("-n");
            args.push(ns);
            args.push("--create-namespace");
        }

        let mut value_strings = Vec::new();
        if let Some(values) = values {
            for (key, value) in values {
                value_strings.push(format!("{}={}", key, value));
            }
        }

        for value_str in &value_strings {
            args.push("--set");
            args.push(value_str);
        }

        let output = Command::new("helm")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to install Helm chart: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Helm install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Upgrade Helm release
    pub async fn helm_upgrade(
        &self,
        release_name: &str,
        chart: &str,
        values: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let helm_config = self
            .config
            .helm
            .as_ref()
            .ok_or_else(|| Error::config("Helm not configured"))?;

        let mut args = vec!["upgrade", release_name, chart, "--install"];

        if let Some(ref ns) = helm_config.namespace {
            args.push("-n");
            args.push(ns);
        }

        let mut value_strings = Vec::new();
        if let Some(values) = values {
            for (key, value) in values {
                value_strings.push(format!("{}={}", key, value));
            }
        }

        for value_str in &value_strings {
            args.push("--set");
            args.push(value_str);
        }

        let output = Command::new("helm")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to upgrade Helm release: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Helm upgrade failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    // ArgoCD operations

    /// List ArgoCD applications
    pub async fn argocd_list_apps(&self) -> Result<Vec<ArgoCDApp>> {
        let argo_config = self
            .config
            .argocd
            .as_ref()
            .ok_or_else(|| Error::config("ArgoCD not configured"))?;

        let mut args = vec!["app", "list", "--output", "json"];

        if argo_config.insecure {
            args.push("--insecure");
        }

        if argo_config.grpc_web {
            args.push("--grpc-web");
        }

        args.push("--server");
        args.push(&argo_config.server);

        let output = Command::new("argocd")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to list ArgoCD apps: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "Failed to list ArgoCD apps: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let apps: Vec<ArgoCDApp> = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::parsing(format!("Failed to parse ArgoCD apps: {}", e)))?;

        Ok(apps)
    }

    /// Sync ArgoCD application
    pub async fn argocd_sync(&self, app_name: &str, prune: bool) -> Result<()> {
        let argo_config = self
            .config
            .argocd
            .as_ref()
            .ok_or_else(|| Error::config("ArgoCD not configured"))?;

        let mut args = vec!["app", "sync", app_name];

        if prune {
            args.push("--prune");
        }

        if argo_config.insecure {
            args.push("--insecure");
        }

        args.push("--server");
        args.push(&argo_config.server);

        let output = Command::new("argocd")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to sync ArgoCD app: {}", e)))?;

        if !output.status.success() {
            return Err(Error::service(format!(
                "ArgoCD sync failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Get configuration
    pub fn get_config(&self) -> &EnhancedCicdConfig {
        &self.config
    }

    /// Get lifecycle manager
    pub fn get_lifecycle(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle
    }

    /// Get security module
    pub fn get_security(&self) -> &SecurityModule {
        &self.security
    }
}

/// CI/CD pipeline status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStatus {
    /// Pipeline is running
    Running,
    /// Pipeline succeeded
    Success,
    /// Pipeline failed
    Failed,
    /// Pipeline is pending
    Pending,
    /// Pipeline is waiting for manual action
    Manual,
    /// Pipeline is cancelled
    Cancelled,
}

/// CI/CD pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Pipeline ID
    pub id: String,
    /// Pipeline name
    pub name: String,
    /// Pipeline status
    pub status: PipelineStatus,
    /// Pipeline provider (e.g., GitHub Actions, GitLab CI, etc.)
    pub provider: String,
    /// Pipeline URL
    pub url: Option<String>,
    /// Pipeline started at timestamp
    pub started_at: Option<String>,
    /// Pipeline finished at timestamp
    pub finished_at: Option<String>,
    /// Pipeline duration in seconds
    pub duration: Option<u64>,
    /// Pipeline commit SHA
    pub commit: Option<String>,
    /// Pipeline branch
    pub branch: Option<String>,
    /// Additional pipeline metadata
    pub metadata: Option<Value>,
}

/// GitHub Actions workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow state
    pub state: String,
    /// Workflow path
    pub path: String,
}

/// Helm release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    /// Release name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Revision
    pub revision: String,
    /// Chart
    pub chart: String,
    /// App version
    pub app_version: String,
    /// Status
    pub status: String,
    /// Updated
    pub updated: String,
}

/// ArgoCD application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgoCDApp {
    /// Name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Project
    pub project: String,
    /// Sync status
    pub sync_status: String,
    /// Health status
    pub health_status: String,
    /// Revision
    pub revision: String,
    /// Destination server
    pub destination_server: String,
    /// Destination namespace
    pub destination_namespace: String,
    /// Repository URL
    pub repo_url: String,
    /// Path
    pub path: String,
    /// Target revision
    pub target_revision: String,
}

/// Default implementation
impl Default for EnhancedCicdConfig {
    fn default() -> Self {
        Self {
            legacy: CicdConfig {
                providers: Vec::new(),
            },
            github_actions: None,
            gitlab_ci: None,
            jenkins: None,
            terraform: None,
            helm: None,
            argocd: None,
            flux: None,
        }
    }
}

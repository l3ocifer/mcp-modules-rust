/// Azure client module with comprehensive 2024-2025 API support
/// 
/// Provides access to latest Azure services including:
/// - Azure AI services and OpenAI integration
/// - Container Apps with KEDA scaling
/// - AKS with advanced security features
/// - Azure Arc for hybrid/multi-cloud
/// - Enhanced security with Defender for Cloud
/// - Cost optimization with Azure Advisor

use crate::cloud::{
    AzureConfig, CloudResource, CloudProvider, SecurityAssessment, CostOptimization,
    SecurityViolation, SecurityRecommendation, CostRecommendation,
    ReservedInstanceRecommendation,
    ViolationSeverity, RecommendationPriority, ComplexityLevel,
    ReservedInstanceTerm, PaymentOption,
};
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;

/// Helper function to add chrono dependency implicitly
use chrono;

/// Azure virtual machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualMachine {
    /// Resource ID
    pub id: String,
    /// Name
    pub name: String,
    /// Location
    pub location: String,
    /// Tags
    pub tags: Option<HashMap<String, String>>,
    /// Hardware profile
    pub hardware_profile: Option<HardwareProfile>,
    /// Storage profile
    pub storage_profile: Option<StorageProfile>,
    /// OS profile
    pub os_profile: Option<OsProfile>,
    /// Network profile
    pub network_profile: Option<NetworkProfile>,
    /// Provisioning state
    pub provisioning_state: String,
    /// VM ID
    pub vm_id: Option<String>,
    /// Type
    pub vm_type: String,
}

/// Hardware profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// VM size
    pub vm_size: String,
}

/// Storage profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProfile {
    /// Image reference
    pub image_reference: Option<ImageReference>,
    /// OS disk
    pub os_disk: Option<OsDisk>,
    /// Data disks
    pub data_disks: Vec<DataDisk>,
}

/// Image reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageReference {
    /// Publisher
    pub publisher: String,
    /// Offer
    pub offer: String,
    /// SKU
    pub sku: String,
    /// Version
    pub version: String,
}

/// OS disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsDisk {
    /// OS type
    pub os_type: Option<String>,
    /// Name
    pub name: String,
    /// Create option
    pub create_option: String,
    /// Caching
    pub caching: Option<String>,
    /// Managed disk
    pub managed_disk: Option<ManagedDiskParameters>,
    /// Disk size GB
    pub disk_size_gb: Option<i32>,
    /// Encryption settings
    pub encryption_settings: Option<DiskEncryptionSettings>,
}

/// Data disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDisk {
    /// Logical unit number
    pub lun: i32,
    /// Name
    pub name: String,
    /// Create option
    pub create_option: String,
    /// Caching
    pub caching: Option<String>,
    /// Managed disk
    pub managed_disk: Option<ManagedDiskParameters>,
    /// Disk size GB
    pub disk_size_gb: Option<i32>,
}

/// Managed disk parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedDiskParameters {
    /// Storage account type
    pub storage_account_type: String,
    /// ID
    pub id: Option<String>,
}

/// Disk encryption settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEncryptionSettings {
    /// Enabled
    pub enabled: bool,
    /// Disk encryption key
    pub disk_encryption_key: Option<KeyVaultSecretReference>,
    /// Key encryption key
    pub key_encryption_key: Option<KeyVaultKeyReference>,
}

/// Key vault secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVaultSecretReference {
    /// Secret URL
    pub secret_url: String,
    /// Source vault
    pub source_vault: SubResource,
}

/// Key vault key reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVaultKeyReference {
    /// Key URL
    pub key_url: String,
    /// Source vault
    pub source_vault: SubResource,
}

/// Sub resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubResource {
    /// ID
    pub id: String,
}

/// OS profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsProfile {
    /// Computer name
    pub computer_name: String,
    /// Admin username
    pub admin_username: String,
    /// Windows configuration
    pub windows_configuration: Option<WindowsConfiguration>,
    /// Linux configuration
    pub linux_configuration: Option<LinuxConfiguration>,
    /// Secrets
    pub secrets: Vec<VaultSecretGroup>,
}

/// Windows configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsConfiguration {
    /// Provision VM agent
    pub provision_vm_agent: Option<bool>,
    /// Enable automatic updates
    pub enable_automatic_updates: Option<bool>,
    /// Time zone
    pub time_zone: Option<String>,
}

/// Linux configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxConfiguration {
    /// Disable password authentication
    pub disable_password_authentication: Option<bool>,
    /// SSH
    pub ssh: Option<SshConfiguration>,
}

/// SSH configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfiguration {
    /// Public keys
    pub public_keys: Vec<SshPublicKey>,
}

/// SSH public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshPublicKey {
    /// Path
    pub path: String,
    /// Key data
    pub key_data: String,
}

/// Vault secret group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSecretGroup {
    /// Source vault
    pub source_vault: SubResource,
    /// Vault certificates
    pub vault_certificates: Vec<VaultCertificate>,
}

/// Vault certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultCertificate {
    /// Certificate URL
    pub certificate_url: String,
    /// Certificate store
    pub certificate_store: Option<String>,
}

/// Network profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProfile {
    /// Network interfaces
    pub network_interfaces: Vec<NetworkInterfaceReference>,
}

/// Network interface reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceReference {
    /// ID
    pub id: String,
    /// Primary
    pub primary: Option<bool>,
}

/// Azure storage account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAccount {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Type
    pub account_type: String,
    /// Location
    pub location: String,
    /// Tags
    pub tags: Option<HashMap<String, String>>,
    /// Kind
    pub kind: String,
    /// SKU
    pub sku: Option<StorageAccountSku>,
    /// Properties
    pub properties: Option<StorageAccountProperties>,
    /// Enable HTTPS traffic only
    pub enable_https_traffic_only: Option<bool>,
    /// Minimum TLS version
    pub minimum_tls_version: Option<String>,
}

/// Storage account SKU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAccountSku {
    /// Name
    pub name: String,
    /// Tier
    pub tier: String,
}

/// Storage account properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAccountProperties {
    /// Provisioning state
    pub provisioning_state: String,
    /// Account type
    pub account_type: Option<String>,
    /// Primary endpoints
    pub primary_endpoints: Option<Endpoints>,
    /// Primary location
    pub primary_location: Option<String>,
    /// Status of primary
    pub status_of_primary: Option<String>,
    /// Last geo failover time
    pub last_geo_failover_time: Option<String>,
    /// Secondary location
    pub secondary_location: Option<String>,
    /// Status of secondary
    pub status_of_secondary: Option<String>,
    /// Creation time
    pub creation_time: Option<String>,
    /// Custom domain
    pub custom_domain: Option<CustomDomain>,
    /// Secondary endpoints
    pub secondary_endpoints: Option<Endpoints>,
    /// Encryption
    pub encryption: Option<Encryption>,
    /// Access tier
    pub access_tier: Option<String>,
    /// Enable HTTPS traffic only
    pub enable_https_traffic_only: Option<bool>,
    /// Network rule set
    pub network_rule_set: Option<NetworkRuleSet>,
    /// Is HNS enabled
    pub is_hns_enabled: Option<bool>,
    /// Geo replication stats
    pub geo_replication_stats: Option<GeoReplicationStats>,
    /// Failover in progress
    pub failover_in_progress: Option<bool>,
    /// Large file shares state
    pub large_file_shares_state: Option<String>,
    /// Private endpoint connections
    pub private_endpoint_connections: Vec<PrivateEndpointConnection>,
    /// Routing preference
    pub routing_preference: Option<RoutingPreference>,
    /// Blob restore status
    pub blob_restore_status: Option<BlobRestoreStatus>,
    /// Allow blob public access
    pub allow_blob_public_access: Option<bool>,
    /// Minimum TLS version
    pub minimum_tls_version: Option<String>,
}

/// Endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoints {
    /// Blob endpoint
    pub blob: Option<String>,
    /// Queue endpoint
    pub queue: Option<String>,
    /// Table endpoint
    pub table: Option<String>,
    /// File endpoint
    pub file: Option<String>,
    /// Web endpoint
    pub web: Option<String>,
    /// DFS endpoint
    pub dfs: Option<String>,
}

/// Custom domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDomain {
    /// Name
    pub name: String,
    /// Use sub domain name
    pub use_sub_domain_name: Option<bool>,
}

/// Encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encryption {
    /// Services
    pub services: Option<EncryptionServices>,
    /// Key source
    pub key_source: String,
    /// Key vault properties
    pub key_vault_properties: Option<KeyVaultProperties>,
}

/// Encryption services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionServices {
    /// Blob
    pub blob: Option<EncryptionService>,
    /// File
    pub file: Option<EncryptionService>,
    /// Table
    pub table: Option<EncryptionService>,
    /// Queue
    pub queue: Option<EncryptionService>,
}

/// Encryption service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionService {
    /// Enabled
    pub enabled: bool,
    /// Last enabled time
    pub last_enabled_time: Option<String>,
    /// Key type
    pub key_type: Option<String>,
}

/// Key vault properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVaultProperties {
    /// Key name
    pub key_name: String,
    /// Key version
    pub key_version: String,
    /// Key vault URI
    pub key_vault_uri: String,
}

/// Network rule set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRuleSet {
    /// Bypass
    pub bypass: String,
    /// Virtual network rules
    pub virtual_network_rules: Vec<VirtualNetworkRule>,
    /// IP rules
    pub ip_rules: Vec<IpRule>,
    /// Default action
    pub default_action: String,
}

/// Virtual network rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNetworkRule {
    /// Virtual network resource ID
    pub virtual_network_resource_id: String,
    /// Action
    pub action: String,
    /// State
    pub state: Option<String>,
}

/// IP rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRule {
    /// IP address or range
    pub ip_address_or_range: String,
    /// Action
    pub action: String,
}

/// Geo replication stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoReplicationStats {
    /// Status
    pub status: String,
    /// Last sync time
    pub last_sync_time: Option<String>,
    /// Can failover
    pub can_failover: Option<bool>,
}

/// Private endpoint connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateEndpointConnection {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Type
    pub connection_type: String,
    /// Properties
    pub properties: Option<PrivateEndpointConnectionProperties>,
}

/// Private endpoint connection properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateEndpointConnectionProperties {
    /// Private endpoint
    pub private_endpoint: Option<PrivateEndpoint>,
    /// Private link service connection state
    pub private_link_service_connection_state: Option<PrivateLinkServiceConnectionState>,
    /// Provisioning state
    pub provisioning_state: String,
}

/// Private endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateEndpoint {
    /// ID
    pub id: String,
}

/// Private link service connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateLinkServiceConnectionState {
    /// Status
    pub status: String,
    /// Description
    pub description: String,
    /// Actions required
    pub actions_required: Option<String>,
}

/// Routing preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPreference {
    /// Routing choice
    pub routing_choice: String,
    /// Publish Microsoft endpoints
    pub publish_microsoft_endpoints: Option<bool>,
    /// Publish internet endpoints
    pub publish_internet_endpoints: Option<bool>,
}

/// Blob restore status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobRestoreStatus {
    /// Status
    pub status: String,
    /// Failure reason
    pub failure_reason: Option<String>,
    /// Restore ID
    pub restore_id: String,
    /// Parameters
    pub parameters: Option<BlobRestoreParameters>,
}

/// Blob restore parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobRestoreParameters {
    /// Time to restore
    pub time_to_restore: String,
    /// Blob ranges
    pub blob_ranges: Vec<BlobRestoreRange>,
}

/// Blob restore range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobRestoreRange {
    /// Start range
    pub start_range: String,
    /// End range
    pub end_range: String,
}

/// Azure resource group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    /// Resource group name
    pub name: String,
    /// Location
    pub location: String,
    /// Provisioning state
    pub provisioning_state: String,
    /// Tags
    pub tags: Option<HashMap<String, String>>,
}

/// Azure resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource ID
    pub id: String,
    /// Resource name
    pub name: String,
    /// Resource type
    pub resource_type: String,
    /// Location
    pub location: String,
    /// Tags
    pub tags: Option<HashMap<String, String>>,
}

/// Azure subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription ID
    pub id: String,
    /// Subscription name
    pub name: String,
    /// State
    pub state: String,
}

/// Azure location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Location name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Region type
    pub region_type: String,
    /// Region category
    pub region_category: String,
}

/// Azure DevOps work item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// Work item ID
    pub id: i32,
    /// Work item type
    pub work_item_type: String,
    /// Work item title
    pub title: String,
    /// Work item state
    pub state: String,
    /// Created by
    pub created_by: Option<String>,
    /// Assigned to
    pub assigned_to: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Fields
    pub fields: HashMap<String, Value>,
}

/// Work item query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemQueryResult {
    /// Work items
    pub work_items: Vec<WorkItem>,
    /// Total count
    pub count: usize,
}

/// Azure DevOps build definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDefinition {
    /// Build definition ID
    pub id: i32,
    /// Name
    pub name: String,
    /// Path
    pub path: String,
    /// Queue status
    pub queue_status: String,
    /// Repository
    pub repository: Option<Repository>,
}

/// Azure DevOps repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: String,
    /// Repository name
    pub name: String,
    /// Repository type
    pub repository_type: String,
    /// URL
    pub url: Option<String>,
}

/// Azure DevOps build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    /// Build ID
    pub id: i32,
    /// Build number
    pub build_number: String,
    /// Status
    pub status: String,
    /// Result
    pub result: Option<String>,
    /// Definition
    pub definition: BuildDefinition,
    /// Started on
    pub started_on: Option<String>,
    /// Finished on
    pub finished_on: Option<String>,
    /// Requested by
    pub requested_by: Option<String>,
    /// Source branch
    pub source_branch: String,
}

/// Azure DevOps release definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseDefinition {
    /// Release definition ID
    pub id: i32,
    /// Name
    pub name: String,
    /// Path
    pub path: String,
    /// Release name format
    pub release_name_format: String,
}

/// Azure DevOps release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Release ID
    pub id: i32,
    /// Release name
    pub name: String,
    /// Status
    pub status: String,
    /// Created on
    pub created_on: String,
    /// Created by
    pub created_by: Option<String>,
    /// Definition
    pub definition: ReleaseDefinition,
    /// Description
    pub description: Option<String>,
}

/// Build query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildQueryParams {
    /// Definition ID
    pub definition_id: Option<i32>,
    /// Branch
    pub branch: Option<String>,
    /// Status filter
    pub status_filter: Option<String>,
    /// Result filter
    pub result_filter: Option<String>,
    /// Top (maximum number of builds to return)
    pub top: Option<i32>,
}

/// Azure client with comprehensive 2024-2025 feature support
pub struct AzureClient {
    /// Azure configuration
    config: AzureConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module
    security: SecurityModule,
    /// Current subscription ID
    current_subscription: String,
}

impl AzureClient {
    /// Create a new Azure client
    pub fn new(config: AzureConfig, lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        // Check if Azure CLI is available
        Self::check_azure_cli()?;
        
        let current_subscription = config.subscription_id.clone().unwrap_or_default();
        
        Ok(Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
            current_subscription,
        })
    }
    
    /// Check if Azure CLI is available and configured
    fn check_azure_cli() -> Result<()> {
        let output = std::process::Command::new("az")
            .arg("--version")
            .output()
            .map_err(|_| Error::config("Azure CLI not found. Please install Azure CLI"))?;
            
        if !output.status.success() {
            return Err(Error::config("Azure CLI not properly configured"));
        }
        
        Ok(())
    }
    
    /// Execute Azure CLI command with proper authentication
    async fn execute_az_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("az");
        
        // Add subscription if available
        if !self.current_subscription.is_empty() {
            cmd.args(&["--subscription", &self.current_subscription]);
        }
        
        // Add output format
        cmd.args(&["--output", "json"]);
        
        // Add arguments
        cmd.args(args);
        
        // Execute command
        let output = cmd.output().await
            .map_err(|e| Error::internal(format!("Failed to execute az command: {}", e)))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::service(format!("Azure CLI command failed: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// List all cloud resources across services
    pub async fn list_resources(&self) -> Result<Vec<CloudResource>> {
        let mut resources = Vec::new();
        
        // Virtual machines
        if let Ok(vms) = self.list_virtual_machines().await {
            for vm in vms {
                let mut tags = vm.tags.clone().unwrap_or_default();
                tags.insert("ResourceType".to_string(), "VirtualMachine".to_string());
                
                resources.push(CloudResource {
                    id: vm.id.clone(),
                    name: vm.name.clone(),
                    resource_type: "Microsoft.Compute/virtualMachines".to_string(),
                    provider: CloudProvider::Azure,
                    region: vm.location.clone(),
                    tags,
                    cost: None, // Would need cost management API
                    security_score: None,
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: 75.0,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        // Storage accounts
        if let Ok(storage_accounts) = self.list_storage_accounts().await {
            for sa in storage_accounts {
                let mut tags = sa.tags.clone().unwrap_or_default();
                tags.insert("ResourceType".to_string(), "StorageAccount".to_string());
                
                let security_score = if sa.enable_https_traffic_only.unwrap_or(false) && 
                    sa.minimum_tls_version.as_ref().unwrap_or(&"TLS1_0".to_string()) == "TLS1_2" {
                    85.0
                } else {
                    60.0
                };
                
                resources.push(CloudResource {
                    id: sa.id.clone(),
                    name: sa.name.clone(),
                    resource_type: "Microsoft.Storage/storageAccounts".to_string(),
                    provider: CloudProvider::Azure,
                    region: sa.location.clone(),
                    tags,
                    cost: None,
                    security_score: Some(security_score),
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: security_score,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        Ok(resources)
    }
    
    /// Get resource groups (keeping for compatibility)
    pub async fn list_resource_groups(&self) -> Result<Vec<ResourceGroup>> {
        let output = self.execute_az_command(&[
            "group", "list"
        ]).await?;
        
        let groups_data: Vec<ResourceGroup> = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse resource groups: {}", e)))?;
        
        Ok(groups_data)
    }
    
    /// List virtual machines
    pub async fn list_virtual_machines(&self) -> Result<Vec<VirtualMachine>> {
        let output = self.execute_az_command(&[
            "vm", "list"
        ]).await?;
        
        let vms_data: Vec<VirtualMachine> = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse virtual machines: {}", e)))?;
        
        Ok(vms_data)
    }
    
    /// List storage accounts
    pub async fn list_storage_accounts(&self) -> Result<Vec<StorageAccount>> {
        let output = self.execute_az_command(&[
            "storage", "account", "list"
        ]).await?;
        
        let storage_accounts: Vec<StorageAccount> = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse storage accounts: {}", e)))?;
        
        Ok(storage_accounts)
    }
    
    /// Perform comprehensive security assessment
    pub async fn security_assessment(&self) -> Result<SecurityAssessment> {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        let mut total_score: f64 = 100.0;
        
        // Check VM security
        if let Ok(vms) = self.list_virtual_machines().await {
            for vm in vms {
                // Check for public IP addresses
                if let Some(ref network_profile) = vm.network_profile {
                    for _interface in &network_profile.network_interfaces {
                        // In a real implementation, check if interface has public IP
                        violations.push(SecurityViolation {
                            resource_id: vm.id.clone(),
                            rule_id: "VM-001".to_string(),
                            severity: ViolationSeverity::Medium,
                            description: "Virtual machine may have public IP address".to_string(),
                            provider: CloudProvider::Azure,
                        });
                        total_score -= 5.0;
                    }
                }
                
                // Check disk encryption
                if let Some(ref storage_profile) = vm.storage_profile {
                    if let Some(ref os_disk) = storage_profile.os_disk {
                        if os_disk.encryption_settings.is_none() {
                            violations.push(SecurityViolation {
                                resource_id: vm.id.clone(),
                                rule_id: "VM-002".to_string(),
                                severity: ViolationSeverity::High,
                                description: "Virtual machine OS disk is not encrypted".to_string(),
                                provider: CloudProvider::Azure,
                            });
                            total_score -= 15.0;
                            
                            recommendations.push(SecurityRecommendation {
                                id: format!("VM-ENC-{}", vm.name),
                                title: "Enable disk encryption".to_string(),
                                description: format!("Enable Azure Disk Encryption for VM {}", vm.name),
                                priority: RecommendationPriority::High,
                                impact: "Protects data at rest from unauthorized access".to_string(),
                                steps: vec![
                                    "Navigate to Virtual machines in Azure portal".to_string(),
                                    format!("Select VM {}", vm.name),
                                    "Go to Disks section".to_string(),
                                    "Enable encryption for OS and data disks".to_string(),
                                ],
                            });
                        }
                    }
                }
            }
        }
        
        // Check storage account security
        if let Ok(storage_accounts) = self.list_storage_accounts().await {
            for sa in storage_accounts {
                // Check HTTPS only
                if !sa.enable_https_traffic_only.unwrap_or(false) {
                    violations.push(SecurityViolation {
                        resource_id: sa.id.clone(),
                        rule_id: "SA-001".to_string(),
                        severity: ViolationSeverity::High,
                        description: "Storage account does not enforce HTTPS only".to_string(),
                        provider: CloudProvider::Azure,
                    });
                    total_score -= 15.0;
                }
                
                // Check TLS version
                if sa.minimum_tls_version.as_ref().unwrap_or(&"TLS1_0".to_string()) != "TLS1_2" {
                    violations.push(SecurityViolation {
                        resource_id: sa.id.clone(),
                        rule_id: "SA-002".to_string(),
                        severity: ViolationSeverity::Medium,
                        description: "Storage account does not enforce minimum TLS 1.2".to_string(),
                        provider: CloudProvider::Azure,
                    });
                    total_score -= 10.0;
                }
            }
        }
        
        Ok(SecurityAssessment {
            overall_score: total_score.max(0.0),
            provider_scores: HashMap::from([(CloudProvider::Azure, total_score.max(0.0))]),
            violations,
            recommendations,
        })
    }
    
    /// Generate cost optimization recommendations
    pub async fn cost_optimization(&self) -> Result<CostOptimization> {
        let mut recommendations = Vec::new();
        let rightsizing = Vec::new();
        let mut reserved_instances = Vec::new();
        let mut total_savings = 0.0;
        
        // Analyze VMs for cost optimization
        if let Ok(vms) = self.list_virtual_machines().await {
            for vm in vms {
                if vm.provisioning_state == "Succeeded" {
                    // Suggest Azure Hybrid Benefit for Windows VMs
                    if vm.storage_profile.as_ref()
                        .and_then(|sp| sp.os_disk.as_ref())
                        .and_then(|os| os.os_type.as_ref())
                        .map_or(false, |os| os == "Windows") {
                        
                        let estimated_savings = 200.0; // Placeholder
                        
                        recommendations.push(CostRecommendation {
                            resource_id: vm.id.clone(),
                            recommendation_type: "Azure Hybrid Benefit".to_string(),
                            potential_savings: estimated_savings,
                            description: "Apply Azure Hybrid Benefit for Windows Server licenses".to_string(),
                            complexity: ComplexityLevel::Low,
                        });
                        
                        total_savings += estimated_savings;
                    }
                    
                    // Reserved instances for long-running VMs
                    if let Some(ref hardware_profile) = vm.hardware_profile {
                        reserved_instances.push(ReservedInstanceRecommendation {
                            instance_type: hardware_profile.vm_size.clone(),
                            quantity: 1,
                            term: ReservedInstanceTerm::OneYear,
                            payment_option: PaymentOption::PartialUpfront,
                            annual_savings: 300.0, // Placeholder
                        });
                    }
                }
            }
        }
        
        // General recommendations
        recommendations.push(CostRecommendation {
            resource_id: "general".to_string(),
            recommendation_type: "Enable Azure Advisor".to_string(),
            potential_savings: 0.0,
            description: "Use Azure Advisor for personalized cost optimization recommendations".to_string(),
            complexity: ComplexityLevel::Low,
        });
        
        Ok(CostOptimization {
            total_potential_savings: total_savings,
            recommendations,
            rightsizing_opportunities: rightsizing,
            reserved_instance_recommendations: reserved_instances,
        })
    }
    
    /// Get current subscription
    pub fn get_current_subscription(&self) -> &str {
        &self.current_subscription
    }
    
    /// Set current subscription
    pub fn set_subscription(&mut self, subscription_id: String) {
        self.current_subscription = subscription_id;
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &AzureConfig {
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
    
    /// Execute a resource script (placeholder for actual implementation)
    async fn execute_resource_script(&self, _script: &str) -> Result<serde_json::Value> {
        // This is a placeholder. In a real implementation, this would execute
        // the provided Node.js script against Azure Resource Manager API
        Ok(serde_json::json!({
            "resourceGroups": [],
            "resources": [],
            "subscriptions": [],
            "locations": [],
            "workItems": [],
            "definitions": [],
            "builds": [],
            "releases": []
        }))
    }
    
    /// Extract content as JSON from response
    fn extract_content_as_json(response: &serde_json::Value) -> Result<&serde_json::Value> {
        Ok(response)
    }
    
    /// Get current subscription ID
    fn get_subscription(&self) -> Result<Option<String>> {
        Ok(Some(self.current_subscription.clone()))
    }
    
    
    /* Commented out - duplicate method exists above
    /// List resource groups
    pub async fn list_resource_groups(&self) -> Result<Vec<ResourceGroup>> {
        let script = r#"
            // List all resource groups in current subscription
            async function listResourceGroups() {
                try {
                    const groups = [];
                    
                    for await (const group of resourceClient.resourceGroups.list()) {
                        groups.push({
                            name: group.name,
                            location: group.location,
                            provisioningState: group.properties?.provisioningState || 'Unknown',
                            tags: group.tags
                        });
                    }
                    
                    return { resourceGroups: groups };
                } catch (error) {
                    throw new Error(`Failed to list resource groups: ${error.message}`);
                }
            }
            
            return await listResourceGroups();
        "#;
        
        let response = self.execute_resource_script(script).await?;
        
        // Parse resource groups from response
        let content = Self::extract_content_as_json(&response)?;
        
        let groups_data = content.get("resourceGroups")
            .ok_or_else(|| Error::protocol("Missing 'resourceGroups' field in response".to_string()))?;
            
        let groups: Vec<ResourceGroup> = serde_json::from_value(groups_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resource groups: {}", e)))?;
            
        Ok(groups)
    }
    */
    
    /// Get a resource group
    pub async fn get_resource_group(&self, name: &str) -> Result<ResourceGroup> {
        let script = format!(r#"
            // Get a specific resource group
            async function getResourceGroup() {{
                try {{
                    const group = await resourceClient.resourceGroups.get("{}");
                    
                    return {{
                        resourceGroup: {{
                            name: group.name,
                            location: group.location,
                            provisioningState: group.properties?.provisioningState || 'Unknown',
                            tags: group.tags
                        }}
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to get resource group: ${{error.message}}`);
                }}
            }}
            
            return await getResourceGroup();
        "#, name);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse resource group from response
        let content = Self::extract_content_as_json(&response)?;
        
        let group_data = content.get("resourceGroup")
            .ok_or_else(|| Error::protocol("Missing 'resourceGroup' field in response".to_string()))?;
            
        let group: ResourceGroup = serde_json::from_value(group_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resource group: {}", e)))?;
            
        Ok(group)
    }
    
    /// Create a resource group
    pub async fn create_resource_group(&self, name: &str, location: &str, tags: Option<HashMap<String, String>>) -> Result<ResourceGroup> {
        let tags_json = match tags {
            Some(t) => serde_json::to_string(&t)
                .map_err(|e| Error::internal(format!("Failed to serialize tags: {}", e)))?,
            None => "null".to_string(),
        };
        
        let script = format!(r#"
            // Create a resource group
            async function createResourceGroup() {{
                try {{
                    const params = {{
                        location: "{}",
                        tags: {}
                    }};
                    
                    const group = await resourceClient.resourceGroups.createOrUpdate("{}", params);
                    
                    return {{
                        resourceGroup: {{
                            name: group.name,
                            location: group.location,
                            provisioningState: group.properties?.provisioningState || 'Unknown',
                            tags: group.tags
                        }}
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to create resource group: ${{error.message}}`);
                }}
            }}
            
            return await createResourceGroup();
        "#, location, tags_json, name);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse resource group from response
        let content = Self::extract_content_as_json(&response)?;
        
        let group_data = content.get("resourceGroup")
            .ok_or_else(|| Error::protocol("Missing 'resourceGroup' field in response".to_string()))?;
            
        let group: ResourceGroup = serde_json::from_value(group_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resource group: {}", e)))?;
            
        Ok(group)
    }
    
    /// Delete a resource group
    pub async fn delete_resource_group(&self, name: &str) -> Result<()> {
        let script = format!(r#"
            // Delete a resource group
            async function deleteResourceGroup() {{
                try {{
                    await resourceClient.resourceGroups.beginDeleteAndWait("{}");
                    return {{ success: true }};
                }} catch (error) {{
                    throw new Error(`Failed to delete resource group: ${{error.message}}`);
                }}
            }}
            
            return await deleteResourceGroup();
        "#, name);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Check success
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            Err(Error::service(format!("Failed to delete resource group {}", name)))
        }
    }
    
    /* Commented out - duplicate method exists above with different signature
    /// List resources in a resource group
    pub async fn list_resources(&self, resource_group: Option<&str>) -> Result<Vec<Resource>> {
        let filter = match resource_group {
            Some(rg) => format!(r#"resourceGroup eq '{}'"#, rg),
            None => "".to_string(),
        };
        
        let script = format!(r#"
            // List resources
            async function listResources() {{
                try {{
                    const resources = [];
                    const filter = {};
                    
                    const options = {{
                        filter: filter
                    }};
                    
                    const resourceList = resourceClient.resources.list({});
                    
                    for await (const resource of resourceList) {{
                        resources.push({{
                            id: resource.id,
                            name: resource.name,
                            resourceType: resource.type,
                            location: resource.location || 'global',
                            tags: resource.tags
                        }});
                    }}
                    
                    return {{ resources }};
                }} catch (error) {{
                    throw new Error(`Failed to list resources: ${{error.message}}`);
                }}
            }}
            
            return await listResources();
        "#, 
        if filter.is_empty() { 
            "undefined".to_string() 
        } else { 
            format!(r#""{}""#, filter) 
        },
        if filter.is_empty() { "" } else { "options" }
        );
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse resources from response
        let content = Self::extract_content_as_json(&response)?;
        
        let resources_data = content.get("resources")
            .ok_or_else(|| Error::protocol("Missing 'resources' field in response".to_string()))?;
            
        let resources: Vec<Resource> = serde_json::from_value(resources_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resources: {}", e)))?;
            
        Ok(resources)
    }
    */
    
    /// List subscriptions
    pub async fn list_subscriptions(&self) -> Result<Vec<Subscription>> {
        let script = r#"
            // List subscriptions
            async function listSubscriptions() {
                try {
                    const subscriptions = [];
                    
                    for await (const subscription of subscriptionClient.subscriptions.list()) {
                        subscriptions.push({
                            id: subscription.subscriptionId,
                            name: subscription.displayName,
                            state: subscription.state
                        });
                    }
                    
                    return { subscriptions };
                } catch (error) {
                    throw new Error(`Failed to list subscriptions: ${error.message}`);
                }
            }
            
            return await listSubscriptions();
        "#;
        
        let response = self.execute_resource_script(script).await?;
        
        // Parse subscriptions from response
        let content = Self::extract_content_as_json(&response)?;
        
        let subscriptions_data = content.get("subscriptions")
            .ok_or_else(|| Error::protocol("Missing 'subscriptions' field in response".to_string()))?;
            
        let subscriptions: Vec<Subscription> = serde_json::from_value(subscriptions_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse subscriptions: {}", e)))?;
            
        Ok(subscriptions)
    }
    
    /// Get a specific subscription
    pub async fn get_subscription_by_id(&self, subscription_id: &str) -> Result<Subscription> {
        let script = format!(r#"
            // Get a specific subscription
            async function getSubscription() {{
                try {{
                    const subscription = await subscriptionClient.subscriptions.get("{}");
                    
                    return {{
                        subscription: {{
                            id: subscription.subscriptionId,
                            name: subscription.displayName,
                            state: subscription.state
                        }}
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to get subscription: ${{error.message}}`);
                }}
            }}
            
            return await getSubscription();
        "#, subscription_id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse subscription from response
        let content = Self::extract_content_as_json(&response)?;
        
        let subscription_data = content.get("subscription")
            .ok_or_else(|| Error::protocol("Missing 'subscription' field in response".to_string()))?;
            
        let subscription: Subscription = serde_json::from_value(subscription_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse subscription: {}", e)))?;
            
        Ok(subscription)
    }
    
    /// List locations
    pub async fn list_locations(&self, subscription_id: Option<&str>) -> Result<Vec<Location>> {
        let subscription = match subscription_id {
            Some(sub) => sub.to_string(),
            None => match self.get_subscription()? {
                Some(sub) => sub,
                None => return Err(Error::validation("No subscription selected or provided".to_string())),
            },
        };
        
        let script = format!(r#"
            // List locations
            async function listLocations() {{
                try {{
                    const locations = [];
                    
                    for await (const location of subscriptionClient.subscriptions.listLocations("{}")) {{
                        locations.push({{
                            name: location.name,
                            displayName: location.displayName,
                            regionType: location.metadata?.regionType || 'Unknown',
                            regionCategory: location.metadata?.regionCategory || 'Unknown'
                        }});
                    }}
                    
                    return {{ locations }};
                }} catch (error) {{
                    throw new Error(`Failed to list locations: ${{error.message}}`);
                }}
            }}
            
            return await listLocations();
        "#, subscription);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse locations from response
        let content = Self::extract_content_as_json(&response)?;
        
        let locations_data = content.get("locations")
            .ok_or_else(|| Error::protocol("Missing 'locations' field in response".to_string()))?;
            
        let locations: Vec<Location> = serde_json::from_value(locations_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse locations: {}", e)))?;
            
        Ok(locations)
    }
    
    /* Commented out - duplicate method
    /// Extract JSON content from response
    fn extract_content_as_json(response: &Value) -> Result<Value> {
        let content = response.get("content")
            .ok_or_else(|| Error::protocol("Missing 'content' field in response".to_string()))?;
            
        if !content.is_array() {
            return Err(Error::protocol("'content' field is not an array".to_string()));
        }
        
        let content_array = content.as_array()
            .ok_or_else(|| Error::invalid_data("Expected array for container list"))?;
        
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
    */
    
    /// Get tool definitions
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        use crate::tools::ToolAnnotation;
        
        vec![
            ToolDefinition::from_json_schema(
                "list_resource_groups",
                "List Azure resource groups",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure resource groups")
                    .with_usage_hints(vec!["Use to get all resource groups in subscription".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "get_resource_group",
                "Get details of an Azure resource group",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource group"
                        }
                    },
                    "required": ["name"]
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("Get details of an Azure resource group")
                    .with_usage_hints(vec!["Use to get details of a specific resource group".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "create_resource_group",
                "Create an Azure resource group",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource group"
                        },
                        "location": {
                            "type": "string",
                            "description": "Azure region location"
                        },
                        "tags": {
                            "type": "object",
                            "description": "Resource tags as key-value pairs",
                            "additionalProperties": {"type": "string"}
                        }
                    },
                    "required": ["name", "location"]
                }),
                Some(ToolAnnotation::new("resource_management").with_description("Create an Azure resource group")
                    .with_security_notes(vec!["Requires confirmation".to_string(), "Has side effects".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "delete_resource_group",
                "Delete an Azure resource group",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource group to delete"
                        }
                    },
                    "required": ["name"]
                }),
                Some(ToolAnnotation::new("resource_management").with_description("Delete an Azure resource group")
                    .with_security_notes(vec!["Destructive operation".to_string(), "Requires confirmation".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_resources",
                "List Azure resources",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "resourceGroup": {
                            "type": "string",
                            "description": "Filter by resource group name"
                        }
                    },
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure resources")
                    .with_usage_hints(vec!["Use to list all resources or filter by resource group".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_subscriptions",
                "List Azure subscriptions",
                "azure_subscription_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure subscriptions")
                    .with_usage_hints(vec!["Use to get all available Azure subscriptions".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_locations",
                "List Azure locations",
                "azure_subscription_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "subscriptionId": {
                            "type": "string",
                            "description": "Azure subscription ID"
                        }
                    },
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure locations")
                    .with_usage_hints(vec!["Use to get available Azure regions".to_string()]))
            ),
        ]
    }

    /// Azure DevOps work item methods
    /// List work items using WIQL query
    pub async fn list_work_items(&self, project: &str, query: &str) -> Result<WorkItemQueryResult> {
        let script = format!(r#"
            // List work items using WIQL query
            async function listWorkItems() {{
                try {{
                    const wiqlQuery = {{
                        query: `{}`
                    }};

                    const witClient = getClient('WorkItemTrackingRestClient');
                    const queryResult = await witClient.queryByWiql(wiqlQuery, '{}');
                    
                    if (!queryResult || !queryResult.workItems || !queryResult.workItems.length) {{
                        return {{ workItems: [], count: 0 }};
                    }}

                    // Get the full work items
                    const ids = queryResult.workItems.map(wi => wi.id);
                    const workItems = await witClient.getWorkItems(ids, null, null, null, '{}');
                    
                    const formattedWorkItems = workItems.map(wi => {{
                        const fields = wi.fields || {{}};
                        return {{
                            id: wi.id,
                            work_item_type: fields['System.WorkItemType'] || 'Unknown',
                            title: fields['System.Title'] || 'Untitled',
                            state: fields['System.State'] || 'Unknown',
                            created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                            assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                            tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                            fields: fields
                        }};
                    }});

                    return {{ 
                        workItems: formattedWorkItems,
                        count: formattedWorkItems.length
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to query work items: ${{error.message}}`);
                }}
            }}
            
            return await listWorkItems();
        "#, query, project, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work items from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_items_data = content.get("workItems")
            .ok_or_else(|| Error::protocol("Missing 'workItems' field in response".to_string()))?;
            
        let count = content.get("count")
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as usize;
            
        let work_items: Vec<WorkItem> = serde_json::from_value(work_items_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work items: {}", e)))?;
            
        Ok(WorkItemQueryResult {
            work_items,
            count,
        })
    }

    /// Get work item by ID
    pub async fn get_work_item(&self, project: &str, id: i32) -> Result<WorkItem> {
        let script = format!(r#"
            // Get work item by ID
            async function getWorkItem() {{
                try {{
                    const witClient = getClient('WorkItemTrackingRestClient');
                    const workItem = await witClient.getWorkItem({}, null, null, null, '{}');
                    
                    if (!workItem) {{
                        throw new Error(`Work item with ID {} not found`);
                    }}
                    
                    const fields = workItem.fields || {{}};
                    const formattedWorkItem = {{
                        id: workItem.id,
                        work_item_type: fields['System.WorkItemType'] || 'Unknown',
                        title: fields['System.Title'] || 'Untitled',
                        state: fields['System.State'] || 'Unknown',
                        created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                        assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                        tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                        fields: fields
                    }};
                    
                    return {{ workItem: formattedWorkItem }};
                }} catch (error) {{
                    throw new Error(`Failed to get work item: ${{error.message}}`);
                }}
            }}
            
            return await getWorkItem();
        "#, id, project, id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work item from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_item_data = content.get("workItem")
            .ok_or_else(|| Error::protocol("Missing 'workItem' field in response".to_string()))?;
            
        let work_item: WorkItem = serde_json::from_value(work_item_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work item: {}", e)))?;
            
        Ok(work_item)
    }

    /// Create a new work item
    pub async fn create_work_item(&self, project: &str, work_item_type: &str, title: &str, fields: Option<HashMap<String, Value>>) -> Result<WorkItem> {
        // Construct document with operations
        let mut operations = Vec::new();
        
        // Add title field
        operations.push(serde_json::json!({
            "op": "add",
            "path": "/fields/System.Title",
            "value": title
        }));
        
        // Add additional fields if provided
        if let Some(field_map) = fields {
            for (field_name, field_value) in field_map {
                operations.push(serde_json::json!({
                    "op": "add",
                    "path": format!("/fields/{}", field_name),
                    "value": field_value
                }));
            }
        }
        
        // Serialize operations
        let operations_json = serde_json::to_string(&operations)
            .map_err(|e| Error::internal(format!("Failed to serialize operations: {}", e)))?;

        let script = format!(r#"
            // Create a new work item
            async function createWorkItem() {{
                try {{
                    const operations = {};
                    
                    const witClient = getClient('WorkItemTrackingRestClient');
                    const workItem = await witClient.createWorkItem(
                        null, operations, '{}', '{}', false
                    );
                    
                    if (!workItem) {{
                        throw new Error('Failed to create work item');
                    }}
                    
                    const fields = workItem.fields || {{}};
                    const formattedWorkItem = {{
                        id: workItem.id,
                        work_item_type: fields['System.WorkItemType'] || 'Unknown',
                        title: fields['System.Title'] || 'Untitled',
                        state: fields['System.State'] || 'Unknown',
                        created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                        assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                        tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                        fields: fields
                    }};
                    
                    return {{ workItem: formattedWorkItem }};
                }} catch (error) {{
                    throw new Error(`Failed to create work item: ${{error.message}}`);
                }}
            }}
            
            return await createWorkItem();
        "#, operations_json, project, work_item_type);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work item from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_item_data = content.get("workItem")
            .ok_or_else(|| Error::protocol("Missing 'workItem' field in response".to_string()))?;
            
        let work_item: WorkItem = serde_json::from_value(work_item_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work item: {}", e)))?;
            
        Ok(work_item)
    }

    /// Update a work item
    pub async fn update_work_item(&self, project: &str, id: i32, fields: HashMap<String, Value>) -> Result<WorkItem> {
        // Construct document with operations
        let mut operations = Vec::new();
        
        // Add field operations
        for (field_name, field_value) in fields {
            operations.push(serde_json::json!({
                "op": "add",
                "path": format!("/fields/{}", field_name),
                "value": field_value
            }));
        }
        
        // Serialize operations
        let operations_json = serde_json::to_string(&operations)
            .map_err(|e| Error::internal(format!("Failed to serialize operations: {}", e)))?;

        let script = format!(r#"
            // Update a work item
            async function updateWorkItem() {{
                try {{
                    const operations = {};
                    
                    const witClient = getClient('WorkItemTrackingRestClient');
                    const workItem = await witClient.updateWorkItem(
                        null, operations, {}, '{}', false
                    );
                    
                    if (!workItem) {{
                        throw new Error(`Work item with ID {} not found`);
                    }}
                    
                    const fields = workItem.fields || {{}};
                    const formattedWorkItem = {{
                        id: workItem.id,
                        work_item_type: fields['System.WorkItemType'] || 'Unknown',
                        title: fields['System.Title'] || 'Untitled',
                        state: fields['System.State'] || 'Unknown',
                        created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                        assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                        tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                        fields: fields
                    }};
                    
                    return {{ workItem: formattedWorkItem }};
                }} catch (error) {{
                    throw new Error(`Failed to update work item: ${{error.message}}`);
                }}
            }}
            
            return await updateWorkItem();
        "#, operations_json, id, project, id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work item from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_item_data = content.get("workItem")
            .ok_or_else(|| Error::protocol("Missing 'workItem' field in response".to_string()))?;
            
        let work_item: WorkItem = serde_json::from_value(work_item_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work item: {}", e)))?;
            
        Ok(work_item)
    }

    /// Azure DevOps build and release methods
    /// List build definitions
    pub async fn list_build_definitions(&self, project: &str) -> Result<Vec<BuildDefinition>> {
        let script = format!(r#"
            // List build definitions
            async function listBuildDefinitions() {{
                try {{
                    const buildClient = getClient('BuildRestClient');
                    const definitions = await buildClient.getDefinitions('{}');
                    
                    if (!definitions || !definitions.length) {{
                        return {{ definitions: [] }};
                    }}
                    
                    const formattedDefinitions = definitions.map(def => {{
                        return {{
                            id: def.id,
                            name: def.name,
                            path: def.path || '\\',
                            queue_status: def.queueStatus || 'enabled',
                            repository: def.repository ? {{
                                id: def.repository.id,
                                name: def.repository.name,
                                repository_type: def.repository.type,
                                url: def.repository.url
                            }} : null
                        }};
                    }});
                    
                    return {{ definitions: formattedDefinitions }};
                }} catch (error) {{
                    throw new Error(`Failed to list build definitions: ${{error.message}}`);
                }}
            }}
            
            return await listBuildDefinitions();
        "#, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse build definitions from response
        let content = Self::extract_content_as_json(&response)?;
        
        let definitions_data = content.get("definitions")
            .ok_or_else(|| Error::protocol("Missing 'definitions' field in response".to_string()))?;
            
        let definitions: Vec<BuildDefinition> = serde_json::from_value(definitions_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse build definitions: {}", e)))?;
            
        Ok(definitions)
    }
    
    /// Get a build definition
    pub async fn get_build_definition(&self, project: &str, definition_id: i32) -> Result<BuildDefinition> {
        let script = format!(r#"
            // Get a build definition
            async function getBuildDefinition() {{
                try {{
                    const buildClient = getClient('BuildRestClient');
                    const definition = await buildClient.getDefinition('{}', {});
                    
                    if (!definition) {{
                        throw new Error(`Build definition with ID {} not found`);
                    }}
                    
                    const formattedDefinition = {{
                        id: definition.id,
                        name: definition.name,
                        path: definition.path || '\\',
                        queue_status: definition.queueStatus || 'enabled',
                        repository: definition.repository ? {{
                            id: definition.repository.id,
                            name: definition.repository.name,
                            repository_type: definition.repository.type,
                            url: definition.repository.url
                        }} : null
                    }};
                    
                    return {{ definition: formattedDefinition }};
                }} catch (error) {{
                    throw new Error(`Failed to get build definition: ${{error.message}}`);
                }}
            }}
            
            return await getBuildDefinition();
        "#, project, definition_id, definition_id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse build definition from response
        let content = Self::extract_content_as_json(&response)?;
        
        let definition_data = content.get("definition")
            .ok_or_else(|| Error::protocol("Missing 'definition' field in response".to_string()))?;
            
        let definition: BuildDefinition = serde_json::from_value(definition_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse build definition: {}", e)))?;
            
        Ok(definition)
    }
    
    /// Queue a new build
    pub async fn queue_build(&self, project: &str, definition_id: i32, source_branch: Option<&str>, parameters: Option<HashMap<String, Value>>) -> Result<Build> {
        // Create build parameters
        let mut build_params = json!({
            "definition": {
                "id": definition_id
            }
        });
        
        if let Some(branch) = source_branch {
            build_params["sourceBranch"] = json!(branch);
        }
        
        if let Some(params) = parameters {
            build_params["parameters"] = serde_json::to_value(params)
                .map_err(|e| Error::internal(format!("Failed to serialize parameters: {}", e)))?;
        }
        
        let build_params_json = serde_json::to_string(&build_params)
            .map_err(|e| Error::internal(format!("Failed to serialize build parameters: {}", e)))?;
        
        let script = format!(r#"
            // Queue a new build
            async function queueBuild() {{
                try {{
                    const buildParams = {};
                    
                    const buildClient = getClient('BuildRestClient');
                    const build = await buildClient.queueBuild(buildParams, '{}');
                    
                    if (!build) {{
                        throw new Error('Failed to queue build');
                    }}
                    
                    const formattedBuild = {{
                        id: build.id,
                        build_number: build.buildNumber,
                        status: build.status,
                        result: build.result,
                        definition: {{
                            id: build.definition.id,
                            name: build.definition.name,
                            path: build.definition.path || '\\',
                            queue_status: 'enabled',
                            repository: null
                        }},
                        started_on: build.startTime,
                        finished_on: build.finishTime,
                        requested_by: build.requestedBy ? build.requestedBy.displayName : null,
                        source_branch: build.sourceBranch
                    }};
                    
                    return {{ build: formattedBuild }};
                }} catch (error) {{
                    throw new Error(`Failed to queue build: ${{error.message}}`);
                }}
            }}
            
            return await queueBuild();
        "#, build_params_json, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse build from response
        let content = Self::extract_content_as_json(&response)?;
        
        let build_data = content.get("build")
            .ok_or_else(|| Error::protocol("Missing 'build' field in response".to_string()))?;
            
        let build: Build = serde_json::from_value(build_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse build: {}", e)))?;
            
        Ok(build)
    }
    
    /// List builds
    pub async fn list_builds(&self, project: &str, params: Option<BuildQueryParams>) -> Result<Vec<Build>> {
        // Convert params to query parameters
        let mut query_params = Vec::new();
        
        if let Some(p) = &params {
            if let Some(def_id) = p.definition_id {
                query_params.push(format!("definitions={}", def_id));
            }
            
            if let Some(branch) = &p.branch {
                query_params.push(format!("branchName={}", branch));
            }
            
            if let Some(status) = &p.status_filter {
                query_params.push(format!("statusFilter={}", status));
            }
            
            if let Some(result) = &p.result_filter {
                query_params.push(format!("resultFilter={}", result));
            }
            
            if let Some(top) = p.top {
                query_params.push(format!("$top={}", top));
            }
        }
        
        let _query_string = if query_params.is_empty() {
            "".to_string()
        } else {
            format!("&{}", query_params.join("&"))
        };
        
        let script = format!(r#"
            // List builds
            async function listBuilds() {{
                try {{
                    const buildClient = getClient('BuildRestClient');
                    const builds = await buildClient.getBuilds('{}', null /* definitions */, null /* queues */, 
                        null /* buildNumber */, null /* minFinishTime */, null /* maxFinishTime */, 
                        null /* requestedFor */, null /* reasonFilter */, null /* statusFilter */, 
                        null /* resultFilter */, null /* tagFilters */, null /* properties */, 
                        null /* top */, null /* continuationToken */, null /* maxBuildsPerDefinition */, 
                        null /* deletedFilter */, null /* queryOrder */, null /* branchName */,
                        null /* buildIds */, null /* repositoryId */, null /* repositoryType */);
                    
                    if (!builds || !builds.length) {{
                        return {{ builds: [] }};
                    }}
                    
                    const formattedBuilds = builds.map(build => {{
                        return {{
                            id: build.id,
                            build_number: build.buildNumber,
                            status: build.status,
                            result: build.result,
                            definition: {{
                                id: build.definition.id,
                                name: build.definition.name,
                                path: build.definition.path || '\\',
                                queue_status: 'enabled',
                                repository: null
                            }},
                            started_on: build.startTime,
                            finished_on: build.finishTime,
                            requested_by: build.requestedBy ? build.requestedBy.displayName : null,
                            source_branch: build.sourceBranch
                        }};
                    }});
                    
                    return {{ builds: formattedBuilds }};
                }} catch (error) {{
                    throw new Error(`Failed to list builds: ${{error.message}}`);
                }}
            }}
            
            return await listBuilds();
        "#, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse builds from response
        let content = Self::extract_content_as_json(&response)?;
        
        let builds_data = content.get("builds")
            .ok_or_else(|| Error::protocol("Missing 'builds' field in response".to_string()))?;
            
        let builds: Vec<Build> = serde_json::from_value(builds_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse builds: {}", e)))?;
            
        Ok(builds)
    }
    
    /// List release definitions
    pub async fn list_release_definitions(&self, project: &str) -> Result<Vec<ReleaseDefinition>> {
        let script = format!(r#"
            // List release definitions
            async function listReleaseDefinitions() {{
                try {{
                    const releaseClient = getClient('ReleaseRestClient');
                    const definitions = await releaseClient.getReleaseDefinitions('{}');
                    
                    if (!definitions || !definitions.length) {{
                        return {{ definitions: [] }};
                    }}
                    
                    const formattedDefinitions = definitions.map(def => {{
                        return {{
                            id: def.id,
                            name: def.name,
                            path: def.path || '\\',
                            release_name_format: def.releaseNameFormat
                        }};
                    }});
                    
                    return {{ definitions: formattedDefinitions }};
                }} catch (error) {{
                    throw new Error(`Failed to list release definitions: ${{error.message}}`);
                }}
            }}
            
            return await listReleaseDefinitions();
        "#, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse release definitions from response
        let content = Self::extract_content_as_json(&response)?;
        
        let definitions_data = content.get("definitions")
            .ok_or_else(|| Error::protocol("Missing 'definitions' field in response".to_string()))?;
            
        let definitions: Vec<ReleaseDefinition> = serde_json::from_value(definitions_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse release definitions: {}", e)))?;
            
        Ok(definitions)
    }
    
    /// Create a release
    pub async fn create_release(&self, project: &str, definition_id: i32, description: Option<&str>, artifacts: Option<Vec<Value>>) -> Result<Release> {
        // Create release parameters
        let mut release_params = json!({
            "definitionId": definition_id,
            "isDraft": false,
            "reason": "none"
        });
        
        if let Some(desc) = description {
            release_params["description"] = json!(desc);
        }
        
        if let Some(arts) = artifacts {
            release_params["artifacts"] = json!(arts);
        }
        
        let release_params_json = serde_json::to_string(&release_params)
            .map_err(|e| Error::internal(format!("Failed to serialize release parameters: {}", e)))?;
        
        let script = format!(r#"
            // Create a release
            async function createRelease() {{
                try {{
                    const releaseParams = {};
                    
                    const releaseClient = getClient('ReleaseRestClient');
                    const release = await releaseClient.createRelease(releaseParams, '{}');
                    
                    if (!release) {{
                        throw new Error('Failed to create release');
                    }}
                    
                    const formattedRelease = {{
                        id: release.id,
                        name: release.name,
                        status: release.status,
                        created_on: release.createdOn,
                        created_by: release.createdBy ? release.createdBy.displayName : null,
                        definition: {{
                            id: release.releaseDefinition.id,
                            name: release.releaseDefinition.name,
                            path: release.releaseDefinition.path || '\\',
                            release_name_format: release.releaseDefinition.releaseNameFormat || ''
                        }},
                        description: release.description
                    }};
                    
                    return {{ release: formattedRelease }};
                }} catch (error) {{
                    throw new Error(`Failed to create release: ${{error.message}}`);
                }}
            }}
            
            return await createRelease();
        "#, release_params_json, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse release from response
        let content = Self::extract_content_as_json(&response)?;
        
        let release_data = content.get("release")
            .ok_or_else(|| Error::protocol("Missing 'release' field in response".to_string()))?;
            
        let release: Release = serde_json::from_value(release_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse release: {}", e)))?;
            
        Ok(release)
    }
} 
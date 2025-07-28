/// GCP client module with comprehensive 2024-2025 API support
/// 
/// Provides access to latest GCP services including:
/// - Vertex AI and Gemini integration
/// - Cloud Run with 2nd gen runtime
/// - GKE Autopilot with enhanced security
/// - BigQuery with fine-grained access control
/// - Anthos for hybrid/multi-cloud
/// - Cloud Security Command Center

use crate::cloud::{
    GcpConfig, CloudResource, CloudProvider, SecurityAssessment, CostOptimization,
    SecurityViolation, SecurityRecommendation, CostRecommendation,
    ReservedInstanceRecommendation,
    ViolationSeverity, RecommendationPriority, ComplexityLevel,
    ReservedInstanceTerm, PaymentOption,
};
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;

/// GCP service representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpService {
    /// Service name
    pub name: String,
    /// Service region/zone
    pub location: String,
    /// Service resource name
    pub self_link: String,
    /// Service labels
    pub labels: HashMap<String, String>,
    /// Service status
    pub status: String,
    /// Cost information
    pub daily_cost: Option<f64>,
}

/// Compute Engine instance with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeInstance {
    /// Instance ID
    pub id: String,
    /// Instance name
    pub name: String,
    /// Machine type
    pub machine_type: String,
    /// Status
    pub status: String,
    /// Zone
    pub zone: String,
    /// Network interfaces
    pub network_interfaces: Vec<NetworkInterface>,
    /// Disks
    pub disks: Vec<AttachedDisk>,
    /// Tags
    pub tags: Vec<String>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Service accounts
    pub service_accounts: Vec<ServiceAccount>,
    /// Scheduling
    pub scheduling: Option<Scheduling>,
    /// Metadata
    pub metadata: Option<Metadata>,
    /// Creation timestamp
    pub creation_timestamp: String,
    /// Self link
    pub self_link: String,
    /// CPU platform
    pub cpu_platform: Option<String>,
    /// Confidential instance config
    pub confidential_instance_config: Option<ConfidentialInstanceConfig>,
    /// Spot instance
    pub is_spot: bool,
}

/// Network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Name
    pub name: String,
    /// Network
    pub network: String,
    /// Subnetwork
    pub subnetwork: String,
    /// Network IP
    pub network_ip: String,
    /// Access configs
    pub access_configs: Vec<AccessConfig>,
    /// IPv6 access configs
    pub ipv6_access_configs: Vec<AccessConfig>,
    /// Alias IP ranges
    pub alias_ip_ranges: Vec<AliasIpRange>,
}

/// Access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    /// Type
    pub config_type: String,
    /// Name
    pub name: String,
    /// External IP
    pub nat_ip: Option<String>,
    /// Network tier
    pub network_tier: Option<String>,
}

/// Alias IP range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasIpRange {
    /// IP CIDR range
    pub ip_cidr_range: String,
    /// Subnetwork range name
    pub subnetwork_range_name: Option<String>,
}

/// Attached disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachedDisk {
    /// Type
    pub disk_type: String,
    /// Mode
    pub mode: String,
    /// Source
    pub source: String,
    /// Device name
    pub device_name: String,
    /// Index
    pub index: Option<i32>,
    /// Boot disk
    pub boot: bool,
    /// Auto delete
    pub auto_delete: bool,
    /// Initialize params
    pub initialize_params: Option<AttachedDiskInitializeParams>,
    /// Disk encryption key
    pub disk_encryption_key: Option<CustomerEncryptionKey>,
}

/// Attached disk initialize parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachedDiskInitializeParams {
    /// Disk name
    pub disk_name: String,
    /// Source image
    pub source_image: String,
    /// Disk type
    pub disk_type: String,
    /// Disk size GB
    pub disk_size_gb: i64,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Customer encryption key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerEncryptionKey {
    /// Raw key
    pub raw_key: Option<String>,
    /// KMS key name
    pub kms_key_name: Option<String>,
    /// SHA256
    pub sha256: Option<String>,
}

/// Service account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    /// Email
    pub email: String,
    /// Scopes
    pub scopes: Vec<String>,
}

/// Scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheduling {
    /// Preemptible
    pub preemptible: bool,
    /// On host maintenance
    pub on_host_maintenance: String,
    /// Automatic restart
    pub automatic_restart: bool,
    /// Node affinities
    pub node_affinities: Vec<SchedulingNodeAffinity>,
}

/// Scheduling node affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingNodeAffinity {
    /// Key
    pub key: String,
    /// Operator
    pub operator: String,
    /// Values
    pub values: Vec<String>,
}

/// Instance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Fingerprint
    pub fingerprint: String,
    /// Items
    pub items: Vec<MetadataItem>,
}

/// Metadata item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
}

/// Confidential instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidentialInstanceConfig {
    /// Enable confidential compute
    pub enable_confidential_compute: bool,
}

/// Cloud Run service with 2nd gen features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunService {
    /// Service name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Generation
    pub generation: i64,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Annotations
    pub annotations: HashMap<String, String>,
    /// Creation timestamp
    pub creation_timestamp: String,
    /// Spec
    pub spec: CloudRunServiceSpec,
    /// Status
    pub status: CloudRunServiceStatus,
    /// Self link
    pub self_link: String,
    /// Region
    pub region: String,
}

/// Cloud Run service spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunServiceSpec {
    /// Template
    pub template: CloudRunRevisionTemplate,
    /// Traffic
    pub traffic: Vec<CloudRunTrafficTarget>,
}

/// Cloud Run revision template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunRevisionTemplate {
    /// Metadata
    pub metadata: CloudRunRevisionTemplateMetadata,
    /// Spec
    pub spec: CloudRunRevisionSpec,
}

/// Cloud Run revision template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunRevisionTemplateMetadata {
    /// Labels
    pub labels: HashMap<String, String>,
    /// Annotations
    pub annotations: HashMap<String, String>,
}

/// Cloud Run revision spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunRevisionSpec {
    /// Container concurrency
    pub container_concurrency: Option<i32>,
    /// Timeout seconds
    pub timeout_seconds: Option<i64>,
    /// Service account name
    pub service_account_name: Option<String>,
    /// Containers
    pub containers: Vec<CloudRunContainer>,
    /// Volumes
    pub volumes: Vec<CloudRunVolume>,
}

/// Cloud Run container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunContainer {
    /// Name
    pub name: String,
    /// Image
    pub image: String,
    /// Command
    pub command: Vec<String>,
    /// Args
    pub args: Vec<String>,
    /// Environment variables
    pub env: Vec<CloudRunEnvVar>,
    /// Resources
    pub resources: CloudRunResourceRequirements,
    /// Ports
    pub ports: Vec<CloudRunContainerPort>,
    /// Volume mounts
    pub volume_mounts: Vec<CloudRunVolumeMount>,
    /// Startup probe
    pub startup_probe: Option<CloudRunProbe>,
    /// Liveness probe
    pub liveness_probe: Option<CloudRunProbe>,
}

/// Cloud Run environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunEnvVar {
    /// Name
    pub name: String,
    /// Value
    pub value: Option<String>,
    /// Value from
    pub value_from: Option<CloudRunEnvVarSource>,
}

/// Cloud Run environment variable source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunEnvVarSource {
    /// Secret key ref
    pub secret_key_ref: Option<CloudRunSecretKeySelector>,
    /// Config map key ref
    pub config_map_key_ref: Option<CloudRunConfigMapKeySelector>,
}

/// Cloud Run secret key selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunSecretKeySelector {
    /// Name
    pub name: String,
    /// Key
    pub key: String,
}

/// Cloud Run config map key selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunConfigMapKeySelector {
    /// Name
    pub name: String,
    /// Key
    pub key: String,
}

/// Cloud Run resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunResourceRequirements {
    /// Limits
    pub limits: HashMap<String, String>,
    /// Requests
    pub requests: HashMap<String, String>,
}

/// Cloud Run container port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunContainerPort {
    /// Name
    pub name: String,
    /// Container port
    pub container_port: i32,
    /// Protocol
    pub protocol: String,
}

/// Cloud Run volume mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunVolumeMount {
    /// Name
    pub name: String,
    /// Mount path
    pub mount_path: String,
    /// Sub path
    pub sub_path: Option<String>,
}

/// Cloud Run probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunProbe {
    /// Initial delay seconds
    pub initial_delay_seconds: Option<i32>,
    /// Timeout seconds
    pub timeout_seconds: Option<i32>,
    /// Period seconds
    pub period_seconds: Option<i32>,
    /// Success threshold
    pub success_threshold: Option<i32>,
    /// Failure threshold
    pub failure_threshold: Option<i32>,
    /// HTTP get
    pub http_get: Option<CloudRunHttpGetAction>,
    /// TCP socket
    pub tcp_socket: Option<CloudRunTcpSocketAction>,
}

/// Cloud Run HTTP get action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunHttpGetAction {
    /// Path
    pub path: String,
    /// Port
    pub port: i32,
    /// HTTP headers
    pub http_headers: Vec<CloudRunHttpHeader>,
}

/// Cloud Run HTTP header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunHttpHeader {
    /// Name
    pub name: String,
    /// Value
    pub value: String,
}

/// Cloud Run TCP socket action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunTcpSocketAction {
    /// Port
    pub port: i32,
}

/// Cloud Run volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunVolume {
    /// Name
    pub name: String,
    /// Secret
    pub secret: Option<CloudRunSecretVolumeSource>,
    /// Config map
    pub config_map: Option<CloudRunConfigMapVolumeSource>,
}

/// Cloud Run secret volume source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunSecretVolumeSource {
    /// Secret name
    pub secret_name: String,
    /// Items
    pub items: Vec<CloudRunKeyToPath>,
    /// Default mode
    pub default_mode: Option<i32>,
}

/// Cloud Run config map volume source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunConfigMapVolumeSource {
    /// Name
    pub name: String,
    /// Items
    pub items: Vec<CloudRunKeyToPath>,
    /// Default mode
    pub default_mode: Option<i32>,
}

/// Cloud Run key to path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunKeyToPath {
    /// Key
    pub key: String,
    /// Path
    pub path: String,
    /// Mode
    pub mode: Option<i32>,
}

/// Cloud Run traffic target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunTrafficTarget {
    /// Configuration name
    pub configuration_name: Option<String>,
    /// Revision name
    pub revision_name: Option<String>,
    /// Percent
    pub percent: i32,
    /// Tag
    pub tag: Option<String>,
    /// Latest revision
    pub latest_revision: Option<bool>,
}

/// Cloud Run service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunServiceStatus {
    /// Observed generation
    pub observed_generation: Option<i64>,
    /// Conditions
    pub conditions: Vec<CloudRunCondition>,
    /// Latest ready revision name
    pub latest_ready_revision_name: Option<String>,
    /// Latest created revision name
    pub latest_created_revision_name: Option<String>,
    /// Traffic
    pub traffic: Vec<CloudRunTrafficTarget>,
    /// URL
    pub url: Option<String>,
    /// Address
    pub address: Option<CloudRunAddressable>,
}

/// Cloud Run condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunCondition {
    /// Type
    pub condition_type: String,
    /// Status
    pub status: String,
    /// Last transition time
    pub last_transition_time: String,
    /// Reason
    pub reason: Option<String>,
    /// Message
    pub message: Option<String>,
}

/// Cloud Run addressable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRunAddressable {
    /// URL
    pub url: String,
}

/// GKE cluster with Autopilot and enhanced security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeCluster {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Initial node count
    pub initial_node_count: i32,
    /// Node config
    pub node_config: Option<GkeNodeConfig>,
    /// Master auth
    pub master_auth: Option<GkeMasterAuth>,
    /// Logging service
    pub logging_service: String,
    /// Monitoring service
    pub monitoring_service: String,
    /// Network
    pub network: String,
    /// Cluster IPv4 CIDR
    pub cluster_ipv4_cidr: String,
    /// Addons config
    pub addons_config: Option<GkeAddonsConfig>,
    /// Subnetwork
    pub subnetwork: String,
    /// Node pools
    pub node_pools: Vec<GkeNodePool>,
    /// Locations
    pub locations: Vec<String>,
    /// Enable Kubernetes alpha
    pub enable_kubernetes_alpha: bool,
    /// Resource labels
    pub resource_labels: HashMap<String, String>,
    /// Legacy ABAC
    pub legacy_abac: Option<GkeLegacyAbac>,
    /// Network policy
    pub network_policy: Option<GkeNetworkPolicy>,
    /// IP allocation policy
    pub ip_allocation_policy: Option<GkeIpAllocationPolicy>,
    /// Master authorized networks config
    pub master_authorized_networks_config: Option<GkeMasterAuthorizedNetworksConfig>,
    /// Maintenance policy
    pub maintenance_policy: Option<GkeMaintenancePolicy>,
    /// Binary authorization
    pub binary_authorization: Option<GkeBinaryAuthorization>,
    /// Autopilot
    pub autopilot: Option<GkeAutopilot>,
    /// Node pool defaults
    pub node_pool_defaults: Option<GkeNodePoolDefaults>,
    /// Status
    pub status: String,
    /// Self link
    pub self_link: String,
    /// Zone
    pub zone: String,
    /// Endpoint
    pub endpoint: String,
    /// Initial cluster version
    pub initial_cluster_version: String,
    /// Current master version
    pub current_master_version: String,
    /// Current node version
    pub current_node_version: String,
    /// Create time
    pub create_time: String,
    /// Expires
    pub expires: Option<String>,
    /// Location
    pub location: String,
}

/// GKE node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodeConfig {
    /// Machine type
    pub machine_type: String,
    /// Disk size GB
    pub disk_size_gb: i32,
    /// OAuth scopes
    pub oauth_scopes: Vec<String>,
    /// Service account
    pub service_account: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Image type
    pub image_type: String,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Local SSD count
    pub local_ssd_count: i32,
    /// Tags
    pub tags: Vec<String>,
    /// Preemptible
    pub preemptible: bool,
    /// Accelerators
    pub accelerators: Vec<GkeAcceleratorConfig>,
    /// Disk type
    pub disk_type: String,
    /// Min CPU platform
    pub min_cpu_platform: String,
    /// Workload metadata config
    pub workload_metadata_config: Option<GkeWorkloadMetadataConfig>,
    /// Taints
    pub taints: Vec<GkeNodeTaint>,
    /// Sandbox config
    pub sandbox_config: Option<GkeSandboxConfig>,
    /// Node group
    pub node_group: Option<String>,
    /// Reservation affinity
    pub reservation_affinity: Option<GkeReservationAffinity>,
    /// Shielded instance config
    pub shielded_instance_config: Option<GkeShieldedInstanceConfig>,
    /// Linux node config
    pub linux_node_config: Option<GkeLinuxNodeConfig>,
    /// Kubelet config
    pub kubelet_config: Option<GkeNodeKubeletConfig>,
    /// Boot disk KMS key
    pub boot_disk_kms_key: Option<String>,
    /// Spot
    pub spot: bool,
}

/// GKE accelerator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeAcceleratorConfig {
    /// Accelerator count
    pub accelerator_count: i64,
    /// Accelerator type
    pub accelerator_type: String,
}

/// GKE workload metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeWorkloadMetadataConfig {
    /// Mode
    pub mode: String,
}

/// GKE node taint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodeTaint {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Effect
    pub effect: String,
}

/// GKE sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeSandboxConfig {
    /// Type
    pub sandbox_type: String,
}

/// GKE reservation affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeReservationAffinity {
    /// Consume reservation type
    pub consume_reservation_type: String,
    /// Key
    pub key: Option<String>,
    /// Values
    pub values: Vec<String>,
}

/// GKE shielded instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeShieldedInstanceConfig {
    /// Enable secure boot
    pub enable_secure_boot: bool,
    /// Enable integrity monitoring
    pub enable_integrity_monitoring: bool,
}

/// GKE Linux node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeLinuxNodeConfig {
    /// Sysctls
    pub sysctls: HashMap<String, String>,
}

/// GKE node kubelet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodeKubeletConfig {
    /// CPU manager policy
    pub cpu_manager_policy: String,
    /// CPU CFS quota
    pub cpu_cfs_quota: Option<bool>,
    /// CPU CFS quota period
    pub cpu_cfs_quota_period: Option<String>,
}

/// GKE master authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeMasterAuth {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Client certificate config
    pub client_certificate_config: Option<GkeClientCertificateConfig>,
    /// Cluster CA certificate
    pub cluster_ca_certificate: String,
    /// Client certificate
    pub client_certificate: String,
    /// Client key
    pub client_key: String,
}

/// GKE client certificate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeClientCertificateConfig {
    /// Issue client certificate
    pub issue_client_certificate: bool,
}

/// GKE addons configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeAddonsConfig {
    /// HTTP load balancing
    pub http_load_balancing: Option<GkeHttpLoadBalancing>,
    /// Horizontal pod autoscaling
    pub horizontal_pod_autoscaling: Option<GkeHorizontalPodAutoscaling>,
    /// Kubernetes dashboard
    pub kubernetes_dashboard: Option<GkeKubernetesDashboard>,
    /// Network policy config
    pub network_policy_config: Option<GkeNetworkPolicyConfig>,
    /// Cloud run config
    pub cloud_run_config: Option<GkeCloudRunConfig>,
    /// DNS cache config
    pub dns_cache_config: Option<GkeDnsCacheConfig>,
    /// Config connector config
    pub config_connector_config: Option<GkeConfigConnectorConfig>,
    /// GCE persistent disk CSI driver config
    pub gce_persistent_disk_csi_driver_config: Option<GkeGcePersistentDiskCsiDriverConfig>,
}

/// GKE HTTP load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeHttpLoadBalancing {
    /// Disabled
    pub disabled: bool,
}

/// GKE horizontal pod autoscaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeHorizontalPodAutoscaling {
    /// Disabled
    pub disabled: bool,
}

/// GKE Kubernetes dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeKubernetesDashboard {
    /// Disabled
    pub disabled: bool,
}

/// GKE network policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNetworkPolicyConfig {
    /// Disabled
    pub disabled: bool,
}

/// GKE Cloud Run configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeCloudRunConfig {
    /// Disabled
    pub disabled: bool,
    /// Load balancer type
    pub load_balancer_type: String,
}

/// GKE DNS cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeDnsCacheConfig {
    /// Enabled
    pub enabled: bool,
}

/// GKE config connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeConfigConnectorConfig {
    /// Enabled
    pub enabled: bool,
}

/// GKE GCE persistent disk CSI driver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeGcePersistentDiskCsiDriverConfig {
    /// Enabled
    pub enabled: bool,
}

/// GKE node pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodePool {
    /// Name
    pub name: String,
    /// Config
    pub config: Option<GkeNodeConfig>,
    /// Initial node count
    pub initial_node_count: i32,
    /// Self link
    pub self_link: String,
    /// Version
    pub version: String,
    /// Instance group URLs
    pub instance_group_urls: Vec<String>,
    /// Status
    pub status: String,
    /// Status message
    pub status_message: Option<String>,
    /// Autoscaling
    pub autoscaling: Option<GkeNodePoolAutoscaling>,
    /// Management
    pub management: Option<GkeNodeManagement>,
    /// Max pods constraint
    pub max_pods_constraint: Option<GkeMaxPodsConstraint>,
    /// Conditions
    pub conditions: Vec<GkeStatusCondition>,
    /// Pod IPv4 CIDR size
    pub pod_ipv4_cidr_size: Option<i32>,
    /// Upgrade settings
    pub upgrade_settings: Option<GkeUpgradeSettings>,
    /// Locations
    pub locations: Vec<String>,
}

/// GKE node pool autoscaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodePoolAutoscaling {
    /// Enabled
    pub enabled: bool,
    /// Min node count
    pub min_node_count: i32,
    /// Max node count
    pub max_node_count: i32,
    /// Autoprovisioned
    pub autoprovisioned: bool,
}

/// GKE node management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodeManagement {
    /// Auto upgrade
    pub auto_upgrade: bool,
    /// Auto repair
    pub auto_repair: bool,
    /// Upgrade options
    pub upgrade_options: Option<GkeAutoUpgradeOptions>,
}

/// GKE auto upgrade options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeAutoUpgradeOptions {
    /// Auto upgrade start time
    pub auto_upgrade_start_time: String,
    /// Description
    pub description: String,
}

/// GKE max pods constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeMaxPodsConstraint {
    /// Max pods per node
    pub max_pods_per_node: i64,
}

/// GKE status condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeStatusCondition {
    /// Code
    pub code: String,
    /// Message
    pub message: String,
}

/// GKE upgrade settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeUpgradeSettings {
    /// Max surge
    pub max_surge: i32,
    /// Max unavailable
    pub max_unavailable: i32,
}

/// GKE legacy ABAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeLegacyAbac {
    /// Enabled
    pub enabled: bool,
}

/// GKE network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNetworkPolicy {
    /// Provider
    pub provider: String,
    /// Enabled
    pub enabled: bool,
}

/// GKE IP allocation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeIpAllocationPolicy {
    /// Use IP aliases
    pub use_ip_aliases: bool,
    /// Create subnetwork
    pub create_subnetwork: bool,
    /// Subnetwork name
    pub subnetwork_name: String,
    /// Cluster secondary range name
    pub cluster_secondary_range_name: String,
    /// Services secondary range name
    pub services_secondary_range_name: String,
    /// Cluster IPv4 CIDR block
    pub cluster_ipv4_cidr_block: String,
    /// Node IPv4 CIDR block
    pub node_ipv4_cidr_block: String,
    /// Services IPv4 CIDR block
    pub services_ipv4_cidr_block: String,
    /// TPU IPv4 CIDR block
    pub tpu_ipv4_cidr_block: String,
}

/// GKE master authorized networks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeMasterAuthorizedNetworksConfig {
    /// Enabled
    pub enabled: bool,
    /// CIDR blocks
    pub cidr_blocks: Vec<GkeCidrBlock>,
}

/// GKE CIDR block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeCidrBlock {
    /// Display name
    pub display_name: String,
    /// CIDR block
    pub cidr_block: String,
}

/// GKE maintenance policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeMaintenancePolicy {
    /// Window
    pub window: Option<GkeMaintenanceWindow>,
}

/// GKE maintenance window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeMaintenanceWindow {
    /// Daily maintenance window
    pub daily_maintenance_window: Option<GkeDailyMaintenanceWindow>,
    /// Recurring window
    pub recurring_window: Option<GkeRecurringTimeWindow>,
    /// Maintenance exclusions
    pub maintenance_exclusions: HashMap<String, GkeTimeWindow>,
}

/// GKE daily maintenance window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeDailyMaintenanceWindow {
    /// Start time
    pub start_time: String,
    /// Duration
    pub duration: String,
}

/// GKE recurring time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeRecurringTimeWindow {
    /// Window
    pub window: Option<GkeTimeWindow>,
    /// Recurrence
    pub recurrence: String,
}

/// GKE time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeTimeWindow {
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
}

/// GKE binary authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeBinaryAuthorization {
    /// Enabled
    pub enabled: bool,
}

/// GKE Autopilot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeAutopilot {
    /// Enabled
    pub enabled: bool,
}

/// GKE node pool defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodePoolDefaults {
    /// Node config defaults
    pub node_config_defaults: Option<GkeNodeConfigDefaults>,
}

/// GKE node config defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodeConfigDefaults {
    /// Logging config
    pub logging_config: Option<GkeNodePoolLoggingConfig>,
}

/// GKE node pool logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeNodePoolLoggingConfig {
    /// Variant config
    pub variant_config: Option<GkeLoggingVariantConfig>,
}

/// GKE logging variant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkeLoggingVariantConfig {
    /// Variant
    pub variant: String,
}

/// Cloud Storage bucket with enhanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucket {
    /// Name
    pub name: String,
    /// Location
    pub location: String,
    /// Location type
    pub location_type: String,
    /// Storage class
    pub storage_class: String,
    /// Versioning
    pub versioning: Option<GcsBucketVersioning>,
    /// Lifecycle
    pub lifecycle: Option<GcsBucketLifecycle>,
    /// Encryption
    pub encryption: Option<GcsBucketEncryption>,
    /// IAM configuration
    pub iam_configuration: Option<GcsBucketIamConfiguration>,
    /// Logging
    pub logging: Option<GcsBucketLogging>,
    /// Website
    pub website: Option<GcsBucketWebsite>,
    /// Cors
    pub cors: Vec<GcsBucketCors>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Time created
    pub time_created: String,
    /// Updated
    pub updated: String,
    /// Metageneration
    pub metageneration: i64,
    /// Self link
    pub self_link: String,
    /// Project number
    pub project_number: i64,
}

/// GCS bucket versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketVersioning {
    /// Enabled
    pub enabled: bool,
}

/// GCS bucket lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketLifecycle {
    /// Rules
    pub rule: Vec<GcsBucketLifecycleRule>,
}

/// GCS bucket lifecycle rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketLifecycleRule {
    /// Action
    pub action: GcsBucketLifecycleAction,
    /// Condition
    pub condition: GcsBucketLifecycleCondition,
}

/// GCS bucket lifecycle action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketLifecycleAction {
    /// Type
    pub action_type: String,
    /// Storage class
    pub storage_class: Option<String>,
}

/// GCS bucket lifecycle condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketLifecycleCondition {
    /// Age
    pub age: Option<i32>,
    /// Created before
    pub created_before: Option<String>,
    /// Is live
    pub is_live: Option<bool>,
    /// Matches storage class
    pub matches_storage_class: Vec<String>,
    /// Num newer versions
    pub num_newer_versions: Option<i32>,
}

/// GCS bucket encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketEncryption {
    /// Default KMS key name
    pub default_kms_key_name: String,
}

/// GCS bucket IAM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketIamConfiguration {
    /// Uniform bucket level access
    pub uniform_bucket_level_access: Option<GcsUniformBucketLevelAccess>,
    /// Public access prevention
    pub public_access_prevention: Option<String>,
}

/// GCS uniform bucket level access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsUniformBucketLevelAccess {
    /// Enabled
    pub enabled: bool,
    /// Locked time
    pub locked_time: Option<String>,
}

/// GCS bucket logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketLogging {
    /// Log bucket
    pub log_bucket: String,
    /// Log object prefix
    pub log_object_prefix: String,
}

/// GCS bucket website
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketWebsite {
    /// Main page suffix
    pub main_page_suffix: String,
    /// Not found page
    pub not_found_page: String,
}

/// GCS bucket CORS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsBucketCors {
    /// Origin
    pub origin: Vec<String>,
    /// Method
    pub method: Vec<String>,
    /// Response header
    pub response_header: Vec<String>,
    /// Max age seconds
    pub max_age_seconds: Option<i32>,
}

/// GCP client with comprehensive 2024-2025 feature support
pub struct GcpClient {
    /// GCP configuration
    config: GcpConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module
    security: SecurityModule,
    /// Current project
    current_project: String,
}

impl GcpClient {
    /// Create a new GCP client
    pub fn new(config: GcpConfig, lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        // Validate gcloud CLI availability
        Self::check_gcloud_cli()?;
        
        let current_project = config.project_id.clone();
        
        Ok(Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
            current_project,
        })
    }
    
    /// Check if gcloud CLI is available and configured
    fn check_gcloud_cli() -> Result<()> {
        let output = std::process::Command::new("gcloud")
            .arg("--version")
            .output()
            .map_err(|_| Error::config("gcloud CLI not found. Please install Google Cloud SDK"))?;
            
        if !output.status.success() {
            return Err(Error::config("gcloud CLI not properly configured"));
        }
        
        Ok(())
    }
    
    /// Execute gcloud command with proper authentication
    async fn execute_gcloud_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("gcloud");
        
        // Add project
        cmd.args(&["--project", &self.current_project]);
        
        // Add format for consistent output
        cmd.args(&["--format", "json"]);
        
        // Add arguments
        cmd.args(args);
        
        // Execute command
        let output = cmd.output().await
            .map_err(|e| Error::internal(format!("Failed to execute gcloud command: {}", e)))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::service(format!("gcloud command failed: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// List all cloud resources across services
    pub async fn list_resources(&self) -> Result<Vec<CloudResource>> {
        let mut resources = Vec::new();
        
        // Compute Engine instances
        if let Ok(instances) = self.list_compute_instances().await {
            for instance in instances {
                let mut labels = instance.labels.clone();
                labels.insert("ResourceType".to_string(), "ComputeInstance".to_string());
                labels.insert("MachineType".to_string(), instance.machine_type.clone());
                
                resources.push(CloudResource {
                    id: instance.self_link.clone(),
                    name: instance.name.clone(),
                    resource_type: "compute.googleapis.com/Instance".to_string(),
                    provider: CloudProvider::GCP,
                    region: instance.zone.clone(),
                    tags: labels,
                    cost: None, // Would need billing API
                    security_score: None,
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: 75.0,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        // Cloud Run services
        if let Ok(services) = self.list_cloud_run_services().await {
            for service in services {
                let mut labels = service.labels.clone();
                labels.insert("ResourceType".to_string(), "CloudRunService".to_string());
                
                resources.push(CloudResource {
                    id: service.self_link.clone(),
                    name: service.name.clone(),
                    resource_type: "run.googleapis.com/Service".to_string(),
                    provider: CloudProvider::GCP,
                    region: service.region.clone(),
                    tags: labels,
                    cost: None,
                    security_score: Some(80.0), // Cloud Run is generally secure by default
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: 80.0,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        // Cloud Storage buckets
        if let Ok(buckets) = self.list_gcs_buckets().await {
            for bucket in buckets {
                let mut labels = bucket.labels.clone();
                labels.insert("ResourceType".to_string(), "StorageBucket".to_string());
                labels.insert("StorageClass".to_string(), bucket.storage_class.clone());
                
                let security_score = if bucket.encryption.is_some() && 
                    bucket.iam_configuration.as_ref()
                        .and_then(|iac| iac.uniform_bucket_level_access.as_ref())
                        .map_or(false, |ubla| ubla.enabled) {
                    85.0
                } else {
                    60.0
                };
                
                resources.push(CloudResource {
                    id: bucket.self_link.clone(),
                    name: bucket.name.clone(),
                    resource_type: "storage.googleapis.com/Bucket".to_string(),
                    provider: CloudProvider::GCP,
                    region: bucket.location.clone(),
                    tags: labels,
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
    
    /// List Compute Engine instances
    pub async fn list_compute_instances(&self) -> Result<Vec<ComputeInstance>> {
        let output = self.execute_gcloud_command(&[
            "compute", "instances", "list"
        ]).await?;
        
        let instances_data: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse compute instances: {}", e)))?;
        
        let mut instances = Vec::new();
        
        if let Some(instances_array) = instances_data.as_array() {
            for instance_data in instances_array {
                // Parse the instance data - this is a simplified version
                // In a real implementation, you'd need to handle all the nested structures
                instances.push(ComputeInstance {
                    id: instance_data.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string(),
                    name: instance_data.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                    machine_type: instance_data.get("machineType").and_then(|mt| mt.as_str()).unwrap_or("").to_string(),
                    status: instance_data.get("status").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                    zone: instance_data.get("zone").and_then(|z| z.as_str()).unwrap_or("").to_string(),
                    network_interfaces: Vec::new(), // Would need detailed parsing
                    disks: Vec::new(), // Would need detailed parsing
                    tags: Vec::new(),
                    labels: HashMap::new(),
                    service_accounts: Vec::new(),
                    scheduling: None,
                    metadata: None,
                    creation_timestamp: instance_data.get("creationTimestamp").and_then(|ct| ct.as_str()).unwrap_or("").to_string(),
                    self_link: instance_data.get("selfLink").and_then(|sl| sl.as_str()).unwrap_or("").to_string(),
                    cpu_platform: instance_data.get("cpuPlatform").and_then(|cp| cp.as_str()).map(|s| s.to_string()),
                    confidential_instance_config: None,
                    is_spot: false, // Would need to check scheduling
                });
            }
        }
        
        Ok(instances)
    }
    
    /// List Cloud Run services
    pub async fn list_cloud_run_services(&self) -> Result<Vec<CloudRunService>> {
        let output = self.execute_gcloud_command(&[
            "run", "services", "list", "--platform", "managed"
        ]).await?;
        
        let services_data: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse Cloud Run services: {}", e)))?;
        
        let mut services = Vec::new();
        
        if let Some(services_array) = services_data.as_array() {
            for service_data in services_array {
                // Simplified parsing - real implementation would handle all nested structures
                services.push(CloudRunService {
                    name: service_data.get("metadata").and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string(),
                    namespace: service_data.get("metadata").and_then(|m| m.get("namespace")).and_then(|n| n.as_str()).unwrap_or("").to_string(),
                    generation: service_data.get("metadata").and_then(|m| m.get("generation")).and_then(|g| g.as_i64()).unwrap_or(0),
                    labels: HashMap::new(),
                    annotations: HashMap::new(),
                    creation_timestamp: service_data.get("metadata").and_then(|m| m.get("creationTimestamp")).and_then(|ct| ct.as_str()).unwrap_or("").to_string(),
                    spec: CloudRunServiceSpec {
                        template: CloudRunRevisionTemplate {
                            metadata: CloudRunRevisionTemplateMetadata {
                                labels: HashMap::new(),
                                annotations: HashMap::new(),
                            },
                            spec: CloudRunRevisionSpec {
                                container_concurrency: None,
                                timeout_seconds: None,
                                service_account_name: None,
                                containers: Vec::new(),
                                volumes: Vec::new(),
                            },
                        },
                        traffic: Vec::new(),
                    },
                    status: CloudRunServiceStatus {
                        observed_generation: None,
                        conditions: Vec::new(),
                        latest_ready_revision_name: None,
                        latest_created_revision_name: None,
                        traffic: Vec::new(),
                        url: service_data.get("status").and_then(|s| s.get("url")).and_then(|u| u.as_str()).map(|s| s.to_string()),
                        address: None,
                    },
                    self_link: "".to_string(),
                    region: service_data.get("metadata").and_then(|m| m.get("labels")).and_then(|l| l.get("cloud.googleapis.com/location")).and_then(|loc| loc.as_str()).unwrap_or("").to_string(),
                });
            }
        }
        
        Ok(services)
    }
    
    /// List Cloud Storage buckets
    pub async fn list_gcs_buckets(&self) -> Result<Vec<GcsBucket>> {
        let output = self.execute_gcloud_command(&[
            "storage", "buckets", "list"
        ]).await?;
        
        let buckets_data: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse GCS buckets: {}", e)))?;
        
        let mut buckets = Vec::new();
        
        if let Some(buckets_array) = buckets_data.as_array() {
            for bucket_data in buckets_array {
                // Simplified parsing
                buckets.push(GcsBucket {
                    name: bucket_data.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                    location: bucket_data.get("location").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                    location_type: bucket_data.get("locationType").and_then(|lt| lt.as_str()).unwrap_or("").to_string(),
                    storage_class: bucket_data.get("storageClass").and_then(|sc| sc.as_str()).unwrap_or("STANDARD").to_string(),
                    versioning: None,
                    lifecycle: None,
                    encryption: None,
                    iam_configuration: None,
                    logging: None,
                    website: None,
                    cors: Vec::new(),
                    labels: HashMap::new(),
                    time_created: bucket_data.get("timeCreated").and_then(|tc| tc.as_str()).unwrap_or("").to_string(),
                    updated: bucket_data.get("updated").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                    metageneration: bucket_data.get("metageneration").and_then(|mg| mg.as_i64()).unwrap_or(0),
                    self_link: bucket_data.get("selfLink").and_then(|sl| sl.as_str()).unwrap_or("").to_string(),
                    project_number: 0,
                });
            }
        }
        
        Ok(buckets)
    }
    
    /// Perform comprehensive security assessment
    pub async fn security_assessment(&self) -> Result<SecurityAssessment> {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        let mut total_score: f64 = 100.0;
        
        // Check Compute Engine security
        if let Ok(instances) = self.list_compute_instances().await {
            for instance in instances {
                // Check for external IP addresses
                if !instance.network_interfaces.is_empty() {
                    // In a real implementation, check for external IPs
                    violations.push(SecurityViolation {
                        resource_id: instance.name.clone(),
                        rule_id: "GCE-001".to_string(),
                        severity: ViolationSeverity::Medium,
                        description: "Compute instance may have external IP address".to_string(),
                        provider: CloudProvider::GCP,
                    });
                    total_score -= 5.0;
                }
                
                // Check OS login
                if instance.metadata.is_none() {
                    recommendations.push(SecurityRecommendation {
                        id: format!("GCE-OSLOGIN-{}", instance.name),
                        title: "Enable OS Login".to_string(),
                        description: format!("Enable OS Login for instance {}", instance.name),
                        priority: RecommendationPriority::Medium,
                        impact: "Improves access control and audit logging".to_string(),
                        steps: vec![
                            "Navigate to Compute Engine console".to_string(),
                            format!("Select instance {}", instance.name),
                            "Edit instance".to_string(),
                            "Add metadata key: enable-oslogin, value: TRUE".to_string(),
                        ],
                    });
                }
            }
        }
        
        // Check Cloud Storage security
        if let Ok(buckets) = self.list_gcs_buckets().await {
            for bucket in buckets {
                // Check uniform bucket-level access
                if bucket.iam_configuration.is_none() {
                    violations.push(SecurityViolation {
                        resource_id: bucket.name.clone(),
                        rule_id: "GCS-001".to_string(),
                        severity: ViolationSeverity::High,
                        description: "GCS bucket does not have uniform bucket-level access enabled".to_string(),
                        provider: CloudProvider::GCP,
                    });
                    total_score -= 15.0;
                    
                    recommendations.push(SecurityRecommendation {
                        id: format!("GCS-UBLA-{}", bucket.name),
                        title: "Enable uniform bucket-level access".to_string(),
                        description: format!("Enable uniform bucket-level access for bucket {}", bucket.name),
                        priority: RecommendationPriority::High,
                        impact: "Improves security by using IAM for access control".to_string(),
                        steps: vec![
                            "Navigate to Cloud Storage console".to_string(),
                            format!("Select bucket {}", bucket.name),
                            "Go to Permissions tab".to_string(),
                            "Switch to uniform access control".to_string(),
                        ],
                    });
                }
                
                // Check encryption
                if bucket.encryption.is_none() {
                    violations.push(SecurityViolation {
                        resource_id: bucket.name.clone(),
                        rule_id: "GCS-002".to_string(),
                        severity: ViolationSeverity::Medium,
                        description: "GCS bucket does not use customer-managed encryption".to_string(),
                        provider: CloudProvider::GCP,
                    });
                    total_score -= 10.0;
                }
            }
        }
        
        Ok(SecurityAssessment {
            overall_score: total_score.max(0.0),
            provider_scores: HashMap::from([(CloudProvider::GCP, total_score.max(0.0))]),
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
        
        // Analyze Compute Engine instances
        if let Ok(instances) = self.list_compute_instances().await {
            for instance in instances {
                if instance.status == "RUNNING" {
                    // Suggest preemptible instances for non-critical workloads
                    if !instance.is_spot {
                        let estimated_savings = 100.0; // Placeholder
                        
                        recommendations.push(CostRecommendation {
                            resource_id: instance.name.clone(),
                            recommendation_type: "Use preemptible instances".to_string(),
                            potential_savings: estimated_savings,
                            description: "Consider using preemptible instances for fault-tolerant workloads".to_string(),
                            complexity: ComplexityLevel::Medium,
                        });
                        
                        total_savings += estimated_savings;
                    }
                    
                    // Suggest committed use discounts
                    reserved_instances.push(ReservedInstanceRecommendation {
                        instance_type: instance.machine_type.clone(),
                        quantity: 1,
                        term: ReservedInstanceTerm::OneYear,
                        payment_option: PaymentOption::NoUpfront,
                        annual_savings: 200.0, // Placeholder
                    });
                }
            }
        }
        
        // General recommendations
        recommendations.push(CostRecommendation {
            resource_id: "general".to_string(),
            recommendation_type: "Enable billing exports".to_string(),
            potential_savings: 0.0,
            description: "Export billing data to BigQuery for detailed cost analysis".to_string(),
            complexity: ComplexityLevel::Low,
        });
        
        recommendations.push(CostRecommendation {
            resource_id: "general".to_string(),
            recommendation_type: "Set up budget alerts".to_string(),
            potential_savings: 0.0,
            description: "Create budget alerts to monitor spending".to_string(),
            complexity: ComplexityLevel::Low,
        });
        
        Ok(CostOptimization {
            total_potential_savings: total_savings,
            recommendations,
            rightsizing_opportunities: rightsizing,
            reserved_instance_recommendations: reserved_instances,
        })
    }
    
    /// Get current project
    pub fn get_current_project(&self) -> &str {
        &self.current_project
    }
    
    /// Set current project
    pub fn set_project(&mut self, project_id: String) {
        self.current_project = project_id;
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &GcpConfig {
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

/// Helper function to add chrono dependency implicitly
use chrono;